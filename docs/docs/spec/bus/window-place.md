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
| scale | `{factor,anchor:{x,y}}`, factor 0 or 1e-6–64 and window-local logical anchor | factor 1, anchor `{x:0,y:0}` |
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

Placing `opacity`, `scale` or `translate` during a motion run immediately
takes ownership of those fields, even when the value equals the stored base.
Their tracks are removed, so the run's completion cannot overwrite the placed
values. Other tracks retain their existing clock and endpoint. `translate`
takes both axes; `scale` takes the factor and anchor. Explicit hide cancels
all visual tracks and suppresses any exit image immediately, including when
the client is destroyed before the next lifecycle tick.

When placement removes the final visual track, the pending motion reply
resolves promptly with `status:"superseded"`; it does not wait for the original
duration or reply budget. Geometry belonging to that superseded run stops.

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
content fit. Zero draws nothing. Factors below 1e-6 (including intermediate
animated values) accept no positional input; retained pointer and touch grabs
are cancelled on their next positional event. The inverse rejects any
non-finite result. Placement rejects nonzero factors below 1e-6.
Scale and translation do not change keyboard focus. Opacity
zero still receives input, as before. Clip continues to crop pixels only;
it does not narrow input regions. Logical geometry read through the Bus remains
the untransformed geometry. A closing window's retained image is never an input
target. Scaling does not promote windows across compositor band barriers.
An existing pointer or touch grab keeps its original surface focus until
release, including outside its bounds; its absolute event coordinates continue
through the current inverse transform. Relative-pointer deltas retain their
native device semantics.

A hidden window may still be raised; its new order takes effect when shown.
Unregistered same-layer drawables also act as stacking barriers. Clip corners
are clamped to the framebuffer after adding the window origin and projecting,
so an extreme origin, clip or zoom cannot overflow a physical rectangle.

Touch cancellation on hide applies to all active slots if any slot targets the hidden surface tree, following Smithay grab semantics. Clip coordinates and endpoints use the inclusive i32 range (-2147483648 to 2147483647); sizes remain non-negative.

Scale also accepts `anchor:"center"`, resolved from the window geometry at
composition time. Motion window targets accept `anchor:"center"` or a
logical `{x,y}` anchor alongside tracks/preset. Lifecycle entrances use the
window centre, including windows without prior placement. Window rows expose
the current sampled opacity, rather than just the requested endpoint.
Superseded replies add `by:"run"|"place"|"lifecycle"`.
