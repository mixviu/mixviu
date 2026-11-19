# Widget rectangles

The shared Mix application host provides `<app>.widgets` without application
callbacks. Arguments are an object with optional `ids` and `kinds` arrays of
strings, and `visible_only` (boolean, default false). Filters intersect; an
empty array matches nothing. Unknown ids produce no entries. Unknown argument
keys, invalid filter types and non-object arguments refuse with rc 10 and
`error_code: "ARGUMENT"`.

The reply is an array:

```json
[{"id":"save","kind":"button","label":"Save",
  "rect":{"x":12,"y":48,"w":80,"h":28},
  "window":{"id":7,"generation":2},"enabled":true,"focused":false}]
```

Ids are the authored scene node keys. A `control` definition must use the
same id as its scene node (scene validation enforces this). Kinds are scene families, or the control's
declared kind. Internal compound-control children and repeated template nodes
are omitted: they have no unique scene node identity. No generated identities
are exposed. Hidden nodes and nodes not laid out in the latest pass are absent.
`visible_only` requires a positive intersection with the layout clip; it does
not account for other windows covering this surface. Bounds remain the full
allocation, rather than the clipped fragment. Labels are optional authored
labels/text; they are not a substitute for stable ids.
Mounting a frame with any non-root id also present in the application scene
is refused at load. The frame root is omitted, leaving one flat namespace
of unique authored ids. Scene validation also refuses multiple parents.

Coordinates are surface-local logical units, including client decorations.
Join `window.id` and `window.generation` with `comp.windows.list`, then add
the window's output-local `x,y` to the rectangle origin. Add the output origin
for desktop-global coordinates. Apply any compositor presentation transform
and clip separately; physical pixels require the output scale. Discard a
result when its window generation no longer matches.

The host resolves the compositor identity on demand using the app host's own
process identity. It refuses `WINDOW_UNAVAILABLE` if the window catalogue is unavailable,
the surface is not yet mapped, or several windows belong to the process; it
never guesses an anchor. Empty tables return immediately without consulting
the compositor. A headless instance that has not laid out a frame returns `[]`.

The host validates and queues queries, requests a frame, and builds the table
only during that frame. Requests pending together share the capture. The latch
clears after capture; ordinary frames do not construct, update or scale a
rectangle table. Each capture rebuilds labels and kinds from the resolved
scene, so metadata never relies on a revision-static reuse path. Replies use
the existing Bus runtime/client for compositor lookup; no thread or runtime is
created per query. No compositor polling runs between queries.

The standalone renderer API pairs `request_widget_rectangles()` with the next
`show()` and then `widget_rectangles(args)`. Without a request it retains the
previous requested capture, which the Bus host never serves as a fresh reply.

Coverage currently includes the root scenes of Mix application bundles. Child
viewports and the separate furniture scene host are not included. Multiple
mapped windows in one process cause `WINDOW_UNAVAILABLE`; a stable per-surface
compositor identity is required to extend this safely. This does not yet close
the complete all-hosts contract.

The headless `rectangle_frame_cost` test uses 100 buttons, 20 warm-up frames
and 500 frames per case, with no GPU. Run it with `cargo test --release -p
lib-viu --features test-support --test widget_rectangles rectangle_frame_cost
-- --nocapture --test-threads=1`. It compares ordinary frames with a pending
query on every frame. The query case includes capture, metadata construction,
filtering and reply value allocation. It excludes wire encoding, transport,
compositor lookup and painting; it is not a desktop frame budget.
