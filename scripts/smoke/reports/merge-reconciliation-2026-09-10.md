# #8 merge reconciliation — 2026-09-10

This records reconciliation and focused regression checks, not merge approval or a complete live-platform qualification. The initial repair pass removed incomplete private workload-control plumbing; the subsequent coordinated implementation and its remaining failures are recorded below. The maintainer requested committing and pushing this progress to #8 with those failures still open. That publication does not authorize merging the PR or claim the stack is ready.

## Transport follow-up — live qualification in progress

The maintainer subsequently requested and approved the coordinated transport fix, including both host writers, readers, guest dispatch, the coordinator, and saved transport counters. The integrated implementation compiles. Freeze/thaw uses a bounded in-process route to the existing primary writer; ordinary input remains ordered and gated at complete-frame boundaries. There is no additional socket or public API. Native guest execution and fresh Mac/Linux live qualification are being completed; compilation alone is not a qualification pass.

The initial safety-review hold was resolved by the maintainer's subsequent approval to complete the coordinated repair. Full snapshots produced by earlier unreleased development builds without the required transport counters are rejected. Released disk snapshots and the frozen generation-8 protocol schema are unchanged.

The agreed input accounting uses cumulative control/bulk wire-byte and frame counters, including headers, with 8 MiB/256 ordinary control frames and 32 MiB/256 bulk records. Combined-port raw bulk records use bulk accounting too. These limits fit the existing 512-entry guest consumer queues when charges remain owned through consumption. Accepted guest input and its remaining credit debt survive restore; unadmitted host input remains source-owned. No counter reset or new public socket is intended. Private replies must match operation and attempt, and gate release requires confirmed thaw. Reverse output backpressure still needs a bounded-prefix review; an ordinary output-budget wait cannot be described as consumer-independent lifecycle progress.

The integrated follow-up passes 271 host runtime tests, 196 native Linux guest tests, 61 protocol tests, four schema checks, and 21 Unix agent-client tests. Runner/client-only runtime and local SDK library Clippy checks pass; native Linux guest Clippy also passes with `-D warnings`. The ARM64 musl guest cross-check passes (existing musl `time_t` deprecation is allowed only in that cross-Clippy invocation). Released `schema/gen-8.json` remains byte-identical with SHA-256 `0b9ec1348f019430fdcb2d4acf3cac48f2267b1091523ed523be1daf5831d911`. The live harness has 26 passing transport helper tests and reuses 24 passing cleanup tests. Local socket fixtures require execution permission to bind their temporary Unix socket; the permitted rerun passes. These are not candidate VM qualification.

The tested source bundle is `/private/tmp/msb-transport-repair.PF6XsF/candidate-source-2.tar.gz`, SHA-256 `5ef502deb821420a3bc671622e450999dd8e6a22e5527608108189d593b7b60e`; OVH extracted it into `/home/ubuntu/msb-transport-repair.AKNpfY/candidate-2`. Guest tests and release guest builds use that identical source. The Mac release binary is codesigned with this checkout's hypervisor entitlements. Baseline binaries remain preserved separately.

### Candidate-2 live finding: inherited stdin exhausts control frames

The Mac pilot passed idle pause/resume, blocked pipe resume, exact source 16 MiB input plus EOF, and the restored child's retained prefix plus pipe EOF. However, full capture took 667.57 ms and the user-visible child restore command took 19,888.95 ms, coinciding with the autonomous consumer gate. This is a qualification failure, not an acceptable restore result. The runtime itself activated in 61.042 ms (626 µs private thaw), then fresh child clients waited roughly 19.6 seconds. The checkpoint has both sent and granted `control_frames=2433`: all 256 outstanding frame credits belong to 8 KiB stdin messages, leaving no frame credit for new leases or exec requests despite 6,279,936 bytes of headroom. A longer fixture timer would hide nothing; it would merely extend the wait.

The proposed correction is to charge inline stdin, TCP input/EOF, filesystem input/EOF and raw records to the existing data window, leaving control credit for leases and ordinary requests. It reuses already-decoded message types and existing writers/FIFOs; it does not reset debt or add a queue/socket. New `data_*` counter names would reject superseded unreleased captures. This material accounting change is awaiting maintainer approval. The pilot's raw report is `/private/tmp/msb-transport-repair.PF6XsF/mac-candidate-pilot/candidate/report.json`; its isolated home `/tmp/msb-t-b556ro7o` is retained for logs/artifacts, with all owned VMs stopped and no cleanup errors.

The matching Linux pilot subsequently reproduced the same gate-overlap failure after blocked pipe pause/resume passed; it did not proceed to branch. Raw report: `/home/ubuntu/msb-transport-repair.AKNpfY/transport-candidate-harness.3IOpBC/linux-candidate2-pilot/candidate/report.json`. Its failed evidence home `/tmp/msb-t-yg7zr1vs` is retained but stopped. Both hosts have no remaining owned test VMs.

