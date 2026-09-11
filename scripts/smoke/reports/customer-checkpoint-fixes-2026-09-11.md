# Customer checkpoint fixes — partial qualification

This report tracks Microsandbox #1557 (base `releases/v0.7.0`) and libkrun #130 (base `krun`), not completion of all six customer issues. No packages were released. Earlier results below identify the earlier dependency pin; the September 11 follow-up results used matching local development sources and freshly built guest agents.

The low-level follow-up is committed and pushed as libkrun `9a9aa965f65f872578f940a20d75cfea5086d834`. This Microsandbox integration uses that exact Git revision without local dependency overrides. The approved eager preparation/activation separation and public Stop contract are now implemented in the follow-up candidate. Mac qualification passes, but Linux exposed a missing guest-kernel poweroff handler previously hidden by the removed host kill timer. Stop is therefore not Linux-qualified or merge-ready. A firmware fix requires approval to expand beyond the originally requested two PRs.

## Current six-issue status

| Customer issue | Implementation and evidence | Remaining boundary |
|---|---|---|
| 1. Unused `max_memory` dominates capture/restore | Capture skips actual unplugged blocks, unplug acknowledgement follows zeroing, and eager restore retains the fresh-zero-mapping optimization. Mac/Linux ceiling read counts and shrink/replug checks pass. | Full 8 GiB boot/256 MiB dirty performance distributions and Windows live qualification are not established. |
| 2. Checkpoint after memory growth cannot restore | Original construction geometry, actual block occupancy, requested targets and CPU state are restored separately; eager/forked/branch checks pass. | The small-boot Mac pressure case below remains a failed resize cell, not a qualified pass. |
| 3. Additional mounts prevent full capture | External directory/file providers, strict/relaxed remapping, and private restored managed disks are implemented. See detailed coverage below. | Open-unlinked/special external objects fail closed; relaxed clean guest cache is not forcibly invalidated. |
| 4. Ubuntu live ext4 growth returns EIO | Bounded `WRITE_ZEROES + UNMAP` uses the budgeted explicit-zero writer. Ubuntu 10→12→14 GiB and checksum checks pass. | Recovery of an already partially failed filesystem was not live-tested. |
| 5. Stop then remove races runtime teardown | Public Stop waits for same-run ownership release; its explicit timeout never kills. Rust, Python, Node, Go and Ruby expose the contract. Mac Stop→Remove and zero-budget safety pass. | Linux x86 guest poweroff halts without exiting the VMM; the kernel completion path must be fixed before this candidate can ship. Older running runtime/agent bundles retain their older shutdown implementation. |
| 6. Slow first forked restore times out misleadingly | Preparation progress, delayed first restore, cancellation/reaping, catalog reconciliation, and structured diagnostics are implemented. Eager RAM/CPU/device reconstruction now finishes before Activating and bounded activation waits begin. | The pre-existing cancellation exposures listed below are not claimed fixed. Windows/Linux ARM64 have not been live-rerun for this follow-up. |

## Unused-capacity capture and shrink/replug

These debug-build causal checks booted 256 MiB guests, not the customer's full 8 GiB workload. The increasing ceiling did not increase guest RAM reads. Times are single first-capture observations, not release latency percentiles or pause-time comparisons.

| Host | Ceiling | Guest bytes read | Unplugged bytes skipped | Capture measurement |
|---|---:|---:|---:|---:|
| macOS ARM64 | 8 GiB | 268,435,456 | 8,321,499,136 | 0.641 s RAM capture; 1.060 s CLI |
| macOS ARM64 | 16 GiB | 268,435,456 | 16,911,433,728 | 0.650 s RAM capture; 1.051 s CLI |
| macOS ARM64 | 32 GiB | 268,435,456 | 34,091,302,912 | 0.637 s RAM capture; 1.036 s CLI |
| Linux x86-64 | 8 GiB | 306,774,016 | 8,321,499,136 | 0.68 s CLI |
| Linux x86-64 | 32 GiB | 306,774,016 | 34,091,302,912 | 0.69 s CLI |

