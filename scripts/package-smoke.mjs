import { existsSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
const pkg = JSON.parse(await readFile("package.json", "utf8"));
const required = ["dist/index.js", "dist/index.css", "index.ts", "components/index.ts",
  "patterns/index.ts", "theme/css-variables.css", "components/sidebar.css", "components/settings-shell.css"];
for (const path of required) if (!existsSync(path)) throw new Error(`packed artifact missing ${path}`);
for (const [name, target] of Object.entries(pkg.exports)) {
  const entry = typeof target === "string" ? target : (target.import ?? target.types);
  if (!existsSync(entry.replace(/^\.\//, ""))) throw new Error(`export ${name} points to missing ${entry}`);
}
if (!readFileSync("dist/index.js", "utf8").trim()) throw new Error("empty dist/index.js");
console.log("package exports and CSS assets are present");
