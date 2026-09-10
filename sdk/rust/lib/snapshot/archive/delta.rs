//! Explicit disk-prefix and immutable RAM-object dependencies for incremental exports.

use microsandbox_image::checkpoint::{DiskLayerExportPlan, DiskLayerRef};
use microsandbox_image::snapshot::{DiskLayer, Manifest};

use super::*;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

pub(super) const REQUIREMENT: &str = "msb-snapshot-dependencies-v1";

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "layer",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
enum LayerIdentity {
    File(DiskLayer),
    Checkpoint(DiskLayerRef),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredLayer {
    path: String,
    identity: LayerIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Dependencies {
    disks: Vec<RequiredLayer>,
    memory: Vec<ObjectId>,
}

struct PhysicalLayer {
    required: RequiredLayer,
    source: PathBuf,
}

struct BaseSnapshot {
    snapshot: Snapshot,
    // Keep archive staging alive until all required payloads belong to the destination.
    _stage: Option<tempfile::TempDir>,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

pub(super) async fn selection(
    local: &LocalBackend,
    head: &Snapshot,
    opts: &SaveOpts,
) -> MicrosandboxResult<Option<Dependencies>> {
    if opts.since.is_none() && opts.last_layers.is_none() {
        return Ok(None);
    }
    if opts.with_parents || (opts.since.is_some() && opts.last_layers.is_some()) {
        return Err(MicrosandboxError::InvalidConfig(
            "incremental export takes either since or last_layers, without with_parents".into(),
        ));
    }
    let layers = physical_layers(head.manifest(), head.path())?;
    let mut memory = Vec::new();
    let required = if let Some(base) = &opts.since {
        // Base archives carry buffered decoder/verification futures; keep them off the caller's
        // stack, including when this planner is nested inside a direct restore or SDK call.
        let base = Box::pin(open_base(local, base)).await?;
        let baseline = physical_layers(base.snapshot.manifest(), base.snapshot.path())?;
        let available = memory_objects(&base.snapshot)?;
        memory = memory_objects(head)?
            .intersection(&available)
            .cloned()
            .collect();
        // Tmpfs-root full snapshots have no disks, but may still depend on RAM objects.
        // Do not let disk completeness suppress an independent memory dependency.
        if layers.is_empty() && baseline.is_empty() {
            0..0
        } else {
            DiskLayerExportPlan::since(
                &layers
                    .iter()
                    .map(|layer| &layer.required.identity)
                    .collect::<Vec<_>>(),
                &baseline
                    .iter()
                    .map(|layer| &layer.required.identity)
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| MicrosandboxError::InvalidConfig(error.to_string()))?
            .required()
        }
    } else {
        DiskLayerExportPlan::last(layers.len(), opts.last_layers.expect("selector checked"))
            .map_err(|error| MicrosandboxError::InvalidConfig(error.to_string()))?
            .required()
    };
    if required.is_empty() && memory.is_empty() {
        return Ok(None);
    }
    Ok(Some(Dependencies {
        disks: layers[required]
            .iter()
            .map(|layer| layer.required.clone())
            .collect(),
        memory,
    }))
}

pub(super) fn apply(
    inventory: &mut ArchiveInventory,
    dependencies: &Dependencies,
) -> MicrosandboxResult<()> {
    let paths = dependency_paths(&inventory.head, dependencies);
    let mut found = 0;
    for entry in &mut inventory.entries {
        if !paths.contains(entry.path.as_str()) {
            continue;
        }
        found += 1;
        entry.included = false;
        entry.encoded_size = 0;
        entry.sparse_ranges.clear();
        entry.transport_integrity = None;
    }
    if found != paths.len() {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "required payload is absent from archive inventory".into(),
        ));
    }
    inventory.completeness = "dependent".into();
    inventory.requires.push(REQUIREMENT.into());
    inventory.requires.sort();
    inventory
        .extensions
        .insert(REQUIREMENT.into(), serde_json::to_value(dependencies)?);
    inventory.limits.entry_count = inventory
        .entries
        .iter()
        .filter(|entry| entry.included)
        .count() as u64;
    inventory.limits.encoded_bytes = inventory
        .entries
        .iter()
        .filter(|entry| entry.included)
        .map(|entry| entry.encoded_size)
        .sum();
    inventory.limits.apparent_bytes = inventory
        .entries
        .iter()
        .filter(|entry| entry.included)
        .map(|entry| entry.apparent_size)
        .sum();
    Ok(())
}

pub(super) fn validate(inventory: &ArchiveInventory) -> MicrosandboxResult<Option<Dependencies>> {
    let extension = inventory.extensions.get(REQUIREMENT);
    let required = inventory.requires.iter().any(|value| value == REQUIREMENT);
    if inventory.completeness == "boot-complete" && !required && extension.is_none() {
        if inventory.entries.iter().any(|entry| !entry.included) {
            return Err(MicrosandboxError::SnapshotIntegrity(
                "complete archive cannot omit payloads".into(),
            ));
        }
        return Ok(None);
    }
    if inventory.completeness != "dependent" || !required || extension.is_none() {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "invalid snapshot dependency capability/completeness binding".into(),
        ));
    }
    let dependencies: Dependencies = serde_json::from_value(extension.unwrap().clone())?;
    if (dependencies.disks.is_empty() && dependencies.memory.is_empty())
        || dependencies.disks.len() > 256
        || dependencies.memory.len() > inventory.entries.len()
        || dependencies
            .memory
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "invalid snapshot dependency count or object ordering".into(),
        ));
    }
    let entries: HashMap<_, _> = inventory
        .entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let paths = dependency_paths(&inventory.head, &dependencies);
    if paths.len() != dependencies.disks.len() + dependencies.memory.len() {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "duplicate snapshot dependency".into(),
        ));
    }
    for path in &paths {
        let entry = entries.get(path.as_str()).ok_or_else(|| {
            MicrosandboxError::SnapshotIntegrity("dependency lacks an inventory entry".into())
        })?;
        let is_memory = dependencies
            .memory
            .binary_search_by(|id| memory_archive_path(&inventory.head, id).cmp(path))
            .is_ok();
        let valid_kind = if is_memory {
            entry.kind == "checkpoint-object"
        } else {
            matches!(
                entry.kind.as_str(),
                "file-payload" | "checkpoint-disk-layer"
            )
        };
        if entry.included
            || !valid_kind
            || entry.owner_snapshot.as_deref() != Some(inventory.head.as_str())
            || entry.encoded_size != 0
            || !entry.sparse_ranges.is_empty()
            || entry.transport_integrity.is_some()
        {
            return Err(MicrosandboxError::SnapshotIntegrity(
                "invalid omitted payload binding".into(),
            ));
        }
    }
    if inventory
        .entries
        .iter()
        .filter(|entry| !entry.included)
        .count()
        != paths.len()
    {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "archive omits an undeclared dependency".into(),
        ));
    }
    Ok(Some(dependencies))
}