The x86 guest has an additional fixed boot region. Mac also passed eager/forked restore from the 32 GiB-ceiling snapshot (0.708/0.740 s), 768→384 MiB shrink, restore, replug to 768 MiB, incremental capture and branching, with RAM checksums. Linux passed shrink, eager/forked restore, replug with a new 384 MiB random payload, incremental capture, and forked restore of that delta. Linux's first forked trial hit `EDQUOT` on the host tmpfs; the successful repeat used ordinary disk storage without changing quotas. Low-level tests cover failed unplug without ownership acknowledgement, private-backing sibling isolation, and incremental zero extents replacing stale baseline bytes.

Evidence: Mac `/tmp/cbh-memory-_u7wznkw/results.json`; OVH `/tmp/cbh-memory-ceilings.uLp2Zn`, `/tmp/cbh-1rn1csmz`, and `/home/ubuntu/msb-customer-fixes.tjHWpS/memory.U5oKoi`. All owned VMs were stopped. Three derived Linux `.ram` cache files were removed to recover roughly 1.2 GiB; authoritative snapshot payloads, manifests and logs remain, so those caches are regenerable.

### Unresolved small-boot kernel-zone pressure

The latest Mac flat-root case booted 256 MiB, grew to 768 MiB, filled 384 MiB of tmpfs, captured, eagerly restored and verified the checksum. A subsequent 1 GiB/CPU1 request exposed a kernel allocation panic. Isolated repeat restores retained checksums, but memory growth plateaued at 896 MiB and guest `dmesg` reported hotplug `ENOMEM`. The snapshot already contained a pre-capture vmemmap allocation warning. Only about 2.65 MiB of non-CMA boot-zone memory remained; hundreds of MiB free in the Movable zone could not satisfy those kernel allocations.

Replacing only a disposable child's payload with a 16 MiB sentinel restored approximately 136 MiB of boot-zone headroom; that same restored VM then reached 1 GiB/CPU1 and retained its checksum. A fresh non-restored control also passed but had more initial kernel-zone headroom, so it was not an equal-pressure control. The original panic's exact timing was not reproduced. No kernel zoning/minimum-memory policy was changed to hide this limitation. Evidence: `/tmp/cbh-cpvu7r2j/{results,memory-diagnostics,memory-convergence,memory-kernel-probe,memory-causal-controls}.json`; all diagnostic children stopped.

## Slow preparation, diagnostics and cancellation

Mac held the RAM backing cache lock for 185 seconds, then released it. First forked restore completed in 187.631 seconds with visible waiting progress and the captured RAM checksum intact, demonstrating removal of the old 180-second preparation limit for this path. Cancelling a separate restore while it waited on that lock terminated/reaped the owned runtime and reconciled its catalog in 73 ms; immediate removal then succeeded. Evidence: `/tmp/cbh-9k68spkd/slow-preparation.json` and `sdk/rust/tests/creation_progress_live.rs`.

Preparation errors are persisted before startup-channel EOF; asynchronous activation failures also publish their actual cause before process exit. Tests cover delayed diagnostic publication and activation-channel completion. Failed pin/workdir finalization uses lifecycle-lock-gated rollback, and Starting sandboxes protect referenced volumes against deletion. A newly added additional-disk copy worker owns unique temporary storage until atomic publication; its cancellation/replacement-sentinel test passes.

The TypeScript ordinary create and both progress terminals now preserve the returned native handle's lifecycle ownership, so an automatically detached restore is not stopped by wrapper disposal. Go's synchronous progress open/close no longer allocate cancellation tokens that those native functions cannot consume; cancellation after allocation closes the exact returned stream before reporting the error. Creation and receive retain their existing cancellable paths. These follow-up SDK fixes are included in this integration.

Cleanup is not claimed universal: initial named-volume provisioning before the creation guard, pre-existing root-layer/overlay blocking workers, and cancellation of an in-flight direct-branch capture still need separate ownership audits/fixes. Bounded termination that cannot confirm exit retains durable restore intent/storage, not every in-memory lease indefinitely. The public `StopTimeout` error is a source-compatibility change for exhaustive Rust error matches. Timeout/cancellation of the wait itself does not kill; dropping the last attached lifecycle owner or an independently configured lifetime policy can still terminate its VM.

### Eager preparation boundary and Stop follow-up

