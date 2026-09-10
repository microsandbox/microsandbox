# Snapshot groups qualification — 2026-09-10

## Result

Snapshot groups, scoped selectors, ancestry-aware head advancement, explicit selection, archive import, and CLI/Rust/Python/TypeScript/Go surfaces were initially implemented and qualified in an isolated detached worktree based on `e20269e1b260149e4cf629f43fac142a35e1458b`. The initial qualification below did not edit the concurrent #8 checkout. Subsequent integration checks are recorded separately so these original measurements retain their context.

The final macOS ARM64/HVF live matrix passed with flat, managed/layered, and tmpfs roots. These runs verify group integration with existing execution state and archive workflows; they are not a fresh Linux or Windows qualification of the stack.

## Live coverage

| Check | Flat 512 MiB | Managed 512 MiB | Tmpfs 128 MiB |
| --- | --- | --- | --- |
| Capture, stable ID directory, alias, recorded parent | Pass | Pass | Pass, full only |
| Automatic fast-forward and explicit head selection | Pass | Pass | Pass |
| Restore exact old member versus selected group head | Pass | Pass | Pass |
| Concurrent sibling captures retain both members; one wins head | Pass | Pass | Pass |
| Full capture and capture from user-paused source | Pass | Pass | Pass |
| Direct branch child records source's last captured ancestor | Pass | Pass | Pass |
| Duplicate name rejects without changing existing member/head/cursor | Pass | Pass | Pass |
| Base + `--since` export/load, preserved aliases and IDs | Pass | Pass | Pass, self-contained export |
| Missing required base rejected before group creation | Pass | Pass | No dependency omitted in this workload |
| Old/idempotent import retains head; `--set-head` explicitly rewinds | Pass | Pass | Pass |
| Same archive in another group; ambiguous global ID refused | Pass | Pass | Pass |
| Head deletion protection, even with `--force` | Pass | Pass | Pass |
| Eager and forked full restore reproduce disk/RAM markers | Pass | Pass | Pass |
| Direct full archive capture and restore install no snapshot member | Pass | Pass | Pass |
| Stopped-source disk capture and restore | Pass | Pass | Correctly refused |

The final matrices ran 66, 66, and 64 CLI commands respectively, including expected negative cases and explicit stops. Test-owned VMs are stopped in `finally`, including failed creates. The tmpfs source's `--since` archive had no reusable object omissions, so its inventory correctly declared `boot-complete`; the fixture checks whether a base is required from the actual inventory rather than assuming every `--since` request produces a dependent archive. RAM dependency reconstruction is independently covered by the 12-checkpoint RAM-only and disk-plus-RAM artifact tests.

## Timing context

These are individual end-to-end **debug-build** CLI wall times, not pause duration, release performance, or a regression comparison. The flat/managed matrices overlapped, while the final tmpfs retry ran separately. Sources had two vCPUs and 256 MiB of RAM with small disk/RAM marker files. They are intentionally small correctness fixtures, not representative large working sets.

| Operation | Flat | Managed | Tmpfs |
| --- | ---: | ---: | ---: |
| Initial group head read | 7.98 ms | 7.73 ms | 10.15 ms |
| Second disk capture | 123.37 ms | 128.96 ms | Not applicable |
| Full capture (`full1`) | 1356.53 ms | 1099.99 ms | 791.07 ms |
| Capture while paused | 1260.54 ms | 1162.57 ms | 770.73 ms |
| Installed eager full restore | 936.21 ms | 725.02 ms | 522.50 ms |
| Installed forked full restore | 920.67 ms | 733.85 ms | 546.03 ms |
| Direct local branch | 922.42 ms | 865.51 ms | 607.58 ms |
| Direct full archive capture | 2279.01 ms | 1863.23 ms | 1493.84 ms |
| Direct forked archive restore | 2017.03 ms | 1650.82 ms | 1257.52 ms |

