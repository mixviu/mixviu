// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The HDR composite: the scene drawn into an FP16 blending-space texture
//! (colour-managed surfaces decoded on the way in), then that texture encoded
//! to BT.2020 PQ in one pass onto the 10-bit scanout buffer.
//!
//! The intermediate is one persistent texture with its own damage tracker, so
//! a frame that changes a cursor's worth of pixels redraws and re-encodes a
//! cursor's worth of pixels. What HDR costs: no direct scanout or overlay
//! planes while it is on (every surface is composited, as KWin does whenever
//! the output's colour pipeline is not offloaded) and one FP16 output-sized
//! texture per HDR output.

use super::describe::{OutputColor, Tagged};
use super::glsl;
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::{Element, Id, Kind, RenderElement, UnderlyingStorage};
use smithay::backend::renderer::gles::{
    GlesError, GlesFrame, GlesRenderer, GlesTexProgram, GlesTexture, Uniform,
};
use smithay::backend::renderer::utils::{
    CommitCounter, DamageBag, DamageSet, DamageSnapshot, OpaqueRegions,
};
use smithay::backend::renderer::{Bind, Offscreen};
use smithay::utils::user_data::UserDataMap;
use smithay::utils::{Buffer, Physical, Point, Rectangle, Scale, Size, Transform};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// The decode in force for the element being drawn: the plain decode
    /// program and its uniforms. The texture-program seam reads it, so an
    /// element that swaps in its own program (decor's corner clip) swaps in
    /// that program's HDR variant and keeps decoding.
    static ACTIVE: RefCell<Option<(GlesTexProgram, Vec<Uniform<'static>>)>> =
        const { RefCell::new(None) };
}

/// The decode for the element being drawn, if any (see [`ACTIVE`]).
pub fn active_decode() -> Option<(GlesTexProgram, Vec<Uniform<'static>>)> {
    ACTIVE.with(|a| a.borrow().clone())
}

/// An element drawn through the decode stage when its surface is tagged.
pub struct Decoded<'a, E> {
    inner: &'a E,
    decode: Option<(GlesTexProgram, Vec<Uniform<'static>>)>,
    /// The surface's description changed (or appeared, or went) since the last
    /// frame: its whole area is stale in the intermediate even without buffer
    /// damage, so report all of it.
    restale: bool,
}

impl<E: Element> Element for Decoded<'_, E> {
    fn id(&self) -> &Id {
        self.inner.id()
    }
    fn current_commit(&self) -> CommitCounter {
        self.inner.current_commit()
    }
    fn location(&self, scale: Scale<f64>) -> Point<i32, Physical> {
        self.inner.location(scale)
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        self.inner.src()
    }
    fn transform(&self) -> Transform {
        self.inner.transform()
    }
    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.inner.geometry(scale)
    }
    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if self.restale {
            let size = self.inner.geometry(scale).size;
            return DamageSet::from_slice(&[Rectangle::from_size(size)]);
        }
        self.inner.damage_since(scale, commit)
    }
    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.inner.opaque_regions(scale)
    }
    fn alpha(&self) -> f32 {
        self.inner.alpha()
    }
    fn kind(&self) -> Kind {
        self.inner.kind()
    }
    fn is_framebuffer_effect(&self) -> bool {
        self.inner.is_framebuffer_effect()
    }
}

impl<E: RenderElement<GlesRenderer>> RenderElement<GlesRenderer> for Decoded<'_, E> {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), GlesError> {
        let Some((program, uniforms)) = &self.decode else {
            return self
                .inner
                .draw(frame, src, dst, damage, opaque_regions, cache);
        };
        ACTIVE.with(|a| *a.borrow_mut() = Some((program.clone(), uniforms.clone())));
        frame.override_default_tex_program(program.clone(), uniforms.clone());
        let drawn = self
            .inner
            .draw(frame, src, dst, damage, opaque_regions, cache);
        frame.clear_tex_program_override();
        ACTIVE.with(|a| *a.borrow_mut() = None);
        drawn
    }

    fn underlying_storage(&self, _renderer: &mut GlesRenderer) -> Option<UnderlyingStorage<'_>> {
        None
    }

    fn capture_framebuffer(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        cache: &UserDataMap,
    ) -> Result<(), GlesError> {
        self.inner.capture_framebuffer(frame, src, dst, cache)
    }
}

