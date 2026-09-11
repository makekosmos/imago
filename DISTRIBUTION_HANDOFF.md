# Distribution handoff

## Published package

- Package: `@makekosmos/visuals@0.1.3`
- Source release SHA: `5ee6bd11273acf7f24bebcd8ceb869cffbb397a9`
- Release tag: `visuals-v0.1.3`
- Registry: `https://npm.pkg.github.com`
- Tarball: `makekosmos-visuals-0.1.3.tgz`
- Size: `109712` bytes
- Integrity: `sha512-TKTd+J5Jd2m1sEVvNRSAe4qddqBKVI+gfph8zB2OjAh75IuXE5JITNmnK8DCPkG/Biv1LibPAyygqOtbFq/uIA==`
- Shasum: `7b69ce0b5a22f4383206ecd44ed3acf53e014c7a`

## Compatibility evidence

- `bun install --frozen-lockfile`: PASS.
- Full `bun run check`: PASS — lint, formatting, type checks, Vite build and package smoke.
- `bunx lefthook run pre-push`: PASS.
- Packed artifact contains `dist/index.js` and `dist/index.css`; all 7 declared exports are present.
- Registry verification: `@makekosmos/visuals@0.1.3` fetched from GitHub Packages, installed in a clean consumer, and imported successfully.
- Public API, package version, peer dependencies and lockfile unchanged; no application dependencies changed.

## Consumer contract

- Package: `@makekosmos/visuals@0.1.3`
- Preserve existing Vue 3 and Vue Router peer compatibility.
- Use the package exports declared in `package.json`; do not depend on sibling checkouts.
