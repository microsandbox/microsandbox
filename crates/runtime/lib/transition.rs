//! Ownership of the launcher-to-runtime transition for a sandbox name.

use std::fs::File;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Short-lived ownership while a launcher prepares or starts a sandbox.
/// Closing the file releases ownership, including when the launcher dies.
pub struct SandboxTransitionGuard {
    _file: File,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Try to acquire launcher ownership without waiting for an ongoing start.
pub fn try_acquire_transition_guard(
    run_dir: &Path,
    name: &str,
) -> std::io::Result<Option<SandboxTransitionGuard>> {
    let path = sandbox_transition_lock_path(run_dir, name);
    std::fs::create_dir_all(path.parent().expect("transition lock has a parent"))?;
    let file = microsandbox_utils::process_lock::open_lock_file(&path)?;
    if microsandbox_utils::process_lock::try_lock_exclusive(&file)? {
        Ok(Some(SandboxTransitionGuard { _file: file }))
    } else {
        Ok(None)
    }
}

/// Derive a stable, filesystem-safe transition-lock path for one sandbox name.
pub fn sandbox_transition_lock_path(run_dir: &Path, name: &str) -> PathBuf {
    let digest = Sha256::digest(name.as_bytes());
    let hash: String = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    // Keep the original on-disk namespace so mixed-version processes still contend on one lock.
    run_dir.join("creation-locks").join(format!("{hash}.lock"))
}

/// Probe every backward-compatible Unix endpoint before recovering an
/// untracked namespace. A successful connection is direct evidence that an
/// older runtime (which predates lifecycle locks) still owns the name.
#[cfg(unix)]
pub fn sandbox_runtime_endpoint_is_live(
    run_dir: &Path,
    sandbox_dir: &Path,
    name: &str,
) -> std::io::Result<bool> {
    let paths = crate::ipc::sandbox_socket_paths(run_dir, name);
    let fallback_agent = sandbox_dir.join("runtime").join("agent.sock");
    let fallback_control = crate::ipc::control_socket_path_for(&fallback_agent);
    for path in [
        paths.agent,
        paths.control,
        paths.legacy_agent,
        paths.legacy_control,
        fallback_agent,
        fallback_control,
    ] {
        if std::fs::symlink_metadata(&path).is_err() {
            continue;
        }
        match std::os::unix::net::UnixStream::connect(&path) {
            Ok(_) => return Ok(true),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
                ) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}

/// Non-Unix runtimes use the lifecycle ownership guard.
#[cfg(not(unix))]
pub fn sandbox_runtime_endpoint_is_live(
    _run_dir: &Path,
    _sandbox_dir: &Path,
    _name: &str,
) -> std::io::Result<bool> {
    Ok(false)
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_guard_contends_with_existing_launcher_lock() {
        let temp = tempfile::tempdir().unwrap();
        // Exact path produced by the previously shipped SDK, independently of the new helper.
        let path = temp
            .path()
            .join("creation-locks/0e0e827720dff0e9fb6cc08970d370d2.lock");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let old_launcher = microsandbox_utils::process_lock::open_lock_file(&path).unwrap();
        assert!(microsandbox_utils::process_lock::try_lock_exclusive(&old_launcher).unwrap());
        assert_eq!(
            sandbox_transition_lock_path(temp.path(), "before-pid"),
            path
        );
        assert!(
            try_acquire_transition_guard(temp.path(), "before-pid")
                .unwrap()
                .is_none()
        );
        drop(old_launcher);
        assert!(
            try_acquire_transition_guard(temp.path(), "before-pid")
                .unwrap()
                .is_some()
        );
    }
}
