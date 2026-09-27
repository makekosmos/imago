import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
const mode = process.argv[2] ?? "--lint";
const files = execFileSync("git", ["ls-files"], { encoding: "utf8" }).trim().split(/\r?\n/).filter(Boolean)
  .filter((file) => !file.startsWith("bun.lock"));
for (const file of files) {
  let source; try { source = readFileSync(file, "utf8"); } catch { continue; }
  if (source.includes("\0")) continue;
  if (/[ \t]+$/m.test(source)) throw new Error(`trailing whitespace: ${file}`);
  if (!source.endsWith("\n")) throw new Error(`missing final newline: ${file}`);
  if (mode === "--lint" && /(^|\n)\s*debugger\s*;/.test(source)) throw new Error(`debugger statement: ${file}`);
  if (mode === "--lint" && /\.(?:vue|tsx?|mjs)$/.test(file)) {
    for (const match of source.matchAll(/\b(?:add|remove)EventListener\(\s*["'`]([^"'`]+)["'`]/g)) {
      if (!/^[a-z][a-z0-9.:-]*$/.test(match[1])) throw new Error(`invalid event name "${match[1]}": ${file}`);
    }
    // Pointer drag sessions must handle pointercancel: without it, a gesture
    // aborted by the browser (touch-scroll takeover, OS drag) leaks the
    // document-level pointermove listener and leaves drag artifacts behind.
    if (/addEventListener\(\s*["'`]pointermove["'`]/.test(source) && !/["'`]pointercancel["'`]/.test(source)) {
      throw new Error(`pointermove drag session without pointercancel handler: ${file}`);
    }
  }
}
for (const file of ["package.json"]) {
  if (file.endsWith(".json")) JSON.parse(readFileSync(file, "utf8"));
}
if (mode === "--types") {
  const rootIndex = readFileSync("index.ts", "utf8");
  const index = readFileSync("packages/vue/src/index.ts", "utf8");
  if (!rootIndex.includes("./packages/vue/src/index")) throw new Error("Vue package exports are missing");
  if (!index.includes("usePlatform")) throw new Error("public composable exports are missing");
}
console.log(`${mode.slice(2)} checks passed (${files.length} files)`);
