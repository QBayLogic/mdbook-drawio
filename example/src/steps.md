# Step-by-step diagrams

A diagram can be shown one step at a time, which helps to explain a process.
Each layer of the page is a step: step 1 shows the first layer, step 2 adds the
second layer, and so on.

```md
{{#drawio path="diagrams/request.drawio" steps="layers" alt="A web request"}}
```

{{#drawio path="diagrams/request.drawio" steps="layers" alt="A web request"}}

Use the buttons to go through the steps, or **Play** to go through them
automatically. When the diagram has focus, the left and right arrow keys work
too.

## Making a step-by-step diagram

1. In draw.io, open the *Layers* panel (<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>L</kbd>).
2. Put what the first step shows in the bottom layer, and add a layer for each
   next step, in order from bottom to top.
3. Give each layer a name: it is shown as the caption of its step.
4. Add `steps="layers"` to the directive.

All steps have the same size, so the diagram does not move between steps.
Readers without JavaScript, and printed pages, see the last step: the whole
diagram.