The terminal `Activating` startup event now follows the real construction pause inside `Vm::enter`, after eager RAM plus CPU/device reconstruction. Loading progress counts required object-backed RAM slices, excluding untouched zero reservations. Completing the RAM byte count alone does not start activation. Existing clock acknowledgement/thaw/readiness bounds remain, and cold-boot startup timing is unchanged. The `restore_activate.total_us` diagnostic now excludes construction waiting, which remains separately available as `wait_paused_us`.

Mac ARM64/HVF used a fresh 256 MiB Alpine guest and a 4 MiB random tmpfs payload. Eager and forked children passed checksums, explicit zero-timeout errors with subsequent successful commands, paused-Stop refusal, resume/checksum, and ordinary Stop followed immediately by Remove. The repeat run measured eager creation at 826 ms and warm forked creation at 207 ms; Stop took 108 ms, and removal 1–2 ms. First-run eager/cold-forked creation took 2,716/543 ms. These are single debug-build diagnostic timings under development-host load, not release percentiles or a before/after benchmark. Evidence: `/private/tmp/cbh-activation.yhQ7pK/mac-live.log`; all owned guests removed, snapshot retained.

Fresh matching Python and Node native bindings also passed both live Sandbox and persisted Handle receivers: zero timeout produced the typed error without stopping the VM; ordinary Stop→Remove succeeded. Python retained `stop(timeout=0)` behaves identically. Matching Ruby passed one live test with 15 assertions, including both receiver types and invalid-duration rejection; its isolated catalog was empty afterward. Evidence: `/private/tmp/msb-stop-node.ehqO40` and `/private/tmp/cbh-rb.v2y58E/results.md`.

Linux x86-64's test-only preload shim delayed one real eager checkpoint-object read by 15 seconds without changing product deadlines or saturating storage. The VM passed reconstruction and the captured checksum; the actual Activating frame followed the delayed read, not its start. Injected object EIO retained the original `restore memory: Input/output error (os error 5)`, with no Activating event; SDK task cancellation during a 45-second delayed read reaped the exact runtime and reconciled ownership/catalog before the delay ended. The combined successful-restore test initially failed later, only at its separate Stop guard; that result is not a whole-test pass.

The final test revision separates activation verification from graceful Stop qualification and explicitly labels its disposable-child cleanup as forced. It compiled locally but was not rerun on Linux: uploading that revised test was blocked pending source-transfer authorization. The earlier live restore, error and cancellation evidence above remains valid; it is not a passing result for the revised whole test. Linux fixtures and exact runtime PIDs were cleaned up. Only validated generated compiler caches were removed to recover build space; source, snapshots and failure evidence were retained.

### Linux graceful-poweroff blocker

The fresh source received `core.shutdown`, completed agentd teardown, and called `reboot(RB_POWER_OFF)`. Its kernel reported `Power off not available: System halted instead`; the KVM runtime retained ownership until the test's explicit emergency cleanup. The ordinary Stop guard expired at 120 seconds. This is not acceptable as a qualified Stop result, and no forced cleanup is counted as success.

The kernel converts poweroff to halt when no poweroff handler exists. The correct proposed repair is a krun-specific x86 kernel poweroff handler using the existing i8042 exit transport after normal kernel shutdown ordering. Ordinary HLT is not a completion proof; changing agentd alone to reboot would not correctly cover foreign PID1 poweroff. No new device or agent protocol is needed, but the firmware/kernel change is outside the original two-repository scope and remains approval-gated. Mac's native shutdown path passed; Windows and Linux ARM64 were not rerun here.

## Grown-memory restore

The disposable CLI fixture creates a 256 MiB VM with a 1 GiB ceiling and one CPU with a two-CPU ceiling, grows it to 768 MiB/two CPUs, and fills tmpfs with 384 MiB of random data. That payload cannot fit in the original boot RAM. A full checkpoint must reconstruct the original guest-physical layout plus the captured hotplug occupancy, rather than booting a new 768 MiB layout.

Both macOS ARM64/HVF and Linux x86-64/KVM passed eager restore, forked restore, and direct branching with the captured RAM checksum intact. Both restored children then grew to 1 GiB and shrank to one CPU; the fixture verifies guest CPU online state and checks the payload again. This caught and fixed a configuration projection bug that otherwise caused a subsequent CPU change to be silently skipped.

