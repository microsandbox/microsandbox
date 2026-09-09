//! Resident pause/resume through the existing host control endpoint.

use microsandbox_runtime::control::ControlRequest;

use crate::backend::{Backend, LocalBackend};
use crate::error::Operation;
use crate::{MicrosandboxError, MicrosandboxResult};

use super::{Sandbox, SandboxHandle, SandboxPauseState, modify};

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl Sandbox {
    /// Internal CLI lookup for an immediately following authoritative control mutation.
    ///
    /// Keep database/runtime reconciliation, but skip the pause observation used by ordinary
    /// `get`/`list`: that observation is already stale by the time the mutation executes.
    #[doc(hidden)]
    pub async fn get_for_control(name: &str) -> MicrosandboxResult<SandboxHandle> {
        let backend = crate::backend::default_backend();
        if let Some(local) = backend.as_local() {
            let (model, pid) = local.sandbox_handle_state(name).await?;
            return Ok(SandboxHandle::from_local_model(backend, model, pid));
        }
        backend.sandboxes().get(backend.clone(), name).await
    }

    /// Suspend this resident VM without creating a snapshot or releasing RAM.
    pub async fn pause(&self) -> MicrosandboxResult<()> {
        lifecycle(self.name(), self.backend().as_ref(), ControlRequest::Pause)
            .await
            .map(|_| ())
    }

    /// Resume the same VM and processes, correcting wall clock before thawing workloads.
    pub async fn resume(&self) -> MicrosandboxResult<()> {
        lifecycle(self.name(), self.backend().as_ref(), ControlRequest::Resume)
            .await
            .map(|_| ())
    }

    /// Inspect the host-confirmed pause state without contacting the suspended guest.
    pub async fn pause_state(&self) -> MicrosandboxResult<SandboxPauseState> {
        lifecycle(
            self.name(),
            self.backend().as_ref(),
            ControlRequest::PauseState,
        )
        .await
    }
}

impl SandboxHandle {
    /// Suspend an existing resident sandbox without connecting to its guest.
    pub async fn pause(&self) -> MicrosandboxResult<()> {
        lifecycle(self.name(), self.backend.as_ref(), ControlRequest::Pause)
            .await
            .map(|_| ())
    }

    /// Resume an existing user-paused sandbox through host control.
    pub async fn resume(&self) -> MicrosandboxResult<()> {
        lifecycle(self.name(), self.backend.as_ref(), ControlRequest::Resume)
            .await
            .map(|_| ())
    }

