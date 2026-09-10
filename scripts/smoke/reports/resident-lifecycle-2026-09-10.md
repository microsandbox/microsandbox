# Resident pause/resume optimization qualification

## Scope and implementation

This pass implements the approved change from automatic guest-wide filesystem syncing to execution-state preservation: resident pause, full capture, and direct branch no longer call guest `sync()`. Full restore retains dirty guest memory; deliberately extracting only the disk is crash-consistent. Host block draining, sealed-layer durability, workload freezing, heartbeat gating, and clock acknowledgement before workload thaw remain in place.

The CLI avoids a second command-tree construction unless `--tree` is present and uses a current-thread Tokio runtime for pause/resume. Ambient local backend resolution reuses its loaded configuration document. A healthy current-catalog control lookup uses a WAL-aware read-only pool without snapshot reconciliation or writer initialization. Schema/install/downgrade checks remain; older catalogs and stale targets use the existing slow path. Go's name-based pause/resume calls use that observation-free lookup.

The independent libkrun clock-observation change suppresses notifications when the entire observed predicate is unchanged, under the same mutex used by waiters. It passes device unit tests but is **not included in the release binaries benchmarked below**; these use published `msb_krun` 0.1.34. No firmware or protocol version was bumped.

The Node shared-ownership rewrite was approved and implemented in the subsequent [Node ownership follow-up](node-lifecycle-concurrency-2026-09-10.md); it is not part of the CLI/direct measurements below. Retained-handle boot fencing and the Python ownership micro-optimization are not implemented in this pass. Existing runtime-authoritative control remains; this report does not claim to have eliminated the existing name-reuse race for stale retained handles.

## Release benchmark results

Measurements are complete request-to-acknowledgement durations, not isolated hypervisor pause instructions. The direct route opens the same existing control socket and invokes the same runtime operation as the CLI. No snapshot occurs between resident pause and resume. Each entry is the median of 30 samples; CLI/direct order alternates within each fixture. Baseline and candidate each use fresh Small/Large fixtures, run sequentially per host, not randomized across builds. Treat small differences and tail estimates accordingly.

| Host | Profile | Interface | Pause before → after, ms | Resume ACK before → after, ms | Resume through app response before → after, ms |
| --- | --- | --- | --- | --- | --- |
| macOS ARM64/HVF | Small | CLI | 11.980 → 5.305 | 6.582 → 4.836 | 7.236 → 5.417 |
| macOS ARM64/HVF | Small | Direct | 5.531 → 0.410 | 0.166 → 0.160 | 0.615 → 0.518 |
| macOS ARM64/HVF | Large | CLI | 11.957 → 5.348 | 6.449 → 4.970 | 7.081 → 5.585 |
| macOS ARM64/HVF | Large | Direct | 5.561 → 0.428 | 0.183 → 0.175 | 0.652 → 0.547 |
| Linux x86-64/KVM | Small | CLI | 2.717 → 1.916 | 2.795 → 2.078 | 3.223 → 2.532 |
| Linux x86-64/KVM | Small | Direct | 0.275 → 0.228 | 0.347 → 0.343 | 0.800 → 0.804 |
| Linux x86-64/KVM | Large | CLI | 2.773 → 1.944 | 2.748 → 2.018 | 3.182 → 2.486 |
| Linux x86-64/KVM | Large | Direct | 0.307 → 0.244 | 0.307 → 0.307 | 0.763 → 0.767 |

All 480 measured baseline/candidate cycles completed successfully, with application identity, memory state, progress, and SQLite integrity checks. This is 240 candidate cycles and 240 baseline cycles across both hosts, including both interfaces. Small uses one vCPU/256 MiB RAM; Large uses two vCPUs/4096 MiB RAM. These are release builds with matching guest agents; macOS binaries were codesigned with the repository's VM entitlements. The existing bounded stage tracing is unchanged between builds; no new per-pause diagnostic logging was added.

Mac Small direct pause improves about 13.5×; its CLI pause improves about 2.26×. Linux Small CLI pause improves about 1.42×. Direct resume is essentially unchanged, particularly on Linux. The remaining CLI cost must not be described as a slow VM resume primitive. This pass does not rerun CH/FC, isolate each optimization's individual contribution, or qualify p99 latency. Mac Large has one candidate CLI pause outlier of 64.8 ms; median improvements do not eliminate scheduling tails.

## Correctness coverage

The live fixture is `scripts/smoke/cli/dirty-memory-checkpoint.py`. It disables ordinary periodic guest dirty-page writeback for the fixture, sets generous dirty thresholds, dirties a 64 MiB shared mapping, and requires at least 32 MiB reported dirty before capture. It separately retains a private mapping, heap bytes, tmpfs data, and a file explicitly persisted with file/directory `fsync`. Guest settings affect only the disposable test VM.

| Invariant | macOS flat | macOS layered | Linux flat | Linux layered |
| --- | --- | --- | --- | --- |
| Running full capture retains dirty page-cache/shared mmap, private mmap, heap and tmpfs | Pass | Pass | Pass | Pass |
| Eager and forked restore after source shutdown | Pass | Pass | Pass | Pass |
| Running/paused direct branch; source remains paused after paused capture/branch | Pass | Pass | Pass | Pass |
| Mutated direct child stays private; branch-of-branch retains mutations | Pass | Pass | Pass | Pass |
| Dirty incremental capture verified by `capture_mode = incremental`, then restored | Pass | Pass | Pass | Pass |
| Disk-only extraction retains explicitly persisted file and excludes tmpfs | Pass | Pass | Pass | Pass |

