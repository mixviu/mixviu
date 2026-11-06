# Generic Mix bundle host (lab)

The [gold frame discovery checkpoint](gold-frame.md) records the opt-in shared
Mix frame prototype and its current root-identity restriction. It is not a
visual acceptance claim for these application specimens.

`viu run APP_DIR [ARGS]` loads `scene.mix` and `app.mix`. Optional
`contract.json.host` selects `behaviour`, `service`, `title`, `size`,
`commands`, `tick_ms` (100 ms minimum) and `surface` (`window` or `background`).
Arguments arrive as `$app_args`. Behaviour paths stay within the bundle.
Mix publishes through the existing `viu-local scenes.model` protocol.
Callbacks, model publication and Bus replies use the existing bounded worker.

The shared host owns native Bus ingress, SceneView, typed native input,
application GPU readback and timer delivery (`viu.tick`). It contains no
gallery command catalogue, layout, reset or state reducer. Those stay in Mix.
Generic endpoints are `viu.state`, `viu.ui.tree`, `viu.ui.*`, `viu.capture`,
and `viu.quit`. App commands are declared as data and dispatched to Mix.
Background surfaces use the retained native layer-shell backend, one per output.
An optional `model.surfaces[output-name]` supplies a complete scene model for
that output; otherwise the common model is drawn. Mix owns those projections.

The reusable `image-fill` widget accepts `{phase,path?,colour:[r,g,b],fit}`.
It supports fill, fit, centre, tile and stretch. File decoding runs off the UI
thread with regular-file, byte, dimension and allocation limits. PNG, JPEG and
WebP use pure Rust decoders. Tiny-image tiling is bounded to 4096 quads.

Round-1 native additions were shared: bundle loader 138 lines, host 431 lines, retained
layer backend 2,211 lines across six files, plus the bounded image-fill and
decoder-selector/widget-admission extensions. Those three bundles add zero Rust.
No gallery header, catalogue or reducer moved into native code. The original
layer backend's logging calls use Viu tracing; its new close hook waits for
native reply delivery. Import hashes and those adaptations are recorded in
`provenance/app-ports-layer.json`; the thumbnail import retains its earlier hash
and records the explicit decoder-selector extension in `dopus-native.json`.

New locked packages are futures/futures-executor (native synchronous GPU setup),
image-webp (pure Rust image decoder), xkbcommon (system keyboard binding) and
its memmap2 version. These follow the retained layer backend; no C source is
vendored or compiled by these additions. System Wayland/XKB remain edge
libraries of Viu; the Mix library gains no desktop dependency.

Layer captures use the compositor's existing capture command; `viu.capture`
explicitly refuses on layers because the eframe readback applies to windows.
The native headless reference exposes one output. Real multi-output/hotplug
acceptance remains open, despite retaining the original per-output backend and
testing independent Mix output projections.

The old gallery comparator remains under `apps/gallery-reference` to retain
its tests and independent Rust oracle; it is not used by the bundle host.

Round 2 adds Screenshot and Prefs through this same host, bringing the total
to five Mix bundles with zero per-app Rust. Shared job delivery, command
deadlines and bounded image export are counted in
[round2-native.md](round2-native.md); application inventories and deferrals are
in [screenshot.md](screenshot.md) and [prefs.md](prefs.md).

## Dopus inventory for a later round

No Dopus rewrite here. Its host additionally owns: application CLI/profile
resolution; native/Rust comparator selection; file intents and key translation;
modal reservations and reply fences; macros/jobs and chooser service wiring;
appearance/settings synchronisation; decorated window/title-band slots;
native menu/action hosting offers and acknowledgement; frame/input/capture
receipts, compact state and command descriptions; asynchronous shutdown with
persisted config and delivered replies. Some are reusable native mechanisms,
others must move to Dopus Mix policy. They cannot simply be copied into the
generic host as application-specific dispatch.
