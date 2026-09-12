import { beforeEach, describe, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({
  ownsLifecycle: false,
  requestedDetached: undefined as boolean | undefined,
  stop: vi.fn(async () => {}),
  cancel: vi.fn(),
  create: vi.fn(),
}));

vi.mock("../../dist/internal/napi.js", () => {
  const sandbox = () => ({
    id: "restored-id",
    backendKind: "local",
    ownsLifecycle: native.ownsLifecycle,
    stop: native.stop,
  });
  const progress = () => ({
    cancel: native.cancel,
    progress: {},
    awaitSandbox: async () => sandbox(),
  });
  return {
    napi: {
      SandboxBuilder: class {
        detached(enabled: boolean): this {
          native.requestedDetached = enabled;
          return this;
        }
        async create() {
          native.create();
          return sandbox();
        }
        async connectOrCreate() { throw new Error("not used by progress tests"); }
        async createWithPullProgress() { return progress(); }
        async createWithProgress() { return progress(); }
      },
    },
  };
});

import { Sandbox } from "../../dist/sandbox.js";

describe("creation result lifecycle ownership", () => {
  beforeEach(() => {
    native.ownsLifecycle = false;
    native.requestedDetached = undefined;
    native.stop.mockClear();
    native.cancel.mockClear();
    native.create.mockClear();
  });

  for (const method of ["create", "createWithProgress", "createWithPullProgress"] as const) {
    for (const ownsLifecycle of [false, true]) {
      it(`${method} disposes only when the returned native handle owns its lifecycle (${ownsLifecycle})`, async () => {
        native.ownsLifecycle = ownsLifecycle;
        // Simulate a full restore that auto-detaches despite an explicit attached request.
        const builder = Sandbox.builder("restored").detached(false);
        const sandbox = method === "create"
          ? await builder.create()
          : await (await builder[method]()).awaitSandbox();

        expect(native.requestedDetached).toBe(false);
        expect(native.create).toHaveBeenCalledTimes(method === "create" ? 1 : 0);
        expect(sandbox.ownsLifecycle).toBe(ownsLifecycle);
        expect(sandbox.id).toBe("restored-id");
        await sandbox[Symbol.asyncDispose]();
        expect(native.stop).toHaveBeenCalledTimes(ownsLifecycle ? 1 : 0);
      });
    }
  }
});
