//! Read-only lookup for a live control target, without unrelated snapshot reconciliation.

use std::time::Duration;

use microsandbox_db::DbReadConnection;
use microsandbox_migration::schema_metadata;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, QueryFilter, Statement};

use super::{
    LocalBackend, acquire_migration_lock, refuse_incomplete_self_downgrade, refuse_schema_ahead,
};
use crate::db::entity::sandbox as sandbox_entity;
use crate::sandbox::SandboxStatus;
use crate::{MicrosandboxError, MicrosandboxResult};

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl LocalBackend {
    /// Return a healthy live target from a current catalog. `None` requests the existing
    /// migration/stale-runtime path; errors must never become an unvalidated fast path.
    pub(crate) async fn try_control_handle_state(
        &self,
        name: &str,
    ) -> MicrosandboxResult<Option<(sandbox_entity::Model, Option<i32>)>> {
        let db_dir = self.config().home().join(microsandbox_utils::DB_SUBDIR);
        let db_path = db_dir.join(microsandbox_utils::DB_FILENAME);
        match std::fs::metadata(&db_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
        // Use the same installation coordination as the slow path. In particular, a rolled
        // back database can look current while an incomplete downgrade still owns its files.
        let _migration_lock = acquire_migration_lock(&db_dir).await?;
        refuse_incomplete_self_downgrade(&db_dir)?;
        let database = &self.config().database;
        let read = if let Some(pools) = self.db.get() {
            pools.read().clone()
        } else {
            DbReadConnection::open_read_only(
                &db_path,
                Duration::from_secs(database.connect_timeout_secs),
                Duration::from_secs(database.busy_timeout_secs),
            )
            .await
            .map_err(|error| {
                MicrosandboxError::Custom(format!(
                    "read control catalog {}: {error}",
                    db_path.display()
                ))
            })?
        };
        microsandbox_runtime::maintenance::refuse_if_install_exclusive_held(&read)
            .await
            .map_err(|error| MicrosandboxError::Runtime(error.to_string()))?;
        refuse_schema_ahead(read.inner()).await?;
        let row = match read
            .query_one_raw(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) FROM seaql_migrations",
            ))
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return Ok(None),
            Err(error) if super::is_missing_migrations_table(&error) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if row.try_get_by_index::<i64>(0)? != schema_metadata::migration_ids().count() as i64 {
            return Ok(None);
        }
        let model = sandbox_entity::Entity::find()
            .filter(sandbox_entity::Column::Name.eq(name))
            .one(&read)
            .await?
            .ok_or_else(|| MicrosandboxError::SandboxNotFound(name.into()))?;
        if !matches!(
            model.status,
            SandboxStatus::Running | SandboxStatus::Draining
        ) {
            return Ok(None);
        }
        let run = Self::load_active_run(&read, model.id).await?;
        let pid = Self::pid_from_run(run.as_ref());
        // Do not clean up sockets from this read-only observation. The slow path rechecks
        // the exact row/run under lifecycle ownership before touching stale artifacts.
        if pid.is_some_and(Self::pid_is_alive) {
            Ok(Some((model, pid)))
        } else {
            Ok(None)
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    async fn fixture() -> (tempfile::TempDir, LocalBackend) {
        let home = tempfile::tempdir().unwrap();
        let backend = LocalBackend::builder().home(home.path()).build_lazy();
        let pools = backend.db().await.unwrap();
        pools.write().execute_unprepared(
            "INSERT INTO sandbox (id, name, config, status, ephemeral) VALUES (1, 'source', '{}', 'Running', 0)",
        ).await.unwrap();
        pools
            .write()
            .execute_raw(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT INTO run (sandbox_id, pid, status) VALUES (1, ?, 'Running')",
                [i64::from(std::process::id()).into()],
            ))
            .await
            .unwrap();
        (home, backend)
    }

    #[tokio::test]
    async fn healthy_lookup_does_not_initialize_writer_or_reconcile_snapshots() {
        let (home, _) = fixture().await;
        // A non-directory snapshot path would fail normal reconciliation. The live
        // control path has no reason to inspect it or mutate the target's catalog.
        let snapshots = home.path().join("snapshots");
        if snapshots.exists() {
            std::fs::remove_dir(&snapshots).unwrap();
        }
        std::fs::write(&snapshots, b"unrelated snapshot inventory").unwrap();
        let backend = LocalBackend::builder().home(home.path()).build_lazy();
        let (model, pid) = backend
            .try_control_handle_state("source")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(model.id, 1);
        assert_eq!(pid, Some(std::process::id() as i32));
        assert!(backend.db.get().is_none());
        assert_eq!(
            std::fs::read(snapshots).unwrap(),
            b"unrelated snapshot inventory"
        );
    }

    #[tokio::test]
    async fn absent_catalog_and_terminal_target_request_slow_path() {
        let home = tempfile::tempdir().unwrap();
        let empty = LocalBackend::builder().home(home.path()).build_lazy();
        assert!(
            empty
                .try_control_handle_state("source")
                .await
                .unwrap()
                .is_none()
        );
        assert!(!home.path().join("db").exists());
        let (_home, backend) = fixture().await;
        backend
            .db()
            .await
            .unwrap()
            .write()
            .execute_unprepared("UPDATE sandbox SET status = 'Stopped' WHERE id = 1")
            .await
            .unwrap();
        assert!(
            backend
                .try_control_handle_state("source")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn schema_and_install_gates_are_not_bypassed() {
        let (home, backend) = fixture().await;
        let writer = backend.db().await.unwrap().write();
        writer.execute_unprepared(
            "INSERT INTO seaql_migrations (version, applied_at) VALUES ('future_unknown_migration', 0)",
        ).await.unwrap();
        assert!(
            backend
                .try_control_handle_state("source")
                .await
                .unwrap_err()
                .to_string()
                .contains("schema is newer")
        );
        writer
            .execute_unprepared(
                "DELETE FROM seaql_migrations WHERE version = 'future_unknown_migration'",
            )
            .await
            .unwrap();
        writer.execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
                "INSERT OR REPLACE INTO maintenance_lease (name, holder_pid, lease_expires_at) VALUES ('install_exclusive', ?, ?)",
            [(std::process::id() as i32).into(), (chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10)).into()],
        )).await.unwrap();
        assert!(
            backend
                .try_control_handle_state("source")
                .await
                .unwrap_err()
                .to_string()
                .contains("install operation in progress")
        );
        writer
            .execute_unprepared("DELETE FROM maintenance_lease WHERE name = 'install_exclusive'")
            .await
            .unwrap();
        let journal_dir = home.path().join("db/self-downgrade/test-operation");
        std::fs::create_dir_all(&journal_dir).unwrap();
        std::fs::write(
            journal_dir.join("journal.json"),
            br#"{"phase":"preparing"}"#,
        )
        .unwrap();
        assert!(
            backend
                .try_control_handle_state("source")
                .await
                .unwrap_err()
                .to_string()
                .contains("self_downgrade_recovery_required")
        );
    }

    #[tokio::test]
    async fn older_schema_and_missing_active_run_fall_back_without_cleanup() {
        let (home, backend) = fixture().await;
        let writer = backend.db().await.unwrap().write();
        writer
            .execute_unprepared("DELETE FROM run WHERE sandbox_id = 1")
            .await
            .unwrap();
        let runtime = home.path().join("sandboxes/source/runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        let hint = runtime.join("runtime-boot-id");
        std::fs::write(&hint, b"do-not-clean-during-observation").unwrap();
        assert!(
            backend
                .try_control_handle_state("source")
                .await
                .unwrap()
                .is_none()
        );
        assert!(hint.exists());
        let last = schema_metadata::migration_ids().last().unwrap();
        writer
            .execute_raw(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "DELETE FROM seaql_migrations WHERE version = ?",
                [last.into()],
            ))
            .await
            .unwrap();
        assert!(
            backend
                .try_control_handle_state("source")
                .await
                .unwrap()
                .is_none()
        );
        assert!(hint.exists());
    }
}
