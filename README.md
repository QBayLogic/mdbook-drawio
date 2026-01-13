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
{{#drawio path="diagrams/hello.drawio" page=0}}
```

## Example
See the `example/` directory for a working book with diagrams.

## Configuration
- `preprocessor.drawio.result-dir`: Output directory for SVGs (default: `mdbook-drawio`)
- `preprocessor.drawio.drawio-bin`: Path to the drawio CLI (default: `drawio`)
