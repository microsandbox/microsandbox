# Snapshot batch-load qualification — 2026-09-10

Archive names in the original qualification below use the current `.msb` convention. Retained raw logs preserve the filenames used at the time; post-rename qualification is recorded separately.

## Result

Batch loading passed on macOS ARM64/HVF with real full-checkpoint archives for both flat and managed root disks, including eager and forked restoration. Real complete disk-only archive batches also passed for both layouts. Fresh disk-only `--since` export is blocked by an existing producer limitation, described below; this is not counted as passing dependent File-archive live coverage.

The final signed development binary was `/private/tmp/msb-batch-load-final`, SHA-256 `269bcba93e00b1e04b98a92386a301571dc0b59f2f2b0d90e52a2aa46e28541a`. Runtime tests used `/private/tmp/msb-forked-build.8HN0Ie/lib/libkrunfw.5.dylib` and `/private/tmp/msb-cow-8.PIhgYp/build/agentd`. Existing real fixtures were 256 MiB RAM, two vCPUs, and 512 MiB root disks. The full matrices ran in parallel in isolated homes. These are CLI wall times from qualification runs, not release-build performance benchmarks or stop-the-world measurements.

## Live coverage

| Check | Flat root | Managed root |
| --- | --- | --- |
| Three full archives, supplied in reverse and shuffled order | Pass | Pass |
| Actual shell-expanded `*.msb` batch | Pass | Pass |
| Returned paths retain input order while group head selects the unique tip | Pass | Pass |
| Automatic dependency resolution from the named destination group | Pass | Pass |
| `--dest` store root and automatic base lookup there | Pass | Pass |
| Missing disk/RAM closure and unrelated external base rejected before member publication | Pass | Pass |
| Complete external base supplies payloads without importing historical ancestors | Pass | Pass |
| Duplicate inputs install once | Pass | Pass |
| Ambiguous new group remains headless; explicit head selection works | Pass | Pass |
| Ambiguous existing group retains its head | Pass | Pass |
| Ambiguous `--set-head` rejected without published members | Pass | Pass |
| Full imported final member survives removal of inputs and its installed ancestors | Pass | Pass |
| Eager/forked restore retains `/disk-marker` and `/dev/shm/marker` | Pass | Pass |
| Complete disk-only archives load in reverse order and select the child head | Pass | Pass |
| Disk-only final member survives removal of inputs and its installed ancestor | Pass | Pass |
| Disk-only cold boot retains disk marker and does not restore tmpfs marker | Pass | Pass |
| Fresh disk-only dependent `--since` archive creation | Blocked: existing layer-ID issue | Blocked: existing layer-ID issue |

Each final full matrix checked 39 command outcomes, including expected errors. Each complete disk-only matrix checked 12: 102 checked command outcomes in the four successful runs. All six VMs started by those final matrices were stopped, as were the two additional fresh-capture probe VMs. A final process inspection found no matching test harness or VM process remaining.

## Observed timings

| CLI operation | Flat root | Managed root |
| --- | ---: | ---: |
| Load three full archives, reverse order | 3,853.53 ms | 2,511.74 ms |
| Load three full archives, shuffled order | 3,604.32 ms | 3,234.55 ms |
| Load full archive shell wildcard | 3,621.72 ms | 2,752.73 ms |
| Load two full deltas using installed group base | 2,070.18 ms | 1,467.89 ms |
| Same automatic dependency lookup under `--dest` | 2,058.32 ms | 1,467.14 ms |
| Load final full delta with explicit complete external base | 1,074.19 ms | 751.28 ms |
| Restore imported final checkpoint, eager | 958.19 ms | 763.53 ms |
| Restore imported final checkpoint, forked | 970.45 ms | 774.82 ms |
| Load two complete disk-only archives, reverse order | 707.11 ms | 474.26 ms |
| Cold boot imported final disk-only snapshot | 422.90 ms | 329.61 ms |

Restore times cover the `msb create` invocation. Guest marker checks were separate successful `msb exec` commands. No claim is made that these times isolate memory mapping, disk preparation, or guest readiness from the rest of the CLI pipeline.

## Producer limitations found during qualification

1. `--with-image` currently requires materialized layered image-cache artifacts, including fsmeta and VMDK, even for a flat snapshot. The old flat-only fixture lacked those files. The successful flat runs exported the original flat snapshot paths using the managed fixture's already populated image cache with the identical pinned image digest. The harness exposes this as `--image-home`; it does not modify either fixture. The first failed attempt is retained in `/private/tmp/sblf/report.json`.
2. Consecutive disk-only snapshots currently receive fresh IDs for every copied disk layer. `build_artifact` and `new_file_manifest` in `sdk/rust/lib/snapshot/create.rs` allocate random layer IDs for the whole source closure. The physical-prefix requirement therefore rejects `msb snapshot save work:child child.msb --since work:parent` even when parent and child were captured consecutively from the same running sandbox. This was reproduced on the final binary with new captures on both layouts, not only old fixtures. No producer implementation was changed in this work, and no archive metadata was fabricated to make the live test pass. The strict dependent-File test remains available and fails at export. Complete File batch import was qualified separately. Synthetic dependent-File import tests cover the loader independently of this existing exporter limitation.

