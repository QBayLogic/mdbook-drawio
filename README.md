[![documentation-badge](https://img.shields.io/badge/example-book-blue)](https://qbaylogic.github.io/mdbook-drawio/)

# mdbook-drawio

A preprocessor for [mdBook](https://github.com/rust-lang/mdBook) that automatically exports [draw.io](https://www.diagrams.net/) diagrams to SVG and embeds them in your book.

## Features
- Supports `{{#drawio path="..." page=N}}` directives in markdown.
- Runs the draw.io CLI to export diagrams as SVGs during `mdbook build`.
- Embeds the resulting SVGs as images in your book, with alt text.
- Animated arrows, and step-by-step diagrams that show one layer at a time.
- Exports in parallel, and only exports a diagram again when it changes.
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
- `steps`: set to `layers` to show the page step by step, see
  [Animation](#animation).

## Animation
- **Animated arrows**: turn on *Flow animation* for an arrow in draw.io. The
  animation is part of the exported SVG, so it plays in the book as is.
- **Step-by-step diagrams**: add `steps="layers"` to show a page one layer at a
  time, with buttons to go through the steps or play them:
  ```md
  {{#drawio path="diagrams/request.drawio" steps="layers" alt="A web request"}}
  ```
  Step 1 shows the bottom layer, and each next step adds the next layer. The
  layer names are the captions of the steps. All steps have the same size, so
  the diagram does not move. Without JavaScript, and in print, the whole
  diagram is shown. The script and style are added to the page, so no
  `book.toml` setting is needed.

## Showing the syntax
Directives inside code blocks and inline code are left alone, so a book can
show the syntax. Elsewhere, write `\{{#drawio ...}}` to show a directive as
text, the same as for mdBook's own `{{#include}}`.

## Example
The `example/` directory has a book that shows every feature, published at
<https://qbaylogic.github.io/mdbook-drawio/>.

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
