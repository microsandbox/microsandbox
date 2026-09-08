#!/usr/bin/env python3
"""Live Unix pin lifetime and same-name reservation checks for direct branching."""

import fcntl
import json
import os
from pathlib import Path
import subprocess

binary = os.environ["MSB_PATH"]
home = Path(os.environ["MSB_HOME"])
prefix = f"branch-own-{os.getpid()}"
names = [prefix, prefix + ".child", prefix + ".race"]


def call(*args, expected=0):
    result = subprocess.run([binary, *args], capture_output=True, text=True, timeout=120)
    if expected is not None:
        assert result.returncode == expected, result.stderr
    return result


def evictable(path):
    # Same OS primitive used by production eviction. Never unlink or modify live backing.
    with path.open("rb") as file:
        try:
            fcntl.flock(file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return False
        return True


try:
    call("create", "alpine", "--name", prefix, "--root-disk", "tmpfs:128M", "--memory", "256M")
    cache = home / "cache" / "memory" / "branches"
    before = set(cache.glob("*.ram"))
    call("branch", prefix, "--name", names[1])
    paths = set(cache.glob("*.ram")) - before
    assert len(paths) == 1
    backing = paths.pop()
    assert backing.stat().st_mode & 0o777 == 0o400
    assert not evictable(backing), "source/child pins disappeared"
    assert not (home / "sandboxes" / names[1] / ".branch-restore").exists()
    attempts = [subprocess.Popen([binary, "branch", prefix, "--name", names[2]], stdout=subprocess.PIPE, stderr=subprocess.PIPE) for _ in range(2)]
    statuses = []
    for attempt in attempts:
        attempt.communicate(timeout=120)
        statuses.append(attempt.returncode)
    assert sorted(statuses) == [0, 1], statuses
    call("stop", prefix)
    assert not evictable(backing), "child depended on the source's pin"
    call("exec", names[1], "--", "true")
    call("stop", names[1])
    assert evictable(backing), "pin leaked after final VM teardown"
    print(json.dumps({"independent_child_pin": "pass", "release_after_teardown": "pass", "same_name_race": "pass", "dot_name": "pass"}))
finally:
    for name in reversed(names):
        call("stop", name, expected=None)
