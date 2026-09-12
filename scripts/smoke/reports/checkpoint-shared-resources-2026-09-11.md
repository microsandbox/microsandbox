# Additional-volume and shared-resource qualification

Follow-up to PR #1557 at `8325189acca123c9a52c1dc2156b68183126e891`, using registry `msb_krun 0.1.37`. Initial qualification was committed as `5c644f42`. The subsequently approved gateway-identity fix is described below; the original failing evidence remains intact. This work does not add arbitrary external-disk capture or transparent TCP connection continuity.

## Managed additional disks

The fixture uses a 512 MiB guest with a 2 GiB memory ceiling, a 512 MiB layered or flat root, and two managed raw 128 MiB disks. Each volume contains an 8 MiB random payload with a checksum. Coverage includes full and subsequent captures, eager and forked restore, writes isolated from the source, baseline plus dependent archive export, reverse-order batch import into an empty home with offline restore, disk-only restore, direct full archive capture without an installed snapshot, direct branching and branch-of-branch, capture while user-paused, and graceful Stop followed immediately by Remove.

| Platform | Layered root | Flat root |
|---|---|---|
| Linux x86-64 / KVM | Pass | Pass |
| Windows ARM64 / WHP | Pass | Pass |

Single debug-build end-to-end CLI timings, not performance distributions or VM pause durations. Linux used `/dev/shm` because the host root filesystem had only 1.1 GiB free; these numbers are not representative of disk-backed storage latency. Windows used its normal temporary directory on disk. Host architecture, filesystem and storage differ, and some independent fixtures overlapped: this is not a controlled cross-platform speed benchmark. The substantially longer Windows paths need dedicated profiling before attributing the difference to any one subsystem.

| Operation | Linux layered | Linux flat | Windows layered | Windows flat |
|---|---:|---:|---:|---:|
| First full capture | 0.922 s | 0.931 s | 24.764 s | 24.088 s |
| Subsequent full capture | 0.527 s | 0.574 s | 14.509 s | 14.248 s |
| Eager restore | 0.655 s | 0.673 s | 15.497 s | 15.220 s |
| First forked restore | 0.658 s | 0.674 s | 14.797 s | 13.319 s |
| Imported dependent snapshot, forked restore | 0.716 s | 0.727 s | 11.977 s | 12.377 s |
| Disk-only restore from full | 0.411 s | 0.423 s | 4.059 s | 4.338 s |
| Direct full archive capture | 1.136 s | 1.284 s | 20.760 s | 22.796 s |
| Direct archive forked restore | 0.873 s | 0.970 s | 13.521 s | 14.154 s |
| Direct branch | 0.917 s | 0.920 s | 12.754 s | 12.856 s |
| Branch of branch | 1.015 s | 1.014 s | 11.946 s | 12.532 s |
| Capture while paused | 0.923 s | 0.930 s | 17.568 s | 15.533 s |

All eight owned VMs per matrix were gracefully stopped and removed without force. Windows Stop took 159–351 ms across both layouts; immediate Remove succeeded. Both the primary and offline-import catalogs were checked separately for leftovers.

## Shared virtio-fs directories

Mac ARM64/HVF, Linux x86-64/KVM, and Windows ARM64/WHP pass the shared-directory fixture: source and restored child concurrently write separate host-visible files; sequential overwrites of a common file reach the same host object; files created by the host after restore are visible to both; and a later strict restore rejects the changed tracked host state. This tests shared access, not application-level conflict resolution or atomic concurrent appends. The final fixture initializes the clients sequentially, waits until both guest writers reach a ready barrier, and releases both using a host-side start file.

External edits do not invalidate all guest caches synchronously. In the Mac probe, changing an eight-byte tracked file to a longer string produced `host-chan`: new host bytes read using the old cached length. Therefore it is inaccurate to promise only either a complete old value or a complete new value. Strict admission is a restore-time check, not ongoing isolation, a host-directory lock, or a coherence guarantee. The documentation now makes that boundary explicit.

## Network reconnect defect: baseline

Mac, Linux, and Windows preserve the source's established TCP connection across full capture. A child does not reuse that connection during the test observation. However, a fresh child TCP connection also fails in the tested restore: this is a bug, not the intended reconnect contract.

The fixture explicitly permits host egress in both VMs. It warms the source's gateway neighbour entry before capture. The child retains the source's guest MAC, IP addresses, routes, and neighbour entry in RAM. `apply_capture_network` restores the effective guest interface, but `SmoltcpNetwork::new` still derives the gateway MAC from the new sandbox slot. The restored guest therefore initially addresses frames to the old gateway MAC.

| Diagnostic | Mac ARM64 | Linux x86-64 | Windows ARM64 |
|---|---:|---:|---:|
| New child connection before intervention | Failed after 10 s | Failed after 10 s | Failed after 10 s |
| New connection after clearing only the disposable child's neighbour cache | Passed in 16 ms | Passed in 11 ms | Passed in 87 ms |

