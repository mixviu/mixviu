# comp.window.place

Version 0.1.12. Requires `id` and the current role `generation`. Geometry
fields remain `output`, `x`, `y`, `width`, `height` (output-local logical
coordinates). Optional visual fields may be sent without geometry:

| Field | Value | Reset (`null`) |
| --- | --- | --- |
| opacity | finite number 0–1 | 1 |
| z | signed 32-bit JSON integer, higher is forward; `1.0` is refused | 0 |
| visible | boolean | true |
| clip | `{x,y,width,height}`, logical window-local numbers | no crop |

Clip extents must be non-negative, all coordinates and endpoints finite and
within signed 32-bit logical bounds. Empty clips draw no pixels. Unknown
fields and malformed values use the existing typed `invalid_args` /
`invalid_argument` replies. Requests validate before changing any state.

Omitted attributes persist until changed, reset, or the window role ends.
They are compositor state, so restarting a shell does not lose them. Read
`comp.windows.list` or `windows.s<ID>.opacity`, `.z`, `.clip`,
`.requested_visible`. The existing `.visible` reports effective visibility.

Opacity affects compositing, including owned popup trees and server chrome;
it does not disable input. `visible:false` suppresses drawing, input and
frame callbacks through the normal invisible-window path, while retaining
mapping and size. Clip crops pixels and does not change input geometry.

Stacking uses the existing workspace band and normal raise order for equal
z values. Compositor-marked drawables remain barriers, and lock, OSD and
other compositor bands remain above client windows. Scale/rotation are
reserved for a separately specified transform contract.

A hidden window may still be raised; its new order takes effect when shown.
Unregistered same-layer drawables also act as stacking barriers. Clip corners
are clamped to the framebuffer after adding the window origin and projecting,
so an extreme origin, clip or zoom cannot overflow a physical rectangle.
