# Incremental full-snapshot archive qualification — 2026-09-10

## Result and scope

RAM-aware `snapshot save --since` passed 12-checkpoint live chains on macOS ARM64/HVF and OVH Linux x86-64/KVM with flat, managed/layered, and tmpfs roots. The final checkpoint resumed with its expected RAM and filesystem contents through direct-archive and installed-snapshot restore, both eager and forked. This qualifies the archive dependency change, not unrelated concurrent lifecycle work. Linux ARM64 and Windows were not rerun for this change.

The implementation replaces the unreleased disk-only dependency encoding with `msb-snapshot-dependencies-v1`. It omits reusable RAM objects as well as the exact physical disk prefix, keeps the target memory manifest and CPU/device state complete, and resolves the explicit base into destination-owned staging. `--last-layers` remains disk-only selection. Standalone archives and snapshot descriptors are unchanged; there is no compatibility shim for earlier unreleased #8 dependency archives.

## macOS build and fixture

- Base commit: `18c8fb8693957ac6e8ded964e42cfbe63f8506ad`, plus this change's archive implementation, tests, and CLI help. An isolated worktree excluded concurrent agentd, database, and control-path edits in the shared #8 checkout.
- Host: macOS 26.3, build `25D2125`, ARM64/HVF.
- Binary: debug build, `net,ssh` features, codesigned with `msb-entitlements.plist`. SHA-256: `dc21397227f028bb32ffeed7f8322f33caf2767f16bac264fae2ede80a8f6cdb`.
- Guest agent SHA-256: `4c467d1e93d9ba78168d0eb3ec6c0a5d03793b972c496e250fe0288a42e51e43`.
- Firmware SHA-256: `ea0d458cdc12a0fa6dac8d192542ddc39717f816da41176582905e31a8bf868c`.
- Each source: Alpine, 256 MiB RAM, two vCPUs. Flat/managed disks: 512 MiB; tmpfs root: 128 MiB.
- Workload: a retained 24 MiB random RAM file plus a RAM marker and root-filesystem marker changed before each capture. Checkpoint 6 was captured while explicitly paused; other captures were from a running source.
- Source and destination used separate `MSB_HOME` directories. Each destination independently populated its OCI image cache. Image bundling was not part of this test.

## macOS size and timing

These are individual end-to-end CLI wall times from sequential debug-build qualification runs, not release-build latency claims. Filesystem caches were warm, and forked restores ran after eager restores. MB means decimal megabytes. Capture times include publication, not just the pause interval. Export comparisons below use the same checkpoint 12 with and without `--since`.

| Checkpoint 12 result | Flat | Managed/layered | Tmpfs |
| --- | ---: | ---: | ---: |
| Standalone archive bytes | 51,778,517 | 48,269,890 | 47,274,984 |
| Incremental archive bytes | 535,798 | 598,498 | 501,731 |
| Archive reduction | 98.97% | 98.76% | 98.94% |
| Omitted RAM objects | 13 | 13 | 13 |
| Omitted RAM object bytes | 131,334,144 | 135,737,344 | 126,586,880 |
| Full capture | 927.64 ms | 733.54 ms | 312.09 ms |
| Standalone export | 3,228.32 ms | 3,020.35 ms | 3,233.28 ms |
| Incremental export | 1,272.95 ms | 771.61 ms | 189.83 ms |
| Load final dependent archive | 2,222.48 ms | 1,717.76 ms | 1,042.21 ms |
| Direct archive → eager child | 3,361.47 ms | 2,687.48 ms | 1,836.25 ms |
| Direct archive → forked child | 3,367.98 ms | 2,936.96 ms | 1,680.98 ms |
| Installed snapshot → eager child | 1,256.04 ms | 1,076.20 ms | 752.70 ms |
| Installed snapshot → forked child | 792.42 ms | 602.98 ms | 237.71 ms |
| Sum of timed test commands | 75.84 s | 52.63 s | 38.96 s |

The size reduction is workload-dependent. Reuse is at the existing RAM-object granularity, not a new byte-diff encoding. An object can contain ranges not selected by the current memory map, so omitted object bytes are not the number of live unchanged guest bytes. Loading still reconstructs a complete local snapshot and verifies the borrowed RAM objects; a small archive does not imply equally small local storage or load work. Capture, live RAM allocation, and CoW materialization were not redesigned here.

## macOS live invariants

