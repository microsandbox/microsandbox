# Registry dependency qualification — 2026-09-09

This follow-up to #8 replaces development Cargo Git patches with published crates. It starts from Microsandbox `8e68722c47d7f06f76e51b78f44563939d47b4da`, retains the #7 stack base, and does not change the firmware pin, agent protocol, snapshot format, or public API. These are debug-build correctness checks, not release performance measurements.

## Published dependencies

| Package | Registry version | Release-source commit |
| --- | --- | --- |
| `msb-vm-memory` | `0.18.0-msb.2` | rust-vmm `c8aad4c`, on `appcypher/windows-private-memory` |
| `msb-imago` | `0.1.7` | imago `cba9c0c`, on `appcypher/release-memory-dependency` |
| All 15 `msb_krun` family crates | `0.1.34` | libkrun `b20d31a`, on `appcypher/registry-memory-dependencies` |

All uploads completed and Cargo confirmed registry availability. Release-source branches were pushed; this does not claim they have merged into their default branches. No Microsandbox package was published.

The memory release contains the Windows private-view implementation already used by #8's pinned `f798d4f` source. Imago's exact memory dependency had to move with it to avoid incompatible copies of the shared memory types; `0.1.7` also retains the previously released `0.1.6` tail-discard fix. Libkrun now uses those registry dependencies without a Git override. Microsandbox's lockfile changes only the intended 13 packages and resolves one `msb-vm-memory` version across imago and libkrun.

## Release checks

- Memory: Linux x86-64 passed 127 unit tests and 34 doctests, plus Clippy. macOS Clippy passed; 121 unit tests passed and five existing 4 KiB page-assumption tests failed on the 16 KiB-page host. Comparing against `0.18.0-msb.1` confirmed no Unix memory source changes. Windows ARM64 and x86-64 production-feature compilation passed; this is not a native Windows test run. Packaged dry run passed.
- Imago: all-feature build, Clippy, formatting, packaged dry run, 37 unit tests and four doctests passed; two tests were ignored. Windows ARM64 and x86-64 compilation passed.
- Libkrun: `cargo build --all --locked`, `cargo test --all --locked` (281 passed, six ignored), `cargo clippy --all --locked -- -D warnings`, formatting, and the coordinated 15-crate packaged dry run passed. `cargo check --locked -p msb_krun --features blk --target aarch64-pc-windows-msvc` passed with registry dependencies.

## Microsandbox checks

```bash
cargo build --locked -p microsandbox-cli --no-default-features --features net,ssh
cargo test --locked -p microsandbox-runtime --no-default-features --features net --lib checkpoint
cargo test --locked -p microsandbox --no-default-features --features net,ssh --lib snapshot
cargo fmt --all -- --check
git diff --check
```

Build and formatting passed; checkpoint tests passed 32/32 and snapshot tests passed 39/39. Cargo reported the existing future-incompatibility warning for `proc-macro-error2 2.0.1`. The CLI was codesigned with `msb-entitlements.plist` before live testing.

## Mac live checks

The existing `scripts/smoke/cli/direct-branch.py` harness ran with `STACK8_MAINTENANCE=1`, a new disposable `MSB_HOME`, flat 512 MiB disk, 256 MiB RAM, and two vCPUs. All 56 recorded steps had their expected result, including the two deliberate refusals:

- First branch plus five repeated branches with retained siblings, without installed snapshot publication.
- Same-name refusal, private RAM/disk writes, source/sibling isolation, and a grandchild retaining its parent's private writes.
- Continuing guest timer-driven counter progress.
- Branch from a paused source, refusal to execute on the still-paused source, and ordinary resume.
- Three durable full captures and forked restores with subsequent guest reads.
- Live root growth to 768 MiB, explicit compaction, and another usable branch.
- Grandchild survival after source and parent stop; successful cleanup of all 13 test VMs. A final process check found no remaining runtimes with the test prefix.

Evidence: `/private/tmp/msb8-registry-live.b1LAEv/results/results.json` and adjacent per-command logs. The signed debug CLI SHA-256 was `cdbfc66c9fae75d1a8d3e4015020889fa0f48b5ee01932c08de87a01282c8ab7`. Firmware SHA-256 was `ea0d458cdc12a0fa6dac8d192542ddc39717f816da41176582905e31a8bf868c`; the embedded agent SHA-256 was `4c467d1e93d9ba78168d0eb3ec6c0a5d03793b972c496e250fe0288a42e51e43`, matching the existing stack test artifacts. Raw guest state is not committed.

This registry-only update was not live-rerun on Linux or Windows, and does not claim a new full platform matrix or language-SDK qualification. The Surface SSH connection timed out during this pass. Earlier successful platform coverage remains recorded separately in [CoW platform fixes](cow-platform-fixes-2026-09-09.md) and [live disk snapshot](live-disk-snapshot-2026-09-09.md); their historical source versions and measurements are unchanged.
