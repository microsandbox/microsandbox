import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { afterEach, beforeEach, expect, it } from "vitest";

const entry = pathToFileURL(resolve("dist/index.js")).href;
const nativeEntry = resolve("native/index.cjs");
const executable = process.platform === "win32" ? "msb.exe" : "msb";
const library = process.platform === "darwin" ? "libkrunfw.5.dylib"
  : process.platform === "win32" ? "libkrunfw.dll" : "libkrunfw.so.5.6.1";
const triple = process.platform === "darwin" ? "darwin-arm64"
  : process.platform === "win32" ? `win32-${process.arch}-msvc` : `linux-${process.arch}-gnu`;
let root: string;
let packageRoot: string;
let env: NodeJS.ProcessEnv;
function pair(home: string) {
  mkdirSync(join(home, "bin"), { recursive: true });
  mkdirSync(join(home, "lib"), { recursive: true });
  writeFileSync(join(home, "bin", executable), "older runtime: do not execute");
  writeFileSync(join(home, "lib", library), "matching firmware");
}
beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "msb-package-"));
  packageRoot = join(root, "node_modules", "@superradcompany", `microsandbox-${triple}`);
  pair(packageRoot);
  writeFileSync(join(packageRoot, "package.json"), JSON.stringify({ name: `@superradcompany/microsandbox-${triple}` }));
  env = { ...process.env, HOME: root, USERPROFILE: root };
  for (const name of ["MSB_HOME", "MSB_PATH", "MSB_LIBKRUNFW_PATH", "MSB_CONFIG_PATH"]) delete env[name];
});
afterEach(() => rmSync(root, { recursive: true, force: true }));
function installed(before = "") {
  // Import the real JS layer and native addon in a fresh process so automatic
  // registration, set-once overrides, and environment reads are all exercised.
  const code = `import { createRequire } from 'node:module';
    const sdk = await import(${JSON.stringify(entry)});
    const native = createRequire(import.meta.url)(${JSON.stringify(nativeEntry)});
    ${before}
    console.log(sdk.isInstalled());`;
  const result = spawnSync(process.execPath, ["--input-type=module", "-e", code], {
    cwd: root, env, encoding: "utf8", timeout: 30000,
  });
  expect(result.status, result.stderr).toBe(0);
  return result.stdout.trim();
}
it.each(["default", "custom"])("native SDK prefers %s home over an incomplete package", (kind) => {
  const home = join(root, kind === "default" ? ".microsandbox" : "custom");
  if (kind === "custom") env.MSB_HOME = home;
  pair(home);
  unlinkSync(join(packageRoot, "lib", library));
  expect(installed()).toBe("true");
});
it("native SDK uses package only when home is absent", () => {
  expect(installed()).toBe("true");
  const home = join(root, ".microsandbox");
  pair(home);
  unlinkSync(join(home, "lib", library));
  expect(installed()).toBe("false");
});
it("explicit setters still override a registered package and complete home", () => {
  pair(join(root, ".microsandbox"));
  expect(installed(`native.setRuntimeMsbPath(${JSON.stringify(join(root, "missing"))});`)).toBe("false");
});
it("package discovery does not pin default home when MSB_HOME is custom", () => {
  unlinkSync(join(packageRoot, "bin", executable));
  pair(join(root, ".microsandbox"));
  env.MSB_HOME = join(root, "absent-custom-home");
  expect(installed()).toBe("false");
});
