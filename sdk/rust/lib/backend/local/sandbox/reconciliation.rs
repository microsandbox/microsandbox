//! Reconcile persisted local sandbox state with launcher and runtime ownership.

use std::path::Path;

use microsandbox_db::pool::DbPools;
use microsandbox_runtime::transition::{
    SandboxTransitionGuard, sandbox_runtime_endpoint_is_live, try_acquire_transition_guard,
};
use sea_orm::EntityTrait;

use super::{LocalBackend, SandboxStatus, sandbox_entity};
use crate::MicrosandboxResult;

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl LocalBackend {
    /// Reconcile a Starting/Running/Draining row against the owning process's
    /// liveness, marking it terminal when the runtime is gone.
    pub(super) async fn reconcile_sandbox_runtime_state(
        &self,
        pools: &DbPools,
        sandbox: sandbox_entity::Model,
    ) -> MicrosandboxResult<sandbox_entity::Model> {
        let run_dir = self.config().run_dir();
        let sandboxes_dir = self.config().sandboxes_dir();
        Self::reconcile_sandbox_runtime_state_with_paths(
            pools,
            sandbox,
            Some((&run_dir, &sandboxes_dir)),
        )
        .await
    }

    /// Reconcile runtime state with optional exact socket roots.
    pub(super) async fn reconcile_sandbox_runtime_state_with_paths(
        pools: &DbPools,
        sandbox: sandbox_entity::Model,
        socket_roots: Option<(&Path, &Path)>,
    ) -> MicrosandboxResult<sandbox_entity::Model> {
        Self::reconcile_sandbox_runtime_state_with_transition(pools, sandbox, socket_roots, None)
            .await
    }

    /// Reuse a start caller's transition ownership instead of trying to lock itself.
    pub(super) async fn reconcile_sandbox_runtime_state_with_transition(
        pools: &DbPools,
        sandbox: sandbox_entity::Model,
        socket_roots: Option<(&Path, &Path)>,
        held_transition: Option<&SandboxTransitionGuard>,
    ) -> MicrosandboxResult<sandbox_entity::Model> {
        if !matches!(
            sandbox.status,
            SandboxStatus::Starting | SandboxStatus::Running | SandboxStatus::Draining
        ) {
            return Ok(sandbox);
        }

        let run = Self::load_active_run(pools.read(), sandbox.id).await?;
        if run
            .as_ref()
            .and_then(|run| run.pid)
            .is_some_and(Self::pid_is_alive)
        {
            return Ok(sandbox);
        }

        // A launcher owns Starting before the child records its PID. Prove that
        // handoff abandoned with the same lock used by existing launchers.
        let transition_guard = if sandbox.status == SandboxStatus::Starting
            && held_transition.is_none()
            && let Some((run_dir, _)) = socket_roots
        {
            let Some(guard) = try_acquire_transition_guard(run_dir, &sandbox.name)? else {
                return Ok(sandbox);
            };
            Some(guard)
        } else {
            None
        };
        let owns_transition = held_transition.is_some() || transition_guard.is_some();

        // A dead-PID snapshot is not sufficient: another process may already
        // have reconciled and restarted this name. Serialize on the runtime
        // ownership lock, then re-read the exact row/run before unlinking.
        let _guard = if let Some((run_dir, _)) = socket_roots {
            let Some(guard) =
                microsandbox_runtime::ipc::try_acquire_lifecycle_guard(run_dir, &sandbox.name)?
            else {
                return Ok(sandbox);
            };
            Some(guard)
        } else {
            None
        };
        let Some(sandbox) = sandbox_entity::Entity::find_by_id(sandbox.id)
            .one(pools.read())
            .await?
        else {
            return Err(crate::MicrosandboxError::SandboxNotFound(sandbox.name));
        };
        if !matches!(
            sandbox.status,
            SandboxStatus::Starting | SandboxStatus::Running | SandboxStatus::Draining
        ) {
            return Ok(sandbox);
        }
        let run = Self::load_active_run(pools.read(), sandbox.id).await?;

        // Only an unowned start can be declared abandoned without a run row.
        // Older runtimes may expose an endpoint without holding lifecycle locks.
        let Some(run) = run else {
            let abandoned_start = sandbox.status == SandboxStatus::Starting && owns_transition;
            if abandoned_start
                && let Some((run_dir, sandboxes_dir)) = socket_roots
                && sandbox_runtime_endpoint_is_live(
                    run_dir,
                    &sandboxes_dir.join(&sandbox.name),
                    &sandbox.name,
                )?
            {
                return Ok(sandbox);
            }
            if sandbox.status == SandboxStatus::Draining || abandoned_start {
                if let Some((run_dir, sandboxes_dir)) = socket_roots {
                    crate::runtime::remove_sandbox_socket_artifacts_at(
                        run_dir,
                        sandboxes_dir,
                        &sandbox.name,
                    )?;
                }
                let (terminal_status, reason) = Self::stale_runtime_terminal_state(sandbox.status);
                Self::mark_sandbox_runtime_stale(
                    pools.write(),
                    sandbox.id,
                    None,
                    terminal_status,
                    reason,
                )
                .await?;

                return sandbox_entity::Entity::find_by_id(sandbox.id)
                    .one(pools.read())
                    .await?
                    .ok_or_else(|| crate::MicrosandboxError::SandboxNotFound(sandbox.name));
            }

            return Ok(sandbox);
        };

        if run.pid.is_some_and(Self::pid_is_alive) {
            return Ok(sandbox);
        }

        if let Some((run_dir, sandboxes_dir)) = socket_roots {
            crate::runtime::remove_sandbox_socket_artifacts_at(
                run_dir,
                sandboxes_dir,
                &sandbox.name,
            )?;
        }
        let (terminal_status, reason) = Self::stale_runtime_terminal_state(sandbox.status);
        Self::mark_sandbox_runtime_stale(
            pools.write(),
            sandbox.id,
            Some(run.id),
            terminal_status,
            reason,
        )
        .await?;

        sandbox_entity::Entity::find_by_id(sandbox.id)
            .one(pools.read())
            .await?
            .ok_or_else(|| crate::MicrosandboxError::SandboxNotFound(sandbox.name))
    }
}
