# Pages and alt text

A draw.io file can have several pages. `page` selects one, counted from 0. It
is optional, and defaults to 0.

```md
{{#drawio path="diagrams/hello.drawio" page=1}}
```

{{#drawio path="diagrams/hello.drawio" page=1}}

## Alt text

Every diagram gets an alternative text for screen readers, and for readers
whose browser cannot load the image. By default it is `Diagram <name> page
<N>`, which says little. Set a description with `alt`:

```md
{{#drawio path="diagrams/hello.drawio" page=0 alt="A box that says Hello World"}}
```

{{#drawio path="diagrams/hello.drawio" page=0 alt="A box that says Hello World"}}
