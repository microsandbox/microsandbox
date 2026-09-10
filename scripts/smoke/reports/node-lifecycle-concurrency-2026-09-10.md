# Node lifecycle concurrency qualification — 2026-09-10

The approved Node ownership change is implemented. The wrapper now acquires an `Arc` under a short admission lock instead of holding the lock across guest execution, filesystem I/O, or lifecycle waits. Normal operations do not clone the sandbox configuration. Detach/removal consume the shared slot, so subsequent calls through the wrapper or an existing filesystem facade fail; admitted operations retain their references. The runtime still decides whether concurrent operations are valid. This is not a guarantee that work completes successfully after stopping or removing its sandbox.

Live tests also found an existing filesystem response-decoding bug: the host relay rejects new guest work while paused using `core.error`, but filesystem helpers attempted to deserialize it as `FsResponse` or ignored it on streams. They now preserve the existing diagnostic through the SDK's existing unexpected-response helper. The wire protocol, snapshot format, and public method signatures are unchanged. No additional state lookup, guest round trip, or retry was introduced.

## Release timing

Each sample measures the Node promise from invocation to completion using `performance.now()`, while a command on the same object is already executing and waiting for an explicit release marker. There are 20 pause/resume pairs per host in the final sample. Sandbox configuration is Alpine, 256 MiB RAM, and a 512 MiB managed root disk. These are SDK call timings, not CLI startup, snapshot restore, or application-response timings.

| Host | Pause median | Resume median | Pause min–max | Resume min–max |
| --- | --- | --- | --- | --- |
| macOS ARM64/HVF | 0.216 ms | 0.182 ms | 0.187–0.407 ms | 0.170–0.367 ms |
| Linux x86-64/KVM | 0.195 ms | 0.313 ms | 0.178–0.457 ms | 0.299–0.549 ms |

A freshly built old macOS Node binding, using the same runtime and firmware, fails the pending-exec regression at the 2,000 ms pause deadline. The command intentionally waits indefinitely for a marker, so this is a demonstrated lock blockage, not a baseline latency distribution or a meaningful speedup ratio. The new binding completes pause/resume without releasing that command, then verifies the original command completes correctly. The old binding's failure and final candidate reports are retained separately. CH/FC and the snapshot/restore benchmark matrix were not rerun in this follow-up.

## Live coverage

Fixture: `sdk/node-ts/tests/lifecycle-concurrency.test.ts`, enabled with `MSB_NODE_LIFECYCLE_LIVE=1`. Set `MSB_NODE_TIMINGS` to save raw call timings. Use a short isolated `MSB_HOME` and explicitly select the matching development `MSB_PATH` and `MSB_LIBKRUNFW_PATH`.

| Case | macOS ARM64 | Linux x86-64 |
| --- | --- | --- |
| Pending exec: 20 pause/resume cycles, public paused state, original command completion | Pass | Pass |
| Filesystem request while paused: useful rejection and successful read after resume | Pass | Pass |
| Detach during exec: consumed-handle errors, idempotent detach, original command and VM survive | Pass | Pass |
| Blocked filesystem upload: pause/resume/detach progress, existing facade rejects new work, admitted upload finishes with correct bytes | Pass | Pass |
| Removal during exec: wrapper consumption is independent of runtime lifecycle locking; admitted command completes | Pass | Pass |
| Terminal-state wait concurrent with stop, followed by successful removal and consumed-handle errors | Pass | Pass |

The upload fixture uses a host FIFO and observes the first bytes in the guest before pausing, proving the guest stream has started rather than racing its initial admission. The removal fixture permits the runtime's existing lifecycle lock to defer removal until shutdown or reject it. If removal wins before stop's final database observation, a missing-row stop error is accepted only after verifying that the concurrent removal succeeded. The tests do not weaken runtime lifecycle locking or silently ignore arbitrary cleanup errors.

Final live test-body totals, including VM creation and cleanup: macOS 1.72 seconds; Linux approximately 13.53 seconds. The longer Linux fixture total includes shutdown and is not the pause/resume latency shown above. Both final runs passed all six cases. No sandbox records or matching runtime processes remained in either isolated test home after cleanup.

Other validation: 137 Node unit tests passed on each host; three Rust shared-ownership tests passed (pending operation, competing consumers, cancellation/lifetime); five Rust filesystem response tests passed (normal success/failure, paused diagnostic, unexpected envelope, read-stream rejection, terminal response without waiting for channel close); TypeScript build/typecheck and separate live-fixture typecheck passed; Node native Clippy with `-D warnings`, focused Rust formatting, and `git diff --check` passed.

Initial attempts are retained rather than counted as passes: the first Mac home exceeded Unix socket path limits; Linux's first runner installation omitted its optional native bundler dependency; macOS archive metadata sidecars were initially collected as tests on Linux and were excluded with `--exclude '**/._*'` while all 137 actual tests ran. Early upload/removal assertions raced guest admission or incorrectly imposed a 2-second shutdown deadline; the final fixture checks the actual contracts above. The paused-filesystem decoding failure was a product bug and was fixed, not suppressed in the test.

## Artifacts and scope

- Local stage: `/private/tmp/msb-resident-perf.qHpm0p`; candidate package `node-candidate`, old binding `node-baseline`, final reports `node-live-results-final.json`, `node-timings-final.json`, and `node-unit-results-final.json`. Linux live/timing reports are also copied here with a `linux-` prefix.
- Linux stage: `/home/ubuntu/msb-resident-perf.zqr4vR`; final reports use the same names. Runtime `/bin/msb` under that stage is the previously qualified lifecycle candidate; live tests explicitly override the SDK's prebuilt discovery paths.
- Test homes: `/private/tmp/nl.u0biju` and `/home/ubuntu/nl.eRfSLi`. Test sandboxes were removed; image caches and reports are retained.
- macOS candidate Node binding SHA-256: `3ee1a932985d102e6977cf23da37a63eac374429906dbf02a8e0df2f5814147f`.
- macOS old Node binding SHA-256: `dae1ef24eb8d9716521f2c803d3c258d690e8902d7af8a2e4e53c12f9fab4bca`.
- Linux candidate Node binding SHA-256: `5467985868389923447bf0495b0cf7b42d8f49619b9b9ae20b4d7182170fda33`.

Runtime and guest-firmware provenance is recorded in [the resident lifecycle report](resident-lifecycle-2026-09-10.md). The local worktree advanced from `18c8fb86` to `e20269e1` through another contributor's archive commit during this work; those unrelated changes were preserved. Linux uses the existing isolated lifecycle source stage plus this Node/FS patch. Qualification preceded the separately authorized commit and push.

This qualifies the Node ownership follow-up on macOS ARM64 and Linux x86-64, not every mobility invariant. Windows, Linux ARM64, cloud execution, every SSH/streaming variant, retained-handle boot fencing, Python ownership optimization, and companion clock-patch integration are outside this run. No generated binding declarations, dependency versions, lockfiles, or submodule pointers were changed by this work.
