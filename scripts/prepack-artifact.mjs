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
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
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
const dependencyFiles = ["package.json", "bun.lock"];
const allowedDependencies = ["vue", "vue-router", "@lucide/vue"];
const buildDependencyRoots = ["vite", "@vitejs/plugin-vue", "@vue/compiler-sfc"];

function walk(root, directory, files = [], rejectSymlinks = false, skipNestedNodeModules = false) {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) =>
    a.name.localeCompare(b.name),
  )) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      if (rejectSymlinks) throw new Error(`input cannot be a symlink: ${fullPath}`);
      continue;
    }
    if (entry.isDirectory()) {
      if (skipNestedNodeModules && entry.name === "node_modules") continue;
      walk(root, fullPath, files, rejectSymlinks, skipNestedNodeModules);
    }
    else if (entry.isFile()) files.push(path.relative(root, fullPath).replaceAll("\\", "/"));
  }
  return files;
}

function defaultBuild(root, mode) {
  execFileSync("bun", ["x", "vite", "build", "--mode", mode], {
    cwd: root,
    stdio: "inherit",
    env: process.env,
  });
}

function resolveBunVersion() {
  if (process.versions.bun) return process.versions.bun;
  return execFileSync("bun", ["--version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
}

export function createArtifactContract(workspaceRoot = repositoryRoot, runBuild = defaultBuild) {
  const root = path.resolve(workspaceRoot);
  const statePath = path.join(root, ".tmp", "imago-prepared-artifact.json");
  const buildBunVersion = runBuild === defaultBuild ? resolveBunVersion() : process.versions.bun ?? null;

  function digestFiles(files, prefix = "") {
    const hash = createHash("sha256");
    hash.update(prefix).update("\0");
    for (const relative of [...files].sort()) {
      const data = readFileSync(path.join(root, relative));
      hash.update(relative).update("\0").update(data).update("\0");
    }
    return hash.digest("hex");
  }

  function isWithinRoot(candidate) {
    const relative = path.relative(root, candidate);
    return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
  }

  function resolvePackageManifest(name, fromManifest = path.join(root, "package.json")) {
    let directory = path.dirname(fromManifest);
    while (isWithinRoot(directory)) {
      const manifest = path.join(directory, "node_modules", ...name.split("/"), "package.json");
      if (existsSync(manifest)) {
        if (lstatSync(manifest).isSymbolicLink()) {
          throw new Error(`build dependency cannot be a symlink: ${manifest}`);
        }
        return manifest;
      }
      if (directory === root) break;
      directory = path.dirname(directory);
    }
    throw new Error(`build dependency is not resolved: ${name}`);
  }

  function buildDependencyClosure() {
    const queue = buildDependencyRoots.map((name) => ({
      name,
      fromManifest: path.join(root, "package.json"),
      optional: false,
    }));
    const packages = new Map();
    const files = new Set();
    const missingOptional = new Set();
    while (queue.length > 0) {
      const { name, fromManifest, optional } = queue.shift();
      if (allowedDependencies.includes(name)) continue;
      let manifest;
      try {
        manifest = resolvePackageManifest(name, fromManifest);
      } catch (error) {
        if (!optional) throw error;
        missingOptional.add(`${path.relative(root, fromManifest).replaceAll("\\", "/")}:${name}`);
        continue;
      }
      const key = path.resolve(manifest);
      if (packages.has(key)) continue;
      const packageRoot = path.dirname(manifest);
      if (lstatSync(packageRoot).isSymbolicLink()) {
        throw new Error(`build dependency cannot be a symlink: ${packageRoot}`);
      }
      const packageJson = JSON.parse(readFileSync(manifest, "utf8"));
      packages.set(key, {
        name: packageJson.name ?? name,
        version: packageJson.version ?? null,
        manifest: path.relative(root, manifest).replaceAll("\\", "/"),
      });
      for (const file of walk(root, packageRoot, [], true, true)) files.add(file);
      const optionalDependencies = packageJson.optionalDependencies ?? {};
      for (const dependency of Object.keys(packageJson.dependencies ?? {})
        .filter((name) => !Object.hasOwn(optionalDependencies, name))
        .sort()) {
        if (!allowedDependencies.includes(dependency)) {
          queue.push({ name: dependency, fromManifest: manifest, optional: false });
        }
      }
      for (const dependency of Object.keys(optionalDependencies).sort()) {
        if (!allowedDependencies.includes(dependency)) {
          queue.push({ name: dependency, fromManifest: manifest, optional: true });
        }
      }
    }

    const requireFromRoot = createRequire(path.join(root, "package.json"));
    const compilerEntry = requireFromRoot.resolve("vue/compiler-sfc");
    const compilerManifest = resolvePackageManifest("vue");
    files.add(path.relative(root, compilerManifest).replaceAll("\\", "/"));
    files.add(path.relative(root, compilerEntry).replaceAll("\\", "/"));
    const resolvedPackages = [...packages.values()].sort((left, right) =>
      left.manifest.localeCompare(right.manifest),
    );
    const resolvedFiles = [...files].sort();
    const unresolvedOptional = [...missingOptional].sort();
    return {
      packages: resolvedPackages,
      files: resolvedFiles,
      missing_optional: unresolvedOptional,
      sha256: digestFiles(
        resolvedFiles,
        JSON.stringify({ packages: resolvedPackages, missing_optional: unresolvedOptional }),
      ),
    };
  }

  function sourceFingerprint() {
    const files = sourceRoots.flatMap((relative) => {
      const fullPath = path.join(root, relative);
      if (!existsSync(fullPath)) throw new Error(`source input is missing: ${relative}`);
      const stats = lstatSync(fullPath);
      if (stats.isSymbolicLink()) throw new Error(`source input cannot be a symlink: ${fullPath}`);
      if (stats.isFile()) return [relative];
      return walk(root, fullPath, [], true);
    });
    return { files, sha256: digestFiles(files) };
  }

  function dependencyFingerprint() {
    for (const file of dependencyFiles) {
      if (!existsSync(path.join(root, file))) {
        throw new Error(`dependency input is missing: ${file}`);
      }
    }
    const build = buildDependencyClosure();
    const files = [...dependencyFiles, ...build.files].sort();
    return {
      files: dependencyFiles,
      build,
      allowed: allowedDependencies,
      sha256: digestFiles(files, JSON.stringify({ allowed: allowedDependencies, build: build.packages })),
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
      bun: buildBunVersion,
      environment_sha256: createHash("sha256").update(JSON.stringify(environment)).digest("hex"),
    };
  }

  function outputFiles() {
    const dist = path.join(root, "dist");
    if (!existsSync(dist)) throw new Error("prepared artifact is missing dist/");
    return walk(root, dist, [], true).map((file) => {
      const data = readFileSync(path.join(root, file));
      return {
        path: file,
        size: data.length,
        sha256: createHash("sha256").update(data).digest("hex"),
      };
    });
  }

  function preparedState(snapshot = {
    input: sourceFingerprint(),
    dependencies: dependencyFingerprint(),
    variant: variant(),
  }) {
    const outputs = outputFiles();
    if (!outputs.some(({ path: file }) => file === "dist/index.js")) {
      throw new Error("prepared artifact is missing dist/index.js");
    }
    return {
      schema_version: 1,
      package: "@makekosmos/visuals",
      ...snapshot,
      outputs,
    };
  }

  function hasValidPreparedState() {
    let saved;
    try {
      saved = JSON.parse(readFileSync(statePath, "utf8"));
    } catch {
      return false;
    }
    let current;
    try {
      current = preparedState();
    } catch {
      return false;
    }
    return JSON.stringify(saved) === JSON.stringify(current);
  }

  function build() {
    rmSync(statePath, { force: true });
    mkdirSync(path.dirname(statePath), { recursive: true });
    const snapshot = {
      input: sourceFingerprint(),
      dependencies: dependencyFingerprint(),
      variant: variant(),
    };
    runBuild(root, snapshot.variant.mode);
    if (JSON.stringify(snapshot) !== JSON.stringify({
      input: sourceFingerprint(),
      dependencies: dependencyFingerprint(),
      variant: variant(),
    })) {
      rmSync(statePath, { force: true });
      throw new Error("Imago build inputs changed during build");
    }
    writeFileSync(statePath, `${JSON.stringify(preparedState(snapshot))}\n`);
  }

  function prepack(preparedRequired) {
    try {
      if (preparedRequired) {
        if (!hasValidPreparedState()) {
          throw new Error("required prepared Imago artifact is missing or stale");
        }
        console.log("reusing verified prepared Imago artifact");
      } else {
        build();
      }
    } finally {
      rmSync(statePath, { force: true });
    }
  }

  return { build, hasValidPreparedState, prepack, statePath };
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]).toLowerCase() === fileURLToPath(import.meta.url).toLowerCase()
) {
  const contract = createArtifactContract(process.env.IMAGO_ARTIFACT_ROOT ?? repositoryRoot);
  if (process.argv.includes("--build")) contract.build();
  else contract.prepack(process.env.IMAGO_PREPARED_ARTIFACT === "required");
}
