//! Cancellation cleanup for an unpublished local create, retaining namespace ownership.

use std::sync::Arc;
use std::time::Duration;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::sync::Mutex;

use super::SandboxTransitionGuard;
use crate::backend::Backend;
use crate::db::entity::sandbox as sandbox_entity;
use crate::runtime::ProcessHandle;
use crate::runtime::spawn::EnsuredNamedVolumes;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Armed before inserting the provisional row, so cancellation during the DB commit is covered.
/// The name transition remains locked until cleanup finishes; a replacement cannot be targeted.
pub(super) struct CreationCleanup {
    state: Option<CleanupState>,
}

struct CleanupState {
    backend: Arc<dyn Backend>,
    name: String,
    _transition: SandboxTransitionGuard,
    volumes: Arc<EnsuredNamedVolumes>,
    process: Option<Arc<Mutex<ProcessHandle>>>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl CreationCleanup {
    pub(super) fn new(
        backend: Arc<dyn Backend>,
        name: String,
        transition: SandboxTransitionGuard,
        volumes: Arc<EnsuredNamedVolumes>,
    ) -> Self {
        Self {
            state: Some(CleanupState {
                backend,
                name,
                _transition: transition,
                volumes,
                process: None,
            }),
        }
    }

    pub(super) fn retain_process(&mut self, process: Option<Arc<Mutex<ProcessHandle>>>) {
        if let Some(state) = &mut self.state {
            state.process = process;
        }
    }

    /// Called only after the complete creation result has been committed.
    pub(super) fn disarm(&mut self) {
        self.state.take();
    }
}

impl CleanupState {
    async fn cleanup(self) -> crate::MicrosandboxResult<()> {
        if let Some(process) = &self.process {
            process.lock().await.terminate_failed_startup().await?;
        }
        let local = self.backend.as_local().ok_or_else(|| {
            crate::MicrosandboxError::Runtime("local creation cleanup lost its backend".into())
        })?;
        // Before publication, StartupProcess owns termination/reaping. Wait for its exact
        // lifecycle lock to be released before reconciling; never mistake cancellation for exit.
        let guard = crate::runtime::acquire_sandbox_lifecycle_guard(
            &local.config().run_dir(),
            &self.name,
            Duration::from_secs(10),
        )
        .await?;
        let pools = local.db().await?;
        let model = sandbox_entity::Entity::find()
            .filter(sandbox_entity::Column::Name.eq(&self.name))
            // Queue behind any cancelled insert/commit on the single writer connection.
            // A WAL reader could otherwise observe "no row" before that commit completes.
            .one(pools.write())
            .await?;
        // The retained transition excludes a new create/start/remove throughout the query
        // and rollback. Release only the lifecycle probe so the existing rollback can own it.
        drop(guard);
        if let Some(model) = model {
            local
                .rollback_failed_startup(pools.write(), model.id, &self.name, &self.volumes)
                .await?;
        } else {
            crate::runtime::rollback_created_named_volumes(local, &self.volumes).await;
        }
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Drop for CreationCleanup {
    fn drop(&mut self) {
        let Some(state) = self.state.take() else {
            return;
        };
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                if let Err(error) = state.cleanup().await {
                    // Retain durable restore intent and storage on uncertain cleanup. Normal
                    // startup maintenance may reconcile only after ownership is actually gone.
                    tracing::error!(%error, "cancelled creation cleanup remains pending");
                }
            });
        } else {
            tracing::error!(sandbox = %state.name, "runtime unavailable for cancelled creation reconciliation");
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::local::LocalBackend;
    use crate::db::entity::run as run_entity;
    use crate::sandbox::{SandboxBuilder, SandboxStatus};
    use sea_orm::Set;

    #[tokio::test]
    async fn cancelled_creation_retains_name_until_ownership_and_catalog_are_reconciled() {
        let directory = tempfile::tempdir().unwrap();
        let rootfs = directory.path().join("rootfs");
        std::fs::create_dir_all(&rootfs).unwrap();
        let backend = Arc::new(
            LocalBackend::builder()
                .home(directory.path().join("home"))
                .build()
                .await
                .unwrap(),
        );
        let config = SandboxBuilder::new("cancelled-create")
            .image(rootfs)
            .build()
            .await
            .unwrap();
        let pools = backend.db().await.unwrap();
        let transition = LocalBackend::acquire_sandbox_transition_guard(
            &backend.config().run_dir(),
            &config.spec.name,
        )
        .await
        .unwrap();
        let runtime = microsandbox_runtime::ipc::try_acquire_lifecycle_guard(
            &backend.config().run_dir(),
            &config.spec.name,
        )
        .unwrap()
        .unwrap();
        let volumes = Arc::new(
            crate::runtime::ensure_named_volumes(&backend, &config)
                .await
                .unwrap(),
        );
        let cleanup = CreationCleanup::new(
            backend.clone(),
            config.spec.name.clone(),
            transition,
            volumes,
        );
        let id = LocalBackend::insert_starting_sandbox_record(pools.write(), &config)
            .await
            .unwrap();
        LocalBackend::update_sandbox_status(pools.write(), id, SandboxStatus::Running)
            .await
            .unwrap();
        run_entity::Entity::insert(run_entity::ActiveModel {
            sandbox_id: Set(id),
            status: Set(run_entity::RunStatus::Running),
            ..Default::default()
        })
        .exec(pools.write())
        .await
        .unwrap();
        drop(cleanup);
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(
            microsandbox_runtime::ipc::try_acquire_transition_guard(
                &backend.config().run_dir(),
                &config.spec.name,
            )
            .unwrap()
            .is_none(),
            "cleanup must exclude a replacement until ownership is gone"
        );
        assert_eq!(
            sandbox_entity::Entity::find_by_id(id)
                .one(pools.read())
                .await
                .unwrap()
                .unwrap()
                .status,
            SandboxStatus::Running
        );
        drop(runtime);
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let model = sandbox_entity::Entity::find_by_id(id)
                    .one(pools.read())
                    .await
                    .unwrap()
                    .unwrap();
                if model.status == SandboxStatus::Stopped {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        let runs = run_entity::Entity::find()
            .filter(run_entity::Column::SandboxId.eq(id))
            .all(pools.read())
            .await
            .unwrap();
        assert!(
            runs.iter()
                .all(|run| run.status == run_entity::RunStatus::Terminated)
        );
    }
}
