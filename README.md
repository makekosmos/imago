# @makekosmos/visuals

Shared Vue 3 components, composables, patterns, and theme tokens for Kosmos applications.

## Verification

    pnpm install --frozen-lockfile
    pnpm check

The aggregate check is the local quality gate: lint, formatting, exported
TypeScript/API checks, library build, and packed-export smoke checks. The
smoke check validates every declared JS/TS/CSS subpath and generated dist
entry points before publishing.

Hosted GitHub Actions are disabled (KOS-76): no workflow runs on `push`,
`pull_request`, tag push, or `schedule`. The quality and publish workflows are
manual `workflow_dispatch` only, so publication happens exclusively on an
explicit operator request. Done means `pnpm check` and the lefthook hooks pass
locally; an absent or red hosted run is not a blocker.

The check's library build creates a verified prepared-artifact marker. Publishing
must consume that exact artifact with `IMAGO_PREPARED_ARTIFACT=required`; a
standalone `npm pack` without that variable rebuilds safely during `prepack`.

    pnpm check
    IMAGO_PREPARED_ARTIFACT=required npm pack --dry-run

Use pinned pnpm 12.4.1. Breaking public component/type changes require a release
note and an explicit version decision.
