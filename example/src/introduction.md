# Introduction

This book shows every feature of
[mdbook-drawio](https://github.com/QBayLogic/mdbook-drawio), a preprocessor
that exports [draw.io](https://www.drawio.com/) diagrams to SVG and embeds
them in an mdBook.

Write a directive in your Markdown:

```md
{{#drawio path="diagrams/hello.drawio"}}
```

and the diagram appears in the book:

{{#drawio path="diagrams/hello.drawio"}}

The diagrams of this book are in the `example/diagrams` folder. Each chapter
shows one feature, and the source of each chapter is a good place to see how
the directives are written.
