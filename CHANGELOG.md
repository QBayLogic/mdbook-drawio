# Changelog for mdbook-drawio

This changelog is generated from the entries in `changelog/entries`, see
[changelog/README.md](changelog/README.md).

## 0.1.0 - *20 May, 2026*

First release.

### Added:
- An mdBook preprocessor that exports the pages of draw.io diagrams to SVG and
  embeds them, with `{{#drawio path="diagrams/hello.drawio" page=0}}`.
- Supports mdBook 0.5.
  [#3](https://github.com/QBayLogic/mdbook-drawio/pull/3)
- A diagram is only exported again when it is newer than its SVG.
  [#1](https://github.com/QBayLogic/mdbook-drawio/pull/1)
- `book.toml` options `result-dir`, `drawio-bin` and `drawio-args`. The build
  fails when draw.io fails.
  [#4](https://github.com/QBayLogic/mdbook-drawio/pull/4)
