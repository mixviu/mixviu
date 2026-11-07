// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::state::Loop;
use smithay::{
    backend::renderer::gles::GlesRenderer,
    utils::{Physical, Rectangle},
};
use ui_surface::capture::CaptureProgram;
pub use ui_surface::{HandleId, UiSpace};
#[derive(Clone, Copy)]
pub struct CaptureHandle {
    pub id: HandleId,
}
impl CaptureHandle {
    pub fn untyped(self) -> HandleId {
        self.id
    }
}
pub fn load(
    state: &mut Loop,
    renderer: &mut GlesRenderer,
    program: CaptureProgram,
    bounds: Rectangle<i32, Physical>,
    space: UiSpace,
    layer: u64,
) -> CaptureHandle {
    let node = state.inner.environment.gpu.clone();
    let sender = state
        .inner
        .surface()
        .surface_message_buffer_channel
        .0
        .clone();
    let Some(registry) = state.inner.surface_mut().registry.as_mut() else {
        return CaptureHandle { id: HandleId(0) };
    };
    let id = registry
        .open(
            program,
            node.as_str(),
            renderer,
            bounds,
            space,
            layer,
            Box::new(move |message| {
                let _ = sender.send(crate::surface::protocol::SurfaceMessage {
                    message: crate::surface::protocol::SurfaceMessageType::Capture(message.clone()),
                });
            }),
        )
        .expect("registered capture UI owner must allocate the requested surface");
    if space == UiSpace::World {
        let tier = if layer & crate::scene::layer::base::Layer::SCENE_SURFACE_GROUP.bits() != 0 {
            crate::order::track::base::DrawLayer::GROUP
        } else {
            crate::order::track::base::DrawLayer::CONTENT
        };
        state
            .inner
            .register_drawable(uuid::Uuid::from_u128(id.0 as u128), tier);
    }
    CaptureHandle { id }
}
pub use load as load_snapshot;
