# Widget rectangles

The shared Mix application host provides `<app>.widgets` without application
callbacks. Arguments are an object with optional `ids` and `kinds` arrays of
strings, and `visible_only` (boolean, default false). Filters intersect; an
empty array matches nothing. Unknown ids produce no entries.

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

Coordinates are surface-local logical units, including client decorations.
Join `window.id` and `window.generation` with `comp.windows.list`, then add
the window's output-local `x,y` to the rectangle origin. Add the output origin
for desktop-global coordinates. Apply any compositor presentation transform
and clip separately; physical pixels require the output scale. Discard a
result when its window generation no longer matches.

The host resolves the compositor identity on demand using the caller's process
identity. It refuses `WINDOW_UNAVAILABLE` if the window catalogue is unavailable,
the surface is not yet mapped, or several windows belong to the process; it
never guesses an anchor. Empty tables return immediately without consulting
the compositor. A headless instance that has not laid out a frame returns `[]`.

The host retains a last-pass table alongside its existing geometry. Each pass
clears it, including when the mounted content disappears. Queries filter and
serialise that table and do not cause layout, render, or application callbacks.
No compositor polling runs between queries.

Coverage currently includes the root scenes of Mix application bundles. Child
viewports and the separate furniture scene host are not included. Multiple
mapped windows in one process cause `WINDOW_UNAVAILABLE`; a stable per-surface
compositor identity is required to extend this safely. This does not yet close
the complete all-hosts contract.

The headless debug test `rectangle_frame_cost` uses 100 buttons, 20 warm-up
frames and 500 frames per case, with no GPU. One serial run measured:

| Case | ms/frame |
|---|---:|
| Collection disabled (test-only baseline) | 5.150 |
| Last-frame table, no query | 5.291 |
| Last-frame table, query each frame | 5.651 |

The table adds about 0.141 ms in this run; materialising an unfiltered reply
adds about 0.360 ms. These are debug CPU timings, not release or desktop
budgets. The query case includes filtering and reply value allocation, and
excludes wire encoding, transport, compositor lookup and painting. Other runs
show noise comparable to the table's cost; rerun the test for comparisons.
