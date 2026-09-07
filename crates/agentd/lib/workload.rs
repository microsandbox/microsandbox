//! Checkpoint-time execution latch for agentd-managed workloads.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

const CGROUP_ROOT: &str = "/sys/fs/cgroup/microsandbox-workload";
const FREEZE_TIMEOUT: Duration = Duration::from_secs(5);
const FREEZE_POLL_INTERVAL: Duration = Duration::from_millis(1);
const MAX_ATTEMPT_ID_BYTES: usize = 128;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Guest-side latch for processes launched through agentd.
///
/// Failure to initialize the freezer does not affect ordinary execution. It
/// only makes full checkpoint preparation unavailable.
pub(crate) struct WorkloadLatch {
    freezer: Option<Box<dyn FreezerControl>>,
    unavailable_reason: Option<String>,
    state: LatchState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LatchState {
    Running { last_thawed: Option<String> },
    Frozen { attempt_id: String },
    RecoveryRequired { attempt_id: String },
}

trait FreezerControl: Send {
    fn placement(&self) -> io::Result<WorkloadPlacement>;
    fn set_frozen(&self, frozen: bool) -> io::Result<()>;
}

struct CgroupFreezer {
    root: PathBuf,
    cgroup_procs: File,
}

/// A child-owned cgroup placement handle prepared before `fork`.
///
/// `place_current` uses only `write(2)`, so it is safe in the restricted
/// fork-to-exec window used by both agentd spawn paths.
pub(crate) struct WorkloadPlacement {
    cgroup_procs: OwnedFd,
}

/// A workload-latch operation failure.
#[derive(Debug, thiserror::Error)]
pub(crate) enum WorkloadLatchError {
    /// The guest kernel or cgroup mount cannot provide the freezer.
    #[error("workload freezer is unavailable: {0}")]
    Unavailable(String),

    /// The attempt identity is unsafe or malformed.
    #[error("invalid checkpoint attempt identity: {0}")]
    InvalidAttempt(String),

    /// Another checkpoint attempt owns the current latch state.
    #[error("workload latch conflict: {0}")]
    Conflict(String),

    /// A cgroup operation failed.
    #[error("workload freezer operation failed: {0}")]
    Io(#[from] io::Error),
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl WorkloadLatch {
    /// Initialize the workload cgroup, preserving ordinary sandbox operation
    /// when the host kernel does not expose a usable cgroup-v2 freezer.
    pub(crate) fn initialize() -> Self {
        match CgroupFreezer::open(Path::new(CGROUP_ROOT)) {
            Ok(freezer) => Self::with_freezer(Box::new(freezer)),
            Err(error) => Self {
                freezer: None,
                unavailable_reason: Some(error.to_string()),
                state: LatchState::Running { last_thawed: None },
            },
        }
    }

    /// Construct a latch whose checkpoint capability is intentionally disabled.
    pub(crate) fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            freezer: None,
            unavailable_reason: Some(reason.into()),
            state: LatchState::Running { last_thawed: None },
        }
    }

    /// Returns why full-checkpoint workload freezing is unavailable, if applicable.
    pub(crate) fn unavailable_reason(&self) -> Option<&str> {
        self.unavailable_reason.as_deref()
    }

    /// Prepare a cgroup handle for placing a newly forked workload process.
    ///
    /// `None` deliberately means the freezer was unavailable at boot; normal
    /// exec remains usable in that case.
    pub(crate) fn placement(&self) -> Result<Option<WorkloadPlacement>, WorkloadLatchError> {
        self.freezer
            .as_ref()
            .map(|freezer| freezer.placement().map_err(WorkloadLatchError::Io))
            .transpose()
    }

    /// Whether an attempt blocks new work, including an uncertain freezer transition.
    pub(crate) fn is_frozen(&self) -> bool {
        !matches!(self.state, LatchState::Running { .. })
    }

    /// Freeze every process in the agentd-managed workload cgroup.
    pub(crate) fn freeze(&mut self, attempt_id: &str) -> Result<(), WorkloadLatchError> {
        validate_attempt_id(attempt_id)?;
        match &self.state {
            LatchState::Frozen {
                attempt_id: current,
            } if current == attempt_id => return Ok(()),
            LatchState::Frozen {
                attempt_id: current,
            }
            | LatchState::RecoveryRequired {
                attempt_id: current,
            } => {
                return Err(WorkloadLatchError::Conflict(format!(
                    "attempt {current:?} owns the latch; thaw it before another freeze"
                )));
            }
            LatchState::Running { .. } => {}
        }

        self.freezer()?;
        // Record ownership before writing: an error may follow a successful cgroup write.
        // Only a confirmed thaw can release an uncertain transition.
        self.state = LatchState::RecoveryRequired {
            attempt_id: attempt_id.to_string(),
        };
        self.freezer()?.set_frozen(true)?;
        // Agentd itself remains outside the workload cgroup, so it can flush every mounted
        // filesystem after user processes stop mutating them and before the host pauses the VM.
        // `sync(2)` has no error return; completion is the durability boundary exposed by Linux.
        unsafe { libc::sync() };
        self.state = LatchState::Frozen {
            attempt_id: attempt_id.to_string(),
        };
        Ok(())
    }

