# @makekosmos/visuals

Shared Vue 3 components, composables, patterns, and theme tokens for Kosmos applications.

## Verification

    bun install --frozen-lockfile
    bunx playwright install chromium
    bun run check

The aggregate check is the CI contract: lint, formatting, exported TypeScript/API
checks, library build, and packed-export smoke checks. The smoke check validates
every declared JS/TS/CSS subpath and generated dist entry points before publishing.

The check's library build creates a verified prepared-artifact marker. Publishing
must consume that exact artifact with `IMAGO_PREPARED_ARTIFACT=required`; a
standalone `npm pack` without that variable rebuilds safely during `prepack`.

    bun run check
    IMAGO_PREPARED_ARTIFACT=required npm pack --dry-run

Use pinned Bun 1.3.14. Breaking public component/type changes require a release
note and an explicit version decision.
