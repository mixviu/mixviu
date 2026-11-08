# Gold frame discovery checkpoint

This is a lab implementation, not an accepted app port. Monitor selects the
shared frame. Its content still requires the full gold-parity rework.

`contract.json.host.frame: "gold"` selects the shared Mix frame scene in
`libraries/viu/mix/gold-frame.scene.mix`. The generic host mounts that scene
around an independently resolved app view. Mix supplies header groups, icon,
menus, toolbar, title, status, appearance and window choices through
`libraries/viu/mix/gold-frame.mix`. Existing Viu menu, command-bar, design,
font, icon and native caption facilities draw those values.

The helper's `apply(model, options)` retains application values. Required
options are `icon`, `title`, `menus`, `toolbar` and `status`; optional
`appearance` and `window` select the existing design/window contracts.
Menu and toolbar actions arrive at `viu.frame.intent`. Caption close arrives
at `viu.frame.close`; the Mix owner performs its own persistence and operation
checks before publishing `window_close: true` to authorise closure.

Mounting refuses reserved node collisions and preserves application IDs,
bindings, focus and root intents. Both scenes validate models before committing
app state. Content is clipped to its allocated region, and draw receipts cover
both scenes. Independent surveys prompted the identity, close, clipping,
placeholder, fallback and draw-receipt repairs. Tests exercise actual mounted
scenes and native root-control input.

The host follows settingsd through an embedded Mix session worker using the
existing native connection. It installs the retained gold toolkit's complete
`session_theme` look, including app overrides, automatic mode, fonts, geometry,
accent/tints and tunables. Lost-authority/custom-source fallback retains the
file theme and last accepted UI settings. Frame bars, marks and title/status
type use live native design metrics. Private headless tests change the look on
one app PID, then test authority disappearance/restart and Mix-owned close.
The final exact-source binary passed those cases. Further automatic-mode,
override and tune-key combinations still require native acceptance coverage.

The Monitor comparison uses one deterministic native Bus fixture, the same
1040×680 client size and paused process state for gold and Mix. The matched pair
has substantial differences in regions, controls, table columns, widths,
striping, density, typography and status information. Side-by-side and SSIM
diff figures are retained privately. No visual gate has passed and no
deviation has been accepted. Normalised SSIM distortion is 0.0957189; details
are in `parity-method.md`. Performance-page composition and the remaining
applications still require their own gold-first captures and acceptance.

Run `tools/gold_parity_gate.mix` through CBC for the scoped Viu library/engine
fmt, locked check, tests, clippy and build. `--frame-only` is a focused repair
check, not the full checkpoint gate. `--prepare` returns worker formatting as
a patch; any worker diff refuses exact-source acceptance. This scoped gate
does not replace the full four-engine integration gates.
