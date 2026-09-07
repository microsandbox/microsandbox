import { expect, it } from "vitest";
import { Sandbox, Snapshot } from "../dist/index.js";

// Opt-in because this starts real VMs with a matching development runtime/kernel bundle.
it.skipIf(process.env.MSB_COW_LIVE !== "1")("captures a resident pause and restores private memory", async () => {
  const name = `cow8-node-${process.pid}`;
  const source = await Sandbox.builder(name).image("alpine").rootDisk(512).memory(256).memorySnapshot("cow").create();
  let child: Sandbox | undefined;
  try {
    await source.exec("sh", ["-c", "echo source > /dev/shm/sdk-marker"]);
    await source.pause();
    const paused = await Sandbox.get(name);
    expect(paused.status).toBe("paused");
    await Snapshot.builder(`${name}-full`).fromSandbox(name).full().create();
    await paused.resume();
    child = await Sandbox.builder(`${name}-child`).fromSnapshot(`${name}-full`).memorySnapshot("cow").create();
    expect((await child.exec("cat", ["/dev/shm/sdk-marker"])).stdout().trim()).toBe("source");
    await child.exec("sh", ["-c", "echo child > /dev/shm/sdk-marker"]);
    expect((await source.exec("cat", ["/dev/shm/sdk-marker"])).stdout().trim()).toBe("source");
    await child.pause();
  } finally {
    await child?.stop();
    await source.stop();
  }
}, 120_000);
