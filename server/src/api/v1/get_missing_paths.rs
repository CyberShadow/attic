use std::collections::HashSet;

use axum::extract::{Extension, Json};
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, QuerySelect};
use tracing::instrument;

use crate::database::entity::cache;
use crate::database::entity::object::{self, Entity as Object};
use crate::error::{ServerError, ServerResult};
use crate::{RequestState, State};
use attic::api::v1::get_missing_paths::{GetMissingPathsRequest, GetMissingPathsResponse};
use attic::nix_store::StorePathHash;

/// The maximum number of hashes bound in a single query.
///
/// SQLite limits a statement to 32766 variables (and PostgreSQL to 65535),
/// while the request body limit admits about 58000 hashes.
const QUERY_BATCH_SIZE: usize = 10_000;

#[derive(FromQueryResult)]
struct StorePathHashOnly {
    store_path_hash: String,
}

/// Gets information on missing paths in a cache.
///
/// Requires "push" permission as it essentially allows probing
/// of cache contents.
#[instrument(skip_all, fields(payload))]
pub(crate) async fn get_missing_paths(
    Extension(state): Extension<State>,
    Extension(req_state): Extension<RequestState>,
    Json(payload): Json<GetMissingPathsRequest>,
) -> ServerResult<Json<GetMissingPathsResponse>> {
    let database = state.database().await?;
    req_state
        .auth
        .auth_cache(database, &payload.cache, |_, permission| {
            permission.require_push()?;
            Ok(())
        })
        .await?;

    let requested_hashes: HashSet<String> = payload
        .store_path_hashes
        .iter()
        .map(|h| h.as_str().to_owned())
        .collect();
    let requested_hashes_vec: Vec<&String> = requested_hashes.iter().collect();

    let mut found_hashes: HashSet<String> = HashSet::new();
    for batch in requested_hashes_vec.chunks(QUERY_BATCH_SIZE) {
        let query_in = batch.iter().map(|h| Value::from(h.as_str()));

        let result: Vec<StorePathHashOnly> = Object::find()
            .select_only()
            .column_as(object::Column::StorePathHash, "store_path_hash")
            .join(sea_orm::JoinType::InnerJoin, object::Relation::Cache.def())
            .join(sea_orm::JoinType::InnerJoin, object::Relation::Nar.def())
            .filter(cache::Column::Name.eq(payload.cache.as_str()))
            .filter(object::Column::StorePathHash.is_in(query_in))
            .into_model::<StorePathHashOnly>()
            .all(database)
            .await
            .map_err(ServerError::database_error)?;

        found_hashes.extend(result.into_iter().map(|row| row.store_path_hash));
    }

    // Safety: All requested_hashes are validated `StorePathHash`es.
    // No need to pay the cost of checking again
    #[allow(unsafe_code)]
    let missing_paths = requested_hashes
        .difference(&found_hashes)
        .map(|h| unsafe { StorePathHash::new_unchecked(h.to_string()) })
        .collect();

    Ok(Json(GetMissingPathsResponse { missing_paths }))
}
