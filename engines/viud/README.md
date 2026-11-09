# viud

The MixViu Wayland compositor: the gold MixOS compositor (compd) without the in-process
egui widget toolkit. Furniture and applications run out of process in `viu`.
`cargo tree --locked -p viud -e normal` must show no `egui`, `egui-wgpu`,
`egui-winit` or `toolkit`.

Its private crates live in `crates/`, its units in `units/` (`viud@`,
`viud-headless@`). Configuration lives in `~/.config/mix/viud`, environment
variables are `VIUD_*` (and `MIX_VIUD_CONFIG`). The Bus service is still
`comp` with its `comp.*` commands: that is the protocol name.

Caller identity step 1 is audit-only. The port prefers verified Unix Bus
registration and keeps supervised re-registration and subscription replay.
A missing native endpoint falls back to TCP with a warning; `comp.callers`
reports `protected_recipient: false`. No caller needs a role or grant in this
step. See [the caller audit contract](../../docs/comp-callers.md).

Step 2 introduces desk-rooted roles and revocable grants after design review.
Step 3 migrates legitimate callers before enforcement; reads stay open and
input is refused while the session is locked. Capture has its own class for
future grants because it exposes screen pixels.
