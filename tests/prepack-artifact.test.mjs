import { existsSync, rmSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, test } from "vitest";

const root = resolve(import.meta.dirname, "..");
const statePath = resolve(root, ".tmp", "imago-prepared-artifact.json");
const script = resolve(root, "scripts", "prepack-artifact.mjs");

test("prepared mode fails closed when its marker is missing", () => {
  rmSync(statePath, { force: true });
  const result = spawnSync(process.execPath, [script], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, IMAGO_PREPARED_ARTIFACT: "required" },
  });

  expect(result.status).not.toBe(0);
  expect(`${result.stdout}${result.stderr}`).toContain("required prepared Imago artifact");
  expect(existsSync(statePath)).toBe(false);
});
