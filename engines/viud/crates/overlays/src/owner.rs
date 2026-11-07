// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::capture::{
    border::RegionBorder, dialog::ContinueDialog, dim::RegionDim, encodialog::EncodingDialog,
    errordialog::ErrorDialog, hud::StopHud, savedialog::SaveDialog, setup::SetupOverlay,
};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    utils::{Physical, Rectangle},
};
use ui::{TypedSurfaces, UiProgram};
use ui_surface::capture::{CaptureProgram, EmitCapture};
use ui_surface::{HandleId, UiSpace, UiSurfaces};
fn create<U: UiProgram<Message = ui_surface::capture::CaptureMessage>>(
    registry: &mut dyn UiSurfaces,
    program: U,
    node: &str,
    renderer: &mut GlesRenderer,
    bounds: Rectangle<i32, Physical>,
    space: UiSpace,
    layer: u64,
    emit: Option<EmitCapture>,
) -> Option<HandleId> {
    let handle = registry
        .create_in_space(
            node,
            program,
            renderer,
            bounds.loc,
            bounds.size,
            space,
            layer,
        )
        .ok()?;
    if let Some(mut emit) = emit {
        registry.set_message_handler(handle, move |message| emit(message));
    }
    Some(handle.id)
}
pub fn open(
    registry: &mut dyn UiSurfaces,
    program: CaptureProgram,
    node: &str,
    renderer: &mut GlesRenderer,
    bounds: Rectangle<i32, Physical>,
    space: UiSpace,
    layer: u64,
    emit: EmitCapture,
) -> Option<HandleId> {
    // These display-only programs receive recorder updates but never send
    // them back through the surface channel.
    let emit = match &program {
        CaptureProgram::Border { .. } | CaptureProgram::Dim { .. } | CaptureProgram::Encoding => {
            None
        }
        _ => Some(emit),
    };
    macro_rules! open {
        ($program:expr) => {
            create(
                registry, $program, node, renderer, bounds, space, layer, emit,
            )
        };
    }
    match program {
        CaptureProgram::Setup {
            width,
            height,
            media,
            kind,
            preselect,
        } => open!(SetupOverlay::new(width, height, media, kind, preselect)),
        CaptureProgram::Border {
            width,
            height,
            region,
        } => open!(RegionBorder::new(width, height, region)),
        CaptureProgram::Dim {
            width,
            height,
            region,
        } => open!(RegionDim::new(width, height, region)),
        CaptureProgram::Stop => open!(StopHud),
        CaptureProgram::Continue(seconds) => open!(ContinueDialog::new(seconds)),
        CaptureProgram::Save {
            label,
            offer_optimized,
        } => open!(SaveDialog::new(label, offer_optimized)),
        CaptureProgram::Error(message) => open!(ErrorDialog::new(message)),
        CaptureProgram::Encoding => open!(EncodingDialog::new()),
    }
}
