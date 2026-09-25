use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

use crate::database::entity::nar::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_000001_restore_sqlite_nar_hash_index"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // On SQLite, m20230112_000005_drop_old_nar_columns recreated the
        // NAR table without the index on nar_hash, so every NAR lookup by
        // hash scanned the entire table.
        if manager.get_database_backend() != DatabaseBackend::Sqlite {
            return Ok(());
        }

        manager
            .create_index(
                Index::create()
                    .name("idx-nar-nar-hash")
                    .table(Entity)
                    .col(Column::NarHash)
                    .to_owned(),
            )
            .await
    }
}
