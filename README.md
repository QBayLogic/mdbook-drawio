[![documentation-badge](https://img.shields.io/badge/example-book-blue)](https://qbaylogic.github.io/mdbook-drawio/)

# mdbook-drawio

A preprocessor for [mdBook](https://github.com/rust-lang/mdBook) that automatically exports [draw.io](https://www.diagrams.net/) diagrams to SVG and embeds them in your book.

## Features
- Supports `{{#drawio path="..." page=N}}` directives in markdown.
- Runs the draw.io CLI to export diagrams as SVGs during `mdbook build`.
- Embeds the resulting SVGs as images in your book.
- Configurable output directory and drawio binary via `book.toml`.

## Installation

### With Nix (recommended)
```sh
nix build
```
The resulting binary will be in `./result/bin/mdbook-drawio`.

### With Cargo
```sh
cargo build --release
```
The binary will be in `target/release/mdbook-drawio`.

## Usage
Add a preprocessor section to your `book.toml`:
```toml
[preprocessor.drawio]
command = "mdbook-drawio"
```

In your markdown, use:
```md
{{#drawio path="diagrams/hello.drawio" page=0 alt="Overview of the system"}}
```

- `path`: the diagram, relative to the book root (the directory with `book.toml`).
- `page`: the page to show, counted from 0. Optional, defaults to 0.
- `alt`: the alternative text of the image, for screen readers. Optional,
  defaults to `Diagram <name> page <N>`.

Directives inside code blocks and inline code are left alone, so a book can
show the syntax. Elsewhere, write `\{{#drawio ...}}` to show a directive as
text, the same as for mdBook's own `{{#include}}`.

## Example
See the `example/` directory for a working book with diagrams.

## Configuration
- `preprocessor.drawio.result-dir`: Output directory for SVGs, relative to the
  book's `src` directory (default: `mdbook-drawio`). It must be inside `src`,
  because mdBook only copies files from there into the book. The SVGs are
  generated, so it is best to add this directory to `.gitignore`.
- `preprocessor.drawio.drawio-bin`: Path to the drawio CLI (default: `drawio`)
- `preprocessor.drawio.drawio-args`: Extra arguments to pass to the drawio
  invocation, for example `["--svg-theme", "light"]`
- `preprocessor.drawio.jobs`: The number of draw.io processes to run at once
  (default: the number of CPUs, at most 8)

Each SVG keeps the directory of its diagram, so
`diagrams/a/overview.drawio` page 0 becomes
`<result-dir>/diagrams/a/overview-page-0.svg`.

## Caching
A diagram is only exported again when the diagram file, the page,
`drawio-args` or the draw.io binary changes. A hash of these is stored in a
comment at the end of each SVG.

## Logging
mdbook-drawio logs to stderr. Set `MDBOOK_DRAWIO_LOG=debug` for more detail,
or `MDBOOK_DRAWIO_LOG=warn` for less.
