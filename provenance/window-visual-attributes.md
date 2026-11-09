# Window visual attributes

The existing Smithay import is unchanged in origin and licence. This change
adds `CropRenderElement::element_mut`, a geometry-preserving access point
used to update the rounded-frame shader uniforms when a cropped window is
drawn as a thumbnail. The existing crop, alpha modulation, damage tracking
and rendering facilities remain in use. No dependency or foreign build step
is added.

`TouchHandle::current_focuses` exposes the existing active-slot focus set so
the host can cancel a touch sequence if any slot targets a newly hidden
window. Checking only the grab's first slot would miss a second contact on
another window.

Review round 2: local Smithay desktop Space membership revision accessor increments on membership insertion/removal, allowing O(1) idle stacking reconciliation. No dependency changes.