| Debug-build CLI duration | macOS ARM64 | Linux x86-64 |
|---|---:|---:|
| Full capture | 3.312 s | 3.260 s |
| Eager restore | 1.442 s | 1.796 s |
| First forked restore | 1.413 s | 1.795 s |
| Direct branch | 1.662 s | 1.106 s |

These are single correctness-run wall-clock timings, not release performance benchmarks, pause durations, or a before/after speedup claim. The tests used matching freshly built agentd binaries. The reported run predates only the final additional resource-query deadline bound; its dependency sources match the pinned companion commit.

Reproduce with `python3 scripts/smoke/cli/checkpoint-memory-growth.py /absolute/path/to/msb /absolute/path/to/agentd /absolute/path/to/libkrunfw`. The exact Mac binary must be codesigned with `msb-entitlements.plist`. `CBH_ROOT_DISK=flat:512M` selects a flat root; the first runs reported above used the layered root.

Evidence: Mac `/tmp/cbh-j3swr8nn/results.json`; OVH `/tmp/cbh-ux6l5nzg/results.json`. The final pinned build, including the resource-query deadline bound and no local dependency overrides, also passed the entire fixture with flat roots on both hosts: Mac `/tmp/cbh-d2__po51/results.json`, OVH `/tmp/cbh-840jpz18/results.json`. All fixture sandboxes were stopped. Raw RAM/disk artifacts are not committed.

## Automated checks

The final September 11 development sources passed 153 macOS libkrun device tests, 47 VM API tests, and 12 memory tests. Against the pushed Git pin, all 86 SDK snapshot tests, 30 creation tests, and three runtime startup-diagnostic tests passed. The first restricted creation run could not bind two Unix socket fixtures; the permitted rerun passed all 30. Runtime unit tests used the matching local build artifacts rather than downloading a released agent. Additional-disk cancellation passed its private-worker/replacement-sentinel regression. Node passed all 219 unit tests, including six native-ownership/disposal cases, and both TypeScript declaration checks. Go passed all packages and six deterministic progress-allocation tests under the race detector. Those Go helper tests use isolated fake streams, not an instrumented native registry.

The follow-up candidate passed 820 Rust SDK tests (three opt-in tests ignored), 296 runtime tests, strict Rust/Python/Node/agentd Clippy, 225 Node tests with rebuilt native bindings, 129 Python tests (two opt-in tests skipped), and 16 Go native tests including bounded identity lookup. Go package tests and actual old-native refusal checks passed. Ruby compiled against the in-tree SDK; its new API regression passed, while the full Ruby unit suite passed 16/17 and the existing fork-rebuild test returned empty child output. This Ruby failure is not silently reclassified as a pass. The following counts describe earlier pinned builds:

- macOS libkrun: 144 device tests, 11 VM API tests, and the eager fresh-mapping topology regression passed.
- Linux libkrun: 164 device tests passed with the existing hanging vsock quiescence test excluded; the new bounded WRITE_ZEROES test passed. The fresh-mapping topology regression passed on Linux x86-64 as well as Mac ARM64.
- Microsandbox: 38 image checkpoint tests passed, one opt-in experiment ignored; 19 restore-related SDK tests passed, including complete control-frame delivery, captured requested-versus-actual targets, and preservation of concurrent desired configuration edits.
- The final pinned build passed 81 SDK snapshot tests and 53 runtime checkpoint tests. These are automated checks, not substitutes for VM qualification.

## Ubuntu live disk growth

`scripts/smoke/cli/bounded-root-grow.py` passed on Linux/KVM with a fresh Ubuntu 24.04 managed raw upper and default writeback limiting. The same running VM grew 10 → 12 → 14 GiB; ext4 reported both filesystem expansions complete, and a 32 MiB random file retained its checksum. CLI modify durations were 26.96 ms and 13.80 ms, single debug diagnostic trials. Repeating an already-completed target correctly retained the existing grow-only rejection. This does not test recovery from a previously partially failed resize.

Evidence: OVH `/tmp/cbg-t14q2e3t/results.json`. An initial 10 → 12 GiB trial also passed, but its fixture incorrectly expected equal-target success; the fixture now asserts the existing rejection. A separate retry hit the host temporary-storage quota. Only three validated, stopped, disposable test homes from this session were removed to reclaim space; earlier JSON results were retained, but those synthetic RAM/disk artifacts are not recoverable. The final test VM was stopped successfully. This work does not disable bounded writeback or modify the guest image/kernel.

