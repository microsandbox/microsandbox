#!/usr/bin/env python3
"""Append a bundled Go SDK preview to a checked helper branch, without a release."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

REPOSITORY = "superradcompany/microsandbox"
MODULE = f"github.com/{REPOSITORY}/sdk/go"
IMAGE = f"ghcr.io/{REPOSITORY}"
ARTIFACT = "go-ffi-linux-x86_64"
BUNDLE = "libmicrosandbox_go_ffi-linux-amd64.so"
CACHE_PATTERN = r'libDir := filepath.Join\(dir, "lib", (?:"v"\+sdkVersion|"preview-[0-9a-f]{40}")\)'


def run(*args: str, cwd: Path | None = None, env: dict | None = None) -> str:
    return subprocess.check_output(args, cwd=cwd, env=env, text=True).strip()


def api(path: str) -> dict:
    return json.loads(run("gh", "api", f"repos/{REPOSITORY}/{path}"))


def wait_for_run(run_id: int) -> dict:
    deadline = time.monotonic() + 3 * 60 * 60
    previous = None
    while time.monotonic() < deadline:
        checked = api(f"actions/runs/{run_id}")
        status = (checked["status"], checked["conclusion"])
        if status != previous:
            print(f"{checked['html_url']}: {status}", flush=True)
            previous = status
        if checked["status"] == "completed":
            if checked["conclusion"] != "success":
                raise SystemExit("Workflow did not succeed; no SDK was published")
            return checked
        time.sleep(30)
    raise SystemExit("Timed out waiting for workflow; no SDK was published")


def validate_check(checked: dict, branch: str) -> str:
    sha = checked["head_sha"]
    if (
        not re.fullmatch(r"[0-9a-f]{40}", sha)
        or checked["conclusion"] != "success"
        or checked["path"] != ".github/workflows/check.yml"
        or checked["head_repository"]["full_name"] != REPOSITORY
        or checked["head_branch"] != branch
        or checked["event"] != "push"
    ):
        raise ValueError("Expected a successful same-repository Check push run on the helper branch")
    return sha


def preview_setup(source: str, sha: str) -> str:
    result, count = re.subn(
        CACHE_PATTERN,
        f'libDir := filepath.Join(dir, "lib", "preview-{sha}")',
        source,
    )
    if count != 1:
        raise ValueError("Go FFI cache layout changed; review preview isolation before publishing")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-id", type=int, required=True, help="successful Check run")
    parser.add_argument("--branch", default="releases/build-pr-1537")
    parser.add_argument("--wait", action="store_true", help="wait for Check to finish")
    parser.add_argument("--publish-runtime", action="store_true",
                        help="dispatch and wait for the commit OCI publisher on main")
    parser.add_argument("--publish", action="store_true", help="sign and push the packaging commit")
    args = parser.parse_args()
    if not re.fullmatch(r"releases/[A-Za-z0-9][A-Za-z0-9/_-]*", args.branch):
        parser.error("branch must be a releases/ helper branch")

    checked = wait_for_run(args.run_id) if args.wait else api(f"actions/runs/{args.run_id}")
    sha = validate_check(checked, args.branch)
    if api(f"git/ref/heads/{args.branch}")["object"]["sha"] != sha:
        raise SystemExit("Helper branch moved since this Check run; use its latest successful run")

    if args.publish_runtime:
        dispatched = run(
            "gh", "workflow", "run", "publish-runtime-commit.yml", "--repo", REPOSITORY,
            "--ref", "main", "-f", f"commit={sha}",
        )
        print(dispatched, flush=True)
        match = re.search(r"github\.com/" + re.escape(REPOSITORY) + r"/actions/runs/(\d+)", dispatched)
        if not match:
            raise SystemExit(
                "Runtime workflow dispatched but gh did not return its run URL. "
                "Wait for it to succeed, then rerun without --publish-runtime."
            )
        wait_for_run(int(match.group(1)))

    # Require the immutable runtime tag to exist before offering an SDK revision.
    image_data = json.loads(run(
        "docker", "buildx", "imagetools", "inspect", f"{IMAGE}:{sha}",
        "--format", "{{json .Manifest}}",
    ))
    digest = image_data["digest"]
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
        raise SystemExit("Runtime image did not return a valid manifest digest")
    if not any(
        item.get("platform", {}).get("os") == "linux"
        and item.get("platform", {}).get("architecture") == "amd64"
        for item in image_data["manifests"]
    ):
        raise SystemExit("Runtime image does not include linux/amd64")

    script = Path(__file__).resolve()
    root = Path(run("git", "rev-parse", "--show-toplevel", cwd=script.parent))
    # Retain the isolated checkout for inspection on both success and failure.
    work = Path(tempfile.mkdtemp(prefix="msb-go-preview-"))
    checkout = work / "checkout"
    remote = f"https://github.com/{REPOSITORY}.git"
    run("git", "fetch", remote, f"refs/heads/{args.branch}", cwd=root)
    run("git", "worktree", "add", "--detach", str(checkout), sha, cwd=root)
    print(f"Preparing {sha} in {checkout}", flush=True)

    artifact_dir = work / "ffi"
    run("gh", "run", "download", str(args.run_id), "--repo", REPOSITORY,
        "--name", ARTIFACT, "--dir", str(artifact_dir))
    library = artifact_dir / "libmicrosandbox_go_ffi.so"
    data = library.read_bytes()
    # ELF64, little endian, EM_X86_64. Never ship an empty sentinel or wrong arch.
    if len(data) < 64 or data[:6] != b"\x7fELF\x02\x01" or data[18:20] != b"\x3e\x00":
        raise SystemExit("CI artifact is not a Linux amd64 ELF shared library")
    if len(data) >= 100 * 1024 * 1024:
        raise SystemExit("FFI exceeds GitHub's per-file limit; strip it in the build job first")

    sdk = checkout / "sdk/go"
    bundles = sdk / "internal/bundle/bundles"
    # A previous preview may contain other binaries. Never carry them forward.
    for path in bundles.iterdir():
        if path.name.startswith("libmicrosandbox_go_ffi-"):
            path.write_bytes(b"")
    (bundles / BUNDLE).write_bytes(data)
    setup = sdk / "setup.go"
    setup.write_text(preview_setup(setup.read_text(), sha))
    manifest = {
        "source_commit": sha,
        "check_run": checked["html_url"],
        "runtime_image": f"{IMAGE}@{digest}",
        "runtime_tag": f"{IMAGE}:{sha}",
        "platforms": ["linux/amd64"],
        "ffi_sha256": hashlib.sha256(data).hexdigest(),
    }
    (sdk / "preview.json").write_text(json.dumps(manifest, indent=2) + "\n")
    # Keep the repeatable publisher and instructions on the packaging branch.
    relative = script.relative_to(root)
    (checkout / relative).write_bytes(script.read_bytes())
    guide = root / "scripts/ci/go-preview.md"
    (checkout / "scripts/ci/go-preview.md").write_bytes(guide.read_bytes())
    tests = root / "scripts/ci/test_package_go_preview.py"
    (checkout / "scripts/ci/test_package_go_preview.py").write_bytes(tests.read_bytes())

    env = {**os.environ, "GOOS": "linux", "GOARCH": "amd64", "CGO_ENABLED": "0"}
    run("go", "build", "./...", cwd=sdk, env=env)
    # Exercise the normal embedded-library path in Linux, without dev build tags.
    probe = work / "probe.go"
    probe.write_text(
        'package main\nimport ("fmt"; m "' + MODULE + '")\n'
        'func main() { v, err := m.RuntimeVersion(); if err != nil { panic(err) }; '
        'if v == "" { panic("empty runtime version") }; fmt.Println(v) }\n'
    )
    run("go", "build", "-o", str(work / "probe"), str(probe), cwd=sdk, env=env)
    version = run(
        "docker", "run", "--rm", "--platform", "linux/amd64", "--network", "none",
        "-v", f"{work / 'probe'}:/preview-probe:ro", "--entrypoint", "/preview-probe",
        f"{IMAGE}@{digest}",
    )
    print(f"Embedded FFI loaded in the runtime image: {version}", flush=True)
    run("git", "diff", "--check", cwd=checkout)
    print(run("git", "diff", "--stat", cwd=checkout), flush=True)
    print(json.dumps(manifest, indent=2), flush=True)
    if not args.publish:
        print(f"Prepared only: {checkout}. Rerun with --publish to sign and push.")
        return
    if api(f"git/ref/heads/{args.branch}")["object"]["sha"] != sha:
        raise SystemExit("Branch moved during packaging; nothing was pushed")
    run("git", "add", "sdk/go/setup.go", "sdk/go/preview.json",
        "sdk/go/internal/bundle/bundles", str(relative), "scripts/ci/go-preview.md", "scripts/ci/test_package_go_preview.py", cwd=checkout)
    run("git", "diff", "--cached", "--check", cwd=checkout)
    run("git", "commit", "-S", "-m", "build(go): package customer preview [skip ci]",
        "-m", f"Embed the Linux amd64 FFI from {sha}.\n\n"
        "Isolate the preview FFI cache and record the matching runtime image.\n"
        "Keep this commit reachable on the helper branch without release tags.", cwd=checkout)
    packaged = run("git", "rev-parse", "HEAD", cwd=checkout)
    # Fast-forward only: a racing source update must fail, never be overwritten.
    run("git", "push", remote, f"HEAD:refs/heads/{args.branch}", cwd=checkout)
    print(f"SDK: go get {MODULE}@{packaged}")
    print(f"Runtime: {IMAGE}@{digest}")
    print(f"Checkout: {checkout}", flush=True)
    # Verify the exact customer dependency through the public Go module proxy.
    proxy_env = {**os.environ, "GOPROXY": "https://proxy.golang.org",
                 "GOPRIVATE": "", "GONOPROXY": "", "GONOSUMDB": "", "GOWORK": "off"}
    downloaded = json.loads(run(
        "go", "mod", "download", "-json", f"{MODULE}@{packaged}", cwd=work, env=proxy_env,
    ))
    downloaded_ffi = Path(downloaded["Dir"]) / "internal/bundle/bundles" / BUNDLE
    if hashlib.sha256(downloaded_ffi.read_bytes()).hexdigest() != manifest["ffi_sha256"]:
        raise SystemExit("Published Go module does not contain the expected FFI")
    print(f"Verified public Go module: {downloaded['Version']}", flush=True)


if __name__ == "__main__":
    main()
