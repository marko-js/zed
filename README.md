# Marko for Zed

[Marko](https://markojs.com/) language support for [Zed](https://zed.dev):
syntax highlighting, embedded-language injections (TypeScript/CSS/SCSS),
bracket matching, outline, and the
[Marko language server](https://github.com/marko-js/language-server).

The grammar is the [@marko/tree-sitter](https://github.com/marko-js/tree-sitter)
parser, fetched and compiled by Zed from the `[grammars.marko]` entry in
`extension.toml`.

## Layout

```
extension.toml          # manifest: [grammars.marko] + [language_servers.marko]
Cargo.toml              # Rust/wasm extension crate
src/lib.rs              # installs @marko/language-server and launches it --stdio
languages/marko/
  config.toml           # filetype, comments, brackets
  highlights.scm        # copy of the grammar repo's queries/
  injections.scm        # copy of the grammar repo's queries/
  brackets.scm          # Zed-specific (@open/@close), maintained here
  outline.scm           # Zed-specific (@item/@name), maintained here
```

The language server (`@marko/language-server`) is installed from npm on demand
into the extension's working directory and run with Zed's Node; it bundles its
own TypeScript, so no project setup is required. For embedded highlighting,
install Zed's TypeScript/CSS/SCSS languages.

## Local development

To build the grammar from a local checkout instead of GitHub, point
`[grammars.marko]` at it with a `file://` URL and a committed `rev` (see the
comment in `extension.toml`). Then, in Zed, run `zed: install dev extension`
and select this directory; Zed compiles the Rust extension to wasm and builds
the grammar. Open a `.marko` file to exercise it, and run `zed --foreground`
from a terminal to surface grammar build, query, and language-server errors.

`highlights.scm` and `injections.scm` are copies of the grammar repo's
tool-agnostic `queries/`; refresh them when the grammar changes:

```sh
cp ../tree-sitter/queries/*.scm languages/marko/
```

`brackets.scm` (delimiter pairs, `@open`/`@close`) and `outline.scm`
(`@item`/`@name`) use Zed-only conventions and have no consumer outside Zed,
so they live here rather than in the grammar repo.
