# Full snapshots from a standalone Go project

This project runs a Go process inside a sandbox and verifies its in-memory
identity, PID, boot ID, value and counter survive full snapshots. All host-side
sandbox operations use the public Go SDK; the guest workload is also Go.

The matrix covers managed, flat and tmpfs roots; running and paused full capture;
normal and copy-on-write restores; parent/child RAM and filesystem isolation;
branching a paused restored child; full archive export/import; direct archive
restore; deleting the input archive after restore; and stopping a paused VM.
Every command checks its exit status, and every created sandbox is stopped and
removed by test cleanup. Snapshot artifacts remain in the printed isolated test
home for inspection. These are functional checks, not physical memory-sharing
measurements or compatibility tests across releases.

The Go module uses a local SDK replacement for repository CI. That SDK must
contain the matching embedded FFI binary for the host platform; an ordinary
source checkout has empty sentinels. CI fills it from the selected Check run.
No `microsandbox_ffi_path` build tag is used.

Run with a matching runtime and firmware:

```sh
MSB_PATH=/absolute/path/to/msb \
MSB_LIBKRUNFW_PATH=/absolute/path/to/libkrunfw.so.5.6.1 \
go test -v -count=1 -timeout=25m ./...
```

Linux requires writable KVM access. On macOS use Apple Silicon, macOS 15+, a
codesigned runtime and `libkrunfw.5.dylib`. The guest Go program is cross-compiled
by `TestMain` for Linux on the host architecture. The host needs Go installed.
