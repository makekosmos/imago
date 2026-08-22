# `@makekosmos/visuals`

Shared Vue 3 components, composables, patterns, and theme tokens for Kosmos
applications.

## Development

```bash
bun install
bun run build
bun run test
```

The package is published independently to GitHub Packages. Applications keep
the historical `@kosmos/visuals` import path through an npm alias, but pin an
explicit released `@makekosmos/visuals` version instead of importing files from
the Kosmos monorepo.
