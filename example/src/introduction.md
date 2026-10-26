# Introduction

This is an example book to verify the dev shell and build toolchain.

{{#drawio path="diagrams/hello.drawio" page=0 alt="Hello world diagram"}}

{{#drawio path="diagrams/subdiagrams/sub.drawio" page=0}}

## Syntax

Directives inside code are left alone, so a book can show the syntax:

```md
{{#drawio path="diagrams/hello.drawio" page=0 alt="Hello world diagram"}}
```

Outside code, escape a directive with a backslash: \{{#drawio path="diagrams/hello.drawio"}}.
