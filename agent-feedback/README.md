# Agent Feedback

Actionable observations that were out of scope for the task that surfaced them. In scope: fix it. Out of scope: file it here. Never expand a task's diff to fix an item recorded here.

One item per file in `items/`, named `YYYY-MM-DD-<slug>.md`.

## When to file

Anything a future contributor should act on:

- `bug`: a suspected defect left unpursued
- `cleanup`: duplication, dead code, inconsistency, refactor opportunity
- `perf`: speed, memory, payload or bundle size, build time
- `dx`: friction in builds, tests, tooling, or repo workflows
- `unclear`: code or docs that were confusing, and what would have clarified them

## Rules

1. **Verify first.** A guess is not feedback. Every item ends with a check that reproduces the claim.
2. **Dedupe first.** `grep -ril '<path or symbol>' agent-feedback/items`. If a file covers it, edit that file only when you add new information.
3. **Check the code site.** An intent comment there means the behavior is deliberate. Do not file it.
4. **Self-contained.** Paths, symbols, reasoning. Never reference conversation context or "earlier analysis".
5. **Cite by stable symbol**, never line number.
6. **State the defect and the check.** Never describe what works. Never narrate a landed fix.
7. **Direction is preventive for `unclear` and `dx`.** Name what would have stopped the trip: a comment, a doc line, a lint rule, a compile error, a debug-only warning. The goal is that the next agent does not hit it.
8. **Resolve by deleting the file in the same PR as the fix.** A partial fix rewrites the file to what remains.
9. **Won't-fix is a maintainer's call, never an agent's.** Add a comment (two lines max) at the code site stating the behavior and why it is deliberate, then delete the file. The comment is what stops re-filing. Never consult git history to learn whether something was resolved; if it is not in `items/` and not commented at the site, it is unresolved.

## Item format

`items/YYYY-MM-DD-<slug>.md`:

```md
---
type: bug | cleanup | perf | dx | unclear
impact: high | med | low
effort: high | med | low
site: <path/to/file.ts> › <nearestStableSymbol>
---

# <one-line imperative title>

<2-6 sentences: the problem, why it matters, a concrete direction. Cut evidence a fixer can re-derive from the site.>

Check: <command, input, or observation that reproduces the claim>
```

`impact`: what breaks or is lost if ignored. `effort`: expected size of the fix. Both are the filer's estimate; triage re-judges.

## Repo notes

The Zed extension for Marko: a small Rust crate (`src/lib.rs`) compiled to wasm by Zed, plus `extension.toml`, `languages/marko/` (language config and query overrides), and no test suite or CI.

**Reproduce a claim.** Install the extension as a dev extension in Zed (Extensions view, "Install Dev Extension", pointed at this checkout), then open a `.marko` file. Highlighting, injection, and language-server behavior are all observed in the editor; say in the item what you opened and what you saw. `cargo build --release --target wasm32-wasip1` catches compile errors without launching Zed.

**Guard tests.** There are none. An item's check line is a Zed observation or a `cargo` command, and a fix is verified by reinstalling the dev extension.

**Pre-ship.** `cargo build` (or the wasm target build) and `cargo fmt`. Reinstall the dev extension and confirm the behavior in a real `.marko` file.

**Gotchas.** The grammar is not vendored: `extension.toml` pins marko-js/tree-sitter by `rev`, so a highlighting defect is usually a grammar or `queries/` defect in that repo, and the fix there is invisible here until the `rev` is bumped. The comment above `[grammars.marko]` documents pointing `repository` at a local checkout for development; do not commit that. Version bumps in `extension.toml` are what the Zed extension registry consumes.