| Check | Flat | Managed/layered | Tmpfs |
| --- | --- | --- | --- |
| Standalone baseline plus 11 dependent exports | Pass | Pass | Pass |
| Every dependent export actually omits RAM payloads | Pass | Pass | Pass |
| Load checkpoints 1–11 without starting checkpoint VMs | Pass | Pass | Pass |
| Delete each prior installed base after resolving the next | Pass | Pass | Pass |
| Missing/nonexistent explicit base fails | Pass | Pass | Pass |
| Capture while paused, then resume source | Pass | Pass | Pass |
| Direct archive eager/forked restore of checkpoint 12 | Pass | Pass | Pass |
| Direct restore does not install the target snapshot | Pass | Pass | Pass |
| Restored RAM blob hash and both markers match checkpoint 12 | Pass | Pass | Pass |
| Child changes do not alter other restores or installed snapshot | Pass | Pass | Pass |
| Installed snapshot eager/forked restore after deleting its base | Pass | Pass | Pass |
| Live children retain RAM and disk writes after base deletion | Not separately injected | Pass | Pass |
| Final snapshot verification and standalone re-export | Pass | Pass | Pass |
| All test-owned VM processes stopped | Pass | Pass | Pass |

The tmpfs root marker is RAM-backed: this exercises a dependent archive with no disk-layer omissions. The flat run stopped its direct children before deleting the base; the later managed/tmpfs runs additionally deleted the base while those children remained running.

## Automated checks

- `cargo test --locked -p microsandbox --no-default-features --features net,ssh --lib snapshot --offline --target-dir /private/tmp/rd-target-10`: **44 passed** in 7.43 s.
- New archive fixtures cover disk+RAM and RAM-only 12-generation chains, packed-object offsets, zero extents, changed object identities, complete CPU/device metadata, installed and standalone-archive bases, direct restore staging, base deletion, corrupted/missing RAM, malformed omissions, undeclared or unreferenced dependencies, truncated input, and standalone re-export. Existing disk-only archive and released-format regression tests remain passing.
- `cargo build --locked -p microsandbox-cli --no-default-features --features net,ssh --offline --target-dir /private/tmp/rd-target-10`: passed.
- `cargo fmt --all -- --check`, `git diff --check`, and Python smoke-script syntax compilation: passed in the isolated worktree.
- Strict Clippy is blocked by pre-existing warnings: `derivable_impls` in `crates/image/lib/snapshot/manifest.rs` and `too_many_arguments` in `sdk/rust/lib/snapshot/create.rs`. The scoped SDK check passed with `--no-deps -- -D warnings -A clippy::too_many_arguments`; no source lint allowances were added.
- A default-stack regression test exposed a large nested async future introduced by RAM verification. Boxing the buffered reader fixed it; the passing suite uses the default test stack, not an increased stack limit.

## Reproduction and local evidence

The repository smoke harness is `scripts/smoke/cli/incremental-full-archive.py`. Use a short, fresh output directory on macOS because runtime socket paths have a length limit:

```bash
MSB_PATH=/path/to/codesigned/msb \
MSB_LIBKRUNFW_PATH=/path/to/matching/libkrunfw.5.dylib \
STACK8_OUT=/private/tmp/ram-delta-flat \
STACK8_LAYOUT=flat:512M \
python3 scripts/smoke/cli/incremental-full-archive.py
```

For managed roots use `STACK8_LAYOUT=512M`; for tmpfs use `STACK8_LAYOUT=tmpfs:128M`. An optional `STACK8_SEED_CACHE` reuses immutable OCI cache artifacts, never VM RAM caches. The harness records command logs, `results.json`, and `archive-sizes.json`, and stops only its own named VMs in cleanup.

Successful run directories: `/private/tmp/rd-f11`, `/private/tmp/rd-m12`, and `/private/tmp/rd-t11`. These include test RAM and disk artifacts and are local evidence, not files to publish. An explicit process check after all runs found no remaining test VMs.

Discarded fixture attempts are not counted as passes: an older installed firmware lacked the matching VMGenID readiness support; an overly long temporary path exceeded the socket-path limit; optional `--with-image` export from a flat-only cache lacked fsmeta; and `managed:512M` was not valid CLI syntax. Final runs used matching firmware, short directories, independent OCI caches, and the verified root-disk syntax. The `--with-image` cache limitation was not fixed by this archive dependency change.

## Linux x86-64/KVM follow-up

All three root layouts passed the same 12-checkpoint live harness on OVH, including deleting the supplied base while direct eager and forked children remained running. Each run exported the baseline standalone, exported checkpoints 2–12 with `--since`, loaded 1–11 sequentially with `--base`, and restored checkpoint 12 both directly from its archive and from its installed snapshot. Guest reads verified the checkpoint-12 RAM/root markers and the original 24 MiB RAM-file hash. Child writes remained private. Missing bases failed, capture while explicitly paused passed, and final verification and standalone re-export passed. No additional implementation fix was needed for Linux.

