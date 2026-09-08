# Restore optimization qualification — 2026-09-08

The optimization batch is implemented and live-tested on macOS/HVF ARM64. This report does not declare the entire #8 branch or every proposed performance optimization complete. Linux and Windows were not rerun in this pass.

## Changes qualified

- Preserve GNU sparse encoding for long checkpoint layer names, with the exact GNU long-name marker already accepted by the previous reader. Increase archive input and extraction buffers to 1 MiB while retaining bounded reads, transport hashing, sparse-map validation, and truncation rejection.
- Merge ordered incremental memory ranges with a consuming sweep rather than rescanning and sorting the entire evolving map for each update. Preserve zero ranges, object identity, slice offsets, and overlap/overflow rejection.
- Resolve flat snapshot image configuration without downloading or materializing unused OCI disks. Probe pinned/original cache keys before the existing digest scan. Keep digest pinning, registry authentication/TLS settings, `--pull never` refusal when metadata is absent, and full artifact requirements for layered roots.
- Make the stopped-sandbox startup check respect flat root disks. Live testing caught its unconditional VMDK requirement when the new metadata-only cache contained no VMDK; the fix was rebuilt and requalified.
- Avoid directory syncs for disposable child checkpoint staging, but keep durable installed-snapshot publication, persistent disk publication, and immutable RAM-cache synchronization. Avoid syncing diagnostic boot/activation records; guest activation/clock acknowledgement/thaw ordering is unchanged.
- Inspect only the bounded, identity-verified checkpoint root during builder planning, and remove the redundant pre-copy source validation. Child/runtime payload admission still runs before consumption.
- Serialize same-identity CoW cache misses with a process lock and recheck after acquiring it. Warm hits remain outside the build lock. Keep no-replacement inode publication for interoperability with earlier builders and pinned VMs.

No snapshot schema, agent protocol, dependency pin, CLI flag, or public language-SDK option changed.

## Environment and reproducibility

Apple M5 Max, 36 GiB RAM, macOS 26.3, APFS, native ARM64/HVF. Guest: Alpine pinned at `sha256:e7a1a92a5bfeee40966aea60f0796b0e7917cc35591542701834f03a68fa3d18`, 256 MiB RAM, two vCPUs, flat 512 MiB root disk, 32 MiB random tmpfs payload, disk/RAM markers, and a shell workload. This is a development workstation, not an otherwise idle dedicated performance host.

Source base: Microsandbox `15ab8838da58061df5dd9560cbd85d648bb72351` plus this optimization commit. Libkrun remains pinned to `51c1ed3b83dc826c02800fb297995538e4eac55d`. Matching guest agent: `/private/tmp/msb-stack8-agentd-arm`; firmware: `/private/tmp/msb-stack8-libkrunfw.5.dylib`. The final signed executable SHA-256 is `32530a3f4385a090fb385d8a5015429638a43b16d990353818c0ecba219c3d9f`.

Build:

```sh
MSB_AGENTD_PATH=/private/tmp/msb-stack8-agentd-arm cargo build --release --locked --no-default-features --features net,ssh,prebuilt -p microsandbox-cli
codesign --entitlements msb-entitlements.plist --force -s - /private/tmp/msb-resume-final.P83RaS/msb
```

The build executable was copied to the final test directory before signing. Final evidence and executable: `/private/tmp/msb-resume-final.P83RaS/`. `bench.py`, `qualify.py`, and `fanout.py` retain the exact test procedures; `results.json`, `summary.json`, `qualification.json`, `qualification-summary.json`, and `concurrent.json` contain individual times and exit codes. Tests use isolated homes, bounded subprocess timeouts, and cleanup in `finally`. Use fresh directories/names when repeating them. The earlier optimization run and corruption test are retained in `/private/tmp/msb-resume-opt.locaTL/`; the pre-optimization benchmark and actual older executable are in `/private/tmp/msb-resume-profile.osiuRj/`.

## Performance

These times measure process launch through successful CLI completion, not just RAM mapping, stop-the-world time, or application readiness. “First command” measures from that same initial launch until a subsequent guest command completes, including a second CLI process. RAM hash checks occur afterward. Warm CoW means the prepared backing exists; cache miss means that file was absent, not that the OS page cache was purged. No p95 claim is made from these sample counts.

The strongest A/B comparison alternates the actual pre-optimization and final release binaries against the same installed snapshot and home, six samples per binary/mode:

| Installed full restore | Before median | After median | Reduction | After range |
| --- | ---: | ---: | ---: | ---: |
| Standard RAM | 278.15 ms | 153.56 ms | 44.8% | 145.83–159.68 ms |
| Warm CoW | 229.64 ms | 115.18 ms | 49.8% | 112.22–116.87 ms |

The complete final-binary matrix below compares with the previous measurement pass. Unlike the alternating A/B above, those passes used separate captures and ran at different times. Improvements describe the combined batch, not an isolated contribution from each optimization.

