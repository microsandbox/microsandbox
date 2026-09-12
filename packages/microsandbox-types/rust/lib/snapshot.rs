//! Snapshot maintenance results shared by SDK and runtime without linking a VM runner.

use serde::{Deserialize, Serialize};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Measured outcome or dry-run projection of an explicit root-disk compaction.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiskCompactionResult {
    /// Whether only selection was performed.
    pub dry_run: bool,
    /// Physical layers before compaction, including the writable head.
    pub input_layers: usize,
    /// Selected oldest layers, including the base, excluding the writable head.
    pub selected_layers: usize,
    /// Physical layers after compaction, including the writable head.
    pub output_layers: usize,
    /// Guest bytes materialized; not a disk-space saving estimate.
    pub materialized_bytes: u64,
    /// Total operation duration in microseconds.
    pub total_us: u64,
    /// Measured VM pause through resume, zero for stopped sources and dry runs.
    pub pause_us: u64,
}
/// How full restore handles unavailable external bind mounts and stale captured objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalMountRestorePolicy {
    /// Refuse activation if a captured external resource cannot be reconstructed.
    #[default]
    Strict,
    /// Keep the mount present and return filesystem errors for unavailable resources.
    Relaxed,
}

/// A degraded external resource retained by an explicitly relaxed full restore.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalMountWarning {
    /// Guest-visible mount path.
    pub guest_path: String,
    /// Actionable reason the resource could not be reconstructed.
    pub reason: String,
    /// Permanently invalid captured node IDs; empty when the whole export is unavailable.
    pub stale_inodes: Vec<u64>,
}