### Linux build and storage

- Same isolated `18c8fb8693957ac6e8ded964e42cfbe63f8506ad` baseline plus archive edits; no concurrent #8 lifecycle changes. The transferred `delta.rs` SHA-256 matched the Mac source: `25894fc39499d9d664c02d6bd126dfd73a3c6c325c55d4f39fd879c23af6460e`.
- Host kernel: `7.0.0-28-generic`, x86-64/KVM. Rust: `1.97.1`. Guest fixture sizes and workload match the Mac tests.
- Rebuilt the matching static x86-64 guest agent from source; its release build took 20.83 s. SHA-256: `31c648803053be07bc4dc8491b2e16035b44dbf79d21da097a69e609c2658814`.
- Debug CLI build with `--locked --no-default-features --features net,ssh --offline` took 38.62 s. SHA-256: `9bdd6c7ed090520be31e95da7776655339fd62d819640f4a8c87af5553acd10b`.
- Matching existing firmware SHA-256: `6acfb3c81238e64f60ab1dcac95e7b2c2c57161a5e6ab10fbab18e457a205d80`.
- Final archive, cache, sandbox, and temporary staging directories were on the host's ext4 `/dev/md3` filesystem. `TMPDIR=/home/ubuntu/rdl.pa6szm` kept SDK temporary staging and launch-configuration files there too. These results are not from host tmpfs storage; the guest tmpfs-root case still correctly stores its filesystem contents in guest RAM.
- The Linux snapshot regression suite passed **44/44 tests** in 0.45 s with the same test command and features as the Mac. The new host build and all live tests used the rebuilt matching agent.

### Linux measurements

One sequential debug-build run per layout, warm filesystem caches; these are end-to-end CLI times, not pause durations or release latency claims. Forked restores ran after eager restores. The final standalone and incremental exports use the same checkpoint 12.

| Checkpoint 12 result | Flat | Managed/layered | Tmpfs |
| --- | ---: | ---: | ---: |
| Standalone archive bytes | 43,430,398 | 39,755,415 | 39,378,027 |
| Incremental archive bytes | 258,923 | 257,546 | 232,087 |
| Archive reduction | 99.40% | 99.35% | 99.41% |
| Omitted RAM objects | 13 | 13 | 13 |
| Omitted RAM object bytes | 112,947,200 | 121,892,864 | 106,696,704 |
| Full capture | 128.87 ms | 125.26 ms | 84.44 ms |
| Standalone export | 992.76 ms | 900.48 ms | 838.71 ms |
| Incremental export | 139.09 ms | 128.20 ms | 104.27 ms |
| Load final dependent archive | 1,592.66 ms | 1,690.74 ms | 1,459.00 ms |
| Direct archive → eager child | 2,382.62 ms | 2,501.35 ms | 2,179.46 ms |
| Direct archive → forked child | 2,451.04 ms | 2,565.34 ms | 2,244.45 ms |
| Installed snapshot → eager child | 806.07 ms | 809.40 ms | 732.56 ms |
| Installed snapshot → forked child | 235.81 ms | 239.48 ms | 198.96 ms |
| Sum of timed test commands | 50.36 s | 44.48 s | 40.72 s |

### Linux evidence and fixture corrections

The isolated source/build directory is `/home/ubuntu/ram-delta-linux.mdyJh7`. Successful disk-backed run directories are `/home/ubuntu/rdl.pa6szm/{flat-2,layered-2,tmpfs-2}`. Timing and size JSON files were copied to `/private/tmp/rd-linux-evidence-10/{flat,layered,tmpfs}` on the Mac; raw RAM and disk artifacts were not copied or published. The final process check found no remaining runtime using the isolated test binary, including the diagnostic probe.

An initial long runtime path exceeded the Unix socket-path limit. A subsequent flat-root run under `/tmp/rdl.W5axYo/flat` passed, but `/tmp` on this host is RAM-backed, so its timings are excluded from the table. The first disk-backed rerun then encountered `EDQUOT` while writing an anonymous launch-configuration file in `/tmp`, not while writing a snapshot. A scoped syscall trace identified that location. Setting only the test process's `TMPDIR` to the private disk-backed directory resolved it; no host quota, system configuration, or unrelated files were changed. The three final disk-backed runs above completed successfully.
