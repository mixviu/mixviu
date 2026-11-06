# Generic Mix bundle host (lab)

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

The old gallery comparator remains under `apps/gallery-reference` to retain
its tests and independent Rust oracle; it is not used by the bundle host.

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