Raw results are `/private/tmp/sgf2/results.json`, `/private/tmp/sgm2/results.json`, and `/private/tmp/sgt2/results.json`, with per-command stdout/stderr alongside them. Initial retries documented fixture issues: an omitted firmware override, selecting an older installed firmware without VM Generation ID readiness, a too-long macOS socket path, and the tmpfs dependency assumption above. No failed run is counted as a passing matrix.

## Automated checks

- Snapshot library tests: 61 passed, including 15 group tests, five ancestry tests, duplicate IDs, generated-name retry without recapture, cancellation-safe cursor locking, missing history, source replacement, scoped deletion, and RAM-aware archive reconstruction.
- Artifact integration tests: 45 passed, including released-flat import normalization, repeated imports, alias-preserving re-export, and explicit legacy paths.
- Migration tests: 27 passed; downgrade preflight and rollback tests passed separately.
- Same-name/different-backend full-capture routing and post-publication resume-failure handling: passed with local socket permissions.
- CLI snapshot tests: 10 passed. TypeScript snapshot/native contract tests: 11 passed. Go unit tests passed; Go integration tests compiled. Native Rust bindings checked for Python, TypeScript, and Go; TypeScript declarations regenerated and typecheck passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Strict broad Clippy encountered pre-existing `derivable_impls` in `crates/image/lib/snapshot/manifest.rs`; targeted `--no-deps -D warnings` encountered pre-existing `too_many_arguments` in `build_artifact`. Targeted Clippy passed with only that existing lint allowed via command-line `-A clippy::too_many_arguments`. No unrelated source was changed to silence lints.
- Full Python/Go VM suites, Linux, Windows, workspace-wide tests, and a release-build performance comparison were not rerun for this change. Python stub Ruff retains two unrelated pre-existing line-length findings.

## Reproduce

Use a short, fresh output path on macOS:

```bash
MSB_PATH=/path/to/codesigned/msb \
MSB_LIBKRUNFW_PATH=/path/to/matching/libkrunfw.5.dylib \
GROUP_TEST_OUT=/tmp/sg-test \
GROUP_TEST_LAYOUT=flat:512M \
python3 scripts/smoke/cli/snapshot-groups.py
```

Repeat with `512M` and `tmpfs:128M`. The script owns an isolated `MSB_HOME` under its output directory and never stops unrelated sandboxes.

The qualified binary SHA-256 was `6b71b08c9d89f87c59a60b5a218903ec048d6ab47066846d9d83c7d674917fa7`; firmware SHA-256 was `ea0d458cdc12a0fa6dac8d192542ddc39717f816da41176582905e31a8bf868c`; embedded agentd SHA-256 was `1425dc4b6974c10983db03e023c2b868c890fc197857ea45d6289a83df593aa8`. This uses the existing #8 guest artifacts; no kernel or agent implementation changes are part of the group feature.

## Publication and recovery boundaries

Group/head and per-source ancestry locks are process-held filesystem locks, not a daemon or a distributed authority. Head/ID lookup opens only the selected member; aliases scan local member-name metadata. An interrupted operation may have published a complete member, so callers should inspect before retrying. A process crash between member/head publication and cursor persistence may leave conservative ancestry that needs explicit head selection; it cannot silently overwrite a sibling. Group identity/name conflicts fail before moving staged members. Missing ancestry is distinct from missing payload data. Explicit removal racing an already resolved open may fail that operation; it never silently changes the chosen snapshot or cold-boots instead.

Friendly names and group heads are local metadata, not new cryptographic identities. Descriptors retain their existing schema and portable IDs. Released flat artifacts remain readable by explicit path; there is no automatic relocation of old directories.

## Integration onto updated #8

The group commit was replayed onto `86873fa68806914a8417cd0fa4e5f6eaa068105b`, retaining both newer commits: `8713ded2` (resident lifecycle performance) and `86873fa6` (Node ownership locking). No newer-main changes were imported. The newer dirty-memory smoke script was updated to use qualified member selectors and the installed path returned by capture. SDK reference examples were checked against the grouped APIs.