The diagnostic `ip neigh flush dev eth0` is not a product fix, and its success does not turn the preceding failure into a pass. It may also restart resolution rather than preserve identity. The fix therefore retains the captured virtual gateway identity in destination network construction, rather than requiring guest cache invalidation.

Old TCP connection state remains a separate boundary: restored applications need reconnect/retry logic because host proxy sockets are not captured. A restored socket may fail or wait for protocol timeout rather than immediately report disconnection. The fixture's two-second observation of the inherited child socket does not establish a maximum TCP timeout.

The initial host echo server is IPv4 loopback. The initial results above do not independently qualify IPv6, UDP, inbound published-port restoration, or transparent connection continuity.

## Approved gateway-identity fix and final live results

Capture now records `gateway_mac` in the existing network resource binding. Full installed/archive restore and direct branch pass it through the strict internal restore configuration, then select it before the fresh network poll thread starts. Recapturing a child preserves that same identity. Host sockets, queues, policy and port ownership remain child-owned; cold-boot slot allocation is unchanged. No new socket, daemon, agent protocol, public CLI flag, snapshot schema version, or steady-state packet check was added.

Networked development full snapshots without this field fail explicitly and need recapture; their old gateway MAC cannot reliably be inferred from user-overridable guest addresses. A live Mac negative test rejected a pre-fix full snapshot with the recapture diagnostic, while `--disk-only` from that exact snapshot booted, executed a command, and stopped/removed successfully. Network-free checkpoints do not require the field.

The final fixture warms guest IPv4 ARP and IPv6 neighbour caches before capture. It then tests eager restore, forked restore, direct branch, branch-of-branch, and direct full archive capture of a child followed by forked archive restore. All five children remain alive together and each repeats both address-family probes. The source's original TCP connection continues carrying messages after capture and after all children are created. No neighbour-cache flush occurs on a passing run.

| Final gateway-fix coverage | Mac ARM64/HVF | Linux x86-64/KVM | Windows ARM64/WHP |
|---|---|---|---|
| Eager/forked restore, IPv4 + IPv6 | Pass | Pass | Pass |
| Direct branch and branch-of-branch, IPv4 + IPv6 | Pass | Pass | Pass |
| Child recapture → direct full archive → forked restore | Pass | Pass | Pass |
| Five retained children; original source TCP connection retained | Pass | Pass | Pass |
| Cleanup and empty catalog | Pass | Pass | Pass |
| Previously failing fresh child connection, CLI wall time | 28 ms | 10 ms | 120 ms |

These are debug correctness observations, not latency percentiles. The dual-stack tests also expose a separate performance tail: some first IPv6 probes after restore/branch took about 1.02 s on Mac/Linux, while repeats took roughly 10–60 ms. Windows's source already took about 2.17 s for the IPv6 probe before capture; some restored first probes took about 3.2 s, with repeats around 2.15–2.21 s. The gateway fix removes the demonstrated stale-MAC failure; it is not a claim that all first-connection latency is optimized. These timings include the entire CLI command and its completion, not an isolated TCP handshake measurement.

Focused validation passed: two network identity/isolation tests (including a custom guest MAC), two captured-binding validation tests, six strict launch-contract tests, and 18 SDK checkpoint tests. Strict production Clippy and formatting checks passed. A no-network runtime build also passed with five existing unused-variable/mutability warnings. The first sandboxed SDK test attempt was denied permission to bind a local socket; all 18 passed when rerun with that permission. No whole-workspace pass is claimed.

Final live evidence: Mac `/private/tmp/cbh-shared-5_ycajk1/results.json`; OVH `/dev/shm/cbh-shared-5fli6zk2/results.json`; Surface `C:/Users/Stephen/AppData/Local/Temp/cbh-shared-xmx34iwl/results.json`. Local Linux/Windows copies are `linux-gateway-fixed.json` and `windows-gateway-fixed.json` under `/private/tmp/msb-resource-evidence/`. Final tested CLI SHA-256s are Mac `b6ff597ec009eedddda41f179e327b79d838cf1e42416726dbb16e60b5eaeaaa`, Linux `3a377227ed15a54cb27250b91e167cc47fd8e36aca6f79e784be4d3574aac0b6`, and Windows `962b08526b90c72e74e53025cece85246126552cf64b504d0e1101bbbdc8214b`. Agent and firmware hashes below are unchanged.

## Separate Windows CLI startup race

Starting two `msb exec` clients simultaneously in the same fresh home failed one command with `another microsandbox install operation is in progress`. This is independent of virtio-fs writes: `connect_and_migrate` acquires an install-exclusive database lease for descriptor reconciliation, but `MigrationLock::acquire` is a no-op on non-Unix systems. Unix holds a file lock around this path. Windows therefore lacks that serialization and competing normal clients can fail instead of waiting for initialization to finish.