/// Resolve only a caller-supplied base; never search ambient directories or backing paths.
pub(super) async fn resolve(
    local: &LocalBackend,
    inventory: &ArchiveInventory,
    snapshots_dir: &Path,
    cache_dir: &Path,
    base: Option<&str>,
) -> MicrosandboxResult<()> {
    let Some(dependencies) = validate(inventory)? else {
        return Ok(());
    };
    let base = base.ok_or_else(|| {
        MicrosandboxError::InvalidConfig(
            "this dependent archive requires an explicit base snapshot or standalone base archive"
                .into(),
        )
    })?;
    let base = Box::pin(open_base(local, base)).await?;
    let available = physical_layers(base.snapshot.manifest(), base.snapshot.path())?;
    if !dependencies.disks.is_empty()
        && (available.len() != dependencies.disks.len()
            || available
                .iter()
                .zip(&dependencies.disks)
                .any(|(layer, required)| layer.required.identity != required.identity))
    {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "supplied base is not the exact required physical disk prefix".into(),
        ));
    }
    let available_memory = memory_objects(&base.snapshot)?;
    if dependencies
        .memory
        .iter()
        .any(|id| !available_memory.contains(id))
    {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "supplied base does not contain the required RAM objects".into(),
        ));
    }

    // Every dependency is copied into operation-owned staging. In particular, a restored child
    // must not inherit a writable hardlink into the base; deleting the base must be harmless.
    for (source, required) in available.iter().zip(&dependencies.disks) {
        let target = inventory_entry_target(&required.path, snapshots_dir, cache_dir)?;
        copy_dependency(&source.source, &target).await?;
    }
    for id in &dependencies.memory {
        let source = checkpoint_object_path(&base.snapshot.path().join(CHECKPOINT_DIRECTORY), id);
        let target = inventory_entry_target(
            &memory_archive_path(&inventory.head, id),
            snapshots_dir,
            cache_dir,
        )?;
        copy_dependency(&source, &target).await?;
        // Verify only the objects actually borrowed, in the destination-owned copy. Export
        // selection is metadata-only for RAM; it must not scan the base's entire guest memory.
        // This reader owns a 64 KiB buffer; boxing prevents every enclosing archive/SDK
        // future from embedding another copy of that buffer in its own stack frame.
        let actual = format!(
            "sha256:{}",
            hex::encode(Box::pin(file_sha256(&target)).await?)
        );
        if actual != id.as_str() {
            return Err(MicrosandboxError::SnapshotIntegrity(format!(
                "base RAM object content does not match {id}"
            )));
        }
    }

    // Open the complete target only after filling omissions. This retains its normal metadata,
    // range, epoch and disk-integrity validation instead of introducing a partial-closure mode.
    let artifact = snapshots_dir.join(&inventory.head);
    let manifest =
        Manifest::from_bytes(&tokio::fs::read(artifact.join(DESCRIPTOR_FILENAME)).await?)
            .map_err(|error| MicrosandboxError::SnapshotIntegrity(error.to_string()))?;
    let target = physical_layers(&manifest, &artifact)?;
    if target.len() < dependencies.disks.len()
        || target
            .iter()
            .zip(&dependencies.disks)
            .any(|(layer, required)| &layer.required != required)
    {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "dependency list is not the target descriptor's exact disk prefix".into(),
        ));
    }
    let target_memory = if dependencies.memory.is_empty() {
        BTreeSet::new()
    } else {
        let snapshot = store::open_snapshot(local, artifact.to_string_lossy().as_ref()).await?;
        memory_objects(&snapshot)?
    };
    if dependencies
        .memory
        .iter()
        .any(|id| !target_memory.contains(id))
    {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "omitted object is not a target RAM payload; metadata must remain included".into(),
        ));
    }
    let entries: HashMap<_, _> = inventory
        .entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    for id in &dependencies.memory {
        let path = memory_archive_path(&inventory.head, id);
        let entry = entries
            .get(path.as_str())
            .expect("dependency inventory was validated");
        let target = inventory_entry_target(&path, snapshots_dir, cache_dir)?;
        if tokio::fs::metadata(target).await?.len() != entry.apparent_size {
            return Err(MicrosandboxError::SnapshotIntegrity(
                "resolved RAM object size differs from inventory".into(),
            ));
        }
    }
    Ok(())
}