## Automated checks

- Snapshot library: 75 tests passed, including a six-archive mixed-codec reverse-order chain, RAM-only and disk-plus-RAM dependency resolution, missing/corrupt RAM, borrowed File payload checks, and dependent File archives in both input orders.
- Snapshot artifact integration: 55 tests passed, covering legacy single load, ordering, duplicates, conflicting aliases/IDs/labels, branch/head behavior, corruption before publication, missing historical ancestors with complete payloads, and independent installed payload ownership.
- CLI snapshot tests: 12 passed.
- Rust native checks: Python, Node, and Go bindings passed.
- Node: native build, 14 focused tests, and type checking passed.
- Python: one stub-surface test and Ruff passed.
- Go: unit/native checks and integration test compilation passed; VM-backed Go integration execution was not run.
- Targeted Microsandbox/CLI Clippy passed with `--no-deps -D warnings -A clippy::too_many_arguments`; formatting and diff checks passed.

The automated counts above include checks run by the coordinating agent and the SDK agent. Live qualification in this report was macOS-only; Linux and Windows were not rerun for this batch-load change.

## Reproduction and retained evidence

Use a fresh output directory for each invocation. The harness never restarts or deletes the retained source fixtures. It removes only its own exported input archives and selected imported ancestor snapshots to verify dependency independence.

```bash
export MSB_LIBKRUNFW_PATH=/private/tmp/msb-forked-build.8HN0Ie/lib/libkrunfw.5.dylib
export MSB_AGENTD_PATH=/private/tmp/msb-cow-8.PIhgYp/build/agentd

python3 scripts/smoke/cli/snapshot-load-batch.py \
  --binary /private/tmp/msb-batch-load-final \
  --fixtures /private/tmp/sgpf \
  --image-home /private/tmp/sgpm/home \
  --output /private/tmp/batch-flat-new-run --live

# Repeat with --fixtures /private/tmp/sgpm and a different output directory.
# Complete File archive coverage: add --file-only --file-standalone.
# Reproduce the producer limitation: add --file-only --fresh-file instead.
```

Raw JSON reports contain every command result, full output, timing, archive inventories, and cleanup results:

| Run | Report |
| --- | --- |
| Final full flat | `/private/tmp/sblf-final/report.json` |
| Final full managed | `/private/tmp/sblm-final/report.json` |
| Final complete File flat | `/private/tmp/sblf-file-standalone-r2/report.json` |
| Final complete File managed | `/private/tmp/sblm-file-standalone-r2/report.json` |
| Fresh File delta producer failure, flat | `/private/tmp/sblf-fresh-file/report.json` |
| Fresh File delta producer failure, managed | `/private/tmp/sblm-fresh-file/report.json` |

An earlier standalone-File harness attempt incorrectly expected the archive completeness spelling `complete`; the assertion was corrected to the actual `boot-complete` enum before the successful final File runs. No product assertion or dependency check was relaxed.

## `.msb` extension follow-up

The source, CLI help, SDK examples, and live harness now use `.msb`. This is only a naming convention: explicit paths are preserved, compressed/plain-tar decoding remains content-based, and the archive and descriptor schemas are unchanged. The direct-archive unit matrix passed all eight compression/filename combinations (`.msb`, `.tar.zst`, `.tar`, and extensionless, each compressed and plain). Snapshot library tests (75), artifact integration tests (55), CLI tests (12), TypeScript tests (14) and type checking, the Python stub test, Go unit tests, formatting, and diff checks passed again.

The signed follow-up binary is `/private/tmp/msb-extension-final`, SHA-256 `3326644fcc666d55bb72c41b851b1f7f0c28b5bb06ca7095914421a0d28b8e24`. The same fixtures and runtime artifacts were used. Both full matrices and both complete File matrices passed again with newly exported `.msb` archives: 102 checked command outcomes, including actual `*.msb` shell expansion, dependent full imports, eager/forked full restore, and disk-only cold boot. All six test VMs were stopped. Linux and Windows were not rerun for this extension-only change.

| Follow-up operation | Flat root | Managed root |
| --- | ---: | ---: |
| Load three full archives, reverse order | 3,442.06 ms | 2,441.31 ms |
| Restore imported full checkpoint, eager | 939.44 ms | 764.74 ms |
| Restore imported full checkpoint, forked | 955.37 ms | 789.54 ms |
| Load two complete disk-only archives | 803.58 ms | 477.47 ms |
| Cold boot imported disk-only snapshot | 430.29 ms | 336.07 ms |

These remain development-build qualification timings, not isolated performance comparisons. Reports: `/private/tmp/sxbf2/report.json` (full flat), `/private/tmp/sxbm2/report.json` (full managed), `/private/tmp/sxbfd2/report.json` (File flat), and `/private/tmp/sxbmd2/report.json` (File managed). Initial attempts under the tool sandbox passed archive checks but were denied VM endpoint creation (`Operation not permitted`); their failed reports remain under `/private/tmp/sxbf`, `/private/tmp/sxbm`, `/private/tmp/sxbfd`, and `/private/tmp/sxbmd`. The passing reruns used host permissions without changing product code or relaxing assertions. The disk-only dependent-export limitation above remains unchanged.
