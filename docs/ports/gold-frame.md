# Gold frame discovery checkpoint

This is a lab prototype, not an accepted app port. No application currently
selects the frame. The existing application scenes and behaviour are unchanged.

`contract.json.host.frame: "gold"` selects the shared Mix frame scene in
`libraries/viu/mix/gold-frame.scene.mix`. The generic bundle loader composes
that scene around the app's content. Mix supplies header groups, icon, menus,
toolbar, title, status, appearance and window choices through
`libraries/viu/mix/gold-frame.mix`. Existing Viu menu, command-bar, design,
font, icon and native caption facilities draw those values.

The helper's `apply(model, options)` retains application values. Required
options are `icon`, `title`, `menus`, `toolbar` and `status`; optional
`appearance` and `window` select the existing design/window contracts.
Menu and toolbar actions arrive at `viu.frame.intent`. Caption close arrives
at `viu.frame.close`; the Mix owner performs its own persistence and operation
checks before publishing `window_close: true` to authorise closure.

Composition refuses reserved node collisions, control roots and roots with
intent handlers. The current adapter relocates a layout-only root to
`gold-content`. It must be replaced with an identity-preserving mount before
Dopus adopts it: Dopus's root dock resize intents are part of its Mix contract.
This restriction is an open implementation item, not an accepted deviation.

An independent whole-prototype survey identified root identity and caption
close ownership as defects. Explicit refusal now prevents silent root changes,
and caption close returns to Mix. The regression exercises the real embedded
interpreter, model publication, scene composition, geometry and root refusals.
Native headless testing separately exercises the generic framed host.

The Monitor comparison uses one deterministic native Bus fixture, the same
1040×680 client size and paused process state for gold and Mix. The baseline
has substantial differences in regions, controls, table columns, widths,
striping, density, typography and status information. Side-by-side and SSIM
diff figures are retained privately. No visual gate has passed and no
deviation has been accepted. Performance-page composition and the remaining
applications still require their own gold-first captures and acceptance.

Run `tools/gold_parity_gate.mix` through CBC for the scoped Viu library/engine
fmt, locked check, tests, clippy and build. `--frame-only` is a focused repair
check, not the full checkpoint gate. `--prepare` returns worker formatting as
a patch; any worker diff refuses exact-source acceptance. This scoped gate
does not replace the full four-engine integration gates.
