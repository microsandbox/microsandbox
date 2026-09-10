//! The typed launch contract between the SDK and the `msb sandbox` process.
//!
//! [`LaunchConfig`] is the bulk of a sandbox's configuration. The SDK builds
//! it, serializes it as JSON, and hands it to `msb sandbox` over an inherited
//! file descriptor (see [`CONFIG_FD`]); the process deserializes it
//! and builds its [`crate::vm::Config`] from it. Only a few operator-readable
//! labels and the real inherited fds stay on the process argv. This keeps the
//! network config and secret-bearing env out of `ps` and `/proc/<pid>/cmdline`
//! — see issue #997.

use std::path::PathBuf;

use microsandbox_protocol::bootstrap::GuestBootstrap;
use microsandbox_types::{CpuPlacement, PlacementProfile, VsockRouteSpec};
use serde::{Deserialize, Serialize};

use microsandbox_types::TransparentHugePagePolicy;

#[cfg(feature = "net")]
use microsandbox_network::ResolvedNetworkConfig;
#[cfg(feature = "net")]
use microsandbox_types::DeploymentProfile;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Fixed fd carrying the bulk `msb sandbox` config as NUL-terminated argument records.
pub const CONFIG_FD: i32 = 96;

/// Fixed fd used to pass the attached-parent watchdog pipe into `msb sandbox`.
pub const PARENT_WATCH_FD: i32 = 97;

/// Fixed fd used to pass startup JSON from `msb sandbox` to its launcher.
pub const STARTUP_FD: i32 = 98;

/// Fixed fd holding the inherited per-sandbox lifecycle ownership lock.
#[cfg(unix)]
pub const LIFECYCLE_LOCK_FD: i32 = 99;

/// Control byte sent by the owner to stop parent-watch monitoring without stopping the sandbox.
pub const PARENT_WATCH_DETACH: u8 = 1;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Hidden CLI handoff describing the metrics slot the host reserved for this sandbox.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsSlotHandoff {
    /// Name of the POSIX shared-memory object holding the registry.
    pub shm_name: String,
    /// Reserved slot index.
    pub slot: u32,
    /// Generation paired with the reservation.
    pub generation: u64,
}

/// User workload that the sandbox process should start after boot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartupCommand {
    /// Path or command name to execute inside the guest.
    pub cmd: String,

    /// Arguments to pass to the command.
    pub args: Vec<String>,

    /// Environment variables as `KEY=VALUE` strings.
    pub env: Vec<String>,

    /// Working directory for the command.
    pub cwd: Option<String>,

    /// Guest user override for the command.
    pub user: Option<String>,
}

/// The bulk `msb sandbox` configuration delivered over the config fd.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Path to the sandbox database file.
    pub db_path: PathBuf,

    /// Timeout when acquiring a sandbox database connection from the pool.
    pub db_connect_timeout_secs: u64,

    /// Directory for log files.
    pub log_dir: PathBuf,

    /// Runtime directory (scripts, heartbeat).
    pub runtime_dir: PathBuf,

    /// Root directory holding every sandbox's persisted state.
    pub sandboxes_dir: PathBuf,

    /// Root directory holding ephemeral host-runtime artifacts.
    #[serde(default)]
    pub run_dir: PathBuf,

    /// Internal directory containing process-held CPU allocation leases.
    pub cpu_lease_dir: PathBuf,

    /// Internal directory containing process-held writeback pressure leases.
    pub writeback_lease_dir: PathBuf,

    /// Requested host CPU placement policy.
    pub cpu_placement: CpuPlacement,

    /// Host-defined profile name retained for diagnostics and missing-profile validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_profile_name: Option<String>,

    /// Host-resolved profile definition; sandbox clients submit only the name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_profile: Option<PlacementProfile>,

    /// Path to the Unix domain socket for the agent relay.
    pub agent_sock: PathBuf,

    /// Path to the libkrunfw shared library.
    pub libkrunfw_path: PathBuf,

    /// Guest transparent huge-page policy selected at boot.
    #[serde(default)]
    pub thp: TransparentHugePagePolicy,

    /// Per-writable-raw-disk hard budget for buffered host dirty data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_writeback_limit_bytes: Option<u64>,

    /// Host-global dirty-credit pool shared fairly by live writable disks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_writeback_pool_bytes: Option<u64>,

    /// User workload to start after boot, if any.
    pub startup: Option<StartupCommand>,

    /// Lifetime bounds for the sandbox.
    pub lifecycle: Lifecycle,

    /// Metrics sampling configuration and the host-reserved slot.
    pub metrics: MetricsConfig,

    /// Root filesystem source.
    pub rootfs: RootfsConfig,

    /// Additional virtio-fs mounts as `tag:host_path[:opts]`.
    pub mounts: Vec<String>,

    /// Isolated host-file mounts handled by the single-file backend.
    #[serde(default)]
    pub file_mounts: Vec<FileMountConfig>,

    /// Disk-image volume mounts as `id:host_path:format[:ro]`.
    pub disks: Vec<String>,

    /// Path to the init binary in the guest.
    pub init_path: Option<PathBuf>,

    /// Typed one-shot configuration delivered to agentd over its console.
    pub bootstrap: GuestBootstrap,

    /// Path to the executable to run in the guest.
    pub exec_path: Option<PathBuf>,

    /// Arguments to pass to the executable.
    pub exec_args: Vec<String>,

    /// Network launch configuration. Present only when the `net` feature is on.
    #[cfg(feature = "net")]
    pub network: Option<ResolvedNetworkConfig>,

    /// Host-runtime isolation profile enforced by backend implementations.
    #[cfg(feature = "net")]
    #[serde(default)]
    pub deployment_profile: DeploymentProfile,

    /// Sandbox slot for deterministic network address derivation.
    #[cfg(feature = "net")]
    pub sandbox_slot: u16,

    /// Host Unix sockets exposed through virtio-vsock.
    #[serde(default)]
    pub vsock: Vec<VsockRouteSpec>,
}

