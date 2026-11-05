# Paired Viu widget galleries

Two lab applications with independent Rust/Mix composition and state reduction,
shared Rust admission checks, and one `viu::widgets` implementation. Each
requests a 740 x 820 logical-pixel
client area, suitable for comparison on a desktop of at least 1536 x 864.
Window captions distinguish the languages; the client content is identical.

Run the CBC-built `viu-gallery` in a MixOS graphical session:

```
viu-gallery rust
viu-gallery mix /path/to/apps/gallery/gallery.mix
```

The applications register as `gallery-rust` and `gallery-mix` on the existing
native Bus (`MIXOS_NODED_URL`). There is no application socket or extra broker
implementation. Both paths have bounded ingress. The Mix interpreter runs on
its own worker; Rust applies its own controller, without invoking Mix.

## Coverage

Label, button, single-line field, multiline editor, checkbox, toggle, radio
group, tabs, slider, numeric input, progress, combo, virtual list/table,
tree disclosure/selection, disclosure, menu, confirmation dialog, draggable
split, colour picker, canvas, local link, tooltip and separator: 24 samples.
The list/table use 1,000 rows with native visible-row rendering.

These are foundation samples, not complete replacements for every existing
toolkit component. The tree has one example parent; the table demonstrates
selection, ordering and two displayed fields. Rich file metadata, nested
trees, arbitrary column resizing and file drag/drop remain later consumers.
The local link records activation and does not open a browser.

`viu::widgets::Widget` and `Widgets::widget` are public Rust APIs usable in
other layouts. Mix supplies equivalent definitions and typed values through
the embedded runtime. Application controllers own values and actions; the
shared library renders widgets and emits intents.
Choices, captions, tab content, tree parent, context action and canvas shapes
come from application-supplied `properties`. Lists read an application-selected
`model.datasets` entry; the shared renderer generates no gallery rows. Both
controllers load the same explicit `rows.json` fixture as their data control.

## Complete action catalogue

`gallery.describe` lists every widget, supported actions and corresponding
Bus command. `HELP` includes each `gallery.widget.ID.ACTION`. Each widget has
a typed `set` action, including read-only display examples so stress tests
can update them. The 36 named widget commands include three state-only setters:
`button.set`, `canvas.set` and `link.set` update the model/status, while their
visible content comes from properties. Interactive widgets additionally expose
activate, submit,
accept/cancel, expand or sort as applicable.
Activation records a widget name/count; this specimen does not retain its
payload or perform row-specific work from it.

```
send gallery-rust gallery.describe
send gallery-mix gallery.widget.slider.set body=json_encode({value:72})
send gallery-rust gallery.widget.field.submit body=json_encode({value:"Hello"})
send gallery-mix gallery.widget.dialog.set body=json_encode({value:true})
send gallery-mix gallery.widget.dialog.accept
send gallery-rust gallery.execute body=json_encode({action:"reset"})
send gallery-rust gallery.state
```

`gallery.execute {id,action,value}` is the equivalent generic form. Unknown
IDs/actions and invalid value types/ranges refuse before mutation. Rust and
Mix use the same admission checks and independent state reducers. Successful
action replies mean the resulting model was applied, not physical presentation.
Screenshot equality confirms equal models and shared-renderer determinism.
Mix supplies composition/behaviour; native Viu performs rendering for both.

Native egui input can also be driven through the Bus:

- `gallery.ui.tree` returns model and widget/subwidget rectangles.
- `gallery.ui.focus {id}` and `gallery.ui.hover {id}`.
- `gallery.ui.click {id,button?:"left"|"right",double?:bool}`.
- `gallery.ui.key {key,ctrl?:bool,shift?:bool,alt?:bool}`.
- `gallery.ui.text {text}` sends text to the focused field.
- `gallery.ui.scroll {id,delta}` sends wheel input over a widget.
- `gallery.ui.drag {id,fraction}` drags horizontally to 0..1 of its rectangle.
- `gallery.quit` closes only this gallery.

Open menus/dialogs expose their child IDs through `gallery.ui.tree` (for
example `menu.Gamma`, `combo.Beta`, `dialog.Accept`, `tree.expand`,
`table.sort`, or `list.1.choose` after right-clicking row `list.1`).
The same input and intent paths serve people and automation. UI replies wait
for the input frames and resulting worker callbacks; they are not photon
latency receipts. Refusal/timeouts must be counted in stress results.

`VIU_GALLERY_WITNESS` enables diagnostic JSON writes per rendered frame.
Leave it unset for timing/resource measurements. Tests compare independently
constructed definitions, every semantic action and reset between controllers,
check refusal, and exercise all native widget renders. Rust verification uses
the native build service, never local Cargo.

With both galleries running on the current session Bus:

```
/opt/mixos/bin/mix /path/to/apps/gallery/check.mix parity
/opt/mixos/bin/mix /path/to/apps/gallery/check.mix stress 20 /tmp/gallery-stress.json
```

The driver exercises every declared widget action, alternates language order
between stress rounds and checks equal models after each pair. It records raw
per-action round-trip samples. A refusal or mismatch stops the run with an
error; an incomplete run cannot be counted as a passing timing sample.
Mix additionally converts/publishes its complete dataset/model on each action;
Rust mutates its model in place before the common Bus response encoding. Raw
times include that asymmetry and UI scheduling, not just interpreter execution.

The serial driver does not saturate ingress. Readiness gating, malformed-model
refusal, stale-generation handling, disappearing focus targets and requests
racing shutdown remain reusable-host hardening work. Run the specimen after
startup has settled; quit should be the final request.