    /// Release the freeze owned by `attempt_id`.
    pub(crate) fn thaw(&mut self, attempt_id: &str) -> Result<(), WorkloadLatchError> {
        validate_attempt_id(attempt_id)?;
        match &self.state {
            LatchState::Running {
                last_thawed: Some(previous),
            } if previous == attempt_id => return Ok(()),
            LatchState::Running { .. } => {
                return Err(WorkloadLatchError::Conflict(
                    "no matching workload freeze is active".into(),
                ));
            }
            LatchState::Frozen {
                attempt_id: current,
            }
            | LatchState::RecoveryRequired {
                attempt_id: current,
            } if current != attempt_id => {
                return Err(WorkloadLatchError::Conflict(format!(
                    "attempt {current:?} owns the freeze"
                )));
            }
            LatchState::Frozen { .. } | LatchState::RecoveryRequired { .. } => {}
        }

        // A failed thaw must not leave a state that freeze retries can acknowledge as frozen.
        self.state = LatchState::RecoveryRequired {
            attempt_id: attempt_id.to_string(),
        };
        self.freezer()?.set_frozen(false)?;
        self.state = LatchState::Running {
            last_thawed: Some(attempt_id.to_string()),
        };
        Ok(())
    }

    fn with_freezer(freezer: Box<dyn FreezerControl>) -> Self {
        Self {
            freezer: Some(freezer),
            unavailable_reason: None,
            state: LatchState::Running { last_thawed: None },
        }
    }

    fn freezer(&self) -> Result<&dyn FreezerControl, WorkloadLatchError> {
        self.freezer.as_deref().ok_or_else(|| {
            WorkloadLatchError::Unavailable(
                self.unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "unknown initialization failure".into()),
            )
        })
    }
}

impl CgroupFreezer {
    fn open(root: &Path) -> io::Result<Self> {
        std::fs::create_dir_all(root)?;
        let freeze = root.join("cgroup.freeze");
        let events = root.join("cgroup.events");
        if !freeze.is_file() || !events.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "cgroup v2 freezer files are absent",
            ));
        }
        let cgroup_procs = OpenOptions::new()
            .write(true)
            .open(root.join("cgroup.procs"))?;
        Ok(Self {
            root: root.to_path_buf(),
            cgroup_procs,
        })
    }

    fn wait_for_state(&self, expected: bool) -> io::Result<()> {
        let deadline = Instant::now() + FREEZE_TIMEOUT;
        loop {
            let events = std::fs::read_to_string(self.root.join("cgroup.events"))?;
            if parse_frozen_event(&events) == Some(expected) {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!(
                        "cgroup did not report frozen={} within {FREEZE_TIMEOUT:?}",
                        expected as u8
                    ),
                ));
            }
            std::thread::sleep(FREEZE_POLL_INTERVAL);
        }
    }
}

