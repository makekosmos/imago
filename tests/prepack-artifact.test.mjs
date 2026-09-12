import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createArtifactContract } from "../scripts/prepack-artifact.mjs";

function fixture() {
  const root = mkdtempSync(join(tmpdir(), "imago-artifact-"));
  for (const directory of ["components", "composables", "patterns", "runtime", "theme", "scripts"]) {
    mkdirSync(join(root, directory), { recursive: true });
    writeFileSync(join(root, directory, "index.ts"), `${directory}\n`);
  }
  for (const file of ["index.ts", "vite.config.mjs", "bun.lock", "scripts/prepack-artifact.mjs"]) {
    writeFileSync(join(root, file), file);
  }
  writeFileSync(join(root, "package.json"), JSON.stringify({ name: "fixture", type: "module" }));
  for (const [name, packageJson] of [
    ["vite", { name: "vite", version: "1", dependencies: { rolldown: "1" } }],
    ["@vitejs/plugin-vue", { name: "@vitejs/plugin-vue", version: "1", dependencies: { "@rolldown/pluginutils": "1" } }],
    ["@vue/compiler-sfc", { name: "@vue/compiler-sfc", version: "1", dependencies: { "@vue/compiler-core": "1" } }],
    ["rolldown", { name: "rolldown", version: "1" }],
    ["@rolldown/pluginutils", { name: "@rolldown/pluginutils", version: "1" }],
    ["@vue/compiler-core", { name: "@vue/compiler-core", version: "1" }],
    ["vue", { name: "vue", version: "1", exports: { "./compiler-sfc": "./compiler-sfc/index.js" } }],
  ]) {
    const directory = join(root, "node_modules", ...name.split("/"));
    mkdirSync(directory, { recursive: true });
    writeFileSync(join(directory, "package.json"), JSON.stringify(packageJson));
    writeFileSync(join(directory, "index.js"), name);
  }
  mkdirSync(join(root, "node_modules", "vue", "compiler-sfc"), { recursive: true });
  writeFileSync(join(root, "node_modules", "vue", "compiler-sfc", "index.js"), "compiler shim");
  return root;
}

function buildStub(calls, mutate) {
  return (root) => {
    calls.push(root);
    if (mutate) writeFileSync(join(root, "components", "index.ts"), "changed during build");
    mkdirSync(join(root, "dist"), { recursive: true });
    writeFileSync(join(root, "dist", "index.js"), "artifact");
  };
}

function run(name, check) {
  try {
    check();
    console.log(`PASS ${name}`);
  } catch (error) {
    console.error(`FAIL ${name}`);
    throw error;
  }
}

run("prepared mode rejects stale inputs and tampered outputs", () => {
  const root = fixture();
  try {
    const calls = [];
    const contract = createArtifactContract(root, buildStub(calls));
    contract.build();
    assert.equal(calls.length, 1);

    writeFileSync(join(root, "components", "index.ts"), "changed");
    assert.throws(() => contract.prepack(true), /required prepared Imago artifact/);
    assert.equal(calls.length, 1);

    writeFileSync(join(root, "components", "index.ts"), "components\n");
    contract.build();
    writeFileSync(join(root, "node_modules", "rolldown", "index.js"), "dependency changed");
    assert.throws(() => contract.prepack(true), /required prepared Imago artifact/);
    assert.equal(calls.length, 2);

    writeFileSync(join(root, "node_modules", "rolldown", "index.js"), "rolldown");
    contract.build();
    writeFileSync(join(root, "dist", "index.js"), "tampered");
    assert.throws(() => contract.prepack(true), /required prepared Imago artifact/);
    assert.equal(calls.length, 3);
    assert.equal(existsSync(contract.statePath), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

run("failed or input-changing builds leave no prepared marker", () => {
  const root = fixture();
  try {
    const failed = createArtifactContract(root, () => {
      throw new Error("build failed");
    });
    assert.throws(() => failed.build(), /build failed/);
    assert.equal(existsSync(failed.statePath), false);

    const changing = createArtifactContract(root, buildStub([], true));
    assert.throws(() => changing.build(), /changed during build/);
    assert.equal(existsSync(changing.statePath), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

run("prepared packaging skips build while default packaging rebuilds", () => {
  const root = fixture();
  try {
    const calls = [];
    const contract = createArtifactContract(root, buildStub(calls));
    contract.build();
    const marker = readFileSync(contract.statePath, "utf8");
    contract.prepack(true);
    assert.equal(calls.length, 1);
    assert.equal(existsSync(contract.statePath), false);

    writeFileSync(contract.statePath, marker);
    contract.prepack(false);
    assert.equal(calls.length, 2);
    assert.equal(existsSync(contract.statePath), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
