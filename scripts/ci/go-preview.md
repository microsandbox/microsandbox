# Go customer previews

`releases/build-pr-1537` is an append-only build and packaging branch. It does
not create release tags, GitHub releases, or stable package versions.

## Publish an update

1. In a clean checkout of the helper branch, merge the desired feature-branch
   revision. Preserve earlier packaging commits in its history. Resolve conflicts
   explicitly, review the diff, and push a signed commit without `[skip ci]`.
   Do not force-push the branch. The Go CI tests use the newly built external FFI,
   so embedded binaries from a previous preview are not used for those tests.
2. For the complete Check -> runtime image -> bundled SDK sequence, run:

   ```sh
   python3 scripts/ci/package-go-preview.py --run-id <check-run> \
     --wait --publish-runtime --publish
   ```

   The command stops on a failed build. It publishes only once all checks pass.
   If the runtime image is already published, omit `--publish-runtime`.

   Alternatively, wait for that commit's **Check** run to succeed and perform
   the runtime and SDK publication separately:
3. Dispatch **Publish Runtime Commit** on `main`, with `commit` set to the full
   checked helper-branch SHA, and wait for it to succeed:

   ```sh
   gh workflow run publish-runtime-commit.yml \
     --repo superradcompany/microsandbox --ref main -f commit=<checked-sha>
   ```

4. Run the packaging command from this repository:

   ```sh
   python3 scripts/ci/package-go-preview.py --run-id <successful-check-run> --publish
   ```

   Without `--publish`, it prepares and validates an isolated checkout only.
   It requires authenticated `gh`, signed Git commits, Go, and Docker with Linux
   amd64 support. It downloads the FFI from the exact Check run, verifies the
   matching OCI tag, builds the SDK without development tags, and loads its
   embedded FFI inside the matching runtime image. This check does not start a VM.
   A racing branch update aborts publication. The signed packaging commit uses
   `[skip ci]` so it does not start another build cycle.

5. Share the printed packaging SHA and immutable runtime image reference together.
   `sdk/go/preview.json` records their source commit, Check run and FFI checksum.
   The customer must pin the packaging SHA, not the unbundled feature-branch SHA.

## Customer setup

These previews currently support **Linux amd64**. Other platforms retain empty
bundle sentinels and are not supported by this preview publisher.

```sh
go get github.com/superradcompany/microsandbox/sdk/go@<packaging-sha>
```

Build the application normally, without `microsandbox_ffi_path`. Use the matching
runtime image from `preview.json` as the application's final image, copy in the
application, and override the image's `msb` entrypoint:

```dockerfile
FROM ghcr.io/superradcompany/microsandbox@sha256:<manifest-digest>
COPY app /usr/local/bin/app
ENV MSB_PATH=/usr/local/bin/msb
ENV MSB_LIBKRUNFW_PATH=/usr/local/lib/libkrunfw.so.5.6.1
ENTRYPOINT ["/usr/local/bin/app"]
```

Do not call `EnsureInstalled()` in this container: the runtime is preinstalled,
whereas that helper downloads stable versioned release artifacts. The application
user needs a writable home directory for FFI extraction. Each preview extracts
into its own source-SHA cache directory, allowing installed previews to coexist.
Normal Linux runtime requirements, including access to KVM, still apply.

Updates are explicit: change both the Go revision and runtime image together.
Keep old revisions pinned for reproducible builds and rollback; do not use a
moving branch name or a mutable image tag in the customer's deployment.

The source branch requires macOS 15+ for native macOS runtime builds because
snapshot capture/restore uses Hypervisor GIC APIs introduced in macOS 15. This
does not add macOS FFI bundles to the Linux amd64 customer preview.