async fn copy_dependency(source: &Path, target: &Path) -> MicrosandboxResult<()> {
    if tokio::fs::symlink_metadata(target).await.is_ok() {
        return Err(MicrosandboxError::SnapshotIntegrity(
            "dependency collides with an extracted member".into(),
        ));
    }
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let source = source.to_path_buf();
    let target = target.to_path_buf();
    tokio::task::spawn_blocking(move || microsandbox_utils::copy::fast_copy(&source, &target))
        .await
        .map_err(|error| MicrosandboxError::Runtime(format!("base payload copy: {error}")))??;
    Ok(())
}

fn memory_archive_path(snapshot_id: &str, id: &ObjectId) -> String {
    let hash = id
        .as_str()
        .strip_prefix("sha256:")
        .expect("validated ObjectId");
    format!(
        "checkpoints/{snapshot_id}/objects/sha256/{}/{hash}",
        &hash[..2]
    )
}

fn checkpoint_object_path(root: &Path, id: &ObjectId) -> PathBuf {
    let hash = id
        .as_str()
        .strip_prefix("sha256:")
        .expect("validated ObjectId");
    root.join("objects")
        .join("sha256")
        .join(&hash[..2])
        .join(hash)
}

fn dependency_paths(head: &str, dependencies: &Dependencies) -> BTreeSet<String> {
    dependencies
        .disks
        .iter()
        .map(|layer| layer.path.clone())
        .chain(
            dependencies
                .memory
                .iter()
                .map(|id| memory_archive_path(head, id)),
        )
        .collect()
}

