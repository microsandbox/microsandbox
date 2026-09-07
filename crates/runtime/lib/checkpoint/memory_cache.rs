//! Immutable local realizations of complete portable memory manifests.
//!
//! The cache is trusted host storage, not a second portable snapshot format. Publication is
//! atomic; readers retain read-only handles, so removing a snapshot never invalidates live RAM.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use microsandbox_image::checkpoint::{MemoryExtentContent, MemoryManifest, ObjectId};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// One native-aligned, contiguous guest address span in a flat cache file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedMemoryRegion {
    /// Start of the guest physical span.
    pub guest_address: u64,
    /// Length of the span in bytes.
    pub length: u64,
    /// Byte offset in the immutable cache file.
    pub file_offset: u64,
}

/// An opened realization pinned against cooperative eviction until the handle is dropped.
pub struct CachedMemory {
    path: PathBuf,
    identity: ObjectId,
    /// Read-only backing ownership. Transfer this handle to the VMM, not merely its pathname.
    pub file: File,
    /// Exact guest coverage, with address holes omitted from physical storage.
    pub regions: Vec<CachedMemoryRegion>,
    /// Whether existing verified bytes were reused without rereading portable objects.
    pub cache_hit: bool,
    /// Whether this construction cloned its baseline using a filesystem reflink.
    pub reflink: bool,
    /// Time spent resolving or constructing this backing, in microseconds.
    pub prepare_us: u128,
}

