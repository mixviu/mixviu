# Window visual attributes

The existing Smithay import is unchanged in origin and licence. This change
adds `CropRenderElement::element_mut`, a geometry-preserving access point
used to update the rounded-frame shader uniforms when a cropped window is
drawn as a thumbnail. The existing crop, alpha modulation, damage tracking
and rendering facilities remain in use. No dependency or foreign build step
is added.