The final filesystem fixture's barrier avoids simultaneous client initialization while retaining concurrent guest writes; it does not fix or qualify concurrent CLI startup. A Windows cross-process migration lock with ownership/drop semantics matching the Unix guard needs its own contention, cancellation, process-exit, and real installation-exclusion tests. The original failure is retained at Surface `C:/Users/Stephen/AppData/Local/Temp/cbh-shared-8a9j0fya/results.json`.

## Reproduction and evidence

Fixtures: `scripts/smoke/cli/checkpoint-managed-volumes.py` and `scripts/smoke/cli/checkpoint-shared-resources.py`. Pass explicit CLI, matching agentd, and firmware paths. Set `CBH_TEST_ROOT` to a short, writable directory on Unix; use `CBH_ROOT_DISK=flat:512M` for the flat-root volume repeat. Shared-resource modes are `fs` and `net`. On Windows use distinct sandbox names across concurrent homes because named pipes are machine-wide; the shared-resource fixture supplies unique names automatically.

All initial setup failures remain separate from product results: macOS's default temporary path exceeded the Unix socket limit; `/tmp` in a bind root hit the existing no-symlink-root policy; and host TCP access needs explicit network admission. Corrected runs use short canonical host paths and explicit `allow@host` rules. No security policy was weakened globally.

Evidence locations:

- Mac shared-directory pass: `/private/tmp/cbh-shared-0fisffkg/results.json`.
- Final barrier-based Mac/Linux shared-directory passes: `/private/tmp/cbh-shared-ffigwhvd/results.json` and OVH `/dev/shm/cbh-shared-_0b2mxu1/results.json`.
- Mac network failure and successful cache-clear diagnostic: `/private/tmp/cbh-shared-t2lohzr1/results.json`.
- OVH layered/flat managed-volume passes: `/dev/shm/cbh-volumes-75vficcj/results.json` and `/dev/shm/cbh-volumes-7nkziokd/results.json`.
- OVH shared-directory pass and network regression: `/dev/shm/cbh-shared-7hs0l7f8/results.json` and `/dev/shm/cbh-shared-vh5g3yo7/results.json`.
- Local copies of Linux evidence: `/private/tmp/msb-resource-evidence/`.
- Windows layered-volume pass: `C:/Users/Stephen/AppData/Local/Temp/cbh-volumes-9a1y1kvw/results.json`.
- Windows flat-volume pass: `C:/Users/Stephen/AppData/Local/Temp/cbh-volumes-c_b3p5pc/results.json`.
- Windows shared-directory pass and network regression: `C:/Users/Stephen/AppData/Local/Temp/cbh-shared-xww_6uzu/results.json` and `C:/Users/Stephen/AppData/Local/Temp/cbh-shared-8bmx2rbe/results.json`; local copies are also under `/private/tmp/msb-resource-evidence/`.

Artifact SHA-256s:

| Artifact | Mac ARM64 | Linux x86-64 |
|---|---|---|
| CLI | `4ee2291800f102fe598b3a670124bb5c017ce627c174a74487b0de64ff527524` | `77960bfea65ba68c139365be3d4c213091db7f711b2a73dad309f3e4e5299469` |
| agentd | `085e229a324c719a284809a29bc349618b9d056bfe899848512ae9e37f32202a` | `f0e3514eb926cd4687a34c74b207ff4ddfa4f1417712a63f5b8b52b69088ff4a` |
| libkrunfw | `ea0d458cdc12a0fa6dac8d192542ddc39717f816da41176582905e31a8bf868c` | `7d036cd4264a4f897d2600259f7524adcb8b690605f679294fdb5b77eab075d4` |

Windows hashes are CLI `05becf032b916ac3eeb48f9b8908ec2c9e6b6f729dfbd91a056aa52582eddfd8`, agentd `085e229a324c719a284809a29bc349618b9d056bfe899848512ae9e37f32202a`, and libkrunfw `9b2733b7f2bd759261cb8fca162e137f4d4d98a18cc8e12098254491da93e8be`. The Windows fixture uses the official Python 3.13.13 ARM64 embeddable package, extracted only inside the qualification folder after verifying SHA-256 `1230310118a6330cd6385cfc04de48bc77c7d18c240fd5fa23d054e50b1ebb85`; no system Python installation or persistent PATH change was made.

Initial qualification validation: all three native debug CLI builds completed; Python syntax compilation and `git diff --check` passed. That initial stage changed only tests and documentation; the approved Rust gateway fix and its validation are reported separately above. Snapshot artifacts, host fixture files and JSON logs were retained as evidence; no existing user sandbox or volume was removed.
