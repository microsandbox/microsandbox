//! Private reconstruction of required sandbox-owned backing.

use std::path::Path;

use microsandbox_image::snapshot::{OwnedVolumeCapture, OwnedVolumeData};
use microsandbox_types::VolumeMount;

use crate::{MicrosandboxError, MicrosandboxResult};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Materialize every owned mount, independently of external-resource inheritance choices.
pub(crate) async fn materialize_owned_volumes(
    volumes: &[OwnedVolumeCapture],
    source: &Path,
    child: &Path,
    choices: &crate::sandbox::restore_resources::RestoreResources,
) -> MicrosandboxResult<Vec<VolumeMount>> {
    microsandbox_image::snapshot::validate_owned_volumes(volumes)?;
    // Check all destinations before starting workers: explicit replacement would silently
    // discard captured state and is never an alternate authorization for owned storage.
    for volume in volumes {
        if microsandbox_types::owned_volume_mount_id(&volume.mount.guest) != volume.mount_id {
            return Err(invalid(
                "owned mount identity differs from its canonical guest path",
            ));
        }
        if choices.mapped.contains(&volume.mount.guest) {
            return Err(invalid(&format!(
                "owned mount {} cannot be replaced by an explicit mapping",
                volume.mount.guest
            )));
        }
    }
    let staging_parent = child
        .parent()
        .ok_or_else(|| invalid("owned storage child has no parent"))?;
    for volume in volumes {
        let directory = child.join("owned-volumes").join(&volume.mount_id);
        match &volume.data {
            OwnedVolumeData::Disk { generation } => {
                let layer = &generation.layers[0];
                let source = source
                    .join("layers")
                    .join(format!("{}.{}", layer.layer_id, layer.format));
                let parent = staging_parent.to_path_buf();
                let integrity = layer.integrity_root.clone();
                let worker = tokio::task::spawn_blocking(move || {
                    super::additional_disks::stage_additional_disk(
                        &source, &parent, &integrity, false,
                    )
                });
                super::additional_disks::publish_staged_additional_disk(
                    worker,
                    &directory.join("disk.raw"),
                )
                .await?;
            }
            OwnedVolumeData::Directory { descriptor, files } => {
                let source = source.join(volume.directory_path());
                let parent = staging_parent.to_path_buf();
                let descriptor = descriptor.clone();
                let files = files.clone();
                let staging = tokio::task::spawn_blocking(
                    move || -> MicrosandboxResult<tempfile::TempDir> {
                        let snapshot =
                            microsandbox_filesystem::OwnedDirectorySnapshot::open_expected(
                                &source,
                                &descriptor.digest,
                            )?;
                        let payloads = snapshot.payloads();
                        if snapshot.descriptor_bytes()?.len() as u64 != descriptor.bytes
                            || payloads.len() != files.len()
                            || payloads.iter().zip(&files).any(|(actual, expected)| {
                                actual.digest != expected.digest || actual.bytes != expected.bytes
                            })
                        {
                            return Err(invalid(
                                "owned namespace descriptor and payload inventory disagree",
                            ));
                        }
                        let staging = tempfile::Builder::new()
                            .prefix(".owned-directory-restore-")
                            .tempdir_in(parent)?;
                        snapshot.materialize(&source, &staging.path().join("data"))?;
                        Ok(staging)
                    },
                )
                .await
                .map_err(|error| invalid(&format!("owned directory restore worker: {error}")))??;
                // A cancelled copy knows only unique staging. Publish while this async poll
                // still holds the caller's sandbox transition and lifecycle guards.
                std::fs::create_dir_all(&directory)?;
                std::fs::rename(staging.path().join("data"), directory.join("data"))?;
            }
        }
    }
    Ok(volumes
        .iter()
        .map(|volume| volume.mount.to_mount())
        .collect())
}

fn invalid(message: &str) -> MicrosandboxError {
    MicrosandboxError::SnapshotIntegrity(message.into())
}
