//! Guest-side writeback boundary for external filesystem checkpoints.

use std::{
    ffi::CString,
    fs::File,
    io,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    sync::atomic::{AtomicBool, Ordering},
};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

static SYNC_ACTIVE: AtomicBool = AtomicBool::new(false);

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Pins an in-flight flush even after the async caller times out.
pub(crate) struct ExternalSyncPermit;

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Drop for ExternalSyncPermit {
    fn drop(&mut self) {
        SYNC_ACTIVE.store(false, Ordering::Release);
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Reserve before scheduling, so queued and timed-out blocking work both stay visible.
pub(crate) fn try_start_sync() -> Option<ExternalSyncPermit> {
    SYNC_ACTIVE
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .ok()
        .map(|_| ExternalSyncPermit)
}

/// Flush each virtiofs superblock while application tasks and external input are frozen.
///
/// `syncfs` reports writeback errors, unlike `sync`. This never unmounts a filesystem or
/// discards dirty pages. The caller must exclude independently running agent write streams.
pub(crate) fn sync_external_mounts(expected_tags: Vec<String>) -> io::Result<()> {
    if expected_tags.is_empty() {
        return Ok(());
    }
    let mut pending = expected_tags
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mounts = std::fs::read_to_string("/proc/self/mountinfo")?;
    for line in mounts.lines() {
        let Some((fields, filesystem)) = line.split_once(" - ") else {
            return Err(io::Error::other("malformed guest mount inventory"));
        };
        let mut filesystem = filesystem.split_whitespace();
        if filesystem.next() != Some("virtiofs") {
            continue;
        }
        let tag = filesystem
            .next()
            .ok_or_else(|| io::Error::other("missing virtiofs tag"))?;
        if !pending.contains(tag) {
            continue;
        }
        let mountpoint = fields
            .split_whitespace()
            .nth(4)
            .ok_or_else(|| io::Error::other("missing guest mountpoint"))?;
        let mountpoint = decode_mountpoint(mountpoint)?;
        let path =
            CString::new(mountpoint).map_err(|_| io::Error::other("NUL in guest mountpoint"))?;
        // O_PATH is not accepted by syncfs. Opening the mount root read-only does not
        // alter host data and also keeps the exact superblock pinned through the flush.
        let file = File::open(std::path::Path::new(std::ffi::OsStr::from_bytes(
            path.as_bytes(),
        )))?;
        if unsafe { libc::syncfs(file.as_raw_fd()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        pending.remove(tag);
    }
    if pending.is_empty() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "external mounts are absent from the agent mount namespace: {}",
            pending.into_iter().collect::<Vec<_>>().join(", ")
        )))
    }
}

fn decode_mountpoint(encoded: &str) -> io::Result<Vec<u8>> {
    let input = encoded.as_bytes();
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'\\' {
            let escape = input
                .get(index + 1..index + 4)
                .ok_or_else(|| io::Error::other("truncated mountpoint escape"))?;
            if !escape.iter().all(|byte| (b'0'..=b'7').contains(byte)) {
                return Err(io::Error::other("invalid mountpoint escape"));
            }
            let value = u16::from(escape[0] - b'0') * 64
                + u16::from(escape[1] - b'0') * 8
                + u16::from(escape[2] - b'0');
            output.push(
                u8::try_from(value).map_err(|_| io::Error::other("invalid mountpoint octet"))?,
            );
            index += 4;
        } else {
            output.push(input[index]);
            index += 1;
        }
    }
    Ok(output)
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{decode_mountpoint, try_start_sync};

    #[tokio::test]
    async fn timed_out_worker_keeps_later_capture_from_certifying_clean() {
        let permit = try_start_sync().unwrap();
        let (release, waiting) = std::sync::mpsc::channel();
        let worker = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            waiting.recv().unwrap();
        });
        // Timeout drops only the wait future, not the blocking worker's permit.
        let mut worker = worker;
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(1), &mut worker)
                .await
                .is_err()
        );
        assert!(try_start_sync().is_none());
        release.send(()).unwrap();
        worker.await.unwrap();
        assert!(try_start_sync().is_some());
    }

    #[test]
    fn mount_inventory_escapes_are_decoded_without_path_substitution() {
        assert_eq!(
            decode_mountpoint(r"/work\040dir\134name").unwrap(),
            b"/work dir\\name"
        );
        assert!(decode_mountpoint(r"/bad\0").is_err());
        assert!(decode_mountpoint(r"/bad\xyz").is_err());
    }
}