/// Lifetime bounds for the sandbox.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    /// Hard cap on total sandbox lifetime in seconds.
    pub max_duration_secs: Option<u64>,

    /// Idle timeout in seconds.
    pub idle_timeout_secs: Option<u64>,
}

/// Metrics sampling configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Sampling interval in milliseconds.
    pub sample_interval_ms: u64,

    /// Disable sampling; overrides `sample_interval_ms`.
    pub disabled: bool,

    /// Host-reserved shared-memory slot, if metrics are enabled.
    pub slot: Option<MetricsSlotHandoff>,
}

/// Root filesystem source for the sandbox.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RootfsConfig {
    /// Root filesystem path for direct passthrough mounts.
    pub path: Option<PathBuf>,

    /// Follow symlinks when resolving a bind (`path`) rootfs.
    ///
    /// Defaults to `false` (resolve following no symlink), matching the
    /// `--mount` protection for the caller/tenant-provided rootfs path.
    #[serde(default)]
    pub follow_root_symlinks: bool,

    /// Disk image file path for virtio-blk rootfs.
    pub disk: Option<PathBuf>,

    /// Disk image format (qcow2, raw, vmdk).
    pub disk_format: Option<String>,

    /// Mount the disk image as read-only.
    pub disk_readonly: bool,

    /// Writable upper block device for OCI rootfs overlay.
    pub upper: Option<PathBuf>,

    /// Upper disk image format ("raw", "qcow2"). Absent means raw — the
    /// managed `upper.ext4` fast path. Set for user-supplied disk-image
    /// root disks so the runner attaches with the right format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upper_format: Option<String>,
}

/// Host-side configuration for one isolated file mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMountConfig {
    /// `tag:host_path[:opts]` specification parsed by the runtime.
    pub mount: String,

    /// Filename presented at the root of the synthetic virtio-fs share.
    pub filename: String,
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{FileMountConfig, LaunchConfig};

    #[test]
    fn isolated_file_mount_survives_the_client_runner_handoff() {
        let config = LaunchConfig {
            file_mounts: vec![FileMountConfig {
                mount: "config:/host/secret.txt:ro,uid=1000,gid=1000".into(),
                filename: "secret.txt".into(),
            }],
            ..Default::default()
        };
        let encoded = serde_json::to_value(&config).unwrap();
        let decoded: LaunchConfig = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded.file_mounts[0].mount, config.file_mounts[0].mount);
        assert_eq!(decoded.file_mounts[0].filename, "secret.txt");
        assert!(decoded.mounts.is_empty());

        // An omitted additive field must not turn an ordinary directory into a file mount.
        let mut without_files = encoded;
        without_files.as_object_mut().unwrap().remove("file_mounts");
        let decoded: LaunchConfig = serde_json::from_value(without_files).unwrap();
        assert!(decoded.file_mounts.is_empty());
    }

    #[cfg(feature = "net")]
    #[test]
    fn network_slot_handoff_rejects_out_of_range_values() {
        let config = LaunchConfig {
            sandbox_slot: u16::MAX,
            ..Default::default()
        };
        let mut encoded = serde_json::to_value(&config).unwrap();
        let decoded: LaunchConfig = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded.sandbox_slot, u16::MAX);
        encoded["sandbox_slot"] = serde_json::json!(u32::from(u16::MAX) + 1);
        assert!(serde_json::from_value::<LaunchConfig>(encoded).is_err());
    }
}
