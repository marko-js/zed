# Marko for Zed

The Zed extension for Marko: a Rust crate in `src/` compiled to wasm by Zed, plus `extension.toml` and `languages/marko/`. The grammar is not vendored; `extension.toml` pins marko-js/tree-sitter by `rev`.

## Agent feedback

Anything actionable but out of scope for the current task (suspected bug, cleanup, perf or size win, tooling friction, confusing code) must be filed in [`agent-feedback/`](agent-feedback/README.md) before finishing. Never drop it silently. Never fix it inside an unrelated diff.