The exact pre-follow-up source baseline is preserved at `/private/tmp/msb-transport-repair.PF6XsF/baseline-source.tar.gz`, SHA-256 `9f624caa07a16f496698ca64dd591dba49f6329d6c4dd1e3c3b258409d586dc8`. Fresh Mac and Linux release baseline binaries and matching ARM64/x86-64 guest agents were built. Initial live harness attempts exposed harness socket-path and output-only stdin-lifetime problems; their partial timings are not a valid performance comparison. Candidate live testing and before/after performance qualification remain outstanding.

Corrected baseline runs subsequently passed on macOS ARM64 and OVH Linux x86-64: three idle pause/resume cycles, three pipe and three PTY workloads, exact 16 MiB stdin hash/EOF checks, sequenced control output, checksummed 64 MiB uploads/downloads, and subsequent framing checks. Both reports confirm no cleanup errors, no remaining owned sandboxes/runtime PIDs, and removal of the isolated homes. No blocked-input capture, branch, or restore qualification was performed in these baseline runs.

| Baseline, three samples per measurement | macOS ARM64 p50 / p95 | Linux x86-64 p50 / p95 |
| --- | --- | --- |
| Idle pause | 12.46 / 13.43 ms | 7.56 / 7.65 ms |
| Idle resume | 11.88 / 12.83 ms | 3.53 / 7.48 ms |
| Pipe stdin throughput | 48.78 / 51.86 MiB/s | 40.22 / 47.65 MiB/s |
| PTY stdin throughput | 11.49 / 11.50 MiB/s | 20.05 / 20.41 MiB/s |

These are pre-fix baseline observations, not improvements or candidate passes. Raw reports: `/private/tmp/msb-transport-repair.PF6XsF/mac-baseline-perf-3/baseline/report.json` and `/private/tmp/msb-transport-repair.PF6XsF/linux-baseline-evidence/linux-baseline-perf-3/baseline/report.json`. The reports retain workload parameters, binary/agent/firmware hashes, architecture-specific image digests, CPU/throughput samples, and cleanup evidence. Percentiles use nearest rank; with only three samples, p95 is the maximum observed sample.

Candidate-2's separate idle/throughput runs pass byte, EOF, framing, and cleanup checks on both hosts, but do not establish capture/restore correctness or acceptable final performance. Each uses three samples, the same image digest per host, 16 MiB stdin, and concurrent 64 MiB bulk transfers. The following p50 observations are provisional; a same-session Linux baseline repeat is being used to check host/time variability.

| Metric | Mac baseline → candidate-2 | Linux baseline → candidate-2 |
| --- | --- | --- |
| Idle pause | 12.46 → 13.18 ms | 7.56 → 3.57 ms |
| Idle resume | 11.88 → 11.57 ms | 3.53 → 3.50 ms |
| Pipe stdin | 48.78 → 67.15 MiB/s | 40.22 → 24.24 MiB/s |
| PTY stdin | 11.49 → 9.64 MiB/s | 20.05 → 5.73 MiB/s |

The Linux PTY slowdown is significant and unresolved, not an accepted cost of correctness. Source runtime CPU measurements include concurrent filesystem transfers, which run longer and transfer more total data in a slower stdin case; they do not isolate stdin overhead. The host writer comparison found extra admission locking/notification work, but no new payload copy, changed console batching, or verified busy loop. These are optimization candidates, not a proven causal explanation. Reports: `/private/tmp/msb-transport-repair.PF6XsF/mac-candidate2-perf/candidate/report.json` and `/private/tmp/msb-transport-repair.PF6XsF/linux-candidate2-evidence/linux-candidate-perf/candidate/report.json`. Both runs removed their isolated homes and report no remaining owned VM processes.

Read-only follow-up identified a guest scheduling candidate: after the first partial stdin write, subsequent input queues without probing newly available pipe/PTY capacity, and queued input drains only in bounded turns of the outer reactor loop. A bounded, non-awaiting FIFO drain per decoded input batch may reduce that backlog without reinstating the original blocking write. Separately, host admission currently registers notifications and locks state on two loop turns per frame; arming a waiter only when blocked could reduce overhead while preserving atomic gate checks. Neither optimization has been implemented or proven causal. The Linux mixed-load PTY test completed 30 downloads and 11 uploads in candidate-2 versus five and three in the original baseline; fixed-volume/stdin-only checks are needed before interpreting this as an intrinsic per-byte slowdown.

