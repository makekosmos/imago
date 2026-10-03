# Imago: agent instructions

## Scope and entry points

Imago is the single Kosmos GPUI design layer — a Rust-only workspace.

- `crates/imago-gpui`: palettes (`palettes`), theme tokens and runtime theme
  helpers (`theme`), shared button/sidebar chrome (`button`, `chrome`).
- `crates/mundus-gpui-kit`: shell widgets (`widgets`), view primitives and
  JSON accessors (`fields`), and the Engine lock/RPC client (`engine`,
  `engine_ws`, `engine_error`) — merged in from the archived
  `makekosmos/kosmos-gpui-kit` repo (KOS-319), faithful extraction only: no
  API redesign vs the `cortex/manager-gpui` originals.
- `toolchain.json`: pinned Rust toolchain, identical to cortex's.
- `.github/workflows/ci.yml`, `lefthook.yml`: the Quality gate.

The Vue/TS library `@makekosmos/visuals` was removed under KOS-319; nothing
in this repo is published to npm anymore.

## Setup and verification

Run from this repository root. Do not bypass hooks to obtain a green result.

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Use the pinned toolchain (`toolchain.json`, rust 1.95.0). The lefthook
pre-push hook runs the same gate.

## Contracts to preserve

- Consumers pin both crates by the **same** git rev of this repo
  (`imago-gpui = { git = "…/imago.git", rev = … }` and
  `mundus-gpui-kit = { git = "…/imago.git", rev = … }`). Never let the pins
  drift apart.
- `gpui` (renamed `gpui-kit`), `gpui-component` and `gpui-base` are
  exact-pinned and MUST stay identical across workspace members and to
  `cortex/manager-gpui`'s specs — two different gpui versions in one build
  produce type errors. Bump consumers and these crates together.
- `mundus-gpui-kit` depends on `imago-gpui` by workspace `path`; consumers
  get the same rev's `imago-gpui` automatically through the git checkout.
- Preserve keyboard access, focus behavior, accessible names and theme token
  usage in shared components.
- Do not propagate changes into application repositories. Hand off a tested
  rev plus compatibility notes to their owners.
- Clean up listeners, timers and subscriptions on disposal.

## Parallel work

- This chat owns only its assigned repositories; sibling repositories are
  read-only unless explicitly assigned.
- Each writer uses a separate worktree. Never switch branches, reset, clean
  or stash another writer's checkout.
- Pin external dependencies by version/SHA.
- Hand off contract changes with operation/type, inputs, outputs, errors,
  version and compatibility evidence.

## Completion

- Prefer existing code and tools; avoid unrelated cleanup and new
  abstractions. Trace callers before fixing a shared bug.
- Add the smallest meaningful regression check for changed nontrivial
  behavior.
- Run `cargo fmt --all` before every commit, then the targeted
  check/clippy/test for the crates you touched; run the full workspace gate
  before pushing.
- Report exact SHA, commands and PASS / FAIL / NOT_RUN with reasons.
- Do not publish releases, tags or crates unless that action is authorized.
