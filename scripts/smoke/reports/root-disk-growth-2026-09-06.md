# Root-disk growth qualification — 2026-09-06

Stack: Microsandbox `appcypher/live-root-disk-growth`, directly based on #6 (`ed4167f7`), with libkrun `ec9f119dcc144f1c011fe62ebd224d660c2191a3`. No newer main changes were imported. Publication and Linux/Windows source transfers await explicit approval.

## Coverage

The reproducible matrix is `scripts/smoke/cli/root-disk-growth.py`. Set `MSB_BIN`, an isolated `MSB_HOME`, matching `MSB_LIBKRUNFW_PATH`, `QUAL_ROOT` and a fresh `QUAL_PREFIX`. It always stops its own sandboxes, including on failure.

macOS/HVF passed 58 CLI checks and ten direct control-phase measurements in both debug and release builds. Hardware: Apple M5 Max, 36 GiB RAM. Guest: Alpine, 256 MiB RAM, initially 512 MiB ext4 root. The release matrix covers both managed and flat roots: raw growth to 768 MiB; a 600 MiB file written and synced beyond the original capacity; full checkpoint rollover; qcow2 growth to 1 GiB and then 1280 MiB; unchanged ancestor SHA-256 and chain length; shrink rejection; a second full snapshot; compaction followed by growth; a partial final block group at 1700 MiB; stopped growth to 1792 MiB; next-start growth to 2 GiB; a synced 1800 MiB file; stopped snapshot creation and verification; old/new full restores at 768/1280 MiB with payload checksums and tmpfs memory markers intact. All test sandboxes were confirmed stopped afterward.

The initial live tests found that Linux online growth leaves valid lazy group flags which the older offline resizer rejected. The resizer now preserves those flags and initializes a lazy partial block bitmap before extending it. The failed attempt modified only staging; the original head and journal remained usable.

## Release phase measurements

These are individual observations, not percentiles or latency guarantees. Runtime total includes preflight, persistence, pause/resume and guest acknowledgment. Pause includes draining the block worker. Guest time includes online ext4 expansion, sync and verification, after VM resume.

| Operation | Managed total / pause / guest (ms) | Flat total / pause / guest (ms) |
| --- | ---: | ---: |
| Raw 512 → 768 MiB | 69.14 / 7.80 / 45.38 | 62.49 / 7.91 / 38.18 |
| Qcow2 768 → 1024 MiB | 69.44 / 7.11 / 45.79 | 67.76 / 8.03 / 43.03 |
| Qcow2 1024 → 1280 MiB | 50.49 / 4.15 / 30.32 | 55.39 / 4.18 / 35.12 |
| Compacted 1280 → 1536 MiB | 54.50 / 7.06 / 30.97 | 52.09 / 3.95 / 32.09 |
| Partial group 1536 → 1700 MiB | 50.10 / 4.02 / 29.98 | 52.24 / 3.80 / 32.07 |

Stopped CLI growth from 1700 to 1792 MiB took 45.65 ms managed and 43.66 ms flat. The matrix's online CLI rows are same-target configuration reconciliation after a separately measured control request; they must not be quoted as end-to-end first-growth latency. Raw outputs and phase JSON are in `/private/tmp/msb-grow-qual.6VfI8S/release`; debug results are in its `matrix2` sibling.

## Other checks

- libkrun device suite: 135 passed, including preserved data, zero-filled added capacity, repeated targets, and read-only/shrink rejection.
- Runtime disk tests: five passed, including both layouts, immutable ancestors, pending-target snapshot/compaction refusal, recovery before boot, lost-completion acknowledgment, and failed staging without journal publication.
- Rust SDK modification tests: 51 passed, including explicit restart/next-start policies, new root capability and old control sockets.
- Full Rust SDK library suite: 666 passed and three ignored with `cargo test --offline -p microsandbox --lib -- --test-threads=1`. A parallel run had one failure in the unchanged stale Unix-socket detection test; that test passed in isolation and in the serial suite. The initial sandboxed run also denied four socket bindings; validation was repeated with local socket access.
- Image ext4 tests and four external `e2fsck` tests passed; dedicated tests cover qcow2 growth and lazy partial block groups.
- Protocol generation-9 schema and append-only checks: four passed.
- Pinned CLI build/check succeeds offline after seeding Cargo's cache from the exact signed local libkrun commit. The companion commit must be pushed before other machines can fetch it.
- Strict Clippy initially found pre-existing `derivable_impls` and `too_many_arguments` warnings in #6. Focused linting allows those two baseline classes; new lint findings were fixed.

## Remaining qualification and limitations

Linux/KVM and Windows/WHP have not been tested for this item. Both machines were reachable, but transferring unreleased source was blocked pending explicit approval. SDK bindings already route the existing modification options through Rust; fresh Python/TypeScript/Go native live runs have not been performed for #7. Direct/dependent archives after growth, sustained concurrent-I/O latency, request cancellation, process/power-loss injection, and every admission-failure variant remain additional qualification work. Do not describe this report as exhaustive fault testing.

Stopped growth currently retains previous private file bindings after publishing the replacement head. It does not add chain depth or alter snapshots, but retained files can consume host space; automatic reclamation was not implemented because its deletion scope needs explicit approval/stronger ownership proof. Pending online growth is forward-only. Older runtimes refuse the pending journal field; finish recovery before switching back to an older runtime. A failed operation may leave saved desired configuration behind physical capacity until the caller retries the same requested size.