/// Return reusable RAM payload IDs, never metadata objects, even if bytes happen to coincide.
fn memory_objects(snapshot: &Snapshot) -> MicrosandboxResult<BTreeSet<ObjectId>> {
    let SnapshotState::Checkpoint(state) = &snapshot.manifest().state else {
        return Ok(BTreeSet::new());
    };
    let expected = ObjectId::new(&state.checkpoint_root)
        .map_err(|error| MicrosandboxError::SnapshotIntegrity(error.to_string()))?;
    let closure = CheckpointClosure::open_portable(
        snapshot.path().join(CHECKPOINT_DIRECTORY),
        Some(&expected),
    )
    .map_err(|error| MicrosandboxError::SnapshotIntegrity(error.to_string()))?;
    let checkpoint = closure.checkpoint();
    let mut objects: BTreeSet<_> = closure
        .memory()
        .extents
        .iter()
        .filter_map(|extent| match &extent.content {
            MemoryExtentContent::Object(content) => Some(content.object.clone()),
            MemoryExtentContent::Zero => None,
        })
        .collect();
    objects.remove(&checkpoint.memory);
    objects.remove(&checkpoint.execution_state);
    for id in &checkpoint.disks {
        objects.remove(id);
    }
    for device in &checkpoint.devices {
        objects.remove(&device.state);
    }
    Ok(objects)
}

fn physical_layers(
    manifest: &Manifest,
    directory: &Path,
) -> MicrosandboxResult<Vec<PhysicalLayer>> {
    match &manifest.state {
        SnapshotState::File(file) => file
            .layers
            .iter()
            .map(|layer| {
                Ok(PhysicalLayer {
                    required: RequiredLayer {
                        path: portable_archive_path(&file.layer_path(layer))?,
                        identity: LayerIdentity::File(layer.clone()),
                    },
                    source: directory.join(file.layer_path(layer)),
                })
            })
            .collect(),
        SnapshotState::Checkpoint(state) => {
            let root = directory.join(CHECKPOINT_DIRECTORY);
            let expected = ObjectId::new(&state.checkpoint_root)
                .map_err(|error| MicrosandboxError::SnapshotIntegrity(error.to_string()))?;
            let closure = CheckpointClosure::open_portable(&root, Some(&expected))
                .map_err(|error| MicrosandboxError::SnapshotIntegrity(error.to_string()))?;
            if closure.disks().len() > 1 {
                return Err(MicrosandboxError::InvalidConfig(
                    "disk-layer selection supports at most one checkpoint disk".into(),
                ));
            }
            Ok(closure
                .disks()
                .iter()
                .flat_map(|disk| &disk.layers)
                .map(|layer| PhysicalLayer {
                    required: RequiredLayer {
                        path: format!(
                            "checkpoints/{}/layers/{}.{}",
                            manifest.snapshot_id, layer.layer_id, layer.format
                        ),
                        identity: LayerIdentity::Checkpoint(layer.clone()),
                    },
                    source: closure.disk_layer_path(layer),
                })
                .collect())
        }
    }
}