impl WorkloadPlacement {
    /// Move the calling child into the workload cgroup before it executes user code.
    pub(crate) fn place_current(&self) -> io::Result<()> {
        let bytes = b"0";
        loop {
            let written = unsafe {
                libc::write(
                    self.cgroup_procs.as_raw_fd(),
                    bytes.as_ptr().cast(),
                    bytes.len(),
                )
            };
            if written == bytes.len() as isize {
                return Ok(());
            }
            if written < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "short write to cgroup.procs",
            ));
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl FreezerControl for CgroupFreezer {
    fn placement(&self) -> io::Result<WorkloadPlacement> {
        Ok(WorkloadPlacement {
            cgroup_procs: self.cgroup_procs.try_clone()?.into(),
        })
    }

    fn set_frozen(&self, frozen: bool) -> io::Result<()> {
        std::fs::write(
            self.root.join("cgroup.freeze"),
            if frozen { b"1" } else { b"0" },
        )?;
        self.wait_for_state(frozen)
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

fn validate_attempt_id(attempt_id: &str) -> Result<(), WorkloadLatchError> {
    if attempt_id.is_empty()
        || attempt_id.len() > MAX_ATTEMPT_ID_BYTES
        || !attempt_id.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(WorkloadLatchError::InvalidAttempt(
            "must be 1-128 printable ASCII bytes without spaces".into(),
        ));
    }
    Ok(())
}

fn parse_frozen_event(events: &str) -> Option<bool> {
    events.lines().find_map(|line| {
        let (key, value) = line.split_once(' ')?;
        if key != "frozen" {
            return None;
        }
        match value {
            "0" => Some(false),
            "1" => Some(true),
            _ => None,
        }
    })
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use super::*;

    struct FakeFreezer {
        states: Arc<Mutex<Vec<bool>>>,
    }

    struct FailingFreezer {
        outcomes: Mutex<VecDeque<bool>>,
    }

    impl FreezerControl for FailingFreezer {
        fn placement(&self) -> io::Result<WorkloadPlacement> {
            unreachable!()
        }

        fn set_frozen(&self, _frozen: bool) -> io::Result<()> {
            // Model either a failed write or a write that succeeded before acknowledgement failed.
            if self.outcomes.lock().unwrap().pop_front().unwrap() {
                Ok(())
            } else {
                Err(io::Error::other("injected freezer transition failure"))
            }
        }
    }

    #[test]
    fn failed_freeze_retains_ownership_until_confirmed_thaw() {
        let mut latch = WorkloadLatch::with_freezer(Box::new(FailingFreezer {
            outcomes: Mutex::new(VecDeque::from([false, false, true])),
        }));
        assert!(latch.freeze("a").is_err());
        assert!(latch.is_frozen());
        assert!(latch.freeze("a").is_err());
        assert!(latch.freeze("b").is_err());
        assert!(latch.thaw("b").is_err());
        assert!(latch.thaw("a").is_err());
        assert!(latch.is_frozen());
        latch.thaw("a").unwrap();
        assert!(!latch.is_frozen());
        latch.thaw("a").unwrap();
    }

    #[test]
    fn failed_thaw_never_acknowledges_a_freeze_retry() {
        let mut latch = WorkloadLatch::with_freezer(Box::new(FailingFreezer {
            outcomes: Mutex::new(VecDeque::from([true, false, true])),
        }));
        latch.freeze("a").unwrap();
        assert!(latch.thaw("a").is_err());
        assert!(latch.is_frozen());
        assert!(latch.freeze("a").is_err());
        latch.thaw("a").unwrap();
        assert!(!latch.is_frozen());
    }

    #[test]
    fn known_unavailable_never_takes_ownership() {
        let mut latch = WorkloadLatch::unavailable("no cgroup freezer");
        for attempt in ["a", "b"] {
            assert!(matches!(
                latch.freeze(attempt),
                Err(WorkloadLatchError::Unavailable(_))
            ));
            assert!(!latch.is_frozen());
        }
    }

    impl FreezerControl for FakeFreezer {
        fn placement(&self) -> io::Result<WorkloadPlacement> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "not needed"))
        }

        fn set_frozen(&self, frozen: bool) -> io::Result<()> {
            self.states.lock().unwrap().push(frozen);
            Ok(())
        }
    }

    #[test]
    fn attempt_retries_are_idempotent_and_cross_attempt_release_is_rejected() {
        let states = Arc::new(Mutex::new(Vec::new()));
        let mut latch = WorkloadLatch::with_freezer(Box::new(FakeFreezer {
            states: Arc::clone(&states),
        }));

        latch.freeze("checkpoint-42").unwrap();
        latch.freeze("checkpoint-42").unwrap();
        assert!(latch.thaw("checkpoint-99").is_err());
        latch.thaw("checkpoint-42").unwrap();
        latch.thaw("checkpoint-42").unwrap();

        assert_eq!(*states.lock().unwrap(), vec![true, false]);
    }

    #[test]
    fn unavailable_freezer_does_not_disable_process_placement() {
        let latch = WorkloadLatch {
            freezer: None,
            unavailable_reason: Some("missing cgroup2".into()),
            state: LatchState::Running { last_thawed: None },
        };

        assert!(latch.placement().unwrap().is_none());
        assert_eq!(latch.unavailable_reason(), Some("missing cgroup2"));
    }

    #[test]
    fn parses_cgroup_v2_frozen_event() {
        assert_eq!(parse_frozen_event("populated 1\nfrozen 1\n"), Some(true));
        assert_eq!(parse_frozen_event("populated 0\nfrozen 0\n"), Some(false));
        assert_eq!(parse_frozen_event("populated 0\n"), None);
    }

    #[test]
    fn validates_bounded_printable_attempt_ids() {
        assert!(validate_attempt_id("checkpoint_42").is_ok());
        assert!(validate_attempt_id("").is_err());
        assert!(validate_attempt_id("contains space").is_err());
        assert!(validate_attempt_id(&"x".repeat(129)).is_err());
    }
}
