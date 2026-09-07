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
    // Do not send a new operation to an old runtime that cannot implement its semantics.
    let capabilities =
        modify::control_request_for(local, name, "{\"op\":\"capabilities\"}\n".into()).await?;
    if !capabilities
        .capabilities
        .is_some_and(|caps| caps.pause_resume)
    {
        return Err(MicrosandboxError::Runtime("resident pause/resume requires a runtime and guest kernel with clock-only resume support".into()));
    }
    let line = format!("{}\n", serde_json::to_string(&request)?);
    let response = modify::control_request_for(local, name, line).await?;
    response
        .pause
        .ok_or_else(|| MicrosandboxError::Runtime("control response omitted pause state".into()))
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
            // One observation plus the capability-gated public lifecycle exchange.
            for _ in 0..3 {
                let (stream, _) = listener.accept().await.unwrap();
                let mut stream = BufReader::new(stream);
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                let response = if line.contains("capabilities") {
                    "{\"ok\":true,\"capabilities\":{\"pause_resume\":true,\"cpu_resize\":false,\"memory_resize\":false,\"secrets_update\":false}}\n"
                } else {
                    "{\"ok\":true,\"pause\":{\"paused\":true,\"recovery_required\":false,\"capture_unavailable\":null}}\n"
                };
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
}
