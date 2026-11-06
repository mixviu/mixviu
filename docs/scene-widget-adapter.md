# Shared scene and widget adapter (lab)

SceneView owns scene parsing, binding/template preparation, layout traversal,
geometry and event adaptation. Widgets::widget is the native control drawing
and intent entry point used by both Rust callers and Mix-authored scenes.
The gallery's grid, tiles and numbered headings belong to apps/gallery/scene.mix,
not the widget library. Both gallery hosts load that file at launch; changing
the declarative layout needs no Rust rebuild. Rust and Mix still independently
author widget definitions and state reduction. The declarative layout is shared.

## Scene vocabulary

`grid` adds children, integer columns (1..16), identity, column_gap, row_gap and
min_column_width. These are general layout ports, without gallery conventions.
`control` carries an object definition decoded by Widget::decode, and optional
on_intent command. A definition may be a model binding, for example:

````
```mix
root: {widget:"control", definition:"= $model.control", on_intent:"app.action"}
```
````

The definition's ID must equal its scene node ID. Every existing widget kind
is available through this leaf, including virtual list/table/tree and icons.
Definitions, bound ports, current values and datasets validate before a revision
commits. Invalid candidates retain the last good model, tree and revision.
Equivalent integral JSON numbers remain valid after scene normalisation to f64.
Dataset row_count must equal the selected dataset length; cells are strings,
with at least two columns for a table and one for list/tree.

Legacy text/button/field/toggle scene families map to the same native widgets.
Rows/columns, legacy list templates and spacer retain layout/routing semantics.
Image content supports known `icon:NAME` assets through the shared SVG renderer,
with explicit dimensions. Arbitrary file/network image sources refuse at load.
Window declarations remain host metadata; window nodes are not renderable
content. Unsupported content families refuse during preparation. Other existing
partial flex/style-port limitations in runtime-lab.md remain explicit; this is
not a complete compd renderer or compositor extraction.

## Identity, geometry and focus

Direct Rust layouts call Widgets::begin_frame once, widget once per control,
then finish_frame once. Geometry clears at begin_frame, and the completed map
remains available until the next frame. SceneView performs that lifecycle.
Widgets disappearing from a frame leave no stale automation target. IDs are
stable; child targets retain ID.suffix conventions. Consumers must choose IDs
that do not collide with another control's subwidget addresses.

SceneView exposes full node/control geometry. The gallery preserves its prior
published control, ID.tile and subwidget map; layout-only header nodes are not
additional gallery automation targets. Generic grid cell geometry accounts for
the full allocated cell width, not just the control's content width.

Focus requests refer to current geometry. A target disappearing before draw
refuses once and clears the latch. The gallery correlates that refusal only to
focus requests, leaving unrelated input replies pending for their own fence.
Malformed values passed directly to the public widget API draw a diagnostic,
emit no intent and register no target rather than panicking the UI thread.

## Events and admission

Widgets emit Intent{id,action,value}. Event::from_intent adds scene/node identity
and an explicit callback command. The gallery routes these to gallery.execute;
application reducers retain policy. Legacy scene callbacks keep their existing
{scene,node,kind,value?} or template-item body shapes, preserving Quoin callers.

Widget::actions, validate_action and admit define the shared native contract.
Row activate uses the same range/type rule as set. Enabled admission uses kind
and current state, not an application-specific widget ID. The host validates
before invoking either language reducer. Superseded completions retire their
inflight sequence and answer the pending request with SUPERSEDED; success does
not inherit an unrelated earlier error code.

The gallery deliberately keeps its fixed 40-command catalogue. Its scene fixes
IDs/order; inconsistent published definitions refuse instead of diverging from
that catalogue. A dynamic application registry is a later scope. Existing native
Bus ingress bounds, NOT_READY/BUSY and model-ready reply boundaries remain.

This adapter is isolated lab code. Existing compd, production/reference apps
and the preserved VT4 binaries remain available. No viud split, replacement
broker, main merge, production acceptance or performance-parity claim follows.

## File application and window primitives

The Dopus comparison extends the same dispatch with virtual `tree-table`,
`menubar`, `command-bar`, `property-grid`, `dialog-form`, `image-view` and
`window-controls`. Navigation, selection, file operations and modal reservations
remain in the application controller. Native controls emit typed intents and
expose current subwidget geometry for Bus input automation.

`dock` allocates application-declared edges and a final fill child. Resizable
edges use `min_fraction` and `max_fraction` for both native dragging and intent
admission. `split` owns a divider, bounded ratio and double-click reset. Numeric
admission uses the same native point precision as pointer-generated values.

`band` arranges app-authored `children` using one `groups` entry per child:
`preferred`, `minimum`, `align` (`left`, `center`, `right`), optional `shrink`
priority and integer `order`. It reuses the retained renderer-independent design
solver. Omit `shrink` for a fixed group; authored Mix uses `nil`, while a JSON
model uses `null`. Lengths are finite nonnegative numbers; `window.captions`
selects the current theme's three-caption width. Hidden or detached slot-only
children consume no band space. Malformed model updates refuse transactionally,
including a required definition, placement or group port removed by a binding.

A scene-declared `window_slot` attaches a dock child to a host-provided rectangle;
`slot_only` suppresses content while that slot is absent. A native host can use
`WindowFrame::show_with_chrome` to lend the whole client title band to a scene
containing `window-controls`. The older `show_with_header` keeps its caption
group and lends only the remaining title area. Rust callers can use the same
caption widget through `Widgets::widget` in their own layout.

Caption controls emit an owner `close` intent; minimise/maximise use ordinary
native viewport commands. The semantic actions are `close`, `minimize` and
`toggle-maximize`. Owners acknowledge automation and persist/cancel their work
before closing. Dopus maps these standard window intents without moving file
policy into Rust. Automatic framing uses the retained native backend detector;
hosts initialise it with their actual display handle, rather than inferring a
backend from environment variables alone.

Menu automation opens actual egui popups against fresh geometry. Bar, compact
button and explicit top-content popup forms share the same application command
data. Form queries disclose effective fallbacks; compositor-hosted SSD menus
and action bars are not yet advertised by this adapter.
