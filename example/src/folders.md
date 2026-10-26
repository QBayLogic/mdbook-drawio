# Diagrams in folders

`path` is relative to the book root, the folder with `book.toml`, so diagrams
can be organised in folders. This book has two diagrams with the same file
name:

```md
{{#drawio path="diagrams/frontend/overview.drawio" alt="Browser and API client"}}
{{#drawio path="diagrams/backend/overview.drawio" alt="HTTP API and database"}}
```

{{#drawio path="diagrams/frontend/overview.drawio" alt="Browser and API client"}}

{{#drawio path="diagrams/backend/overview.drawio" alt="HTTP API and database"}}

Each SVG keeps the folder of its diagram, so they do not overwrite each other:
`gen/drawio/diagrams/frontend/overview-page-0.svg` and
`gen/drawio/diagrams/backend/overview-page-0.svg`. A diagram outside the book
root (a `path` that starts with `../`) goes to a folder named after a hash of
its path.
