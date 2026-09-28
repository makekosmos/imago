# AGENTS.md — mundus-gpui-kit

Shared GPUI crate extracted from `cortex/manager-gpui`: Imago theme tokens
(`theme`), shell chrome (`widgets`), view primitives + JSON accessors
(`fields`) and the Engine lock/RPC client (`engine`). Consumed via pinned
git dependency (`rev = "<sha>"`) by manager-gpui, the unified Cortex GPUI
application and agenda-gpui.

## Checks

```powershell
cargo check
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Pinning rule

`gpui` (renamed `gpui-kit`), `gpui-component` and `imago-gpui` are exact-pinned
and MUST stay identical to `cortex/manager-gpui`'s specs — two different gpui
versions in one build produce type errors. Bump consumers and this crate
together.

## Rules

- Faithful extraction only: no API redesign, no behavior changes vs the
  manager-gpui originals. Entity-bound helpers are generic over the root
  entity type (`Context<T>`, `Slots` trait) instead of naming `ManagerApp`.
- `.cargo/config.toml` sets `git-fetch-with-cli` so the private imago
  dependency authenticates via the system git credentials — keep it.
- Library crate: `Cargo.lock` is gitignored on purpose.