/// `elements` wrapped for a direct SDR draw with tagged surfaces decoded (no
/// intermediate, no re-damage): what a capture replay of the scene must draw so
/// its colours match the composite's. `program` is [`HdrPass::decode_program`].
pub fn decoded_sdr<'a, E: Element>(
    elements: &'a [E],
    program: &GlesTexProgram,
    tagged: &HashMap<Id, Tagged>,
) -> Vec<Decoded<'a, E>> {
    let target = OutputColor::sdr();
    elements
        .iter()
        .map(|e| Decoded {
            inner: e,
            decode: tagged
                .get(e.id())
                .filter(|t| !t.description.is_identity())
                .map(|t| (program.clone(), target.decode_uniforms(&t.description))),
            restale: false,
        })
        .collect()
}

impl<'a, E> Decoded<'a, E> {
    /// The wrapped element.
    pub fn inner(&self) -> &'a E {
        self.inner
    }
}

/// One HDR output's composite state.
pub struct HdrPass {
    size: Size<i32, Physical>,
    scale: Scale<f64>,
    texture: GlesTexture,
    tracker: OutputDamageTracker,
    rendered: bool,
    damage: DamageBag<i32, Physical>,
    decode: GlesTexProgram,
    encode: GlesTexProgram,
    id: Id,
    /// Description generation per tagged element at the last render, so a
    /// description change re-damages that element (see `Decoded::restale`).
    seen: HashMap<Id, u64>,
}

impl HdrPass {
    /// Allocate the FP16 intermediate and compile both stages. `Err` (the
    /// output stays SDR) when the GPU cannot render to half-float or a shader
    /// does not compile.
    pub fn new(
        renderer: &mut GlesRenderer,
        size: Size<i32, Physical>,
        scale: Scale<f64>,
    ) -> Result<Self, String> {
        let texture: GlesTexture = renderer
            .create_buffer(
                Fourcc::Abgr16161616f,
                Size::<i32, Buffer>::from((size.w, size.h)),
            )
            .map_err(|e| format!("FP16 intermediate {}x{}: {e}", size.w, size.h))?;
        let decode = renderer
            .compile_custom_texture_shader(glsl::decode_shader(), &glsl::decode_uniforms())
            .map_err(|e| format!("decode shader: {e:?}"))?;
        let encode = renderer
            .compile_custom_texture_shader(glsl::encode_shader(), &glsl::encode_uniforms())
            .map_err(|e| format!("encode shader: {e:?}"))?;
        Ok(Self {
            size,
            scale,
            texture,
            tracker: OutputDamageTracker::new(size, scale, Transform::Normal),
            rendered: false,
            damage: DamageBag::new(8),
            decode,
            encode,
            id: Id::new(),
            seen: HashMap::new(),
        })
    }

    /// The compiled decode stage, for [`decoded_sdr`].
    pub fn decode_program(&self) -> &GlesTexProgram {
        &self.decode
    }

    /// Whether this pass was made for `size` at `scale` (else rebuild it).
    pub fn fits(&self, size: Size<i32, Physical>, scale: Scale<f64>) -> bool {
        self.size == size && self.scale == scale
    }

    /// The intermediate: the frame in blending space (gamma 2.2, BT.709,
    /// 1.0 = SDR white). Clamped to 0..1 it is the frame's SDR rendition, which
    /// is what captures and screenshots should copy.
    pub fn texture(&self) -> &GlesTexture {
        &self.texture
    }

    /// Draw `elements` into the intermediate. Elements whose id is in
    /// `tagged` (and whose description is not already blending space) draw
    /// through the decode stage.
    pub fn render<E: RenderElement<GlesRenderer>>(
        &mut self,
        renderer: &mut GlesRenderer,
        elements: &[E],
        tagged: &HashMap<Id, Tagged>,
        output: &OutputColor,
    ) -> Result<(), String> {
        let wrapped: Vec<Decoded<'_, E>> = elements
            .iter()
            .map(|e| {
                let t = tagged.get(e.id());
                Decoded {
                    inner: e,
                    decode: t
                        .filter(|t| !t.description.is_identity())
                        .map(|t| (self.decode.clone(), output.decode_uniforms(&t.description))),
                    restale: t.map(|t| t.generation) != self.seen.get(e.id()).copied(),
                }
            })
            .collect();
        let seen: HashMap<Id, u64> = elements
            .iter()
            .filter_map(|e| tagged.get(e.id()).map(|t| (e.id().clone(), t.generation)))
            .collect();
        let age = usize::from(self.rendered);
        let mut target = renderer
            .bind(&mut self.texture)
            .map_err(|e| format!("bind FP16 intermediate: {e}"))?;
        let result = self
            .tracker
            .render_output(renderer, &mut target, age, &wrapped, [0.0, 0.0, 0.0, 1.0])
            .map_err(|e| format!("HDR composite: {e:?}"))?;
        if !self.rendered {
            self.damage.add([Rectangle::from_size(self.size)]);
        } else if let Some(damage) = result.damage
            && !damage.is_empty()
        {
            self.damage.add(damage.iter().copied());
        }
        self.rendered = true;
        self.seen = seen;
        Ok(())
    }

