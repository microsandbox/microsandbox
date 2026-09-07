# CoW memory and resident lifecycle — 2026-09-07

Follow-up: [execution-state fixes and qualification](execution-state-2026-09-07.md) supersedes the Windows post-restore failure status below and adds Linux ARM64 coverage. The observations below describe the earlier backend revision.

Status: development integration and live smoke coverage, not full platform or performance qualification. Microsandbox #8 remains stacked directly on #7 `ce04099b`, with libkrun `94d680b21bf7ea7c2bed5262ed211833bd4379bd` and firmware `6cca413ac248f63e65d4ea4748b3bc36cd1b22f3`. The kernel and agentd used below were built from matching development sources on the authorized OVH host, including the ARM64 guest artifacts used on macOS and Windows.

## Reproduce

Use `scripts/smoke/cli/cow-memory-lifecycle.py` with `MSB_PATH`, an isolated `MSB_HOME`, matching `MSB_LIBKRUNFW_PATH`, an output directory in `STACK8_OUT`, and a fresh `STACK8_PREFIX`. Select `STACK8_LAYOUT=flat:512M`, `512M`, or `tmpfs`, `STACK8_MODE=cow` or `standard`, and optionally `STACK8_PAUSE_SECONDS=10`. The workload uses Alpine, 256 MiB RAM, and two vCPUs. The runner records each command's elapsed wall time, stdout, stderr, and exit code, and attempts to stop every sandbox it starts in `finally`.

Optional `STACK8_LIVE_RESIZE=1` adds a 512 MiB ceiling and exercises targets 384, 256, 512, and 256 MiB before capture. It checks actual guest MemTotal convergence relative to the baseline within 4 MiB and preserves a tmpfs marker. `STACK8_CHECK_COW_REJECTION=1` checks unsupported-platform rejection without a running/paused fallback. `STACK8_KEEP_ARCHIVE=1` retains the direct archive for diagnosis instead of testing unlink survival. Timed-out commands are recorded explicitly, with partial output, before cleanup runs.

The opt-in language SDK tests are `sdk/python/tests/test_cow_lifecycle.py`, `sdk/node-ts/tests/cow-lifecycle.test.ts`, and `sdk/go/cow_lifecycle_test.go`. Set `MSB_COW_LIVE=1`; Go also requires tags `cow_live microsandbox_ffi_path` and `MICROSANDBOX_FFI_PATH` pointing to its matching native library.

## Observed coverage

Linux/KVM x86-64 passed CoW flat, managed, and tmpfs roots, plus a standard-memory flat-root baseline. macOS/HVF ARM64 passed the same root/memory variants. The checks exercise fresh construction, running full capture, idempotent pause/resume, host-observed Paused status, prompt rejection of new guest exec while paused, two successive full captures while retaining pause, installed-snapshot restore into two children, private child writes, direct full `.msnap` capture/restore, survival after input-archive unlink, and stop from paused. Completed runs stopped their test VMs; retained snapshot/cache artifacts remain in the isolated test homes for inspection.

The later Linux flat/managed/tmpfs/standard runs and macOS tmpfs run additionally assert unchanged Linux boot ID across ordinary pause/resume, resumed progress of the original counter workload, and guest wall clock within three seconds of the host after a ten-second pause. These checks do not constitute host-suspend, every clock-failure, or VM Generation ID notification testing.

The Python, Node, and Go live SDK checks passed on macOS: create with explicit CoW, pause, capture while paused, resume through a handle, restore a child, verify tmpfs contents and source/child write isolation, and stop a paused child. A further Linux flat-root CoW run passed the live-resize sequence described above and the subsequent full lifecycle matrix. This verifies basic capacity convergence and retained marker contents, not physical host-memory reclamation or every unplug/replug invariant.

Windows ARM64 firmware build/export/load checks and native `aarch64-pc-windows-msvc` runtime compilation passed. Explicit CoW rejection passed without starting a VM. Standard-memory lifecycle qualification failed as detailed below; Windows CoW remains unsupported.

### Windows post-restore failure

The first standard-memory flat-root run passed fresh boot, running full capture, pause/idempotence/status/admission checks, two paused captures, resume/idempotence, boot identity, ten-second pause clock correction, workload progress, and two installed-snapshot child restores with write isolation. Direct full archive creation took 10,247.52 ms and produced 20,630,604 bytes. Direct restore returned success after 4,912.00 ms, but the first guest command timed out after 120 seconds. That is a failed usable-restore result, not a 4.9-second successful restore benchmark.

