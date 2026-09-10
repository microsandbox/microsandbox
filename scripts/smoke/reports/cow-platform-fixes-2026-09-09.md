# CoW platform fixes — 2026-09-09

Development qualification of the #8 changes on top of Microsandbox `49b6670d`. The tested companion sources are published and pinned to libkrun `862d68422b40ac7c26c731b3577025a5b1f64b89` and rust-vmm `f798d4f274db22a3c458ba756900db2cd03e6fe9`. Firmware is unchanged from the matching execution-state test build. The live test builds used explicit local source overrides for those sources before publication. These are debug-build correctness runs, not release performance benchmarks.

## Changes

- Windows uses native private file views for restored RAM. All GPA slots are slices of one owned view, so 4 KiB slot offsets do not require separate 64 KiB-aligned mappings. Siblings cannot change each other's RAM or the backing file. The final view owner unmaps it exactly once.
- The Windows RAM cache has owner/SYSTEM-only directory ACLs, immutable reader sharing, shared lifetime pins, and identity-checked exclusive eviction. The completed writer closes before readers open the published file.
- Pending restore intent survives database insertion and process failure. It is cleared only after restore activation and create finalization. A failed child cannot later start as an ordinary VM, auto-start through exec, grow or compact its staged disk, or create a snapshot. Remove/recreate it from the original input. Successful children retain normal later stop/start behavior.
- ARM64 KVM restores distributor control before pending levels and interrupt enables, with every vCPU still paused. KVM's userspace `GICD_CTLR` setter changes the enable flag without requeueing pending interrupts; enabling it last can strand an already-high timer line. See the [Linux VGIC implementation](https://github.com/torvalds/linux/blob/v6.12/arch/arm64/kvm/vgic/vgic-mmio-v3.c).
- Resume acknowledgements now get the configured one-second barrier deadline. Only pause uses ten-millisecond sub-waits for periodic kicks. This corrects an accidental ten-millisecond resume deadline, without adding delay to successful acknowledgements.

## Live coverage

| Check | macOS ARM64/HVF | Linux x86-64/KVM | Windows ARM64/WHP |
| --- | --- | --- | --- |
| Full eager/forked restore, flat and layered roots | Pass | Pass | Pass |
| Direct full archive capture/restore and input-archive unlink | Pass | Pass | Pass |
| Pause/resume, idempotence, capture while paused | Pass | Pass | Pass |
| Source/sibling RAM isolation and modified-child capture/restore | Pass | Pass | Pass |
| Direct branch, paused-source branch, branch of branch; flat/layered/tmpfs | Pass | Pass | Pass |
| Independent backing pins, final release, same-name race | Pass | Pass | Pass after porting the test's Unix-only lock probe |
| Failed restore refuses start/exec/modify/compact/snapshot; sealed bytes unchanged | Pass | Pass | Pass |
| Fresh restore after failure, then ordinary stop/start | Pass | Pass | Pass |
| Retained-sibling timer-progress regression | 20/20 | 20/20 | 20/20 |
| Branch after live root growth and compaction | Previously passed; not added to final Mac rerun | Previously passed; not added to final x86 rerun | Pass, flat and layered |
| Delayed incremental archive forked restore: wall clock, monotonic/boottime, timers | Prior coverage retained | Prior coverage retained | Pass |
| Offline CPU retained and subsequently onlined after forked restore | Prior eager coverage retained | Prior eager coverage retained | Pass |

Linux ARM64 nested-KVM also passed all eight final matrix suites, including branch-after-growth/compaction for flat and layered disks. Its failed-restore test passed every refusal, snapshot byte preservation, fresh restore, and later stop/start. Delayed incremental archive forked restore passed wall-clock, monotonic/boottime, and relative-timer checks. The offline-CPU forked restore and later CPU online check passed. The committed timer-progress fixture passed another 8/8 iterations after the 100-iteration diagnostic run. Windows x86-64 was deliberately not tested. This report does not claim every possible failure injection, memory-pressure scenario, NUMA configuration, or SDK-language/platform combination has been qualified.

## ARM64 diagnosis and regression

The original retained-sibling loop repeatedly hung after successful restore activation. The same failure occurred with forced full RAM capture, eager anonymous memory, and a single vCPU; those experiments did not fix it. A cold-start control and an ordinary pause/resume control each passed 100 iterations.

