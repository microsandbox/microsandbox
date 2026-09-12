//! Backing allocation and admission for sandbox-owned, unnamed volumes.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use microsandbox_image::ext4::{Ext4FormatOptions, format_ext4};
use microsandbox_types::{OwnedVolumeStorage, VolumeMount};

use crate::{MicrosandboxError, MicrosandboxResult};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Resolve a canonical mount identity without accepting a serialized host path.
pub(crate) fn backing_path(
    sandbox_dir: &Path,
    guest: &str,
    storage: &OwnedVolumeStorage,
) -> PathBuf {
    sandbox_dir
        .join("owned-volumes")
        .join(microsandbox_types::owned_volume_mount_id(guest))
        .join(match storage {
            OwnedVolumeStorage::Directory { .. } => "data",
            OwnedVolumeStorage::Disk { .. } => "disk.raw",
        })
}

/// Allocate only on initial creation. Restarts and restores must find their existing backing.
pub(crate) async fn prepare(
    sandbox_dir: &Path,
    mounts: &[VolumeMount],
    restored: bool,
) -> MicrosandboxResult<()> {
    let owned: Vec<_> = mounts
        .iter()
        .filter(|mount| matches!(mount, VolumeMount::Owned { .. }))
        .cloned()
        .collect();
    if owned.is_empty() {
        return Ok(());
    }
    validate_tags(mounts)?;
    if restored {
        return validate(sandbox_dir, mounts);
    }
    let destination = sandbox_dir.join("owned-volumes");
    if destination.try_exists()? {
        return Err(MicrosandboxError::InvalidConfig(format!(
            "owned volume backing already exists: {}",
            destination.display()
        )));
    }
    tokio::fs::create_dir_all(sandbox_dir).await?;
    let parent = sandbox_dir
        .parent()
        .ok_or_else(|| MicrosandboxError::InvalidConfig("sandbox directory has no parent".into()))?
        .to_path_buf();
    // The blocking worker owns its operation-unique directory. Cancellation cannot
    // leave a late formatter writing into a replacement sandbox's name.
    let stage = tokio::task::spawn_blocking(move || -> MicrosandboxResult<tempfile::TempDir> {
        let stage = tempfile::Builder::new()
            .prefix(".owned-volume-create-")
            .tempdir_in(parent)?;
        for mount in owned {
            let VolumeMount::Owned { guest, storage, .. } = mount else {
                unreachable!()
            };
            let directory = stage
                .path()
                .join(microsandbox_types::owned_volume_mount_id(&guest));
            std::fs::create_dir(&directory)?;
            match storage {
                OwnedVolumeStorage::Directory { .. } => {
                    std::fs::create_dir(directory.join("data"))?
                }
                OwnedVolumeStorage::Disk { capacity_mib } => {
                    if capacity_mib == 0 {
                        return Err(MicrosandboxError::InvalidConfig(
                            "owned disk size must be positive".into(),
                        ));
                    }
                    format_ext4(
                        &directory.join("disk.raw"),
                        &Ext4FormatOptions {
                            size_bytes: u64::from(capacity_mib) * 1024 * 1024,
                            ..Default::default()
                        },
                    )
                    .map_err(|error| {
                        MicrosandboxError::Custom(format!("format owned disk {guest}: {error}"))
                    })?;
                }
            }
        }
        Ok(stage)
    })
    .await
    .map_err(|error| MicrosandboxError::Custom(format!("owned volume preparation: {error}")))??;
    // No await between publication and returning to the retained creation cleanup.
    std::fs::rename(stage.path(), &destination)?;
    validate(sandbox_dir, mounts)
}

/// Admit existing backing without recreating missing state or following planted symlinks.
pub(crate) fn validate(sandbox_dir: &Path, mounts: &[VolumeMount]) -> MicrosandboxResult<()> {
    if !mounts
        .iter()
        .any(|mount| matches!(mount, VolumeMount::Owned { .. }))
    {
        return Ok(());
    }
    validate_tags(mounts)?;
    for mount in mounts {
        let VolumeMount::Owned { guest, storage, .. } = mount else {
            continue;
        };
        let path = backing_path(sandbox_dir, guest, storage);
        for component in [
            path.parent().and_then(Path::parent),
            path.parent(),
            Some(path.as_path()),
        ]
        .into_iter()
        .flatten()
        {
            let metadata = std::fs::symlink_metadata(component).map_err(|error| {
                MicrosandboxError::InvalidConfig(format!(
                    "owned volume {guest} backing unavailable at {}: {error}",
                    component.display()
                ))
            })?;
            if metadata.file_type().is_symlink() {
                return Err(MicrosandboxError::InvalidConfig(format!(
                    "owned volume {guest} backing must not be a symlink"
                )));
            }
        }
        let metadata = std::fs::metadata(&path)?;
        let correct = match storage {
            OwnedVolumeStorage::Directory { .. } => metadata.is_dir(),
            OwnedVolumeStorage::Disk { capacity_mib } => {
                metadata.is_file() && metadata.len() == u64::from(*capacity_mib) * 1024 * 1024
            }
        };
        if !correct {
            return Err(MicrosandboxError::InvalidConfig(format!(
                "owned volume {guest} backing kind or capacity does not match its configuration"
            )));
        }
    }
    Ok(())
}

