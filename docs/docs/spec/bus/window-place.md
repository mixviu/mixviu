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
| scale | `{factor,anchor:{x,y}}`, uniform factor 0–64 and window-local logical anchor | factor 1, anchor `{x:0,y:0}` |
| translate | `{x,y}`, logical offset | `{x:0,y:0}` |

Clip extents must be non-negative, all coordinates and endpoints finite and
within signed 32-bit logical bounds. Empty clips draw no pixels. Unknown
fields and malformed values use the existing typed `invalid_args` /
`invalid_argument` replies. Requests validate before changing any state.

Omitted attributes persist until changed, reset, or the window role ends.
They are compositor state, so restarting a shell does not lose them. Read
`comp.windows.list` or `windows.s<ID>.opacity`, `.z`, `.clip`,
`.requested_visible`. The existing `.visible` reports effective visibility.
The sampled `.scale` and `.translate` are also exposed on window rows and
the props tree. Anchor and translation coordinates use signed 32-bit logical
bounds. All numbers must be finite; partial objects and unknown keys are refused.

Opacity affects compositing, including owned popup trees and server chrome;
it does not disable input. `visible:false` suppresses drawing, input and
frame callbacks through the normal invisible-window path, while retaining
mapping and size. Clip crops pixels and does not change input geometry.

Stacking uses the existing workspace band and normal raise order for equal
z values. Compositor-marked drawables remain barriers, and lock, OSD and
other compositor bands remain above client windows. The scale and translation
contract follows; rotation remains reserved.

Composition first crops in window-local coordinates, then scales the complete
window tree (client content, owned popups, letterbox, shadow and server chrome)
about `anchor`, then translates it. For window-local point `p`, the drawn
point is `origin + anchor + factor * (p - anchor) + translate`. The camera and
output scale apply afterwards. These fields never change the client's configure,
buffer size, layout, placement state or workspace membership.

Input follows the drawn transform. Pointer, touch, tablet and injected input
hit-test using its inverse, then the existing surface input regions and
content fit. A factor of zero draws nothing and receives no input. Opacity
zero still receives input, as before. Clip continues to crop pixels only;
it does not narrow input regions. Logical geometry read through the Bus remains
the untransformed geometry. A closing window's retained image is never an input
target. Scaling does not promote windows across compositor band barriers.

A hidden window may still be raised; its new order takes effect when shown.
Unregistered same-layer drawables also act as stacking barriers. Clip corners
are clamped to the framebuffer after adding the window origin and projecting,
so an extreme origin, clip or zoom cannot overflow a physical rectangle.

Touch cancellation on hide applies to all active slots if any slot targets the hidden surface tree, following Smithay grab semantics. Clip coordinates and endpoints use the inclusive i32 range (-2147483648 to 2147483647); sizes remain non-negative.
