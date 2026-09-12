"""Restore is a distinct operation, never a create call with an ignored field."""

import pytest

from microsandbox import Sandbox


@pytest.mark.parametrize("option", ["image", "memory", "cpus", "cmd", "replace", "detached"])
def test_restore_rejects_create_options(option):
    with pytest.raises(TypeError, match="unexpected restore option"):
        Sandbox.restore("missing", name="restore-validation", **{option: None})


@pytest.mark.parametrize("option", ["from_snapshot", "forked", "disk_only", "snapshot_base"])
def test_create_rejects_restore_options(option):
    with pytest.raises(TypeError):
        Sandbox.create("restore-validation", image="alpine", **{option: None})


@pytest.mark.asyncio
async def test_restore_missing_artifact_does_not_boot(tmp_path):
    with pytest.raises(FileNotFoundError):
        await Sandbox.restore(tmp_path / "missing", name="restore-validation", forked=True)
