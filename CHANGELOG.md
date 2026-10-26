# Changelog for mdbook-drawio

This changelog is generated from the entries in `changelog/entries`, see
[changelog/README.md](changelog/README.md).

## 0.2.0 - *26 Oct, 2026*

### Added:
- Directives accept an `alt` attribute that sets the alternative text of the diagram, for example `{{#drawio path="diagrams/system.drawio" alt="Block diagram of the system"}}`. Without it, the text is `Diagram <name> page <N>`, as before.
  [#5](https://github.com/QBayLogic/mdbook-drawio/issues/5)
- Step-by-step diagrams: `{{#drawio path="..." steps="layers"}}` shows a page one layer at a time, with buttons to go through the steps or play them. The layer names are the captions of the steps. Without JavaScript, and in print, the whole diagram is shown.

### Changed:
- The `page` attribute is optional and defaults to 0. A malformed directive, such as one without `path`, is now an error that names the chapter, instead of being left in the page.
- An SVG keeps the folder of its diagram: page 0 of `diagrams/a/overview.drawio` is written to `<result-dir>/diagrams/a/overview-page-0.svg`, so that diagrams with the same file name no longer overwrite each other. SVGs from earlier versions are left in the result directory, and can be deleted.
  [#10](https://github.com/QBayLogic/mdbook-drawio/issues/10)
- A diagram is exported again when the diagram file, the page, `drawio-args` or the draw.io program changes. Before, only the modification time of the diagram file was compared.
  [#9](https://github.com/QBayLogic/mdbook-drawio/issues/9)
- The log is written to stderr, which mdBook shows, instead of to `mdbook-drawio.log` inside `src`, which was published with the book. Set `MDBOOK_DRAWIO_LOG=debug` for more detail.
  [#6](https://github.com/QBayLogic/mdbook-drawio/issues/6)
- A `result-dir` that is absolute or outside the book's `src` directory is now an error, as mdBook would not copy the SVGs into the book.
  [#11](https://github.com/QBayLogic/mdbook-drawio/issues/11)

### Fixed:
- `mdbook build` no longer shows a lone `true` line.
  [#13](https://github.com/QBayLogic/mdbook-drawio/issues/13)
- draw.io no longer fails when `ELECTRON_RUN_AS_NODE` is set, as some editors and tools do for their child processes.
  [#12](https://github.com/QBayLogic/mdbook-drawio/issues/12)
- Directives inside code blocks and inline code are left alone, so a book can show the syntax. Outside code, `\{{#drawio ...}}` shows a directive as text, as for mdBook's own directives.
  [#7](https://github.com/QBayLogic/mdbook-drawio/issues/7)
- A damaged diagram file, or a `page` that the diagram does not have, is now an error. Before, draw.io exported part of a damaged diagram without an error.
  [#8](https://github.com/QBayLogic/mdbook-drawio/issues/8)

### Performance:
- Diagrams are exported in parallel, by up to `jobs` draw.io processes at once (default: the number of CPUs, at most 8). A page that is used in several chapters is exported once.
  [#14](https://github.com/QBayLogic/mdbook-drawio/issues/14)

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
