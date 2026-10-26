# Showing the syntax

Directives inside code blocks and inline code, such as
`{{#drawio path="diagrams/hello.drawio"}}`, are left alone. That is how this
book can show the syntax:

```md
{{#drawio path="diagrams/hello.drawio" page=0 alt="Hello"}}
```

Outside code, put a backslash before a directive to show it as text, the same
as for mdBook's own `{{#include}}`. Writing

```md
\\{{#drawio path=diagrams/hello.drawio}}
```

gives: \{{#drawio path=diagrams/hello.drawio}}