A stuck guest had both CPUs idle, virtual timers enabled/unmasked with expired deadlines, and timer interrupt line levels high, but no active interrupt. Reasserting the pending interrupt recovered workload progress in a disposable diagnostic guest. That experiment was not shipped as a workaround. Inspection of KVM's userspace distributor-enable semantics identified the ordering bug above. No forced eager fallback, synthetic timer injection, periodic wakeup, or disabled incremental capture remains in the production changes.

The corrected-order run first exposed the separate resume-deadline bug. With both fixes, a subsequent run passed 20 retained-child iterations before the disposable 32 GiB VM exhausted disk space. Unpinned local RAM cache entries were evicted under exclusive file locks; snapshots and pinned backings were not removed. A fresh run then passed all 100 retained-child iterations, including first exec and continuing timer-driven workload progress, with no failed cleanup. The final broader ARM64 matrix and focused regressions also passed.

## Reproduce and evidence

- `scripts/smoke/cli/failed-restore.py`: requires `MSB_TEST_DISPOSABLE_HOME=1`, deliberately obstructs only the disposable memory cache, retains the failed database row, verifies every refusal and sealed-file digest, and tests a fresh child's later stop/start.
- `scripts/smoke/cli/branch-timer-progress.py`: repeated direct branches, retained siblings, atomic counter publication, first-command success and continued guest timer progress. Set unique `STACK8_PREFIX`, `STACK8_OUT`, optional `STACK8_REPEATS` and `STACK8_RETAIN`, plus the usual runtime/home/firmware environment.
- Existing `cow-memory-lifecycle.py`, `direct-branch.py`, `branch-ownership.py`, `checkpoint-clock.py`, and `checkpoint-cpu-state.py` supply the broader matrix. `CPU_FORKED=1` selects forked restore for the offline-CPU fixture.
- Mac final matrix: `/private/tmp/msb-cow-fix-final-mac-results`; failure/restart: `/private/tmp/msb-cow-fix-restart-mac`; timer stress: `/private/tmp/msb-cow-fix-mac-stress`.
- OVH final matrix: `/home/ubuntu/msb-cow-fix-final-results`; failure/restart: `/home/ubuntu/msb-cow-fix-restart-results`. A separate benchmark task overlapped some qualification work; do not use these timings as an uncontended performance comparison.
- Surface evidence under `C:\Users\Stephen\AppData\Local\Temp`: `msb-cow-fix-49b6670d\results`, `msb-cow-fix-restart-results`, `msb-cow-fix-maint-flat`, `msb-cow-fix-maint-layered`, `msb-cow-fix-clock-results`, `msb-cow-fix-cpu-results`, and `msb-cow-fix-win-stress`.
- Nested ARM64: `/root/msb-cow-fix-qualified-results` and `/root/msb-cow-fix-arm-final-results` in the disposable QEMU VM; text-only evidence copied to `/private/tmp/msb-cow-fix-evidence/arm64` on the Mac.

All runners stop their own test VMs in `finally`; final process checks found no remaining runtimes from this pass on the Mac, nested ARM64 VM, or Surface. Three pre-existing Surface branch-5 runtimes were left untouched. The disposable nested ARM64 VM was shut down after evidence collection. Unrelated development VMs are not targets for cleanup. Raw artifacts can contain guest state and are not committed.

## Other validation

After publication, the Mac CLI build passed with the pinned Git dependencies and no local source overrides: `cargo build -p microsandbox-cli --no-default-features --features net,ssh`. The generated lockfile changes only the twelve intended dependency sources. Formatting, diff whitespace checks, and Python smoke-script compilation passed.

The Mac libkrun VMM library tests passed 52/52, including the resume-deadline regression. ARM64 VGIC tests passed 3/3, including the distributor/pending-state ordering regression. The failed-restore database regression passed. Runtime checkpoint tests passed 31/31 on Mac and Windows. The Windows public Rust mapping harness passed alias/sibling/backing isolation, bounds, parent-drop, and mapped-file-unlink checks. The standalone vm-memory unit harness cannot currently compile on Windows because its existing vmm-sys-util development dependency references Unix clock APIs; this is not reported as a passing unit run. The production mapping code was exercised by both the standalone Rust binary and the live WHP matrix.
