//! get-missing-paths v1
//!
//! `POST /_api/v1/get-missing-paths`
//!
//! Requires "push" permission.

use serde::{Deserialize, Serialize};

use crate::cache::CacheName;
use crate::nix_store::StorePathHash;

/// The maximum number of store path hashes a client sends per request.
///
/// Each hash takes about 35 bytes of JSON, so this keeps requests well
/// below the server's default request body limit of 2 MiB.
pub const MAX_STORE_PATH_HASHES: usize = 10_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMissingPathsRequest {
    /// The name of the cache.
    pub cache: CacheName,

    /// The list of store paths.
    pub store_path_hashes: Vec<StorePathHash>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMissingPathsResponse {
    /// A list of paths that are not in the cache.
    pub missing_paths: Vec<StorePathHash>,
}
