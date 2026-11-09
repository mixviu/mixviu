# viud

The MixViu Wayland compositor: the gold MixOS compositor (compd) without the in-process
egui widget toolkit. Furniture and applications run out of process in `viu`.
`cargo tree --locked -p viud -e normal` must show no `egui`, `egui-wgpu`,
`egui-winit` or `toolkit`.

Its private crates live in `crates/`, its units in `units/` (`viud@`,
`viud-headless@`). Configuration lives in `~/.config/mix/viud`, environment
variables are `VIUD_*` (and `MIX_VIUD_CONFIG`). The Bus service is still
`comp` with its `comp.*` commands: that is the protocol name.
`comp.window.place` accepts persistent visual attributes alongside geometry:
`opacity` (number 0–1), `z` (signed 32-bit integer), `visible` (boolean), and
`clip` (`{x,y,width,height}` in window-local logical coordinates, non-negative
extents). Omission keeps the previous value; `null` resets it to 1, 0, true,
or no clip respectively. Attribute-only requests also work on constrained
windows and do not move, resize or unmap them. Opacity does not alter input.
Hidden windows retain their mapping and size, contribute no pixels, receive
no input, and use the existing invisible-window frame-callback policy.

Higher `z` stacks windows forward within their existing workspace band;
ties retain the normal raise order. Compositor-marked drawables are barriers,
and layer-shell, lock, OSD and screen UI bands are never promoted across.
`comp.windows.list` and `windows.s<ID>` expose `opacity`, `z`, `clip`, and
`requested_visible`; `visible` remains the effective visibility including
workspace and minimisation. State lasts for the window role's lifetime and
can be re-queried after a shell restart. Scale and rotation need a later
transform contract and are outside this command's current scope.

Hidden windows can still be raised; the order applies when they are shown.
Unregistered same-layer drawables are stacking barriers too. `z` requires a
JSON integer token (`1.0` is refused).