async fn open_base(local: &LocalBackend, input: &str) -> MicrosandboxResult<BaseSnapshot> {
    let path = Path::new(input);
    if !path.is_file() {
        let snapshot = store::open_snapshot(local, input).await?;
        if matches!(snapshot.manifest().state, SnapshotState::File(_)) {
            Box::pin(snapshot.verify()).await?;
        }
        return Ok(BaseSnapshot {
            snapshot,
            _stage: None,
        });
    }
    let stage = tempfile::tempdir()?;
    let snapshots_dir = stage.path().join("snapshots");
    let cache_dir = stage.path().join("cache");
    tokio::fs::create_dir_all(&snapshots_dir).await?;
    tokio::fs::create_dir_all(&cache_dir).await?;
    let mut reader = BufReader::new(tokio::fs::File::open(path).await?);
    let compressed = reader
        .fill_buf()
        .await?
        .starts_with(&[0x28, 0xb5, 0x2f, 0xfd]);
    let unpacked = if compressed {
        Box::pin(unpack_archive(
            ZstdDecoder::new(reader),
            &snapshots_dir,
            &cache_dir,
        ))
        .await?
    } else {
        Box::pin(unpack_archive(reader, &snapshots_dir, &cache_dir)).await?
    };
    if let Some(inventory) = &unpacked.inventory {
        if validate(inventory)?.is_some() {
            return Err(MicrosandboxError::InvalidConfig("the supplied base archive must be standalone; load dependent bases explicitly first".into()));
        }
        materialize_inventory_layers(inventory, &snapshots_dir).await?;
    } else {
        super::super::migration::normalize_staged(local.db().await?, &unpacked.manifest_dirs)
            .await?;
    }
    let imported = verify_imported_snapshots(local, &unpacked.manifest_dirs).await?;
    let head = match unpacked.head {
        Some(head) => imported
            .iter()
            .position(|snapshot| snapshot.id().as_str() == head)
            .ok_or_else(|| {
                MicrosandboxError::SnapshotIntegrity("base archive head is missing".into())
            })?,
        None => select_head_snapshot(&imported)?,
    };
    if let Some(inventory) = &unpacked.inventory {
        validate_inventory_snapshot_bindings(inventory, &imported)?;
    }
    if matches!(imported[head].manifest().state, SnapshotState::File(_)) {
        Box::pin(imported[head].verify()).await?;
    }
    Ok(BaseSnapshot {
        snapshot: imported[head].clone(),
        _stage: Some(stage),
    })
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
#[path = "delta_tests.rs"]
mod memory_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use microsandbox_image::snapshot::{
        DiskLayerId, FileSnapshotState, ImageRef, LayerFileKind, LayerPayload, SnapshotCapture,
        SnapshotConsistency, SnapshotFormat, SnapshotId, SnapshotRootDisk, SnapshotScope,
    };

    #[tokio::test]
    async fn delta_load_and_direct_restore_require_exact_base_and_own_their_closure() {
        let temp = tempfile::tempdir().unwrap();
        let local = LocalBackend::builder()
            .home(temp.path().join("home"))
            .build()
            .await
            .unwrap();
        let base_dir = temp.path().join("base");
        let head_dir = temp.path().join("head");
        tokio::fs::create_dir_all(base_dir.join("layers"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(head_dir.join("layers"))
            .await
            .unwrap();
        let base_layer = DiskLayer {
            layer_id: DiskLayerId::new("layer_00000000000000000000000000000001").unwrap(),
            format: SnapshotFormat::Raw,
            virtual_size: 65536,
            backing: None,
            payload: LayerPayload {
                file_kind: LayerFileKind::Regular,
                integrity: None,
            },
        };
        let top = DiskLayer {
            layer_id: DiskLayerId::new("layer_00000000000000000000000000000002").unwrap(),
            format: SnapshotFormat::Qcow2,
            virtual_size: 65536,
            backing: Some(base_layer.layer_id.clone()),
            payload: base_layer.payload.clone(),
        };
        let descriptor = |id: &str, layers: Vec<DiskLayer>| Manifest {
            schema: "microsandbox.snapshot/1".into(),
            snapshot_id: SnapshotId::new(id).unwrap(),
            scope: SnapshotScope::Disk,
            root_disk: SnapshotRootDisk::Managed,
            state: SnapshotState::File(FileSnapshotState {
                disk_format: layers.last().unwrap().format,
                filesystem: "ext4".into(),
                virtual_size: 65536,
                head: layers.last().unwrap().layer_id.clone(),
                layers,
            }),
            capture: SnapshotCapture {
                created_at: "2026-09-05T00:00:00Z".into(),
                source_lineage: None,
                source_checkpoint: None,
                consistency: SnapshotConsistency::CrashConsistent,
            },
            image: ImageRef {
                reference: "docker.io/library/alpine:3.20".into(),
                manifest_digest: format!("sha256:{}", "0".repeat(64)),
            },
            parent: None,
            extensions: BTreeMap::new(),
            requires: vec![],
        };
        let base = descriptor(
            "snap_00000000000000000000000000000001",
            vec![base_layer.clone()],
        );
        let head = descriptor(
            "snap_00000000000000000000000000000002",
            vec![base_layer.clone(), top.clone()],
        );
        let base_path =
            microsandbox_image::snapshot::layer_path(&base_layer.layer_id, base_layer.format);
        let top_path = microsandbox_image::snapshot::layer_path(&top.layer_id, top.format);
        std::fs::write(base_dir.join(&base_path), vec![91u8; 65536]).unwrap();
        std::fs::copy(base_dir.join(&base_path), head_dir.join(&base_path)).unwrap();
        microsandbox_image::checkpoint::create_qcow2_overlay(
            &head_dir.join(top_path),
            65536,
            &head_dir.join(&base_path),
            "raw",
        )
        .await
        .unwrap();
        std::fs::write(
            base_dir.join(DESCRIPTOR_FILENAME),
            base.to_canonical_bytes().unwrap(),
        )
        .unwrap();
        std::fs::write(
            head_dir.join(DESCRIPTOR_FILENAME),
            head.to_canonical_bytes().unwrap(),
        )
        .unwrap();
        let base_name = base_dir.to_str().unwrap();
        let head_name = head_dir.to_str().unwrap();
        let archive = temp.path().join("delta.tar.zst");
        save_snapshot(
            &local,
            head_name,
            &archive,
            SaveOpts {
                since: Some(base_name.into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(load_snapshot(&local, &archive, None).await.is_err());
        assert!(
            load_snapshot_with_base(&local, &archive, None, Some(head_name))
                .await
                .is_err()
        );
        let loaded = load_snapshot_with_base(&local, &archive, None, Some(base_name))
            .await
            .unwrap();
        assert!(loaded.path().join(&base_path).exists());
        let child = temp.path().join("child");
        let result = materialize_archive_for_child_with_base(
            &local,
            &archive,
            &child,
            false,
            Some(base_name),
        )
        .await
        .unwrap();
        assert_eq!(result.upper_layers.len(), 3);
        let base_archive = temp.path().join("base.tar");
        save_snapshot(
            &local,
            base_name,
            &base_archive,
            SaveOpts {
                plain_tar: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let last = temp.path().join("last.tar");
        save_snapshot(
            &local,
            head_name,
            &last,
            SaveOpts {
                last_layers: Some(1),
                plain_tar: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let other_dest = temp.path().join("imported");
        load_snapshot_with_base(
            &local,
            &last,
            Some(&other_dest),
            Some(base_archive.to_str().unwrap()),
        )
        .await
        .unwrap();
        std::fs::remove_dir_all(&base_dir).unwrap();
        assert_eq!(
            std::fs::read(loaded.path().join(&base_path)).unwrap(),
            vec![91u8; 65536]
        );
        assert!(result.upper_layers.iter().all(|layer| layer.path.exists()));
        for count in [0, 3] {
            assert!(
                save_snapshot(
                    &local,
                    head_name,
                    &temp.path().join("invalid.tar"),
                    SaveOpts {
                        last_layers: Some(count),
                        ..Default::default()
                    }
                )
                .await
                .is_err()
            );
        }
        // An intact head must not hide a corrupt recorded ancestor in a file-state chain.
        let mut recorded = head.clone();
        let SnapshotState::File(file) = &mut recorded.state else {
            unreachable!()
        };
        file.layers[0].payload.integrity = Some(
            super::super::super::verify::compute_merkle_integrity(&head_dir.join(&base_path))
                .await
                .unwrap(),
        );
        std::fs::write(
            head_dir.join(DESCRIPTOR_FILENAME),
            recorded.to_canonical_bytes().unwrap(),
        )
        .unwrap();
        let snapshot = store::open_snapshot(&local, head_name).await.unwrap();
        snapshot.verify().await.unwrap();
        std::fs::write(head_dir.join(&base_path), vec![92u8; 65536]).unwrap();
        assert!(snapshot.verify().await.is_err());
    }
}
