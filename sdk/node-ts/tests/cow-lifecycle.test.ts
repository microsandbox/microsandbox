import { expect, it } from "vitest";
import { Sandbox, Snapshot } from "../dist/index.js";

// Opt-in because this starts real VMs with a matching development runtime/kernel bundle.
it.skipIf(process.env.MSB_COW_LIVE !== "1")("captures a resident pause and restores private memory", async () => {
  const name = `cow8-node-${process.pid}`;
  const source = await Sandbox.builder(name).image("alpine").rootDisk(512).memory(256).create();
  let child: Sandbox | undefined;
  const branches: Sandbox[] = [];
  try {
    await source.exec("sh", ["-c", "echo source > /dev/shm/sdk-marker"]);
    await source.pause();
    const paused = await Sandbox.get(name);
    expect(paused.status).toBe("paused");
    const branched = await paused.branch(`${name}-paused-branch`);
    branches.push(branched);
    expect((await branched.exec("cat", ["/dev/shm/sdk-marker"])).stdout().trim()).toBe("source");
    const snapshot = await Snapshot.builder(`${name}-full`).fromSandbox(name).full().create();
    await paused.resume();
    child = await Sandbox.restore(snapshot.path).name(`${name}-child`).forked().restore();
    expect((await child.exec("cat", ["/dev/shm/sdk-marker"])).stdout().trim()).toBe("source");
    await child.exec("sh", ["-c", "echo child > /dev/shm/sdk-marker"]);
    const descendant = await child.branch(`${name}-branch`);
    branches.push(descendant);
    expect((await descendant.exec("cat", ["/dev/shm/sdk-marker"])).stdout().trim()).toBe("child");
    expect((await source.exec("cat", ["/dev/shm/sdk-marker"])).stdout().trim()).toBe("source");
    await child.pause();
    await child.resume();
  } finally {
    // A paused VM cannot stop gracefully. Attempt every cleanup even when one
    // fails so an assertion or cleanup error does not strand sibling VMs.
    const results = await Promise.allSettled([...branches, ...(child ? [child] : []), source].map(async (sandbox) => {
      if ((await Sandbox.get(sandbox.name)).status === "paused") await sandbox.resume();
      await sandbox.stop();
    }));
    for (const result of results) if (result.status === "rejected") throw result.reason;
  }
}, 120_000);