## External directory and isolated-file mounts

The macOS ARM64/HVF qualification used a codesigned debug CLI, the matching ARM64 guest agent, Alpine, 256 MiB guest memory, and task-owned host paths. Directory and isolated single-file mounts both passed full capture with an open read/write descriptor, strict same-host restore, explicit destination remapping, and direct branching. The single-file remap used a different host basename while retaining its captured guest filename. Restored descriptors read the expected bytes and wrote only the selected remap targets; the original host contents stayed unchanged.

| Case | Observed result |
|---|---|
| Referenced file changed after capture | Strict restore refused before activation with the stale object ID. |
| Explicit relaxed restore of changed file | Structured warning identified the guest path and stale inode; old handles could not overwrite newer host bytes. Fresh lookups could see the current namespace. |
| Missing directory or single-file export | Relaxed restore retained the mount/device and returned `EIO`; it did not recreate the host path or expose the covered guest directory. Strict restore returned a mount-specific missing-path diagnostic. |
| Mixed directory and single-file portable archive | Full capture, `snapshot save --with-image`, and direct `create --from-snapshot archive.msb` with both explicit mappings passed. Both inherited descriptors worked. |
| Source explicitly paused before capture | Capture succeeded and `inspect` still reported `paused`; no temporary user-workload resume was required. |

The final-format mixed fixture is `/private/tmp/cbh-mounts.YeZ4yT/snapshots/ext-final/snap_ea7952b86bde0963433b56a7cdf14125`; its standalone archive is `/private/tmp/cbh-mounts.YeZ4yT/external-final.msb`. It used `/private/tmp/msb-customer-fixes.uHd8b5/msb-customer-live-final1`. This rerun did not reproduce the earlier debug Tokio stack overflow during direct archive creation. Earlier directory and single-file matrices remain under the same disposable home. All completed mount test guests were stopped; test data and archives were retained, not committed. These archive runs precede the final external-flush timeout/worker-reservation correction and relocation of host-only authorization/warning records outside the guest-writable runtime share. Neither changes the captured external format; older development source-local records are not migrated, so those fixtures need explicit mappings with the final layout.

A fresh final-source Mac rerun then passed full capture (0.98 s), automatic same-host eager restore (0.61 s), explicit directory/different-basename file remapping (0.61 s), and relaxed restore of changed objects (0.66 s). Both inherited read/write descriptors worked. Remapped writes left originals unchanged; same-host writes intentionally used the shared originals. Authorization and healthy/degraded warning records existed only in the host sandbox directory and were absent from physical `runtime/` and guest `/.msb`. The CLI displayed both stale-object warnings. Evidence: `/private/tmp/cbh-final3.Irtfrj/results.md`; all four guests stopped. The tested signed binary was `msb-customer-live-final3` with matching `agentd-final-arm64`.

Final matching Linux x86-64/KVM binaries passed a mixed directory/single-file fixture after those corrections: full capture with open descriptors (0.648 s), strict same-host eager restore (0.635 s), and explicit remapping including a different file basename (0.631 s). Restored inherited descriptors read and wrote only the selected remap targets; originals remained unchanged. Missing directory and file exports each failed strict admission with mount-specific ENOENT; relaxed admission warned, returned guest EIO, and did not recreate host paths. An unaffected directory remained readable when only the file export was unavailable. Host-only warning records were verified in `sandboxes/<child>/restore-mount-warnings.json`. All fixture VMs stopped. Evidence: OVH `/home/ubuntu/msb-customer-fixes.tjHWpS/e._cuial7h/results.json`. These small correctness-run timings do not establish a performance distribution, Linux archive/branch coverage, or Windows execution.

Automated qualification passed all 654 filesystem tests, all 61 runtime checkpoint tests, and three SDK named-directory admission tests. The Windows ARM64 filesystem test target compiled successfully; Windows execution is not claimed here. Focused tests cover source path replacement, destination reopen races, exact handle identity after guest rename, symlinks without following outside targets, malformed state rejection even under relaxed admission, and single-file sibling isolation. Named-directory tests prove that restoring a missing catalog entry does not recreate its record or path, inherited create-if-absent intent is removed, and an external filesystem transport cannot silently become a named disk. Host-record tests prove guest-writable lookalike files cannot supply source authorization or replace restore warnings.