| Path | n | Previous CLI median | Final CLI median | Final first-command median |
| --- | ---: | ---: | ---: | ---: |
| Resident resume, standard | 8 | 12.97 ms | 11.40 ms | 25.81 ms |
| Resident resume, CoW | 8 | 12.91 ms | 10.57 ms | 24.04 ms |
| Fresh boot, standard | 5 | 180.48 ms | 158.89 ms | 194.53 ms |
| Fresh boot, CoW configured | 5 | 179.43 ms | 166.89 ms | 219.02 ms |
| Stopped disk snapshot | 5 | 206.80 ms | 190.45 ms | 226.67 ms |
| Full snapshot, disk-only restore | 5 | 209.38 ms | 181.63 ms | 218.00 ms |
| Installed full, standard | 8 | 278.76 ms | 152.31 ms | 219.32 ms |
| Installed full, warm CoW | 8 | 228.47 ms | 111.66 ms | 176.29 ms |
| Installed full, CoW cache miss | 5 | 301.18 ms | 162.85 ms | 232.38 ms |
| Full archive, standard | 5 | 405.12 ms | 236.38 ms | 287.45 ms |
| Full archive, warm CoW | 5 | 347.50 ms | 200.94 ms | 256.26 ms |

The final full archive was 54,767,005 bytes. A genuine incremental checkpoint produced by repeated capture while paused restored in median 165.20 ms with standard RAM and 120.20 ms with CoW (five samples each). That paused incremental case has no intentionally dirtied RAM between captures; it verifies the incremental path but does not measure heavily fragmented dirty-memory merge throughput. A separate post-workload capture fell back to full and is correctly recorded as `delta-*` with `capture_mode=full`, not presented as incremental performance.

Single observations, not distributions: full capture 493.35 ms; subsequent full-fallback capture 466.94 ms; direct full archive capture 442.86 ms; stopped disk capture 37.38 ms. In the qualification run, a full baseline capture took 458.52 ms and the following genuinely incremental paused capture took 247.57 ms. These are caller-observed operation durations, not vCPU pause durations.

Three simultaneous cold-CoW archive restores completed in 278.70–286.37 ms each and published one prepared RAM file. This is one fanout experiment, not a concurrency percentile. An empty image-cache archive restore took 3,138.93 ms including registry network access; only one metadata JSON file was cached, with no OCI disk artifacts. A subsequent offline `--pull never` CoW restore succeeded. Avoid interpreting network-bound cold metadata fetch as warm restore latency.

## Correctness and compatibility evidence

- 92 focused tests passed: 27 runtime checkpoint tests, 28 registry tests, 31 SDK snapshot tests, five checkpoint resolver tests, and the new flat-restart regression test.
- The memory merge was compared against an independent per-byte oracle over 1,000 generated fragmented cases, including holes, zero updates, nonzero object offsets, unsorted input, and updates spanning multiple old extents. Malformed overlap, empty, and overflowing ranges are rejected.
- Sparse long-name output is checked with a separate tar parser and encoded-size assertion, then loaded and restored. The actual pre-optimization #8 release binary successfully restored the new plain-tar and zstd archives without reader changes. The final binary restored a real pre-optimization archive and verified its tmpfs payload hash. macOS system tar listed both new archive encodings successfully. This is not qualification against every historical release.
- The live matrix checked full-state RAM SHA-256, retained workload markers, cold disk-only semantics, standard/CoW modes, installed/direct-archive inputs, prepared-cache hits/misses, resident resume, unchanged boot ID across resident resume, and cleanup.
- Missing metadata with `--pull never` refused cleanly; metadata-only network fetch succeeded; reuse with `--pull never` succeeded. Unit tests retain layered artifact requirements and reject moved-tag digest mismatches.
- Two children retained independent RAM/disk contents after deleting their input archive. Stopping/restarting the modified child preserved its private disk marker without OCI layers or VMDK. An isolated snapshot copy with corrupted execution-state object bytes was refused before successful VM creation.
- Three concurrent restores shared a cold cache entry, retained their markers, and isolated private RAM writes. A truncated zstd archive was refused with `zstd stream did not finish`.
- A process inventory after the runs found no runtimes belonging to either optimization test directory. Two unrelated pre-existing development VMs were left untouched. Test artifacts were retained for inspection.

The first live cold-cache attempt used an overlong Unix socket path and was rerun with a shorter home. The later restart check exposed and fixed the real unconditional-VMDK bug described above. Both are retained in the earlier run's evidence; neither is counted as a passing initial attempt.

`cargo fmt --all -- --check` and `git diff --check` passed. Strict Clippy was blocked by the existing `derivable_impls` warning in `crates/image/lib/snapshot/manifest.rs`; allowing that lint exposed the existing `too_many_arguments` warning on `RuntimeControlExecutor::new`. No unrelated lint cleanup was included. This pass did not rerun every workspace or language-SDK test.

## Remaining performance work

Warm full CoW branching is roughly twice as fast, but it is not consistently below 90 ms. The broader proposals for startup readiness notification, pipelined eager object reads, restore-only zero-preserving RAM construction, and avoiding immutable qcow2 ancestor relocation copies are not implemented by this batch. The remaining first-command latency also deserves separate measurement; CLI completion is not a substitute for measuring a ready application or a persistent SDK connection. Large-memory, deep-chain, heavily fragmented dirty-memory throughput, cache-builder process-death, and cross-platform performance qualification remain outside this pass.
