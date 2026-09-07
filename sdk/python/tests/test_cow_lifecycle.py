"""Opt-in live CoW lifecycle check using a matching runtime/kernel bundle."""

import os

import pytest

from microsandbox import MemorySnapshotMode, Sandbox, Snapshot


@pytest.mark.skipif(os.environ.get("MSB_COW_LIVE") != "1", reason="requires matching live bundle")
@pytest.mark.asyncio
async def test_cow_resident_capture_and_child_isolation():
    name = f"cow8-python-{os.getpid()}"
    source = await Sandbox.create(
        name, image="alpine", memory=256, memory_snapshot=MemorySnapshotMode.COW
    )
    child = None
    try:
        await source.exec("sh", ["-c", "echo source > /dev/shm/sdk-marker"])
        await source.pause()
        paused = await Sandbox.get(name)
        assert str(paused.status) == "paused"
        await Snapshot.create(f"{name}-full", from_sandbox=name, full=True)
        await paused.resume()
        child = await Sandbox.create(
            f"{name}-child", from_snapshot=f"{name}-full", memory_snapshot=MemorySnapshotMode.COW
        )
        result = await child.exec("cat", ["/dev/shm/sdk-marker"])
        assert result.stdout_text.strip() == "source"
        await child.exec("sh", ["-c", "echo child > /dev/shm/sdk-marker"])
        result = await source.exec("cat", ["/dev/shm/sdk-marker"])
        assert result.stdout_text.strip() == "source"
        await child.pause()
    finally:
        if child is not None:
            await child.stop()
        await source.stop()