/// Host-local, immutable memory cache. No entry is ever modified in place.
pub struct MemoryCache {
    root: PathBuf,
    page_size: u64,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl MemoryCache {
    /// Open a dedicated cache directory using this host's native mapping alignment.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        #[cfg(unix)]
        {
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
            if page_size <= 0 {
                return Err(io::Error::last_os_error());
            }
            let root = root.into();
            std::fs::create_dir_all(&root)?;
            // Cache contents are guest RAM, not public image data. Restrict traversal even
            // when the caller's umask permits other local users to read ordinary cache files.
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))?;
            Ok(Self {
                root,
                page_size: page_size as u64,
            })
        }
        #[cfg(not(unix))]
        {
            let _ = root;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "private memory cache is not qualified on this backend",
            ))
        }
    }

    /// Resolve a complete memory image, verifying portable objects only on a cache miss.
    ///
    /// `read_object` must return identity-verified bytes. It is called once per distinct packed
    /// object, not once per extent. The complete canonical manifest identity names the cache;
    /// neither an unverified partial delta nor a mutable file may be published under that name.
    pub fn materialize(
        &self,
        manifest: &MemoryManifest,
        identity: &ObjectId,
        read_object: impl FnMut(&ObjectId) -> io::Result<Vec<u8>>,
    ) -> io::Result<CachedMemory> {
        self.materialize_with_baseline(manifest, identity, None, read_object)
    }

    /// Reuse a pinned complete baseline before overlaying immutable changed object slices.
    /// The source VM is never read or remapped here; both inputs are completed captures.
    pub fn materialize_with_baseline(
        &self,
        manifest: &MemoryManifest,
        identity: &ObjectId,
        baseline: Option<(&MemoryManifest, &CachedMemory)>,
        mut read_object: impl FnMut(&ObjectId) -> io::Result<Vec<u8>>,
    ) -> io::Result<CachedMemory> {
        let started = Instant::now();
        let canonical = manifest.to_canonical_bytes().map_err(io::Error::other)?;
        if ObjectId::from_bytes(&canonical).map_err(io::Error::other)? != *identity {
            return Err(invalid(
                "memory cache identity does not match its complete manifest",
            ));
        }
        let regions = memory_regions(manifest, self.page_size)?;
        let length = regions
            .last()
            .and_then(|r| r.file_offset.checked_add(r.length))
            .ok_or_else(|| invalid("empty or overflowing memory topology"))?;
        let path = self.entry_path(identity);
        if let Some(file) = open_pinned(&path, length)? {
            return Ok(CachedMemory {
                path,
                identity: identity.clone(),
                file,
                regions,
                cache_hit: true,
                reflink: false,
                prepare_us: started.elapsed().as_micros(),
            });
        }

        let staging_dir = tempfile::Builder::new()
            .prefix(".memory-")
            .tempdir_in(&self.root)?;
        let staging_path = staging_dir.path().join("memory");
        let baseline = baseline.filter(|(_, cached)| cached.regions == regions);
        if let Some((previous, cached)) = baseline {
            let bytes = previous.to_canonical_bytes().map_err(io::Error::other)?;
            if ObjectId::from_bytes(&bytes).map_err(io::Error::other)? != cached.identity {
                return Err(invalid("cache baseline does not match its pinned manifest"));
            }
        }
        let mut reflink = false;
        if let Some((_, cached)) = baseline {
            let (_, strategy) =
                microsandbox_utils::copy::fast_copy_with_strategy(&cached.path, &staging_path)?;
            reflink = strategy == microsandbox_utils::copy::FastCopyStrategy::Reflink;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&staging_path, std::fs::Permissions::from_mode(0o600))?;
            }
        }
        let mut staging = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&staging_path)?;
        // A fresh sparse file supplies all zero extents without allocating or writing RAM-sized
        // buffers. Only immutable nonzero object slices are copied into it.
        staging.set_len(length)?;
        let previous = baseline.map(|(manifest, _)| {
            manifest
                .extents
                .iter()
                .map(|extent| (extent.start, extent))
                .collect::<BTreeMap<_, _>>()
        });
        let mut objects = BTreeMap::<ObjectId, Vec<(u64, u64, u64)>>::new();
        let mut region_index = 0;
        for extent in &manifest.extents {
            while extent.start >= regions[region_index].guest_address + regions[region_index].length
            {
                region_index += 1;
            }
            if previous.as_ref().and_then(|map| map.get(&extent.start)) == Some(&extent) {
                continue;
            }
            let region = &regions[region_index];
            let offset = region.file_offset + (extent.start - region.guest_address);
            if let MemoryExtentContent::Object(content) = &extent.content {
                objects.entry(content.object.clone()).or_default().push((
                    offset,
                    content.object_offset,
                    extent.length,
                ));
            } else if baseline.is_some() {
                // A newly zero range must overwrite the cloned bytes, never resurrect them.
                // Bound the temporary allocation independently of guest RAM size.
                staging.seek(SeekFrom::Start(offset))?;
                let zeros = [0u8; 64 * 1024];
                let mut remaining = extent.length;
                while remaining > 0 {
                    let count = remaining.min(zeros.len() as u64) as usize;
                    staging.write_all(&zeros[..count])?;
                    remaining -= count as u64;
                }
            }
        }
        for (id, slices) in objects {
            let bytes = read_object(&id)?;
            for (target, offset, count) in slices {
                let start = usize::try_from(offset)
                    .map_err(|_| invalid("memory object offset overflows"))?;
                let count = usize::try_from(count)
                    .map_err(|_| invalid("memory object length overflows"))?;
                let end = start
                    .checked_add(count)
                    .ok_or_else(|| invalid("memory object slice overflows"))?;
                let bytes = bytes
                    .get(start..end)
                    .ok_or_else(|| invalid("memory object slice exceeds verified bytes"))?;
                staging.seek(SeekFrom::Start(target))?;
                staging.write_all(bytes)?;
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            staging.set_permissions(std::fs::Permissions::from_mode(0o400))?;
        }
        staging.sync_all()?;
        // Publish the inode without replacement. Concurrent builders may do duplicate work, but
        // no winner can overwrite backing another VM has already pinned or mapped.
        match std::fs::hard_link(&staging_path, &path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        #[cfg(unix)]
        File::open(&self.root)?.sync_all()?;
        let file = open_pinned(&path, length)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "memory cache was evicted before it could be pinned; retry restore",
            )
        })?;
        Ok(CachedMemory {
            path,
            identity: identity.clone(),
            file,
            regions,
            cache_hit: false,
            reflink,
            prepare_us: started.elapsed().as_micros(),
        })
    }

    /// Remove an unpinned immutable entry. `false` means absent or still owned by a VM.
    ///
    /// Never truncate or hole-punch a live entry. POSIX open-handle lifetime also protects a
    /// reader that opened the inode immediately before an eviction acquired its exclusive lock.
    pub fn evict(&self, identity: &ObjectId) -> io::Result<bool> {
        let path = self.entry_path(identity);
        let file = match open_readonly(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        };
        if !microsandbox_utils::process_lock::try_lock_exclusive(&file)? {
            return Ok(false);
        }
        // A competing evictor can have removed this same inode while we waited to acquire it.
        // Do not unlink a new realization published at the old name in the meantime.
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let opened = file.metadata()?;
            let current = match std::fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
                Err(error) => return Err(error),
            };
            if (opened.dev(), opened.ino()) != (current.dev(), current.ino()) {
                return Ok(false);
            }
        }
        std::fs::remove_file(path)?;
        Ok(true)
    }

    fn entry_path(&self, identity: &ObjectId) -> PathBuf {
        // Geometry lives in the identity-bearing manifest. Native alignment is local realization
        // policy, so a cache prepared on a different page-size host must not collide with it.
        self.root.join(format!(
            "{}-{}.ram",
            identity.as_str().replace(':', "-"),
            self.page_size
        ))
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

fn memory_regions(
    manifest: &MemoryManifest,
    page_size: u64,
) -> io::Result<Vec<CachedMemoryRegion>> {
    let mut regions: Vec<CachedMemoryRegion> = Vec::new();
    let mut file_length = 0u64;
    for extent in &manifest.extents {
        if let Some(last) = regions.last_mut()
            && last.guest_address.checked_add(last.length) == Some(extent.start)
        {
            last.length = last
                .length
                .checked_add(extent.length)
                .ok_or_else(|| invalid("memory topology overflows"))?;
        } else {
            regions.push(CachedMemoryRegion {
                guest_address: extent.start,
                length: extent.length,
                file_offset: file_length,
            });
        }
        file_length = file_length
            .checked_add(extent.length)
            .ok_or_else(|| invalid("memory cache size overflows"))?;
    }
    for region in &regions {
        if !region.guest_address.is_multiple_of(page_size)
            || !region.length.is_multiple_of(page_size)
            || !region.file_offset.is_multiple_of(page_size)
        {
            return Err(invalid(
                "guest memory topology is not aligned for private mappings on this host",
            ));
        }
    }
    Ok(regions)
}

fn open_readonly(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    options.open(path)
}

fn open_pinned(path: &Path, length: u64) -> io::Result<Option<File>> {
    let file = match open_readonly(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != length {
        return Err(invalid(
            "memory cache entry has invalid type or length; evict and rebuild it",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        loop {
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_SH) } == 0 {
                break;
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }
    Ok(Some(file))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use microsandbox_image::checkpoint::{ContentRef, MemoryCaptureMode, MemoryExtent};
    use std::os::unix::fs::FileExt;

    fn fixture(page: u64) -> (MemoryManifest, ObjectId, Vec<u8>) {
        let bytes = vec![0x5a; page as usize];
        let object = ObjectId::from_bytes(&bytes).unwrap();
        let manifest = MemoryManifest {
            schema: "microsandbox.memory/1".into(),
            architecture: std::env::consts::ARCH.into(),
            guest_page_size: 4096,
            topology_generation: 1,
            generation: 1,
            capture_mode: MemoryCaptureMode::Full,
            pause_generation: 1,
            extents: vec![
                MemoryExtent {
                    start: 0,
                    length: page,
                    content: MemoryExtentContent::Object(ContentRef {
                        object: object.clone(),
                        object_offset: 0,
                    }),
                },
                MemoryExtent {
                    start: page,
                    length: page,
                    content: MemoryExtentContent::Zero,
                },
                MemoryExtent {
                    start: page * 4,
                    length: page,
                    content: MemoryExtentContent::Object(ContentRef {
                        object,
                        object_offset: 0,
                    }),
                },
            ],
        };
        let id = ObjectId::from_bytes(&manifest.to_canonical_bytes().unwrap()).unwrap();
        (manifest, id, bytes)
    }

    #[test]
    fn materialize_once_reuse_pinned_bytes_and_evict_after_last_owner() {
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (manifest, id, bytes) = fixture(cache.page_size);
        let mut reads = 0;
        let first = cache
            .materialize(&manifest, &id, |_| {
                reads += 1;
                Ok(bytes.clone())
            })
            .unwrap();
        assert_eq!(reads, 1);
        assert!(!first.cache_hit);
        assert_eq!(first.regions.len(), 2);
        assert_eq!(first.file.metadata().unwrap().len(), cache.page_size * 3);
        let mut zero = vec![1; cache.page_size as usize];
        first
            .file
            .read_exact_at(&mut zero, cache.page_size)
            .unwrap();
        assert!(zero.iter().all(|byte| *byte == 0));
        let second = cache
            .materialize(&manifest, &id, |_| {
                panic!("warm cache reread a portable object")
            })
            .unwrap();
        assert!(second.cache_hit);
        assert!(!cache.evict(&id).unwrap());
        drop(first);
        assert!(!cache.evict(&id).unwrap());
        drop(second);
        assert!(cache.evict(&id).unwrap());
        assert!(!cache.evict(&id).unwrap());
    }

    #[test]
    fn descendant_reuses_unchanged_objects_and_clears_new_zero_ranges() {
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (manifest, id, bytes) = fixture(cache.page_size);
        let baseline = cache
            .materialize(&manifest, &id, |_| Ok(bytes.clone()))
            .unwrap();
        let mut descendant = manifest.clone();
        descendant.generation += 1;
        descendant.extents[0].content = MemoryExtentContent::Zero;
        let changed = vec![0x7c; cache.page_size as usize];
        let changed_id = ObjectId::from_bytes(&changed).unwrap();
        descendant.extents[1].content = MemoryExtentContent::Object(ContentRef {
            object: changed_id.clone(),
            object_offset: 0,
        });
        let descendant_id =
            ObjectId::from_bytes(&descendant.to_canonical_bytes().unwrap()).unwrap();
        let mut reads = 0;
        let child = cache
            .materialize_with_baseline(
                &descendant,
                &descendant_id,
                Some((&manifest, &baseline)),
                |id| {
                    assert_eq!(id, &changed_id, "unchanged object was reread");
                    reads += 1;
                    Ok(changed.clone())
                },
            )
            .unwrap();
        assert_eq!(reads, 1);
        let mut result = vec![0; cache.page_size as usize * 3];
        child.file.read_exact_at(&mut result, 0).unwrap();
        assert!(
            result[..cache.page_size as usize]
                .iter()
                .all(|byte| *byte == 0)
        );
        assert_eq!(
            &result[cache.page_size as usize..cache.page_size as usize * 2],
            &changed
        );
        assert_eq!(&result[cache.page_size as usize * 2..], &bytes);
        baseline
            .file
            .read_exact_at(&mut result[..cache.page_size as usize], 0)
            .unwrap();
        assert_eq!(
            &result[..cache.page_size as usize],
            &bytes,
            "baseline was mutated"
        );
        assert!(!cache.evict(&id).unwrap());
    }

    #[test]
    fn reject_a_manifest_paired_with_the_wrong_baseline() {
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (manifest, id, bytes) = fixture(cache.page_size);
        let baseline = cache
            .materialize(&manifest, &id, |_| Ok(bytes.clone()))
            .unwrap();
        let mut wrong = manifest.clone();
        wrong.generation += 1;
        let target = ObjectId::from_bytes(&wrong.to_canonical_bytes().unwrap()).unwrap();
        assert!(
            cache
                .materialize_with_baseline(&wrong, &target, Some((&wrong, &baseline)), |_| panic!(
                    "must reject before reads"
                ))
                .is_err()
        );
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn failed_materialization_does_not_publish_or_leave_staging() {
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (manifest, id, _) = fixture(cache.page_size);
        let failure = cache.materialize(&manifest, &id, |_| {
            Err(io::Error::other("injected object read failure"))
        });
        assert!(failure.is_err());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(cache.materialize(&manifest, &id, |_| Ok(vec![])).is_err());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    }

    #[test]
    fn reject_wrong_manifest_identity_and_host_alignment() {
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (mut manifest, id, _) = fixture(cache.page_size);
        manifest.generation += 1;
        assert!(
            cache
                .materialize(&manifest, &id, |_| panic!(
                    "identity rejection must precede reads"
                ))
                .is_err()
        );
        let (mut manifest, _, _) = fixture(4096);
        manifest.extents.truncate(1);
        assert!(memory_regions(&manifest, 16384).is_err());
    }

    #[test]
    fn concurrent_builders_publish_one_immutable_inode() {
        use std::os::unix::fs::MetadataExt;
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (manifest, id, bytes) = fixture(cache.page_size);
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let run = || {
                cache
                    .materialize(&manifest, &id, |_| {
                        barrier.wait();
                        Ok(bytes.clone())
                    })
                    .unwrap()
            };
            let first = scope.spawn(run);
            let second = scope.spawn(run);
            let first = first.join().unwrap();
            let second = second.join().unwrap();
            assert_eq!(
                first.file.metadata().unwrap().ino(),
                second.file.metadata().unwrap().ino()
            );
        });
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn unlinked_backing_survives_without_snapshot_paths() {
        let directory = tempfile::tempdir().unwrap();
        let cache = MemoryCache::open(directory.path()).unwrap();
        let (manifest, id, bytes) = fixture(cache.page_size);
        let memory = cache
            .materialize(&manifest, &id, |_| Ok(bytes.clone()))
            .unwrap();
        std::fs::remove_file(cache.entry_path(&id)).unwrap();
        let mut actual = vec![0; bytes.len()];
        memory
            .file
            .read_exact_at(&mut actual, cache.page_size * 2)
            .unwrap();
        assert_eq!(actual, bytes);
    }
}
