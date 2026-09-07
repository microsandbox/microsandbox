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
resize = os.environ.get("STACK8_LIVE_RESIZE") == "1"
rows = []
names = []

def run(label, *args, expected=0, timeout=120):
    started = time.perf_counter()
    try:
        result = subprocess.run([binary, *args], text=True, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        # TimeoutExpired can carry bytes even when text=True. Preserve the failed
        # command in the evidence instead of recording only subsequent cleanup.
        for stream in ("stdout", "stderr"):
            captured = getattr(error, stream) or b""
            if isinstance(captured, bytes):
                captured = captured.decode("utf-8", errors="replace")
            (root / (label + "." + stream)).write_text(captured)
        row = {"case": label, "ms": round((time.perf_counter() - started) * 1000, 2),
               "exit": "timeout", "timeout_seconds": timeout}
        rows.append(row)
        print(json.dumps(row), flush=True)
        raise
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
    if os.environ.get("STACK8_CHECK_COW_REJECTION") == "1":
        refused = prefix + "-cow-refused"
        names.append(refused)
        result = run("cow-unsupported", "create", "alpine", "-n", refused,
                     "--memory", "256M", "--memory-snapshot", "cow", expected=None)
        assert result.returncode != 0, "unsupported CoW must not silently start eagerly"
        assert "not qualified" in result.stderr or "not qualified" in result.stdout
        inspected = run("cow-refused-inspect", "inspect", refused, "--format", "json", expected=None)
        if inspected.returncode == 0:
            assert json.loads(inspected.stdout)["status"] not in ("Running", "Paused")
    source = prefix + "-source"
    names.append(source)
    run("fresh-" + mode, "run", "-d", "-n", source, "--memory-snapshot", mode,
        "--root-disk", layout, "--memory", "256M", "--cpus", "2",
        *(["--max-memory", "512M"] if resize else []), "alpine",
        "--", "sh", "-c", "mkdir -p /dev/shm; echo captured > /dev/shm/cow-marker; i=0; while :; do echo $i > /tmp/cow-counter; i=$((i+1)); sleep 0.05; done")
    # Detached launch acknowledges the runtime, not the application's first write.
    for attempt in range(30):
        ready = run("application-ready-" + str(attempt), "exec", source, "--", "test", "-s", "/dev/shm/cow-marker", expected=None)
        if ready.returncode == 0:
            break
        time.sleep(0.1)
    else:
        raise RuntimeError("application did not initialize its marker")
    run("marker-source", "exec", source, "--", "cat", "/dev/shm/cow-marker")
    boot_id = run("boot-id-before", "exec", source, "--", "cat", "/proc/sys/kernel/random/boot_id").stdout.strip()
    process = run("process-before", "exec", source, "--", "sh", "-c", "for p in /proc/[0-9]*/cmdline; do tr '\\0' ' ' < $p; echo; done").stdout
    assert "cow-counter" in process
    if resize:
        baseline = int(run("memory-baseline", "exec", source, "--", "sh", "-c",
                           "awk '/MemTotal/ {print $2}' /proc/meminfo").stdout.strip())
        for step, target in enumerate((384, 256, 512, 256)):
            run(f"memory-target-{step}", "modify", source, "--memory", f"{target}M", "--format", "json")
            deadline = time.monotonic() + 30
            sample = 0
            while True:
                observed = int(run(f"memory-convergence-{step}-{sample}", "exec", source,
                                   "--", "sh", "-c", "awk '/MemTotal/ {print $2}' /proc/meminfo").stdout.strip())
                # Hotplug metadata consumes some newly onlined pages. Check actual guest
                # capacity within 4 MiB, not just an accepted host target/configuration.
                if abs(observed - (baseline + (target - 256) * 1024)) <= 4096:
                    break
                assert time.monotonic() < deadline, f"memory target {target} did not converge: {observed} KiB"
                sample += 1
                time.sleep(0.1)
            assert run(f"memory-marker-{step}", "exec", source, "--", "cat", "/dev/shm/cow-marker").stdout.strip() == "captured"
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
    if os.environ.get("STACK8_KEEP_ARCHIVE") != "1":
        Path(archive).unlink()
    assert run("archive-child-exec", "exec", child, "--", "cat", "/dev/shm/cow-marker").stdout.strip() == "captured"
    run("pause-for-stop", "pause", source)
    run("stop-paused", "stop", source, timeout=20)
    assert run("child-after-source-stop", "exec", prefix + "-a", "--", "cat", "/dev/shm/cow-marker").stdout.strip() == "private-a"
finally:
    for name in reversed(names):
        try:
            run("cleanup-" + name, "stop", name, expected=None, timeout=20)
        except Exception as error:
            rows.append({"case": "cleanup-" + name, "error": str(error)})
    (root / "results.json").write_text(json.dumps(rows, indent=2))