    /// This frame's encode element, to hand the scanout compositor.
    /// `hdr`: PQ-encode for an HDR output; otherwise the intermediate is drawn as
    /// it is (an SDR output: blending space is already its encoding, and the
    /// target clips HDR and wide-gamut values).
    pub fn encode_element(&self, output: &OutputColor, hdr: bool) -> HdrEncode {
        HdrEncode {
            texture: self.texture.clone(),
            program: hdr.then(|| self.encode.clone()),
            uniforms: if hdr {
                output.encode_uniforms()
            } else {
                Vec::new()
            },
            size: self.size,
            id: self.id.clone(),
            damage: self.damage.snapshot(),
        }
    }
}

/// Copy `src` of an HDR output's intermediate ([`HdrPass::texture`]) onto
/// `target` (sized `dst_size`): the frame's SDR rendition, HDR highlights
/// clipped at SDR white by an 8-bit target. What captures and screenshots of an
/// HDR output read, instead of the PQ-encoded scanout buffer.
pub fn copy_sdr(
    renderer: &mut GlesRenderer,
    intermediate: &GlesTexture,
    target: &mut smithay::backend::renderer::gles::GlesTarget<'_>,
    src: Rectangle<i32, Physical>,
    dst_size: Size<i32, Physical>,
) -> Result<smithay::backend::renderer::sync::SyncPoint, String> {
    use smithay::backend::renderer::{Frame, Renderer};
    let mut frame = renderer
        .render(target, dst_size, Transform::Normal)
        .map_err(|e| format!("HDR capture frame: {e}"))?;
    let dst = Rectangle::from_size(dst_size);
    let src = Rectangle::<f64, Buffer>::new(
        (src.loc.x as f64, src.loc.y as f64).into(),
        (src.size.w as f64, src.size.h as f64).into(),
    );
    frame
        .clear([0.0, 0.0, 0.0, 1.0].into(), &[dst])
        .map_err(|e| format!("HDR capture clear: {e}"))?;
    frame
        .render_texture_from_to(
            intermediate,
            src,
            dst,
            &[dst],
            &[],
            Transform::Normal,
            1.0,
            None,
            &[],
        )
        .map_err(|e| format!("HDR capture copy: {e}"))?;
    frame
        .finish()
        .map_err(|e| format!("HDR capture finish: {e}"))
}

/// The encode stage as a render element: the whole output, opaque, damaged
/// where the composite changed.
pub struct HdrEncode {
    pub texture: GlesTexture,
    pub program: Option<GlesTexProgram>,
    pub uniforms: Vec<Uniform<'static>>,
    pub size: Size<i32, Physical>,
    id: Id,
    damage: DamageSnapshot<i32, Physical>,
}

impl Element for HdrEncode {
    fn id(&self) -> &Id {
        &self.id
    }
    fn current_commit(&self) -> CommitCounter {
        self.damage.current_commit()
    }
    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size((self.size.w as f64, self.size.h as f64).into())
    }
    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, Physical> {
        Rectangle::from_size(self.size)
    }
    fn transform(&self) -> Transform {
        Transform::Normal
    }
    fn damage_since(
        &self,
        _scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        self.damage
            .damage_since(commit)
            .unwrap_or_else(|| DamageSet::from_slice(&[Rectangle::from_size(self.size)]))
    }
    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::from_slice(&[Rectangle::from_size(self.size)])
    }
    fn alpha(&self) -> f32 {
        1.0
    }
    fn kind(&self) -> Kind {
        Kind::Unspecified
    }
}

impl RenderElement<GlesRenderer> for HdrEncode {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        _cache: Option<&UserDataMap>,
    ) -> Result<(), GlesError> {
        self.draw_gles(frame, src, dst, damage, opaque_regions)
    }

    fn underlying_storage(&self, _renderer: &mut GlesRenderer) -> Option<UnderlyingStorage<'_>> {
        None
    }
}

impl HdrEncode {
    /// The encode draw, reachable from any renderer seam that holds a
    /// `GlesFrame`.
    pub fn draw_gles(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        frame.render_texture_from_to(
            &self.texture,
            src,
            dst,
            damage,
            opaque_regions,
            Transform::Normal,
            1.0,
            self.program.as_ref(),
            &self.uniforms,
        )
    }
}