Post-integration checks passed: 61 snapshot tests, 45 artifact tests, 10 CLI snapshot tests, four read-only control-lookup tests, the unindexed-group downgrade refusal test, and the same-name backend capture-routing test. All 27 migration tests also passed during integration preparation. Native bindings checked for Python, Node and Go; TypeScript typecheck and 16 focused unit tests passed; Go unit tests passed. Formatting and targeted Clippy passed with the previously documented `too_many_arguments` allowance. Native checks used the existing isolated SDK bootstrap home after the default prebuilt installer attempted an unavailable network download; no installed runtime was replaced.

The complete macOS group matrices passed again: 66 commands each for flat and managed roots and 64 for tmpfs. Raw results are `/private/tmp/sgpf/results.json`, `/private/tmp/sgpm/results.json`, and `/private/tmp/sgpt/results.json`. The integrated, codesigned debug binary SHA-256 is `56f901eba49a3e748608cd2c987f9c7495959c4d6e43981bc0e5aa3c47efa067`, retained at `/private/tmp/sg-push-integrated-msb`; firmware and embedded agentd are unchanged from the initial qualification.

| Integrated debug CLI operation | Flat | Managed | Tmpfs |
| --- | ---: | ---: | ---: |
| Initial group head read | 6.80 ms | 9.00 ms | 9.58 ms |
| Second disk capture | 114.73 ms | 137.68 ms | Not applicable |
| Full capture | 1275.77 ms | 966.86 ms | 814.14 ms |
| Capture while paused | 1260.71 ms | 993.24 ms | 777.49 ms |
| Installed eager restore | 949.10 ms | 697.05 ms | 504.40 ms |
| Installed forked restore | 947.55 ms | 718.54 ms | 534.55 ms |
| Direct branch | 843.45 ms | 777.08 ms | 626.83 ms |
| Direct full archive capture | 2527.27 ms | 1788.07 ms | 1502.21 ms |
| Direct forked archive restore | 2510.29 ms | 1571.97 ms | 1254.42 ms |

These remain small debug-build correctness fixtures; some runs overlapped other builds or tests. They are not isolated performance comparisons or VM pause measurements. Linux, Windows, and release-performance qualification were not rerun during this push.

### Existing dirty-memory qualification gap

The additional `dirty-memory-checkpoint.py` smoke test did **not** pass. Its second full capture from a forked child reported `capture_mode: full` rather than the asserted `incremental`. Source/child RAM checks, direct branching, branch-of-branch, paused captures, and eager/forked restoration passed up to that assertion; the later grandchild and disk-only checks in that script were not reached. These failures do not invalidate the separate complete group matrices above, but they must not be described as a complete dirty-memory qualification.

The original poll allocated and copied the entire 64 MiB disk-cache file merely to inspect eight bytes. The integration changes limit that read to eight bytes while retaining the dirty 64 MiB shared mapping and every assertion. This reduced observer overhead but did not resolve the full-capture result: both flat and managed reruns still failed. The runtime has an existing density fallback that chooses a full capture at 60% dirty RAM, but the failed runs do not log the decision reason, so that explanation remains unproven.

Crucially, the exact pre-group #8 code at `86873fa6`, rebuilt separately with the same guest artifacts, reproduces the same assertion using its original script on a managed root. This establishes that the failure exists before the group change; no runtime workaround or relaxed assertion was added. Baseline binary SHA-256: `6d1eb6edaba2794d564c2b85f4c48598bb6a3d95b8726d861fccac97ac8a9c57` (`/private/tmp/sg-push-baseline-msb`). Reports: `/private/tmp/sg-dirty-push-flat/report.json`, `/private/tmp/sg-dirty-push-flat2/report.json`, `/private/tmp/sg-dirty-push-managed2/report.json`, and `/private/tmp/sg-dirty-baseline/report.json`. Every dirty-memory run reported an empty cleanup-error list.