The first fixture iteration captured a restored child's initial full baseline and therefore did not prove incremental dirty capture. The tightened fixture creates the child's baseline first, mutates it, checks the next memory manifest explicitly says `incremental`, and restores that generation. Only the tightened run is credited for the incremental invariant. The paused checks validate public paused state and resulting memory; they are not an instruction-entry-counter proof that no guest instruction executed during capture.

The tightened Linux tmpfs run returned an empty control response during baseline capture from a forked child, followed by SQLite disk-I/O errors during cleanup. Its runtime log ends during capture preparation without an explicit panic; no matching kernel crash/OOM report was found. The shared `/tmp` is a 32 GiB tmpfs and was heavily occupied. This is a suspected host-storage/resource interaction, **not an established root cause** and not a fixed product defect. The failed fixture is retained at `/tmp/dirty-42ts79bx` on OVH; none of its runtime processes remained alive when checked. Disk-backed reruns are tracked separately below.

Disk-backed tightened Linux reruns passed both layouts after freeing tmpfs quota and directing temporary files to disk-backed storage. The intermediate disk-backed attempt had also failed at creation with explicit `Disk quota exceeded (os error 122)` because helper temporary files still used `/tmp`; mount inspection confirmed `/tmp` has `usrquota`. Two completed, stopped earlier fixture homes were moved intact from `/tmp/dirty-aic065db` and `/tmp/dirty-1ob_e60_` to `retained-dirty-flat` and `retained-dirty-layered` under the remote stage, freeing about 5 GiB without deleting their artifacts. Final reports are `dirty-flat-disk-r2/report.json` and `dirty-layered-disk-r2/report.json` under that stage. The quota failure is established; attribution of the earlier empty-response incident specifically to it remains an inference, not a proved runtime fix.

Other checks passed: 17 Linux guest-freezer tests (predicate/event races, EINTR, error paths, bounded fast polling/backoff, timeout and latch ownership); 4 read-pool tests (including WAL visibility and write refusal); 4 control-lookup tests; 25 backend/profile tests; 2 control-socket reply tests; 15 libkrun clock-device tests; Go native binding `cargo check`; CLI `--tree`, `pause --tree`, and `pause --help`; focused Rust formatting and `git diff --check`. The restricted socket test initially failed to bind sockets and passed when rerun with appropriate permissions. The Go check initially lacked network/prebuilt artifacts and passed with an isolated development bundle and dependency access.

Windows ARM64 and Linux ARM64/KVM were not live-tested in this pass. The subsequent [Node ownership follow-up](node-lifecycle-concurrency-2026-09-10.md) qualifies same-object exec/filesystem/lifecycle concurrency on Mac and Linux x86-64. Other unqualified cases include runtime-share custom files/mappings, detailed injected block-queue/ENOSPC failures, and a full per-language SDK timing matrix. Do not present these reports as exhaustive cross-platform correctness qualification or completion of every research recommendation.

## Artifacts and provenance

Base Microsandbox commit: `18c8fb8693957ac6e8ded964e42cfbe63f8506ad`. Worktree: `/private/tmp/msb-cow-8.PIhgYp`, branch `appcypher/cow-memory-lifecycle`. Release source staging copied that commit plus the lifecycle implementation files only, excluding concurrent incremental-archive changes in the shared checkout. Test-only/formatting additions made after staging do not change the measured runtime behavior. Qualification preceded the separately authorized commit and push.

- Local build, harnesses, raw benchmark JSON, and live reports: `/private/tmp/msb-resident-perf.qHpm0p`.
- Linux build, raw results, and logs: `/home/ubuntu/msb-resident-perf.zqr4vR` on OVH.
- macOS candidate binary SHA-256: `23d7d9db282f63cc9d1ee1b1b2e49bed0661814f3cce3e98cbde2f5e4bc2dd101`.
- macOS baseline binary SHA-256: `82eb66d6cfc6370f172c3af597913069d03cb197688ee129bdde125308a01126`.
- Linux candidate binary SHA-256: `f87b4063dd1adc64ce996d306e0d334c3816f7ff9ca3b7a9474dad4acce25372`.
- ARM64 guest agent SHA-256: `1425dc4b6974c10983db03e023c2b868c890fc197857ea45d6289a83df593aa8`.
- x86-64 guest agent SHA-256: `ae70a9d63c7df953340c8d6dc2c674ccad18e9f490c114977ba7972fe63bb6d0`.
- macOS firmware SHA-256: `ea0d458cdc12a0fa6dac8d192542ddc39717f816da41176582905e31a8bf868c`.
- Companion libkrun worktree: `/private/tmp/krun-registry-release.Ordgfp`, baseline `b20d31aba2fcf996512b0540bde0176dd7db7ad8`; only the clock-observation file is modified.

Passed live fixtures stop their own VMs in `finally`; artifacts are retained for inspection rather than deleting unrelated test data. Initial harness failures (old system Python, prohibited inherited host ports, and incorrect layered-root CLI spelling) are retained as failed attempts and are not counted as product passes.
