// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The message type shared by every capture overlay UI.
//!
//! Used three ways:
//! - as each capture UI UI's `UiProgram::Message` (the widgets emit these),
//! - carried over the surface message channel as `SurfaceMessageType::Capture`,
//! - dispatched *into* indicator UIs (`SetRegion` / `SetCountdown`) by the
//!   interface to update their drawn state live.

/// What kind of thing the capture targets. Carries no geometry — the concrete
/// target (with data) is [`crate::capture::session::CaptureTarget`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TargetKind {
    /// The windows currently selected in the canvas (`canvas.Select`).
    Windows,
    /// A rectangle anchored in world: pans/zooms with the camera.
    WorldRegion,
    /// A rectangle fixed in screen/output pixels: stays put under the camera.
    ScreenRegion,
    /// The whole output.
    FullScreen,
}

/// Screenshot vs. video. (Saving/encoding is deferred; this only changes the
/// session lifecycle — video arms the 5-minute keep-alive dialog.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CaptureMedia {
    Screenshot,
    Video,
}

/// Cursor position in overlay-local coordinates. The setup overlay is a
/// full-screen screen-space instance, so these map to output pixels (1:1 at
/// scale 1.0).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OverlayPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OverlayRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CaptureMessage {
    // ---- setup chooser (SetupOverlay → interface) ----
    SelectKind(TargetKind),
    SelectMedia(CaptureMedia),
    /// Toggle the transparent-background ("no background") option — when on, the
    /// capture omits the parallax/UI backdrop and emits transparent where no
    /// window is. Applies to window/world targets, video and screenshot.
    SetNoBackground(bool),
    /// Pointer pressed: begin a region drag at the last-tracked cursor
    /// (region targets only).
    DragStart,
    /// Pointer moved: tracks the cursor and, while dragging, extends the
    /// region rect.
    DragMove(OverlayPoint),
    /// Pointer released: finish the region drag.
    DragEnd,
    /// Commit the setup and begin capturing.
    Confirm,
    /// Abandon setup.
    Cancel,

    // ---- active (StopHud → interface) ----
    /// Stop the active capture (saving deferred — just stops).
    Stop,

    // ---- video keep-alive dialog (ContinueDialog → interface) ----
    ContinueCapture,
    StopFromDialog,

    // ---- save dialog (SaveDialog → interface) ----
    /// Write to the default location (XDG Pictures/Videos).
    SaveDefault,
    /// Open the XDG portal "Save As" dialog.
    SaveAs,
    /// Throw the capture away.
    Discard,
    /// Toggle the "Optimized encoding" checkbox in the save dialog (video only).
    ToggleOptimized(bool),

    // ---- inbound: interface → encoding dialog ----
    /// Update the optimized-encode progress shown (0..=100 percent).
    SetEncodeProgress(u32),

    // ---- inbound: interface → indicator UIs ----
    /// Update the region rect a border/dim element draws (screen pixels).
    SetRegion(OverlayRect),
    /// Update the countdown the continue dialog shows (seconds remaining).
    SetCountdown(u32),
}

/// Committed setup state, read synchronously after input has been applied.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CaptureSetup {
    pub kind: TargetKind,
    pub media: CaptureMedia,
    pub draft: Option<OverlayRect>,
    pub no_background: bool,
    pub output: Option<String>,
    pub windows: Vec<String>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SaveChoice {
    pub optimized: bool,
    pub format: Option<String>,
    pub path: Option<std::path::PathBuf>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CaptureProgram {
    Setup {
        width: i32,
        height: i32,
        media: CaptureMedia,
        kind: TargetKind,
        preselect: Option<OverlayRect>,
    },
    Border {
        width: i32,
        height: i32,
        region: OverlayRect,
    },
    Dim {
        width: i32,
        height: i32,
        region: OverlayRect,
    },
    Stop,
    Continue(u32),
    Save {
        label: String,
        offer_optimized: bool,
    },
    Error(String),
    Encoding,
}
pub type EmitCapture = Box<dyn FnMut(&CaptureMessage) + Send>;
/// Capture UI lifecycle / future Bus boundary.
///
/// The overlays owner registers its factory at compositor startup, before any
/// world's surfaces are created. Recorder opens an instance on the active world
/// and output and retains its handle. Input is applied by the UI frame/input
/// pump; emitted plain messages arrive on recorder's existing surface channel.
/// Recorder reads setup/save_choice synchronously while handling Confirm or
/// Save, before closing the instance. Closing is idempotent and is swept across
/// worlds, so a world switch cannot strand an overlay. Reads of a missing or
/// closed handle return None. No registry or typed program reference is shared
/// with recorder. An absent owner creates no instances and returns None.
pub trait CaptureUi {
    fn setup(&self, id: crate::HandleId) -> Option<CaptureSetup>;
    fn save_choice(&self, id: crate::HandleId) -> Option<SaveChoice>;
    fn capture_message(&mut self, id: crate::HandleId, message: CaptureMessage);
    fn open(
        &mut self,
        program: CaptureProgram,
        node: &str,
        renderer: &mut smithay::backend::renderer::gles::GlesRenderer,
        bounds: smithay::utils::Rectangle<i32, smithay::utils::Physical>,
        space: crate::UiSpace,
        layer: u64,
        emit: EmitCapture,
    ) -> Option<crate::HandleId>;
    fn close(&mut self, id: crate::HandleId);
}
pub type CaptureFactory = fn(
    &mut dyn crate::UiSurfaces,
    CaptureProgram,
    &str,
    &mut smithay::backend::renderer::gles::GlesRenderer,
    smithay::utils::Rectangle<i32, smithay::utils::Physical>,
    crate::UiSpace,
    u64,
    EmitCapture,
) -> Option<crate::HandleId>;
static OWNER: std::sync::OnceLock<CaptureFactory> = std::sync::OnceLock::new();
pub fn register(factory: CaptureFactory) {
    OWNER
        .set(factory)
        .unwrap_or_else(|_| panic!("capture UI owner already registered"));
}
pub fn factory() -> Option<CaptureFactory> {
    OWNER.get().copied()
}
