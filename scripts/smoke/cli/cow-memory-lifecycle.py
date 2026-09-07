#!/usr/bin/env python3
"""Isolated #8 live smoke matrix; every started sandbox is stopped in finally."""
import json
import os
from pathlib import Path
import subprocess
import time

binary = os.environ["MSB_PATH"]
root = Path(os.environ["STACK8_OUT"])
root.mkdir(parents=True, exist_ok=True)
prefix = os.environ.get("STACK8_PREFIX", "cow8")
mode = os.environ.get("STACK8_MODE", "cow")
layout = os.environ.get("STACK8_LAYOUT", "flat:512M")
rows = []
names = []

def run(label, *args, expected=0, timeout=120):
    started = time.perf_counter()
    result = subprocess.run([binary, *args], text=True, capture_output=True, timeout=timeout)
    elapsed = (time.perf_counter() - started) * 1000
    (root / (label + ".stdout")).write_text(result.stdout)
    (root / (label + ".stderr")).write_text(result.stderr)
    row = {"case": label, "ms": round(elapsed, 2), "exit": result.returncode}
    rows.append(row)
    print(json.dumps(row), flush=True)
    if expected is not None and result.returncode != expected:
        raise RuntimeError(f"{label}: {result.stderr[-3000:]}")
    return result

try:
    source = prefix + "-source"
    names.append(source)
    run("fresh-" + mode, "run", "-d", "-n", source, "--memory-snapshot", mode,
        "--root-disk", layout, "--memory", "256M", "--cpus", "2", "alpine",
        "--", "sh", "-c", "mkdir -p /dev/shm; echo captured > /dev/shm/cow-marker; i=0; while :; do echo $i > /tmp/cow-counter; i=$((i+1)); sleep 0.05; done")
    run("marker-source", "exec", source, "--", "cat", "/dev/shm/cow-marker")
    boot_id = run("boot-id-before", "exec", source, "--", "cat", "/proc/sys/kernel/random/boot_id").stdout.strip()
    process = run("process-before", "exec", source, "--", "sh", "-c", "for p in /proc/[0-9]*/cmdline; do tr '\\0' ' ' < $p; echo; done").stdout
    assert "cow-counter" in process
    snap = prefix + "-full"
    run("first-full", "snapshot", "create", snap, "--from", source, "--full", "--info")
    run("pause", "pause", source)
    run("pause-idempotent", "pause", source)
    inspected = run("paused-inspect", "inspect", source, "--format", "json")
    assert json.loads(inspected.stdout)["status"] == "Paused"
    refusal = run("paused-exec", "exec", source, "--", "true", expected=None, timeout=10)
    assert refusal.returncode != 0, "paused exec must fail promptly"
    run("paused-full-1", "snapshot", "create", prefix + "-paused1", "--from", source, "--full", "--info")
    run("paused-full-2", "snapshot", "create", prefix + "-paused2", "--from", source, "--full", "--info")
    time.sleep(float(os.environ.get("STACK8_PAUSE_SECONDS", "5")))
    run("resume", "resume", source)
    run("resume-idempotent", "resume", source)
    assert run("boot-id-after", "exec", source, "--", "cat", "/proc/sys/kernel/random/boot_id").stdout.strip() == boot_id
    guest_time = run("wall-clock-after", "exec", source, "--", "date", "+%s").stdout.strip()
    assert abs(time.time() - int(guest_time)) < 3, f"guest wall clock stale: {guest_time}"
    first_counter = run("counter-after", "exec", source, "--", "cat", "/tmp/cow-counter").stdout.strip()
    time.sleep(0.2)
    next_counter = run("counter-progress", "exec", source, "--", "cat", "/tmp/cow-counter").stdout.strip()
    assert int(next_counter) > int(first_counter), "original workload must continue after resume"
    run("marker-after-resume", "exec", source, "--", "cat", "/dev/shm/cow-marker")
    for suffix in ("a", "b"):
        child = prefix + "-" + suffix
        names.append(child)
        run("restore-" + suffix, "create", "-n", child, "--from-snapshot", snap,
            "--memory-snapshot", mode, "--info")
        result = run("marker-" + suffix, "exec", child, "--", "cat", "/dev/shm/cow-marker")
        assert result.stdout.strip() == "captured"
    run("mutate-a", "exec", prefix + "-a", "--", "sh", "-c", "echo private-a > /dev/shm/cow-marker")
    assert run("isolation-b", "exec", prefix + "-b", "--", "cat", "/dev/shm/cow-marker").stdout.strip() == "captured"
    assert run("isolation-source", "exec", source, "--", "cat", "/dev/shm/cow-marker").stdout.strip() == "captured"
    archive = str(root / "direct.msnap")
    run("direct-full", "snapshot", "create", prefix + "-direct", "--from", source,
        "--full", "--archive", archive, "--info")
    child = prefix + "-archive"
    names.append(child)
    run("direct-restore", "create", "-n", child, "--from-snapshot", archive,
        "--memory-snapshot", mode, "--info")
    Path(archive).unlink()
    run("archive-unlink-survival", "exec", child, "--", "cat", "/dev/shm/cow-marker")
    run("pause-for-stop", "pause", source)
    run("stop-paused", "stop", source, timeout=20)
    run("child-after-source-stop", "exec", prefix + "-a", "--", "cat", "/dev/shm/cow-marker")
finally:
    for name in reversed(names):
        try:
            run("cleanup-" + name, "stop", name, expected=None, timeout=20)
        except Exception as error:
            rows.append({"case": "cleanup-" + name, "error": str(error)})
    (root / "results.json").write_text(json.dumps(rows, indent=2))
