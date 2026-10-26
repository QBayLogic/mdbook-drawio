# Configuration

The preprocessor is configured in the `[preprocessor.drawio]` table of
`book.toml`. This is the `book.toml` of this book:

```toml
{{#include ../book.toml}}
```

- `command`: how mdBook runs the preprocessor. Usually `mdbook-drawio`.
- `result-dir`: where the SVGs are written, relative to `src` (default:
  `mdbook-drawio`). It must be inside `src`, because mdBook only copies files
  from there into the book. The SVGs are generated, so add the folder to
  `.gitignore`.
- `drawio-bin`: the draw.io program (default: `drawio`).
- `drawio-args`: extra arguments for draw.io. This book uses `--no-sandbox` to
  run draw.io in CI. `["--svg-theme", "light"]` would export light diagrams for
  dark themes too.
- `jobs`: how many draw.io processes run at once (default: the number of CPUs,
  at most 8).

## Caching

Exporting a diagram takes a few seconds, so a diagram is only exported again
when the diagram file, the page, `drawio-args` or the draw.io program changes.

## Logging

The preprocessor logs which diagrams it exports. Set `MDBOOK_DRAWIO_LOG=debug`
for more detail, or `MDBOOK_DRAWIO_LOG=warn` for less.
