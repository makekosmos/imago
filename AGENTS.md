# Imago: agent instructions

## Scope and entry points

Shared Vue 3 components, composables, patterns and theme tokens, published as `@makekosmos/visuals`.

- `components/`, `composables/`, `patterns/`: public UI primitives and behavior.
- `theme/`, `runtime/`, `index.ts`: tokens, runtime helpers and public exports.
- `scripts/`: package/API checks.

Read README, current package scripts, relevant tests and any nested AGENTS.md first.
Check current git status and task/PR revision; preserve unrelated changes.
Use current implementation as evidence, not old issue descriptions.

## Setup and verification

Run from this repository root. Prefix shell commands with `rtk`; use `rtk proxy`
when unfiltered output is needed. Do not bypass hooks to obtain a green result.

```powershell
rtk bun install --frozen-lockfile
rtk bunx playwright install chromium
rtk bun run check
```

- Use Bun 1.3.14. Check covers API/type checks, browser tests, library build and packed-export smoke.

## Contracts to preserve

- Preserve public JS/TS/CSS export subpaths and Vue peer dependency compatibility. Validate packed exports, not just imports from sibling source.
- Breaking component/type changes require a review note and explicit version decision; do not silently break consumers.
- Preserve keyboard access, focus behavior, accessible names and theme token usage in shared components.
- Do not propagate changes into application repositories. Hand off a tested SHA/version and compatibility notes to their owners.

- Preserve existing UI language; reuse shared tokens/components and preserve keyboard navigation, focus behavior and accessible names.
- Clean up listeners, timers and subscriptions on disposal. Verify visual changes in the running UI, or report UI verification NOT_RUN with its reason.

## Parallel work

- This chat owns only its assigned repositories; sibling repositories are read-only unless explicitly assigned.
- Each writer uses a separate worktree. Never switch branches, reset, clean or stash another writer's checkout.
- The lead may use a few Luna subagents for bounded independent work. Give each an acceptance condition, file boundary and dependency revision.
- Assign manifests, lockfiles, shared helpers and generated outputs to one writer. Integrate returned commits sequentially.
- Pin external dependencies by version/SHA; if existing tooling needs sibling paths, provision isolated pinned checkouts instead of changing another chat's code.
- Hand off contract changes with operation/type, inputs, outputs, errors, version and compatibility evidence. Finish independent work while a dependency is pending.

## Completion

- Prefer existing code and tools; avoid unrelated cleanup and new abstractions. Trace callers before fixing a shared bug.
- Add the smallest meaningful regression check for changed nontrivial behavior; include failure paths for permissions, migrations or persistence.
- For code, manifest or dependency changes, run relevant checks during work and the full gate on the integrated revision. Documentation-only edits need path/command and diff checks, not an application rebuild.
- For nontrivial work, use an independent Luna review of the integrated revision.
- Report exact SHA (and dirty diff if applicable), dependency revisions, commands and PASS / FAIL / NOT_RUN with reasons. Never claim a missing native or external check passed.
- Follow the current organization quality contract; CI absence is not evidence of failure or success, and local checks do not bypass protected-branch rules.
- Do not publish releases, upload artifacts, alter access or close umbrella issues unless that action is authorized. Never replace bytes of an already published version.

## Worktree bootstrap

- Before delegating, ensure this AGENTS.md exists in the new worktree. Git does not carry uncommitted instructions into worktrees: copy only the approved instruction file if absent; do not copy unrelated working changes.