A repeat intended to retain the archive failed earlier: its first installed-snapshot child reported restored in 2,934.15 ms but its first guest command timed out after 120 seconds. No direct archive or archive unlink had been reached, ruling out archive deletion as the sole explanation. Three bounded restores of that retained installed snapshot subsequently all reported activation in 3,066.11–3,182.01 ms and all timed out at the 15-second guest-command bound. Do not infer successful activation implies a usable guest or use these values as successful restore latency.

Trace evidence shows successful generation/clock acknowledgement and workload-thaw exchange, subsequent command bytes delivered to the virtual console, and continuing filesystem device activity during the command stall. The exact cause remains unresolved; the evidence does not establish a whole-VM freeze, archive corruption, or a specific interrupt/agent defect. All four VMs started by the first matrix and both started by the repeat received successful stop responses; cleanup also covered the rejected CoW sandbox record. A process inventory confirmed no remaining `cow8-windows-standard` or `cow8-windows-keep` runtime. Each bounded diagnostic also runs stop in `finally`. A separate fresh diagnostic snapshot reproduced the command timeout on all three children, but its attempted guest task-stack output did not reach `kernel.log` and supplies no task-stack diagnosis. Earlier unrelated development VMs were not terminated.

## Individual debug-build timings

These are single observations, not p50/p95, release benchmarks, or claims of speedup. Build activity and cache warmth varied. CLI wall time includes client/process/setup work and must not be quoted as stop-the-world duration.

| Operation | Linux managed CoW (ms) | macOS managed CoW (ms) | macOS tmpfs CoW (ms) |
| --- | ---: | ---: | ---: |
| First running full capture | 2389.55 | 2000.17 | 1741.27 |
| Resident pause | 9.26 | 19.24 | 14.58 |
| First capture while user-paused | 2679.02 | 2132.41 | 1039.84 |
| Second capture while user-paused | 471.60 | 743.66 | 452.17 |
| Resident resume | 10.46 | 16.22 | 35.71 |
| Restore child A | 158.56 | 572.77 | 259.46 |
| Restore child B | 155.39 | 590.81 | 241.84 |
| Direct full archive capture | 1968.17 | 3050.15 | 2925.98 |
| Direct full archive restore | 396.31 | 1502.71 | 990.42 |
| Stop user-paused source | 111.19 | 119.05 | 116.72 |

The macOS flat run logged APFS reflink reuse for repeated cache construction, including approximately 8 ms for one unchanged paused generation. This is cache preparation only, not total snapshot latency. A warm cache lookup logged 124 microseconds in one restore; that is not end-to-end restore latency or a physical-sharing measurement.

## Other validation and remaining work

Rust SDK library tests: 668 passed, three ignored. Runtime tests: 179 passed. CLI library tests: 315 passed, plus three enabled CLI integration checks; platform-dependent ignored tests remain ignored. Node: 137 unit tests and typecheck passed. Go unit and native-FFI smoke tests passed. Python's focused API/stub tests passed (33 tests). The new backend-binding regression test verifies that pause observation and lifecycle requests use the handle's local backend rather than an ambient backend with the same sandbox name. CoW cache location is also passed from the owning backend at launch.

CoW virtio-mem unplug currently writes private zeros to prevent old backing bytes from reappearing, but does not reclaim those pages' host RAM. NUMA plus CoW is explicitly rejected. These are outstanding integration/performance limitations, not completed acceptance items. Further work includes backing-aware physical reclamation, deeper resize/balloon live invariants beyond the basic Linux sequence, real shared/private resident-memory measurements, cache eviction and publication failure races, cancellation and lifecycle/maintenance concurrency, unsupported guest preparation, recovery/resume failures, fixing and qualifying the Windows post-restore failure, and repeated release-build performance distributions. Public cache inspection/eviction workflow and comprehensive archive compatibility variants also remain to be completed. Do not mark #8 complete from these smoke results.

Evidence locations: OVH `/home/ubuntu/msb-stack8.ElfKzf/`; macOS `/private/tmp/msb-stack8-mac-{results,managed-results,tmpfs-results,standard-results}/`; Windows isolated worktree `C:\Users\Stephen\AppData\Local\Temp\msb-stack8-20260907`. These are development outputs, not shipped artifacts.
