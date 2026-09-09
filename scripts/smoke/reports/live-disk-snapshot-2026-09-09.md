# Live disk-only snapshots — 2026-09-09

Implemented on the #8 branch above Microsandbox `f68c1329`, using existing pinned libkrun `862d6842`, rust-vmm `f798d4f2`, and matching firmware. No companion changes, dependency overrides, schema change, or new public flags were required.

## Behavior

```sh
# Source stays running. No RAM checkpoint or --full workaround.
msb snapshot create saved --from source
msb create --name child --from-snapshot saved

# Direct archive: no installed snapshot directory or index row.
msb snapshot create exported --from source --archive ./exported.msnap
msb create --name archive-child --from-snapshot ./exported.msnap
```

Running and user-paused managed/flat OCI roots use the serialized runtime control executor and a distinct capability-gated `disk_checkpoint_create` operation. Rollover seals the disk and selects a private successor. Running sources resume before SDK packaging; user-paused sources stay paused. Stopped/crashed copies retain their lifecycle lock. The SDK packages only the immutable disk closure, rechecks source identity, and removes consumed staging.

Live disk cuts are **crash-consistent**, not application/filesystem-quiesced. Unsaved application buffers, guest page-cache writes not submitted to disk, and tmpfs are not promised. Children cold-boot with private writable heads. The block worker is inspected by existing rollover machinery, but no RAM, full CPU/device checkpoint, freezer handshake, or RAM-baseline update is required. Full capture retains its existing behavior. Cloud, tmpfs disk-only, and user-owned disk-image restrictions remain unchanged. Unsupported old runtimes are refused without a full-capture or writable-file-copy fallback.

Existing Rust, Python, TypeScript and Go snapshot APIs already route through the shared Rust implementation. Documentation was corrected; no duplicate API was added. Native language packages were not rebuilt/live-tested in this pass.

## Live coverage

The committed `scripts/smoke/cli/live-disk-snapshot.py` passed on macOS ARM64/HVF, OVH Linux x86-64/KVM, and Surface Windows ARM64/WHP. Each tested flat and layered roots with 512 MiB disk capacity, 256 MiB RAM, and a cached Alpine image.

| Checks, on both root layouts | Mac | Linux | Windows |
| --- | --- | --- | --- |
| Running installed capture and cold restore; optional integrity | Pass | Pass | Pass |
| Direct compressed `.msnap` and plain `.tar`; no installed intermediate | Pass | Pass | Pass |
| User-paused capture remains paused; exec refuses until explicit resume | Pass | Pass | Pass |
| Source RAM/boot ID retained; disk child has a new boot ID and no tmpfs marker | Pass | Pass | Pass |
| Source/child writes isolated; sealed payload hashes unchanged | Pass | Pass | Pass |
| Child usable after snapshot/archive deletion | Pass | Pass | Pass |
| Disk capture leaves RAM store/cache untouched; transient staging released | Pass | Pass | Pass |
| Full after disk; disk between full captures; updated RAM restores | Pass | Pass | Pass |
| Stopped capture after live rollover; duplicate-name and tmpfs refusal | Pass | Pass | Pass |
| Capture during active writes; writer progresses; captured counter cold-boots | Pass | Pass | Pass |

An initial live test caught incorrect acquisition of the runtime-owned lifecycle lock; live SDK capture now uses runtime serialization, while stopped capture retains the lock. The first Windows active-write fixture failed to start its separate long-running host client before capture. The final portable fixture uses a guest-owned background worker, as existing branch tests do, and verifies startup, progress, and termination.

## Timings

Debug-build whole CLI wall times in milliseconds, one observation per case. These are correctness-run observations, not release benchmarks, percentiles, or controlled host-to-host comparisons. Cache state, hardware, sequential chain depth, and workload matter. Guest contents and command success were checked separately after create returned.

| Host / root | Installed capture | Installed cold restore | Compressed archive capture | Archive cold restore |
| --- | ---: | ---: | ---: | ---: |
| Mac / flat | 203.20 | 415.35 | 366.84 | 629.40 |
| Mac / layered | 94.74 | 311.51 | 139.37 | 402.38 |
| Linux / flat | 24.57 | 319.45 | 144.48 | 346.42 |
| Linux / layered | 14.18 | 320.52 | 22.19 | 335.16 |
| Windows / flat | 234.00 | 2359.00 | 906.00 | 1328.00 |
| Windows / layered | 157.00 | 2016.00 | 250.00 | 1141.00 |

These samples come from Mac `live-disk-mac-h`, Linux `live-disk-linux-d`, and Windows `live-disk-win-c`. Runtime `capture_disk.pause_us` separately measures pause request through resume acknowledgement for running sources. First installed captures measured 174.32/70.56 ms for flat/layered on Mac and 145.21/49.04 ms on Windows. Linux's preceding `live-disk-linux-c` run measured 5.91/3.32 ms. User-paused measurements describe rollover work, not the user's entire suspension interval.

Disk-only skips RAM capture but is not constant-time or zero-I/O. Existing rollover still performs layer integrity work and prepares its immutable closure while paused. Allocated bytes, chain depth, copying fallback, outstanding I/O and durability latency can increase pause time; this change does not optimize that existing hot path away. SDK publication and archive compression follow source resume.

## Reproduction and limits

Set `MSB_PATH`, isolated `MSB_HOME`, matching `MSB_LIBKRUNFW_PATH`, unique `STACK8_PREFIX`, and `STACK8_OUT`, then run `python3 scripts/smoke/cli/live-disk-snapshot.py`. The fixture stops only its own VMs in `finally`.

Final portable-fixture evidence: `/private/tmp/msb-live-disk-mac-i/results.json`; `/home/ubuntu/msb-live-disk-linux-e/results.json` on OVH; `C:\Users\Stephen\AppData\Local\Temp\msb-live-disk-win-e\results.json` on Surface. Earlier timing runs retain the corresponding result directories. Guest disks/RAM are not committed.

Runtime control tests passed 6/6; checkpoint-filtered runtime tests 32/32; SDK snapshot-filtered tests 31/31. Formatting and diff whitespace checks passed. CLI builds passed on all three hosts with pinned Git sources; the Mac runnable binary was codesigned.

Linux ARM64/nested KVM and Windows x86-64 were not rerun for this addition. Native SDK packages, exhaustive crash/publication fault injection, disk-full, near-limit chains, concurrent maintenance, large cold-cache workloads and release p50/p95 performance are not qualified by this pass.
