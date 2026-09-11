# Customer checkpoint fixes — partial qualification

This report tracks the existing customer-fix PR pair, not completion of all six customer issues. Microsandbox is based on `releases/v0.7.0` at `13155ca0784e1536a0287fbda6e77dccb97fb3b2`; the companion libkrun checkpoint fixes are `97a8042c6861a5e87130c4fa9d93677bb40c29ea` on top of `krun`. No packages were released.

## Grown-memory restore

The disposable CLI fixture creates a 256 MiB VM with a 1 GiB ceiling and one CPU with a two-CPU ceiling, grows it to 768 MiB/two CPUs, and fills tmpfs with 384 MiB of random data. That payload cannot fit in the original boot RAM. A full checkpoint must reconstruct the original guest-physical layout plus the captured hotplug occupancy, rather than booting a new 768 MiB layout.

Both macOS ARM64/HVF and Linux x86-64/KVM passed eager restore, forked restore, and direct branching with the captured RAM checksum intact. Both restored children then grew to 1 GiB and shrank to one CPU; the fixture verifies guest CPU online state and checks the payload again. This caught and fixed a configuration projection bug that otherwise caused a subsequent CPU change to be silently skipped.

| Debug-build CLI duration | macOS ARM64 | Linux x86-64 |
|---|---:|---:|
| Full capture | 3.312 s | 3.260 s |
| Eager restore | 1.442 s | 1.796 s |
| First forked restore | 1.413 s | 1.795 s |
| Direct branch | 1.662 s | 1.106 s |

These are single correctness-run wall-clock timings, not release performance benchmarks, pause durations, or a before/after speedup claim. The tests used matching freshly built agentd binaries. The reported run predates only the final additional resource-query deadline bound; its dependency sources match the pinned companion commit.

Reproduce with `python3 scripts/smoke/cli/checkpoint-memory-growth.py /absolute/path/to/msb /absolute/path/to/agentd /absolute/path/to/libkrunfw`. The exact Mac binary must be codesigned with `msb-entitlements.plist`. `CBH_ROOT_DISK=flat:512M` selects a flat root; the first runs reported above used the layered root.

Evidence: Mac `/tmp/cbh-j3swr8nn/results.json`; OVH `/tmp/cbh-ux6l5nzg/results.json`. The final pinned build, including the resource-query deadline bound and no local dependency overrides, also passed the entire fixture with flat roots on both hosts: Mac `/tmp/cbh-d2__po51/results.json`, OVH `/tmp/cbh-840jpz18/results.json`. All fixture sandboxes were stopped. Raw RAM/disk artifacts are not committed.

## Automated checks

- macOS libkrun: 144 device tests, 11 VM API tests, and the eager fresh-mapping topology regression passed.
- Linux libkrun: 164 device tests passed with the existing hanging vsock quiescence test excluded; the new bounded WRITE_ZEROES test passed. The fresh-mapping topology regression passed on Linux x86-64 as well as Mac ARM64.
- Microsandbox: 38 image checkpoint tests passed, one opt-in experiment ignored; 19 restore-related SDK tests passed, including complete control-frame delivery, captured requested-versus-actual targets, and preservation of concurrent desired configuration edits.
- The final pinned build passed 81 SDK snapshot tests and 53 runtime checkpoint tests. These are automated checks, not substitutes for VM qualification.

## Ubuntu live disk growth

`scripts/smoke/cli/bounded-root-grow.py` passed on Linux/KVM with a fresh Ubuntu 24.04 managed raw upper and default writeback limiting. The same running VM grew 10 → 12 → 14 GiB; ext4 reported both filesystem expansions complete, and a 32 MiB random file retained its checksum. CLI modify durations were 26.96 ms and 13.80 ms, single debug diagnostic trials. Repeating an already-completed target correctly retained the existing grow-only rejection. This does not test recovery from a previously partially failed resize.

Evidence: OVH `/tmp/cbg-t14q2e3t/results.json`. An initial 10 → 12 GiB trial also passed, but its fixture incorrectly expected equal-target success; the fixture now asserts the existing rejection. A separate retry hit the host temporary-storage quota. Only three validated, stopped, disposable test homes from this session were removed to reclaim space; earlier JSON results were retained, but those synthetic RAM/disk artifacts are not recoverable. The final test VM was stopped successfully. This work does not disable bounded writeback or modify the guest image/kernel.

## Still required

The complete six-issue fix is not ready: unused-ceiling capture elision and coherent startup/progress budgets, additional virtiofs/volume providers, Stop ownership semantics across SDKs, and complete cancellation/catalog cleanup remain unfinished. Unplug-zero ordering changes require additional approval before implementation. Large-ceiling release benchmarks, full grow/shrink/incremental/archive matrices, and Windows/Linux ARM64 qualification are not claimed by this report.

Full checkpoints now require original construction geometry and CPU/memory hotplug device records. The user approved rejecting earlier development full snapshots that lack them. Released disk-only snapshot formats are unchanged.