fn validate_tags(mounts: &[VolumeMount]) -> MicrosandboxResult<()> {
    let mut tags = HashSet::new();
    for mount in mounts {
        let tag = if matches!(mount, VolumeMount::Owned { .. }) {
            microsandbox_types::owned_volume_mount_id(mount.guest())
        } else {
            super::spawn::guest_mount_tag(mount.guest())
        };
        if !tags.insert(tag) {
            return Err(MicrosandboxError::InvalidConfig(
                "volume mount identities collide".into(),
            ));
        }
    }
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::MountBuilder;

    #[test]
    fn no_owned_mounts_leave_legacy_admission_unchanged() {
        // Repeated guest tags are deliberately left to existing mount validation when
        // this feature is absent; owned admission must not inspect paths or allocate tags.
        let mount = MountBuilder::new("/legacy").tmpfs().build().unwrap();
        validate(
            Path::new("nonexistent-owned-admission-root"),
            &[mount.clone(), mount],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn directory_is_empty_private_and_never_recreated_on_restart() {
        let home = tempfile::tempdir().unwrap();
        let sandbox = home.path().join("source");
        let mount = MountBuilder::new("/data").owned().build().unwrap();
        prepare(&sandbox, std::slice::from_ref(&mount), false)
            .await
            .unwrap();
        let path = backing_path(
            &sandbox,
            "/data",
            &OwnedVolumeStorage::Directory { quota_mib: None },
        );
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
        std::fs::write(path.join("kept"), b"persisted").unwrap();
        validate(&sandbox, std::slice::from_ref(&mount)).unwrap();
        assert_eq!(std::fs::read(path.join("kept")).unwrap(), b"persisted");
        assert!(
            prepare(&sandbox, std::slice::from_ref(&mount), false)
                .await
                .is_err()
        );
        std::fs::remove_dir_all(&path).unwrap();
        assert!(prepare(&sandbox, &[mount], true).await.is_err());
        assert!(
            !path.exists(),
            "missing data must not silently become an empty volume"
        );
    }

    #[test]
    fn owned_ids_are_portable_and_unaffected_by_argument_order() {
        for guest in ["/data", "/var/lib/docker", "/résultats/文件", "/work files"] {
            let id = microsandbox_types::owned_volume_mount_id(guest);
            assert!(id.len() <= 20);
            assert!(
                id.bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
            );
            assert_eq!(id, microsandbox_types::owned_volume_mount_id(guest));
        }
        assert_ne!(
            microsandbox_types::owned_volume_mount_id("/var/log"),
            microsandbox_types::owned_volume_mount_id("/var_log")
        );
    }

    #[tokio::test]
    async fn owned_disk_is_sized_ext4_and_detects_truncation() {
        let home = tempfile::tempdir().unwrap();
        let sandbox = home.path().join("source");
        let mount = MountBuilder::new("/data")
            .owned_with(|v| v.disk().size(256_u32))
            .build()
            .unwrap();
        prepare(&sandbox, std::slice::from_ref(&mount), false)
            .await
            .unwrap();
        let path = backing_path(
            &sandbox,
            "/data",
            &OwnedVolumeStorage::Disk { capacity_mib: 256 },
        );
        let file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        assert_eq!(file.metadata().unwrap().len(), 256 * 1024 * 1024);
        file.set_len(1024).unwrap();
        assert!(validate(&sandbox, &[mount]).is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn owned_backing_rejects_symlink_escape() {
        let home = tempfile::tempdir().unwrap();
        let sandbox = home.path().join("source");
        let mount = MountBuilder::new("/data").owned().build().unwrap();
        prepare(&sandbox, std::slice::from_ref(&mount), false)
            .await
            .unwrap();
        let path = backing_path(
            &sandbox,
            "/data",
            &OwnedVolumeStorage::Directory { quota_mib: None },
        );
        std::fs::remove_dir(&path).unwrap();
        std::os::unix::fs::symlink(home.path(), &path).unwrap();
        assert!(validate(&sandbox, &[mount]).is_err());
    }
}
