//! Graceful stop completion for one persisted sandbox and runtime generation.

use std::time::Duration;

use microsandbox_runtime::ipc::try_acquire_lifecycle_guard;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::db::entity::run;
use crate::sandbox::SandboxStatus;
use crate::{MicrosandboxError, MicrosandboxResult};

use super::LocalBackend;

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl LocalBackend {
    /// Send shutdown and prove terminal state plus ownership release for the same run.
    pub(crate) async fn stop_complete(&self, name: &str, id: i32) -> MicrosandboxResult<()> {
        let run_dir = self.config().run_dir();
        let transition = Self::acquire_sandbox_transition_guard(&run_dir, name).await?;
        let (model, _) = self
            .sandbox_handle_state_owned(name, Some(id), true)
            .await?;
        let run_id = self.latest_stop_run(id).await?.map(|run| run.id);
        self.request_stop_owned(name, &model).await?;
        // Exit cleanup also needs transition ownership. Never retain this guard while waiting.
        drop(transition);
        self.wait_stop_complete(name, id, run_id, model.ephemeral)
            .await
    }

    async fn latest_stop_run(&self, id: i32) -> MicrosandboxResult<Option<run::Model>> {
        Ok(run::Entity::find()
            .filter(run::Column::SandboxId.eq(id))
            .order_by_desc(run::Column::Id)
            .one(self.db().await?.read())
            .await?)
    }

    async fn wait_stop_complete(
        &self,
        name: &str,
        id: i32,
        run_id: Option<i32>,
        ephemeral: bool,
    ) -> MicrosandboxResult<()> {
        let run_dir = self.config().run_dir();
        loop {
            let transition = Self::acquire_sandbox_transition_guard(&run_dir, name).await?;
            // Reconcile a crashed owner using the existing recovery rules before inspecting the
            // selected run. A reused name or restarted run must never redirect this operation.
            let model = match self.sandbox_handle_state_owned(name, Some(id), true).await {
                Ok((model, _)) => Some(model),
                Err(MicrosandboxError::SandboxNotFound(_)) if ephemeral => None,
                Err(error) => return Err(error),
            };
            let latest = self.latest_stop_run(id).await?;
            if model.is_some() && latest.as_ref().map(|run| run.id) != run_id {
                return Err(MicrosandboxError::Runtime(format!(
                    "sandbox {name:?} (id {id}) restarted while stopping run {run_id:?}; refusing to follow run {:?}",
                    latest.as_ref().map(|run| run.id)
                )));
            }
            if let Some(_ownership) = try_acquire_lifecycle_guard(&run_dir, name)? {
                // Both guards fence restart/removal during the final check. PID visibility is
                // irrelevant: zombies own no runtime resources, and PIDs can be recycled.
                if let Some(model) = model {
                    let terminal = matches!(
                        model.status,
                        SandboxStatus::Created | SandboxStatus::Stopped | SandboxStatus::Crashed
                    ) && latest
                        .as_ref()
                        .is_none_or(|run| run.status == run::RunStatus::Terminated);
                    if !terminal {
                        // A crashed runtime may not have published its terminal row. Ownership
                        // release proves this generation is gone even if its PID is recycled.
                        let (status, reason) = Self::stale_runtime_terminal_state(model.status);
                        Self::mark_sandbox_runtime_stale(
                            self.db().await?.write(),
                            id,
                            run_id,
                            status,
                            reason,
                        )
                        .await?;
                    }
                }
                return Ok(());
            }
            drop(transition);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use sea_orm::Set;

    use super::*;
    use crate::db::entity::sandbox;
    use crate::sandbox::SandboxConfig;

    async fn fixture(name: &str) -> (tempfile::TempDir, LocalBackend, i32, i32) {
        let home = tempfile::tempdir().unwrap();
        let backend = LocalBackend::builder()
            .home(home.path())
            .build()
            .await
            .unwrap();
        let config = SandboxConfig {
            spec: microsandbox_types::SandboxSpec {
                name: name.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let pools = backend.db().await.unwrap();
        let id = LocalBackend::insert_sandbox_record(pools.write(), &config)
            .await
            .unwrap();
        LocalBackend::update_sandbox_status(pools.write(), id, SandboxStatus::Stopped)
            .await
            .unwrap();
        let run_id = run::Entity::insert(run::ActiveModel {
            sandbox_id: Set(id),
            // A visible PID cannot substitute for the runtime's ownership lock.
            pid: Set(Some(std::process::id() as i32)),
            status: Set(run::RunStatus::Terminated),
            ..Default::default()
        })
        .exec(pools.write())
        .await
        .unwrap()
        .last_insert_id;
        (home, backend, id, run_id)
    }

    #[tokio::test]
    async fn terminal_database_does_not_complete_stop_until_runtime_releases_ownership() {
        let (_home, backend, id, _) = fixture("delayed-teardown").await;
        let ownership =
            try_acquire_lifecycle_guard(&backend.config().run_dir(), "delayed-teardown")
                .unwrap()
                .unwrap();
        assert!(
            tokio::time::timeout(
                Duration::from_millis(80),
                backend.stop_complete("delayed-teardown", id)
            )
            .await
            .is_err()
        );
        assert!(
            try_acquire_lifecycle_guard(&backend.config().run_dir(), "delayed-teardown")
                .unwrap()
                .is_none()
        );
        drop(ownership);
        backend.stop_complete("delayed-teardown", id).await.unwrap();
        crate::sandbox::remove_local_persisted_sandbox(&backend, "delayed-teardown", id)
            .await
            .unwrap();
        assert!(
            sandbox::Entity::find_by_id(id)
                .one(backend.db().await.unwrap().read())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn stopped_wait_rejects_a_new_run_of_the_same_persisted_sandbox() {
        let (_home, backend, id, run_id) = fixture("restarted").await;
        run::Entity::insert(run::ActiveModel {
            sandbox_id: Set(id),
            status: Set(run::RunStatus::Terminated),
            ..Default::default()
        })
        .exec(backend.db().await.unwrap().write())
        .await
        .unwrap();
        let error = backend
            .wait_stop_complete("restarted", id, Some(run_id), false)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("refusing to follow run"));
    }

    #[tokio::test]
    async fn ownership_release_reconciles_a_stale_run_despite_a_visible_pid() {
        let (_home, backend, id, run_id) = fixture("stale-run").await;
        run::Entity::update_many()
            .col_expr(
                run::Column::Status,
                sea_orm::sea_query::Expr::value(run::RunStatus::Running),
            )
            .filter(run::Column::Id.eq(run_id))
            .exec(backend.db().await.unwrap().write())
            .await
            .unwrap();
        backend
            .wait_stop_complete("stale-run", id, Some(run_id), false)
            .await
            .unwrap();
        assert_eq!(
            backend.latest_stop_run(id).await.unwrap().unwrap().status,
            run::RunStatus::Terminated
        );
    }
}
