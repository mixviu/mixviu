// SPDX-License-Identifier: MIT OR Apache-2.0
use ui_surface::{UiSurfaces, HandleId, UiSpace};
use ui_surface::capture::{CaptureProgram, EmitCapture};
use ui::{TypedSurfaces, UiProgram};
use smithay::{backend::renderer::gles::GlesRenderer, utils::{Physical, Rectangle}};
use crate::capture::{setup::SetupOverlay, border::RegionBorder, dim::RegionDim,
    hud::StopHud, dialog::ContinueDialog, savedialog::SaveDialog,
    errordialog::ErrorDialog, encodialog::EncodingDialog};
fn create<U: UiProgram<Message=ui_surface::capture::CaptureMessage>>(
    registry: &mut dyn UiSurfaces, program: U, node: &str, renderer: &mut GlesRenderer,
    bounds: Rectangle<i32, Physical>, space: UiSpace, layer: u64, mut emit: EmitCapture,
) -> Option<HandleId> {
    let handle = registry.create_in_space(node, program, renderer, bounds.loc, bounds.size, space, layer).ok()?;
    registry.set_message_handler(handle, move |message| emit(message));
    Some(handle.id)
}
pub fn open(registry: &mut dyn UiSurfaces, program: CaptureProgram, node: &str,
    renderer: &mut GlesRenderer, bounds: Rectangle<i32, Physical>, space: UiSpace,
    layer: u64, emit: EmitCapture) -> Option<HandleId> {
    macro_rules! open { ($program:expr) => { create(registry, $program, node, renderer, bounds, space, layer, emit) }; }
    match program {
        CaptureProgram::Setup { width, height, media, kind, preselect } => open!(SetupOverlay::new(width,height,media,kind,preselect)),
        CaptureProgram::Border { width, height, region } => open!(RegionBorder::new(width,height,region)),
        CaptureProgram::Dim { width, height, region } => open!(RegionDim::new(width,height,region)),
        CaptureProgram::Stop => open!(StopHud),
        CaptureProgram::Continue(seconds) => open!(ContinueDialog::new(seconds)),
        CaptureProgram::Save { label, optimized } => open!(SaveDialog::new(label, optimized)),
        CaptureProgram::Error(message) => open!(ErrorDialog::new(message)),
        CaptureProgram::Encoding => open!(EncodingDialog::new()),
    }
}