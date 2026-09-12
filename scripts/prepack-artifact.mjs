import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const statePath = path.join(root, ".tmp", "imago-prepared-artifact.json");
const dependencyFiles = ["package.json", "bun.lock"];
const sourceRoots = [
  "index.ts",
  "components",
  "composables",
  "patterns",
  "runtime",
  "theme",
  "vite.config.mjs",
  "scripts/prepack-artifact.mjs",
];
const allowedDependencies = ["vue", "vue-router", "@lucide/vue"];

function walk(directory, files = [], rejectSymlinks = false) {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) =>
    a.name.localeCompare(b.name),
  )) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      if (rejectSymlinks) throw new Error(`source input cannot be a symlink: ${fullPath}`);
      continue;
    }
    if (entry.isDirectory()) walk(fullPath, files, rejectSymlinks);
    else if (entry.isFile()) files.push(path.relative(root, fullPath).replaceAll("\\", "/"));
  }
  return files;
}

function digestFiles(files, prefix = "") {
  const hash = createHash("sha256");
  hash.update(prefix).update("\0");
  for (const relative of [...files].sort()) {
    const data = readFileSync(path.join(root, relative));
    hash.update(relative).update("\0").update(data).update("\0");
  }
  return hash.digest("hex");
}

function sourceFingerprint() {
  const files = sourceRoots.flatMap((relative) => {
    const fullPath = path.join(root, relative);
    if (!existsSync(fullPath)) throw new Error(`source input is missing: ${relative}`);
    const stats = lstatSync(fullPath);
    if (stats.isSymbolicLink()) throw new Error(`source input cannot be a symlink: ${fullPath}`);
    if (stats.isFile()) return [relative];
    return walk(fullPath, [], true);
  });
  return { files, sha256: digestFiles(files) };
}

function dependencyFingerprint() {
  const files = dependencyFiles.filter((file) => existsSync(path.join(root, file)));
  return {
    files,
    allowed: allowedDependencies,
    sha256: digestFiles(files, JSON.stringify(allowedDependencies)),
  };
}

function variant() {
  const environment = Object.fromEntries(
    Object.entries(process.env)
      .filter(([key]) => key === "NODE_ENV" || key.startsWith("VITE_"))
      .sort(([left], [right]) => left.localeCompare(right)),
  );
  return {
    mode: process.env.IMAGO_BUILD_MODE ?? "production",
    platform: process.platform,
    arch: process.arch,
    node: process.version,
    bun: process.versions.bun ?? null,
    environment_sha256: createHash("sha256").update(JSON.stringify(environment)).digest("hex"),
  };
}

function outputFiles() {
  const files = walk(path.join(root, "dist"), [], true).map((file) => file.replace(/^dist\//, ""));
  return files.map((relative) => {
    const data = readFileSync(path.join(root, "dist", relative));
    return {
      path: `dist/${relative}`,
      size: data.length,
      sha256: createHash("sha256").update(data).digest("hex"),
    };
  });
}

function preparedState() {
  if (!existsSync(path.join(root, "dist"))) throw new Error("prepared artifact is missing dist/");
  const outputs = outputFiles();
  if (!outputs.some(({ path: file }) => file === "dist/index.js")) {
    throw new Error("prepared artifact is missing dist/index.js");
  }
  return {
    schema_version: 1,
    package: "@makekosmos/visuals",
    input: sourceFingerprint(),
    dependencies: dependencyFingerprint(),
    variant: variant(),
    outputs,
  };
}

function readPreparedState() {
  try {
    return JSON.parse(readFileSync(statePath, "utf8"));
  } catch {
    return null;
  }
}

function hasValidPreparedState() {
  const saved = readPreparedState();
  if (!saved) return false;
  let current;
  try {
    current = preparedState();
  } catch {
    return false;
  }
  return (
    saved.schema_version === current.schema_version &&
    saved.package === current.package &&
    JSON.stringify(saved.input) === JSON.stringify(current.input) &&
    JSON.stringify(saved.dependencies) === JSON.stringify(current.dependencies) &&
    JSON.stringify(saved.variant) === JSON.stringify(current.variant) &&
    JSON.stringify(saved.outputs) === JSON.stringify(current.outputs)
  );
}

function build() {
  rmSync(statePath, { force: true });
  mkdirSync(path.dirname(statePath), { recursive: true });
  execFileSync("bun", ["x", "vite", "build", "--mode", variant().mode], {
    cwd: root,
    stdio: "inherit",
    env: process.env,
  });
  writeFileSync(statePath, `${JSON.stringify(preparedState())}\n`);
}

if (process.env.IMAGO_PREPARED_ARTIFACT === "required") {
  try {
    if (!hasValidPreparedState()) {
      throw new Error("required prepared Imago artifact is missing or stale");
    }
    console.log("reusing verified prepared Imago artifact");
  } finally {
    rmSync(statePath, { force: true });
  }
} else {
  // Standalone pack/publish must remain self-contained and build every time.
  build();
  rmSync(statePath, { force: true });
}