A subsequent Linux baseline repeat with the same current mixed-load harness passed all nine samples and cleanup. Pipe/PTY stdin p50 was 38.93/20.16 MiB/s, close to the original 40.22/20.05 MiB/s. The candidate-2 mixed-load slowdown therefore persists against a nearby baseline, while the different amount of competing bulk work still prevents causal attribution. The new stdin-only harness case has helper coverage but has not been live-run; further runs are held pending the accounting decision. Baseline-repeat report: `/home/ubuntu/msb-transport-repair.AKNpfY/transport-candidate-harness.3IOpBC/linux-baseline-interleaved/baseline/report.json`.

## Pinned inputs

- Original #8: `24152183b0eace990798e31f1a53f024980f938f`.
- Updated #7 merge input: `2830a4e2465fb1572a2b007e4cb5f3d790bca74c`.
- Common ancestor: `ce04099b660adcd75e4b63a985bc51fb30d30504`.
- Isolated candidate: `/private/tmp/msb-source-flag.KUTTO5/pr8`. During qualification, reviewed working-tree resolutions were deliberately left unstaged in the merge index. The maintainer's subsequent commit/push request includes recording those resolutions; it does not resolve the behavioral gaps below.

The comparison uses both pinned parents, not newer unrelated `main`. The candidate's root `Cargo.toml` and `Cargo.lock` remain identical to the updated #7 input. Published `msb_krun` 0.1.34 and `msb-imago` 0.1.7 remain selected; no new Git dependency override was introduced.

## Intersection map and repairs

| Intersection | Reconciled behavior | Evidence / limitation |
| --- | --- | --- |
| Release lifecycle identity × #8 pause/branch | Retained Sandbox and SandboxHandle receivers reject same-name replacements. Control selection binds the sandbox row, run row, and PID, then checks the connected endpoint's server PID before writing. Branch revalidates the selected run before capture. | Stale receiver, endpoint mismatch, and source-restart regression tests pass. This does not serialize every live configuration mutation; see remaining work. |
| Release startup ownership × #8 restore finalization | The actual child process stays owned through readiness, catalog publication, and restore finalization. Detached creation disarms ownership only after success; failure cleanup avoids reacquiring the transition lock already held by the creator. | Startup/kill and abandoned-Starting tests pass. Pending restore intent remains fail-closed. |
| Windows lock handoff × startup acknowledgement | The parent explicitly releases the non-inheritable Windows lifecycle guard immediately before spawn while retaining the name-transition guard. | A Windows helper-process regression test was added, but native execution and Windows typechecking remain unqualified. |
| Release removal/maintenance × #8 lineage publication | Snapshot lineage uses one stable lock outside the removable sandbox directory. Removal/replacement coordinates with it. Ephemeral maintenance uses a nonblocking claim, including the exit observer that already owns lifecycle, and retries without deleting artifacts when capture owns lineage. | SDK removal/lineage tests and runtime cleanup tests pass. Never wait for lineage while holding lifecycle. |
| Live disk rollover × failure recovery | Tentative sealed-layer hashes no longer mutate the live head before successful rollover. A journal replacement/directory-sync failure is treated as possibly published and leaves the source paused for recovery. | Raw/qcow retry and injected post-rename failure tests pass; these are not live power-loss tests. |
| Portable memory cache × eviction | Publication holds a backing pin before exposing a new cache entry; an existing winning entry is also pinned before use. | Publication/eviction race tests pass. |
| Paused guest refusal × release filesystem bulk protocol | Filesystem negotiation and credit waits preserve terminal `core.error` instead of losing its useful rejection reason. | Both added tests failed before the change and pass afterward; all seven focused filesystem pause tests pass. |
| Released database history × snapshot identity/groups | The complete 25-migration release prefix remains unchanged, followed by snapshot identity and then snapshot groups. All 26 executable migration entries and payloads from updated #7 remain intact. | 31 migration tests pass, including canonical ordering and group migration/refusal checks on synthetic catalogs. |
| Release protocol/feature split × #8 restore | Generation-8 schema bytes are unchanged. Strict restore intent and the internal `--restore` argument remain alongside release file mounts, resolved network configuration, and client/runner feature separation. | Bounded two-parent source comparison and native focused builds/tests. No old-binary live compatibility run in this pass. |
| CLI/SDK/docs × combined surface | Snapshot groups, batch loading, `--from-sandbox`, cloud rejection, and forked restore constraints remain. Python `connect_or_create` now declares the `forked` option already accepted by its native builder. Lifecycle docs describe Paused and SDK resume correctly. | Four Python AST contract checks and documentation language-order checks pass. |
| Release kernel cache action × #8 firmware pin | Retain firmware `6cca413ac248f63e65d4ea4748b3bc36cd1b22f3` with the clock-only restore path. Teach the cache action the exact kernel 6.12.108 tarball checksum. | Resolver tests accept the exact 6.12.99/6.12.108 checksums and URLs and reject an unknown version. No kernel rebuild in this pass. |

