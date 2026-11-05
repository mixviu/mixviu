# Paired Viu widget galleries

Two lab applications with independent Rust/Mix composition and state reduction,
shared Rust admission checks, and one `viu::widgets` implementation. Each
requests a 740 x 820 logical-pixel
client area, suitable for comparison on a desktop of at least 1536 x 864.
Window captions distinguish the languages; the client content is identical.
Both hosts load the application-owned `scene.mix` at launch. Its grid, tiles
and headings flow through `SceneView` into `Widgets::widget`; changing that
layout needs no Rust rebuild. The declarative layout is shared, while widget
definitions and reducers remain independently authored in Rust and Mix.

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
split, colour picker, canvas, local link, tooltip, separator, vector icon button and icon toolbar: 26 samples.
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
Direct layouts call `begin_frame`, `widget` and `finish_frame`; no gallery grid
is embedded in the shared library. See [scene/widget contract](../../docs/scene-widget-adapter.md).
Choices, captions, tab content, tree parent, context action and canvas shapes
come from application-supplied `properties`. Lists read an application-selected
`model.datasets` entry; the shared renderer generates no gallery rows. Both
controllers load the same explicit `rows.json` fixture as their data control.

## Complete action catalogue

`gallery.describe` lists every widget, supported actions and corresponding
Bus command. `HELP` includes each `gallery.widget.ID.ACTION`. Each widget has
a typed `set` action, including read-only display examples so stress tests
can update them. The 40 named widget commands include three state-only setters:
`button.set`, `canvas.set` and `link.set` update the model/status, while their
visible content comes from properties. Interactive widgets additionally expose
activate, submit,
accept/cancel, expand or sort as applicable.
Activation retains its widget ID and payload in model.last_activation. Toolbar
activation also selects that application item; disabled actions refuse.

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

Row activation validates the same index range/type as selection. Disabled
admission is shared by widget kind/current state, regardless of widget ID.
Superseded worker results retire their pending sequence and reply SUPERSEDED.

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

The serial driver does not saturate ingress. State exposes ready; commands
except state/quit refuse NOT_READY before initial publication. Models validate
transactionally, keeping the last good state on malformed definitions/values.
Stale-generation completions are rejected. A disappearing focus target refuses
FOCUS_TARGET_UNAVAILABLE without latching subsequent UI replies. Requests
racing shutdown remain hardening work; quit should be the final request.

## Shared vector icons

libraries/icons contains all 109 SVG assets from the pinned toolkit catalogue,
with name lookup and original ISC/MIT licence text. It has no renderer or
application dependency. viu::icons installs the SVG loader and exposes image/
button functions taking logical size, optional colour, enabled and selected
states. SVG textures cache by identity and requested resolution; tint is
applied at draw time. No file/network image loaders are enabled.

Mix uses the same API through icon widget values:
{name:"copy",size:24,colour:[80,180,230],enabled:true,selected:false}.
Names must exist, size is 8..64 points, and colour is three 0..255 channels.
Set this complete value through gallery.widget.icon.set. Activate takes an
icon name; disabled icon activation refuses. Toolbar actions use
gallery.widget.toolbar.activate {value:"save"}; disabled Trash refuses.
Native subwidget IDs include toolbar.copy, toolbar.save and toolbar.trash.
gallery.describe includes the complete icon name catalogue. The gallery uses
icons on buttons, menu choices, context actions and virtual list/table/tree rows.
