# CoW memory and resident lifecycle — 2026-09-07

Status: development integration and live smoke coverage, not full platform or performance qualification. Microsandbox #8 remains stacked directly on #7 `ce04099b`, with libkrun `94d680b21bf7ea7c2bed5262ed211833bd4379bd` and firmware `6cca413ac248f63e65d4ea4748b3bc36cd1b22f3`. The kernel and agentd used below were built from matching development sources on the authorized OVH host, including the ARM64 guest artifacts used on macOS and Windows.

## Reproduce

Use `scripts/smoke/cli/cow-memory-lifecycle.py` with `MSB_PATH`, an isolated `MSB_HOME`, matching `MSB_LIBKRUNFW_PATH`, an output directory in `STACK8_OUT`, and a fresh `STACK8_PREFIX`. Select `STACK8_LAYOUT=flat:512M`, `512M`, or `tmpfs`, `STACK8_MODE=cow` or `standard`, and optionally `STACK8_PAUSE_SECONDS=10`. The workload uses Alpine, 256 MiB RAM, and two vCPUs. The runner records each command's elapsed wall time, stdout, stderr, and exit code, and attempts to stop every sandbox it starts in `finally`.

The opt-in language SDK tests are `sdk/python/tests/test_cow_lifecycle.py`, `sdk/node-ts/tests/cow-lifecycle.test.ts`, and `sdk/go/cow_lifecycle_test.go`. Set `MSB_COW_LIVE=1`; Go also requires tags `cow_live microsandbox_ffi_path` and `MICROSANDBOX_FFI_PATH` pointing to its matching native library.

## Observed coverage

Linux/KVM x86-64 passed CoW flat, managed, and tmpfs roots, plus a standard-memory flat-root baseline. macOS/HVF ARM64 passed the same root/memory variants. The checks exercise fresh construction, running full capture, idempotent pause/resume, host-observed Paused status, prompt rejection of new guest exec while paused, two successive full captures while retaining pause, installed-snapshot restore into two children, private child writes, direct full `.msnap` capture/restore, survival after input-archive unlink, and stop from paused. Completed runs stopped their test VMs; retained snapshot/cache artifacts remain in the isolated test homes for inspection.

The later Linux flat/managed/tmpfs/standard runs and macOS tmpfs run additionally assert unchanged Linux boot ID across ordinary pause/resume, resumed progress of the original counter workload, and guest wall clock within three seconds of the host after a ten-second pause. These checks do not constitute host-suspend, every clock-failure, or VM Generation ID notification testing.

The Python, Node, and Go live SDK checks passed on macOS: create with explicit CoW, pause, capture while paused, resume through a handle, restore a child, verify tmpfs contents and source/child write isolation, and stop a paused child. Windows ARM64 firmware build/export/load checks passed; native runtime compilation and live lifecycle coverage are still in progress. Windows explicit CoW requests remain unsupported and must fail without an eager fallback.

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

CoW virtio-mem unplug currently writes private zeros to prevent old backing bytes from reappearing, but does not reclaim those pages' host RAM. NUMA plus CoW is explicitly rejected. These are outstanding integration/performance limitations, not completed acceptance items. Further work includes backing-aware physical reclamation, resize/balloon live invariants, real shared/private resident-memory measurements, cache eviction and publication failure races, cancellation and lifecycle/maintenance concurrency, unsupported guest preparation, recovery/resume failures, Windows standard lifecycle qualification, and repeated release-build performance distributions. Public cache inspection/eviction workflow and comprehensive archive compatibility variants also remain to be completed. Do not mark #8 complete from these smoke results.

Evidence locations: OVH `/home/ubuntu/msb-stack8.ElfKzf/`; macOS `/private/tmp/msb-stack8-mac-{results,managed-results,tmpfs-results,standard-results}/`; Windows isolated worktree `C:\Users\Stephen\AppData\Local\Temp\msb-stack8-20260907`. These are development outputs, not shipped artifacts.
