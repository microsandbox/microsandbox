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
