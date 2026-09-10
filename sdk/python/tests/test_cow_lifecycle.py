"""Opt-in live CoW lifecycle check using a matching runtime/kernel bundle."""

import os
from pathlib import Path

import pytest

from microsandbox import Sandbox, Snapshot


@pytest.mark.skipif(os.environ.get("MSB_COW_LIVE") != "1", reason="requires matching live bundle")
@pytest.mark.asyncio
async def test_cow_resident_capture_and_child_isolation():
    name = f"cow8-python-{os.getpid()}"
    source = await Sandbox.create(
        name, image="alpine", memory=256
    )
    child = None
    branches = []
    try:
        await source.exec("sh", ["-c", "echo source > /dev/shm/sdk-marker"])
        await source.pause()
        paused = await Sandbox.get(name)
        assert str(paused.status) == "paused"
        branched = await paused.branch(f"{name}-paused-branch")
        branches.append(branched)
        assert (await branched.exec("cat", ["/dev/shm/sdk-marker"])).stdout_text.strip() == "source"
        snapshot = await Snapshot.create(f"{name}-full", from_sandbox=name, full=True)
        assert (Path(snapshot.path) / "snapshot.json").is_file()
        await paused.resume()
        # The returned artifact path selects the exact member in its snapshot group.
        child = await Sandbox.create(
            f"{name}-child", from_snapshot=snapshot.path, forked=True
        )
        result = await child.exec("cat", ["/dev/shm/sdk-marker"])
        assert result.stdout_text.strip() == "source"
        await child.exec("sh", ["-c", "echo child > /dev/shm/sdk-marker"])
        descendant = await child.branch(f"{name}-branch")
        branches.append(descendant)
        result = await descendant.exec("cat", ["/dev/shm/sdk-marker"])
        assert result.stdout_text.strip() == "child"
        result = await source.exec("cat", ["/dev/shm/sdk-marker"])
        assert result.stdout_text.strip() == "source"
        await child.pause()
    finally:
        for branched in reversed(branches):
            await branched.stop()
        if child is not None:
            await child.stop()
        await source.stop()