The kernel checksum was compared against kernel.org's [signed checksum-index text](https://cdn.kernel.org/pub/linux/kernel/v6.x/sha256sums.asc) retrieved over HTTPS; this is not a claim that its OpenPGP signature was independently verified.

## Focused checks

Native Rust checks used macOS ARM64, principally `CARGO_TARGET_DIR=/private/tmp/rd-target-10`. `MSB_AGENTD_PATH=/private/tmp/msb-cow-8.PIhgYp/build/agentd` was only a compile fixture; it is not evidence of a fresh matching live guest build. Migration/client-only checks used `/private/tmp/msb-merge-client-target`.

| Check | Result |
| --- | --- |
| SDK `backend::local` tests, `local,net` | 60 passed |
| SDK `sandbox::pause` tests | 4 passed |
| SDK persisted-removal tests | 3 passed |
| SDK `snapshot::lineage` tests | 6 passed |
| SDK `sandbox::fs::pause_tests` | 7 passed |
| Runtime maintenance tests after ephemeral-cleanup repair | 15 passed |
| Runtime checkpoint tests, client-only / runner | 18 / 48 passed |
| Runtime `restored_` tests | 7 passed; includes the two characterization tests described below |
| `cargo test -p microsandbox-migration --lib --offline` | 31 passed |
| `cargo clippy -p microsandbox --lib --no-default-features --features local,net -- -D warnings` | Passed |
| `cargo clippy -p microsandbox-runtime --lib --no-default-features --features runner,net -- -D warnings` | Passed |
| Runtime Clippy with `--all-targets` | Fails `items_after_test_module` in client/control.rs, client/logging.rs, and runner/exec_log.rs; not silently suppressed |
| Python creation-stub contract functions | 4 passed using `runpy`; system Python lacks pytest, so this is not a pytest-suite pass |
| Node SDK native build, TypeScript build/typecheck, and unit tests | Passed; 153 tests across 12 files, using the newly built native binding |
| `python3 -m unittest discover -s scripts/smoke/cli -p test_snapshot_branch.py -v` | 24 passed; harness tests, not live sandbox tests |
| `python3 scripts/check-docs-language-order.py` | 117 multilingual groups and SDK navigation passed |
| `cargo fmt --all -- --check`; `git diff --check` | Passed before the final report update |

Local Unix-socket fixtures initially failed to bind under tool sandbox restrictions and passed with the required permissions. Windows ARM64 cross-check stopped in dependency C compilation because the host lacks Windows CRT headers (`stdlib.h`, `assert.h`, and `setjmp.h`); it did not establish Windows type correctness. The Surface SSH connection timed out.

Node checks ran in `/private/tmp/msb-merge-node.K1eep5`, a copy of this candidate's SDK. The repository's CI pruning script removed unpublished platform optional dependencies in that disposable copy before offline `npm ci`; tracked package manifests and lockfiles were not changed. Tests initially lacked built JavaScript/native artifacts, then passed after `npm run build:ts` and a fresh `cargo build --offline -p microsandbox-node`. That build used a disposable `MSB_HOME` and `/private/tmp/msb-v070-integration.BKd4tB/artifacts` solely to satisfy the build-time runtime-pair requirement; no VM was started from those artifacts. The debug native link reported a large `__eh_frame` compact-unwind warning. No release-build performance claim is made.

## Remaining work — not repaired or qualified

1. **Transport qualification and retained-input control starvation.** The original loopback/FIFO deadlock and missing frame-boundary plumbing have been replaced by the coordinated implementation described above. Native tests pass, but the fresh Mac pilot exposed inherited stdin exhausting ordinary control-frame credits. The data/control accounting correction awaits approval. Complete blocked-input capture/branch/restore and performance qualification remain required after that correction; the candidate is not merge-ready.
2. **Concurrent host-policy changes during branch.** Exact run validation prevents selecting one runtime and capturing another, but a same-run live secret update can still race with cloning the source's host-side configuration. The capture refreshes CPU/memory/interface values, not a transactionally consistent host secret policy. This inherited branch/modify interaction remains a separate decision and implementation item; no broad modification-lock refactor was made.
3. **Older-macOS dependency delivery.** The separately prepared libkrun optional-HVF-symbol fix at `31f3ce3` is not part of the published `msb_krun` 0.1.34 dependency used by this candidate. This pass did not publish a replacement crate or change that pin.
4. **Fresh platform qualification.** The initial repair pass used no live VMs. The transport follow-up now has matching release binaries, native guest tests, and fresh Mac/Linux live evidence as detailed above; it is not a complete platform pass. Windows ARM64 is not tested in this follow-up. Historical live reports must not be presented as tests of this candidate.

The isolated merge remains available for inspection and continuation. None of these checks establishes that #8 or the full release stack is merge-ready.