    /// Inspect resident suspension without opening an agent connection.
    pub async fn pause_state(&self) -> MicrosandboxResult<SandboxPauseState> {
        lifecycle(
            self.name(),
            self.backend.as_ref(),
            ControlRequest::PauseState,
        )
        .await
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Overlay resident suspension on database lifecycle without persisting a stale pause on crash.
pub(crate) async fn projected_status(
    local: &LocalBackend,
    name: &str,
    status: super::SandboxStatus,
) -> super::SandboxStatus {
    if status != super::SandboxStatus::Running {
        return status;
    }
    // Old runtimes have no pause endpoint. Bound observation so a busy or unavailable host
    // never makes ordinary list/get wait for an entire checkpoint operation.
    let request = modify::control_request_for(local, name, "{\"op\":\"pause_state\"}\n".into());
    match tokio::time::timeout(std::time::Duration::from_millis(250), request).await {
        Ok(Ok(response))
            if response
                .pause
                .as_ref()
                .is_some_and(|state| state.paused || state.recovery_required) =>
        {
            super::SandboxStatus::Paused
        }
        _ => status,
    }
}

async fn lifecycle(
    name: &str,
    backend: &dyn Backend,
    request: ControlRequest,
) -> MicrosandboxResult<SandboxPauseState> {
    let operation = if matches!(request, ControlRequest::Resume) {
        Operation::SandboxResume
    } else {
        Operation::SandboxPause
    };
    let local = backend
        .as_local()
        .ok_or_else(|| MicrosandboxError::local_only(operation))?;
    // The mutation itself is authoritative. Unknown operations fail on older runtimes, and
    // successful replies must carry pause state; neither case can silently become a no-op.
    let line = format!("{}\n", serde_json::to_string(&request)?);
    let response = modify::control_request_for(local, name, line).await?;
    let state = response
        .pause
        .ok_or_else(|| MicrosandboxError::Runtime("control response omitted pause state".into()))?;
    // An acknowledgement must confirm the requested transition, not just contain some
    // observation. State inspection itself must still be able to report recovery required.
    let expected = match request {
        ControlRequest::Pause => Some(true),
        ControlRequest::Resume => Some(false),
        _ => None,
    };
    if expected.is_some_and(|paused| state.paused != paused || state.recovery_required) {
        return Err(MicrosandboxError::Runtime(
            "control response did not confirm the requested pause transition".into(),
        ));
    }
    Ok(state)
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(all(test, unix))]
mod tests {
    use std::sync::Arc;

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    use super::*;
    use crate::backend::with_backend;

    #[tokio::test]
    async fn pause_observation_uses_bound_backend_outside_its_ambient_scope() {
        // macOS's per-user TMPDIR may already consume most of the Unix socket path limit.
        let ambient_home = tempfile::tempdir_in("/tmp").unwrap();
        let bound_home = tempfile::tempdir_in("/tmp").unwrap();
        let ambient: Arc<dyn Backend> = Arc::new(
            LocalBackend::builder()
                .home(ambient_home.path())
                .build()
                .await
                .unwrap(),
        );
        let bound = LocalBackend::builder()
            .home(bound_home.path())
            .build()
            .await
            .unwrap();
        let agent =
            crate::runtime::sandbox_agent_socket_path_candidates_for(&bound, "same-name").remove(0);
        let path = microsandbox_runtime::control::control_socket_path_for(&agent);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let listener = tokio::net::UnixListener::bind(path).unwrap();
        let server = tokio::spawn(async move {
            // Each observation is one exchange; it needs no capabilities preflight.
            for _ in 0..2 {
                let (stream, _) = listener.accept().await.unwrap();
                let mut stream = BufReader::new(stream);
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                assert_eq!(line, "{\"op\":\"pause_state\"}\n");
                let response = "{\"ok\":true,\"pause\":{\"paused\":true,\"recovery_required\":false,\"capture_unavailable\":null}}\n";
                stream
                    .get_mut()
                    .write_all(response.as_bytes())
                    .await
                    .unwrap();
            }
        });
        with_backend(ambient, async {
            assert_eq!(
                projected_status(&bound, "same-name", super::super::SandboxStatus::Running).await,
                super::super::SandboxStatus::Paused
            );
            assert!(
                lifecycle("same-name", &bound, ControlRequest::PauseState)
                    .await
                    .unwrap()
                    .paused
            );
        })
        .await;
        server.await.unwrap();
    }

    #[tokio::test]
    async fn lifecycle_sends_one_mutation_and_requires_an_authoritative_reply() {
        for (operation, response, accepted) in [
            (
                "pause",
                "{\"ok\":true,\"pause\":{\"paused\":true,\"recovery_required\":false}}\n",
                true,
            ),
            (
                "resume",
                "{\"ok\":true,\"pause\":{\"paused\":false,\"recovery_required\":false}}\n",
                true,
            ),
            // Old runtime unknown-operation errors and unsupported current kernels must fail.
            (
                "pause",
                "{\"ok\":false,\"error\":\"unknown variant pause\"}\n",
                false,
            ),
            (
                "resume",
                "{\"ok\":false,\"error\":\"pause/resume unavailable\"}\n",
                false,
            ),
            ("resume", "{\"ok\":true}\n", false),
            (
                "pause",
                "{\"ok\":true,\"pause\":{\"paused\":false,\"recovery_required\":false}}\n",
                false,
            ),
            (
                "resume",
                "{\"ok\":true,\"pause\":{\"paused\":true,\"recovery_required\":false}}\n",
                false,
            ),
            (
                "pause",
                "{\"ok\":true,\"pause\":{\"paused\":true,\"recovery_required\":true}}\n",
                false,
            ),
        ] {
            let home = tempfile::tempdir_in("/tmp").unwrap();
            let backend = LocalBackend::builder()
                .home(home.path())
                .build()
                .await
                .unwrap();
            let agent =
                crate::runtime::sandbox_agent_socket_path_candidates_for(&backend, "source")
                    .remove(0);
            let path = microsandbox_runtime::control::control_socket_path_for(&agent);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let listener = tokio::net::UnixListener::bind(path).unwrap();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut stream = BufReader::new(stream);
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                assert_eq!(line, format!("{{\"op\":\"{operation}\"}}\n"));
                stream
                    .get_mut()
                    .write_all(response.as_bytes())
                    .await
                    .unwrap();
            });
            let request = if operation == "pause" {
                ControlRequest::Pause
            } else {
                ControlRequest::Resume
            };
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                lifecycle("source", &backend, request),
            )
            .await
            .unwrap();
            assert_eq!(result.is_ok(), accepted, "{operation}: {response}");
            server.await.unwrap();
        }
    }
}