Strict admission remains the default. Portable snapshot data does not authorize arbitrary host paths: restore needs an explicit existing volume binding or an exact source-local authorization record. Mappings retain captured guest mount flags and must satisfy the provider's existing quota/configuration requirements. External contents stay shared dependencies; no host-tree snapshot, copy, ejection, or silent conversion to managed storage is performed.

Capture fails closed for open-unlinked external objects, special objects, unsupported backend state, or missing guest writeback evidence. Reopenable guest-renamed objects and symlink objects are supported; link targets are not recursively hashed or copied. Relaxed mode permits captured clean cache pages: a read or memory mapping can return saved bytes until it contacts the backend, where stale IDs return `ESTALE` and unavailable exports return `EIO`. It is not an immediate guest cache-invalidation guarantee. Dirty guest writes must be synchronized before capture; a timed-out flush is not clean evidence. Ordinary input gating/control/thaw retain their 10-second budgets, while an external freeze request has 30 seconds after gating to accommodate the guest's bounded 20-second flush. A blocking flush retains its reservation after an async timeout, preventing a later capture from certifying clean while that worker is still active.

## Additional managed disk volumes

The macOS ARM64 layered-root fixture passed with two separate 128 MiB managed disk volumes mounted at `/data` and `/other`. Each held an 8 MiB random payload and checksum. Full capture, eager restore, forked restore, direct branch, and a branch of that branch preserved both payloads. Writes in restored children did not change the source volume generations. A capture from a user-paused source and subsequent resume preserved the data.

Standalone and dependent archive save/load passed, including loading the dependent archive before its base and then restoring the imported checkpoint. A cold boot with `--disk-only` from a full snapshot also passed after explicit guest `sync` before capture. That sync is significant: a full snapshot retains dirty guest disk cache in RAM, whereas discarding RAM for disk-only boot provides the captured disk's crash-consistent state, not a promise to retain unsynchronized application writes.

Layered-root evidence is `/tmp/cbh-volumes-5hurajlt/results.json`; all commands completed successfully and its guests were stopped. The updated flat-root fixture also passed at `/tmp/cbh-volumes-ygdvqumh/results.json`, including a fresh destination home with `--pull never` and explicit paused-state assertion after capture. It caught and fixed `--with-image` incorrectly requiring layered EROFS files for a flat-only cache: flat exports now bundle image configuration, while layered roots still require their complete base-image cache. Flat debug timings were 2.193 s full capture, 1.054 s eager restore, 1.115 s first forked restore, 1.826 s branch, and 1.600 s offline imported restore. These are single correctness-run measurements, not release benchmarks. Reproduce with `python3 scripts/smoke/cli/checkpoint-managed-volumes.py /absolute/path/to/msb /absolute/path/to/agentd /absolute/path/to/libkrunfw`; `CBH_ROOT_DISK=flat:512M` selects the flat root. All fixture guests were stopped.

Managed additional disks use same-pause-epoch immutable generations and child-private restored copies without rebinding or changing ownership of the source volume. External/shared block disks, unsupported disk formats/chains, or missing managed provenance continue to fail explicitly rather than silently acquiring snapshot ownership. This qualification does not establish cross-host or Windows execution for additional managed disks.

## Still required

The complete six-issue fix is not merge-ready: Stop integration exposed the missing Linux x86 poweroff handler described above. Eager preparation/activation separation is implemented and its independent error/cancellation paths are tested. Unplug-zero ordering and external/managed-volume providers are implemented; do not confuse focused checks with universal qualification. Small-boot kernel-zone pressure and the pre-existing cancellation paths remain explicit limitations. Large-ceiling release distributions and complete Windows/Linux ARM64 live matrices are not claimed. The Git-only companion dependency also still needs the separately authorized release/pin workflow before registry publication.

Full checkpoints now require original construction geometry and CPU/memory hotplug device records. The user approved rejecting earlier development full snapshots that lack them. Released disk-only snapshot formats are unchanged.
