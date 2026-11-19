// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! One-shot Bus file captures, serviced on the compositor's render thread.

use crate::{Readback, Source, source_row, start_readback};
pub use comp_model::capture::CaptureSource;
use comp_model::capture::{CaptureFormat, CaptureFrameSpec, CaptureWindow};
pub use comp_model::reply::ControlReply;
use serde_json::Value;
use smithay::backend::renderer::ExportMem;
use smithay::backend::renderer::{
    ErasedContextId, Renderer,
    gles::{GlesMapping, GlesRenderer},
    sync::SyncPoint,
};
use smithay::output::Output;
use smithay::utils::Rectangle;
use std::cell::RefCell;
use std::io::Write;
use std::time::{Duration, Instant};

const READ_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_POLL_GAP: Duration = Duration::from_millis(50);

type Answer = Box<dyn FnOnce(Result<Value, ControlReply>)>;
pub type Pixels = (u32, u32, std::sync::Arc<Vec<u8>>, u64);
type PixelAnswer = Box<dyn FnOnce(Result<Pixels, ControlReply>)>;
thread_local! {
    static PIXEL_ANSWERS: RefCell<std::collections::HashMap<String, PixelAnswer>> = RefCell::new(std::collections::HashMap::new());
    // Retain at most three RGB allocations. A writer holding a frame prevents
    // reuse: Arc::get_mut succeeds only after every consumer has released it.
    static PIXEL_POOL: RefCell<Vec<std::sync::Arc<Vec<u8>>>> = const { RefCell::new(Vec::new()) };
    static RELEASE_POOL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A raw capture uses the same target fencing, crop and offscreen window path
/// as a file capture. Its private token is never opened as a filesystem path.
pub fn raw_answer(token: String, answer: impl FnOnce(Result<Pixels, ControlReply>) + 'static) {
    PIXEL_ANSWERS.with_borrow_mut(|answers| {
        answers.insert(token, Box::new(answer));
    });
}

pub fn raw_fail(token: &str, error: ControlReply) {
    if let Some(answer) = PIXEL_ANSWERS.with_borrow_mut(|answers| answers.remove(token)) {
        answer(Err(error));
    }
}
struct Request {
    id: u64,
    output: String,
    spec: CaptureFrameSpec,
    answer: Answer,
    raw: bool,
}

thread_local! {
    static REQUESTS: RefCell<Vec<Request>> = const { RefCell::new(Vec::new()) };
    static READS: RefCell<Vec<PendingRead>> = const { RefCell::new(Vec::new()) };
    static NEXT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

struct PollBudget {
    previous: Instant,
    spent: Duration,
}

impl PollBudget {
    fn new(now: Instant) -> Self {
        Self {
            previous: now,
            spent: Duration::ZERO,
        }
    }

    /// Charge only serviced reads. A KMS/offscreen hand-over or a suspended
    /// event loop can leave a long gap; it costs at most one polling interval.
    /// Paused sessions still poll offscreen and must detect a real GPU hang.
    fn unsignalled(&mut self, now: Instant) -> bool {
        self.spent += now
            .saturating_duration_since(self.previous)
            .min(MAX_POLL_GAP);
        self.previous = now;
        self.spent >= READ_TIMEOUT
    }
}

struct PendingRead {
    requests: Vec<Request>,
    mapping: GlesMapping,
    rgba: bool,
    readback: Readback,
    context: ErasedContextId,
    fence: SyncPoint,
    kind: CaptureSource,
    copy_us: u64,
    latency: ledger::frame_trace::WaitSpan,
    budget: PollBudget,
}

fn reads_pending(output: &Output, cursor: Option<bool>) -> bool {
    READS.with_borrow(|reads| {
        reads.iter().any(|read| {
            read.requests.iter().any(|r| {
                r.output == output.name() && cursor.is_none_or(|cursor| r.spec.cursor == cursor)
            })
        })
    })
}

fn wake_readback() {
    graphics::bridge::publish::wake::offer_capture_deadline(
        std::time::Instant::now() + std::time::Duration::from_millis(1),
    );
}

/// Test-only fault injection, with real PBO ownership and a never-ready fence.
#[cfg(feature = "test-support")]
pub mod testing {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    pub fn render_node() -> String {
        std::env::var("MIX_TEST_VIUD_RENDER_NODE").unwrap_or_else(|_| {
            smithay::backend::egl::EGLDevice::enumerate()
                .ok()
                .and_then(|mut devices| devices.find_map(|device| device.render_device_path().ok()))
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| "/dev/dri/no-render-device".into())
        })
    }
    thread_local! {
        static STALL: RefCell<Option<Arc<AtomicUsize>>> = const { RefCell::new(None) };
        pub(super) static POLLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    pub fn poll_count() -> usize {
        POLLS.get()
    }
    #[derive(Debug)]
    struct NeverFence(Arc<AtomicUsize>);
    impl Drop for NeverFence {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    impl smithay::backend::renderer::sync::Fence for NeverFence {
        fn is_signaled(&self) -> bool {
            false
        }
        fn wait(&self) -> Result<(), smithay::backend::renderer::sync::Interrupted> {
            panic!("stalled fence must never be waited on")
        }
        fn is_exportable(&self) -> bool {
            false
        }
        fn export(&self) -> Option<std::os::fd::OwnedFd> {
            None
        }
    }
    pub fn stall_next_read() -> Arc<AtomicUsize> {
        let drops = Arc::new(AtomicUsize::new(0));
        STALL.with_borrow_mut(|slot| *slot = Some(drops.clone()));
        drops
    }
    pub(super) fn replace_fence(fence: SyncPoint) -> SyncPoint {
        STALL
            .with_borrow_mut(|slot| slot.take())
            .map_or(fence, |drops| SyncPoint::from(NeverFence(drops)))
    }
    pub fn read_count() -> usize {
        READS.with_borrow(Vec::len)
    }
    pub fn pool_count() -> usize {
        PIXEL_POOL.with_borrow(Vec::len)
    }
    pub fn take_wake() -> bool {
        graphics::bridge::publish::wake::take_capture_deadline().is_some()
    }
}

pub fn enqueue(
    output: &Output,
    spec: CaptureFrameSpec,
    answer: impl FnOnce(Result<Value, ControlReply>) + 'static,
) -> u64 {
    let id = NEXT_ID.with(|next| {
        let id = next.get();
        next.set(id.wrapping_add(1));
        id
    });
    let raw = PIXEL_ANSWERS.with_borrow(|answers| answers.contains_key(&spec.path));
    REQUESTS.with_borrow_mut(|requests| {
        requests.push(Request {
            id,
            output: output.name(),
            spec,
            answer: Box::new(answer),
            raw,
        })
    });
    id
}

pub fn pending(output: &Output) -> bool {
    reading(output) || requested(output)
}

/// A submitted snapshot needs polling, rather than another scene render.
pub fn reading(output: &Output) -> bool {
    reads_pending(output, None)
}

/// Queued work needs a source render; a submitted PBO only needs polling.
pub fn requested(output: &Output) -> bool {
    REQUESTS.with_borrow(|requests| {
        requests
            .iter()
            .any(|request| request.output == output.name())
    })
}

pub fn pending_output(output: &Output) -> bool {
    REQUESTS.with_borrow(|requests| {
        requests
            .iter()
            .any(|request| request.output == output.name() && request.spec.window.is_none())
    })
}

pub fn pending_picture(output: &Output, cursor: bool) -> bool {
    REQUESTS.with_borrow(|requests| {
        requests.iter().any(|request| {
            request.output == output.name()
                && request.spec.window.is_none()
                && request.spec.listed_windows.is_none()
                && request.spec.cursor == cursor
        })
    })
}

pub fn pending_windows() -> bool {
    REQUESTS.with_borrow(|requests| requests.iter().any(|request| request.spec.window.is_some()))
}

pub fn pending_listed() -> bool {
    REQUESTS.with_borrow(|requests| requests.iter().any(|r| r.spec.listed_windows.is_some()))
}

/// Window requests never consume an output framebuffer, including the KMS tap.
pub(crate) fn service_windows(
    output: &Output,
    mut capture: impl FnMut(
        CaptureWindow,
        bool,
    ) -> Result<(Readback, std::sync::Arc<Vec<u8>>), ControlReply>,
) {
    for request in take(|request| request.output == output.name() && request.spec.window.is_some())
    {
        let target = request.spec.window.expect("window request");
        let started = std::time::Instant::now();
        let result = capture(target, request.raw).and_then(|(readback, pixels)| {
            complete_timed(
                &request.spec,
                readback,
                &pixels,
                output,
                CaptureSource::Offscreen,
                started.elapsed().as_micros() as u64,
                request.raw,
            )
            .map_err(capture_failed)
        });
        (request.answer)(result);
    }
}

pub fn capture_failed(message: impl Into<String>) -> ControlReply {
    ControlReply::Refused {
        error: "capture_failed",
        detail: serde_json::json!({"message": message.into()}),
    }
}

fn take(keep: impl Fn(&Request) -> bool) -> Vec<Request> {
    REQUESTS.with_borrow_mut(|requests| {
        let (taken, rest) = std::mem::take(requests).into_iter().partition(keep);
        *requests = rest;
        taken
    })
}

fn cancel(keep: impl Fn(&Request) -> bool, reason: &str) {
    let mut cancelled = take(&keep);
    READS.with_borrow_mut(|reads| {
        for read in reads.iter_mut() {
            let (taken, rest) = std::mem::take(&mut read.requests)
                .into_iter()
                .partition(&keep);
            cancelled.extend(taken);
            read.requests = rest;
        }
        reads.retain(|read| !read.requests.is_empty());
    });
    for request in cancelled {
        (request.answer)(Err(capture_failed(reason)));
    }
}

fn clear_idle_pool() {
    if RELEASE_POOL.get() && READS.with_borrow(Vec::is_empty) {
        PIXEL_POOL.with_borrow_mut(Vec::clear);
        RELEASE_POOL.set(false);
    }
}

/// Touch the bounded RGB ring before acknowledging the first recording frame.
/// Otherwise the next two captures fault in fresh full-output allocations while
/// the writer still owns the first image, spending the live sampling budget.
pub fn prepare_raw_pool(first: &std::sync::Arc<Vec<u8>>) {
    PIXEL_POOL.with_borrow_mut(|pool| {
        // Stitched multi-target frames have their own allocation. Warming from
        // one would copy bytes that this per-source conversion pool cannot use.
        if !pool
            .iter()
            .any(|frame| std::sync::Arc::ptr_eq(frame, first))
        {
            return;
        }
        while pool.len() < 3 {
            pool.push(std::sync::Arc::new(first.as_ref().clone()));
        }
    });
}

/// Retire one recording's private raw tokens without invoking its callbacks.
/// This is safe while its Session is borrowed during discard/timeout/seal.
pub fn release_raw(prefix: &str) {
    PIXEL_ANSWERS.with_borrow_mut(|answers| answers.retain(|token, _| !token.starts_with(prefix)));
    REQUESTS.with_borrow_mut(|requests| {
        requests.retain(|r| !(r.raw && r.spec.path.starts_with(prefix)))
    });
    READS.with_borrow_mut(|reads| {
        for read in reads.iter_mut() {
            read.requests.retain(|r| !r.spec.path.starts_with(prefix));
        }
        reads.retain(|read| !read.requests.is_empty());
    });
    RELEASE_POOL.set(true);
    clear_idle_pool();
}

fn expire_read(read: PendingRead) {
    // Drop the mapping and fence before delivering an error which may
    // synchronously seal the recorder and admit a new take.
    let requests = read.requests;
    drop(read.mapping);
    drop(read.fence);
    for request in requests {
        raw_fail(
            &request.spec.path,
            capture_failed("readback fence timed out"),
        );
        (request.answer)(Err(capture_failed("readback fence timed out")));
    }
}

pub fn fail(id: u64, reason: &str) {
    cancel(|request| request.id == id, reason);
}

pub fn fail_output(output: &Output, reason: &str) {
    cancel(|request| request.output == output.name(), reason);
}

pub fn fail_output_capture(output: &Output, reason: &str) {
    cancel(
        |request| request.output == output.name() && request.spec.window.is_none(),
        reason,
    );
}

pub fn fail_all(reason: &str) {
    cancel(|_| true, reason);
}

pub fn fail_windows(reason: &str) {
    cancel(
        |request| request.spec.window.is_some() || request.spec.listed_windows.is_some(),
        reason,
    );
}

/// Share one snapshot between callers. Raw recording requests retain the PBO
/// behind an EGL fence; screenshots still map synchronously. GL ordering makes
/// the PBO independent of subsequent source texture reuse. Mapping may change
/// the current GL target; nested restores its EGL surface afterwards.
pub fn service(
    renderer: &mut GlesRenderer,
    source: Source<'_, '_, GlesRenderer>,
    output: &Output,
    kind: CaptureSource,
    cursor: bool,
) {
    poll_reads(renderer, output, cursor);
    let requests = take(|request| {
        request.output == output.name()
            && request.spec.window.is_none()
            && request.spec.listed_windows.is_none()
            && request.spec.cursor == cursor
    });
    if requests.is_empty() {
        return;
    }
    let _service = ledger::frame_trace::span("capture_file_service", 0);
    let readback = source.readback;
    let started = std::time::Instant::now();
    if requests.iter().all(|request| request.raw) {
        // Each PBO is a snapshot ordered before later rendering on this context.
        // Recording owns one request at a time; cap global in-flight memory too.
        if READS.with_borrow(|reads| reads.len() < 3) {
            let latency = ledger::frame_trace::wait_span("capture_map_latency", 0);
            let copy = ledger::frame_trace::span("capture_copy_submit", 0);
            let result = start_pixels(renderer, source).and_then(|(mapping, rgba)| {
                let display = renderer.egl_context().display().clone();
                let fence = renderer
                    .with_context(|gl| {
                        let fence = smithay::backend::egl::fence::EGLFence::create(&display);
                        // SAFETY: with_context made this renderer's GL context current.
                        unsafe {
                            gl.Flush();
                        }
                        fence
                    })
                    .map_err(|e| format!("readback context: {e}"))?
                    .map_err(|e| format!("readback fence: {e}"))?;
                Ok((mapping, rgba, SyncPoint::from(fence)))
            });
            drop(copy);
            match result {
                Ok((mapping, rgba, fence)) => {
                    #[cfg(feature = "test-support")]
                    let fence = testing::replace_fence(fence);
                    READS.with_borrow_mut(|reads| {
                        reads.push(PendingRead {
                            requests,
                            mapping,
                            rgba,
                            readback,
                            fence,
                            kind,
                            context: renderer.context_id().erased(),
                            copy_us: started.elapsed().as_micros() as u64,
                            latency,
                            budget: PollBudget::new(Instant::now()),
                        })
                    });
                    wake_readback();
                }
                Err(error) => {
                    for request in requests {
                        (request.answer)(Err(capture_failed(&error)));
                    }
                }
            }
            return;
        }
        for request in requests {
            (request.answer)(Err(capture_failed("readback ring full")));
        }
        return;
    }
    let pixels = read_pixels(renderer, source);
    let readback_us = started.elapsed().as_micros() as u64;
    for request in requests {
        let result = pixels
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|pixels| {
                complete_timed(
                    &request.spec,
                    readback,
                    pixels,
                    output,
                    kind,
                    readback_us,
                    request.raw,
                )
            })
            .map_err(capture_failed);
        (request.answer)(result);
    }
}

/// Filtered recordings never consume a presentation framebuffer. The builder
/// returns None for an unchanged filtered scene, before any GPU readback.
/// A zero-sized internal raw answer denotes a repeat; it is never a file image.
pub fn service_listed(
    renderer: &mut GlesRenderer,
    output: &Output,
    mut picture: impl FnMut(
        &mut GlesRenderer,
        &CaptureFrameSpec,
    ) -> Result<
        Option<smithay::backend::renderer::gles::GlesTexture>,
        ControlReply,
    >,
) {
    for request in take(|r| r.output == output.name() && r.spec.listed_windows.is_some()) {
        let began = Instant::now();
        let result = (|| {
            let Some(mut texture) = picture(renderer, &request.spec)? else {
                if let Some(answer) =
                    PIXEL_ANSWERS.with_borrow_mut(|a| a.remove(&request.spec.path))
                {
                    answer(Ok((0, 0, Default::default(), 0)));
                }
                return Ok(Value::Null);
            };
            use smithay::backend::renderer::{Bind, Texture};
            let size = (texture.size().w, texture.size().h).into();
            let framebuffer = renderer
                .bind(&mut texture)
                .map_err(|e| capture_failed(e.to_string()))?;
            let readback = Readback {
                size,
                origin_bottom_left: false,
            };
            let pixels = read_pixels(
                renderer,
                Source {
                    framebuffer: &framebuffer,
                    readback,
                },
            )
            .map_err(capture_failed)?;
            complete_timed(
                &request.spec,
                readback,
                &pixels,
                output,
                CaptureSource::Offscreen,
                began.elapsed().as_micros() as u64,
                request.raw,
            )
            .map_err(capture_failed)
        })();
        (request.answer)(result);
    }
}

fn poll_reads(renderer: &mut GlesRenderer, output: &Output, cursor: bool) {
    if !reads_pending(output, Some(cursor)) {
        return;
    }
    #[cfg(feature = "test-support")]
    testing::POLLS.set(testing::POLLS.get() + 1);
    let reads = READS.with_borrow_mut(std::mem::take);
    let mut waiting = Vec::new();
    for mut read in reads {
        read.requests
            .retain(|r| PIXEL_ANSWERS.with_borrow(|answers| answers.contains_key(&r.spec.path)));
        if read.requests.is_empty() {
            continue;
        }
        if !read
            .requests
            .iter()
            .any(|r| r.output == output.name() && r.spec.cursor == cursor)
        {
            waiting.push(read);
            continue;
        }
        if read.context != renderer.context_id().erased() {
            for request in read.requests {
                (request.answer)(Err(capture_failed("readback renderer changed")));
            }
            continue;
        }
        if !read.fence.is_reached() {
            if read.budget.unsignalled(Instant::now()) {
                expire_read(read);
                continue;
            }
            waiting.push(read);
            wake_readback();
            continue;
        }
        read.latency.finish();
        let _service = ledger::frame_trace::span("capture_file_service", 0);
        let started = std::time::Instant::now();
        let pixels = finish_pixels(renderer, &read.mapping, read.rgba, read.readback);
        let readback_us = read.copy_us + started.elapsed().as_micros() as u64;
        for request in read.requests {
            let result = pixels
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|pixels| {
                    complete_timed(
                        &request.spec,
                        read.readback,
                        pixels,
                        output,
                        read.kind,
                        readback_us,
                        true,
                    )
                })
                .map_err(capture_failed);
            (request.answer)(result);
        }
    }
    // A callback can cancel other tokens while the queue is temporarily local.
    waiting.iter_mut().for_each(|read| {
        read.requests
            .retain(|r| PIXEL_ANSWERS.with_borrow(|answers| answers.contains_key(&r.spec.path)))
    });
    READS.with_borrow_mut(|reads| {
        reads.extend(waiting.into_iter().filter(|r| !r.requests.is_empty()))
    });
    clear_idle_pool();
}

/// Complete ready recording copies before deciding whether another capture
/// render is needed. Unsignalled fences remain queued without a GPU wait.
pub fn poll(renderer: &mut GlesRenderer, output: &Output) {
    poll_reads(renderer, output, false);
    poll_reads(renderer, output, true);
}

fn start_pixels<R: ExportMem>(
    renderer: &mut R,
    source: Source<'_, '_, R>,
) -> Result<(R::TextureMapping, bool), String> {
    let region =
        crate::framebuffer_region(Rectangle::from_size(source.readback.size), &source.readback);
    match renderer.copy_framebuffer(
        source.framebuffer,
        region,
        smithay::backend::allocator::Fourcc::Abgr8888,
    ) {
        Ok(mapping) => Ok((mapping, true)),
        Err(_) => start_readback(
            renderer,
            source.framebuffer,
            Rectangle::from_size(source.readback.size),
            &source.readback,
        ),
    }
}

pub(crate) fn read_pixels<R: ExportMem>(
    renderer: &mut R,
    source: Source<'_, '_, R>,
) -> Result<std::sync::Arc<Vec<u8>>, String> {
    let size = source.readback.size;
    if size.w <= 0 || size.h <= 0 {
        return Err("capture has no usable size".to_owned());
    }
    let copy = ledger::frame_trace::span("capture_copy_submit", 0);
    // File/recording consumers normalise to RGB, so prefer the offscreen
    // texture's native RGBA layout over the protocol's XRGB shm layout.
    let readback = source.readback;
    let (mapping, rgba) = start_pixels(renderer, source)?;
    drop(copy);
    finish_pixels(renderer, &mapping, rgba, readback)
}

fn finish_pixels<R: ExportMem>(
    renderer: &mut R,
    mapping: &R::TextureMapping,
    rgba: bool,
    readback: Readback,
) -> Result<std::sync::Arc<Vec<u8>>, String> {
    let map = ledger::frame_trace::span("capture_map_wait", 0);
    let bytes = renderer
        .map_texture(mapping)
        .map_err(|err| format!("map capture: {err}"))?;
    drop(map);
    let _convert = ledger::frame_trace::span("capture_rgb_copy", 0);
    pooled_rgb(
        bytes,
        readback.size.w as usize,
        readback.size.h as usize,
        rgba,
        readback.origin_bottom_left,
    )
}

fn pooled_rgb(
    bytes: &[u8],
    width: usize,
    height: usize,
    rgba: bool,
    bottom_up: bool,
) -> Result<std::sync::Arc<Vec<u8>>, String> {
    PIXEL_POOL.with_borrow_mut(|pool| {
        for frame in pool.iter_mut() {
            if let Some(pixels) = std::sync::Arc::get_mut(frame) {
                rgb_into(bytes, width, height, rgba, bottom_up, pixels)?;
                return Ok(frame.clone());
            }
        }
        let frame = std::sync::Arc::new(rgb(bytes, width, height, rgba, bottom_up)?);
        if pool.len() < 3 {
            pool.push(frame.clone());
        } else {
            pool.remove(0);
            pool.push(frame.clone());
        }
        Ok(frame)
    })
}

fn complete(
    spec: &CaptureFrameSpec,
    readback: Readback,
    pixels: &[u8],
    output: &Output,
    kind: CaptureSource,
) -> Result<Value, String> {
    complete_timed(
        spec,
        readback,
        &std::sync::Arc::new(pixels.to_vec()),
        output,
        kind,
        0,
        false,
    )
}

fn complete_timed(
    spec: &CaptureFrameSpec,
    readback: Readback,
    pixels: &std::sync::Arc<Vec<u8>>,
    output: &Output,
    kind: CaptureSource,
    readback_us: u64,
    raw: bool,
) -> Result<Value, String> {
    if let Some(expected) = spec.output_generation {
        let topology = output
            .user_data()
            .get::<comp_model::capture::OutputGeneration>();
        if topology.is_some_and(|state| {
            *state.1.lock().expect("output signature") != output_signature(output)
        }) {
            return Err("output topology changed after capture admission".into());
        }
        let actual = output
            .user_data()
            .get::<comp_model::capture::OutputGeneration>()
            .map(|generation| generation.0.load(std::sync::atomic::Ordering::Relaxed));
        if actual != Some(expected) {
            return Err("output changed after capture admission".into());
        }
    }
    let size = readback.size;
    let (width, height, cropped) = crop_region(
        spec,
        size.w as u32,
        size.h as u32,
        pixels,
        output.current_scale().fractional_scale(),
    )?;
    let answer = if raw {
        PIXEL_ANSWERS.with_borrow_mut(|answers| answers.remove(&spec.path))
    } else {
        None
    };
    if let Some(answer) = answer {
        answer(Ok((
            width,
            height,
            if spec.region.is_none() {
                pixels.clone()
            } else {
                std::sync::Arc::new(cropped.into_owned())
            },
            readback_us,
        )));
    } else if raw {
        return Err("raw capture was cancelled".into());
    } else {
        write(spec, width, height, &cropped)?;
    }
    Ok(comp_model::capture::reply(
        &spec.path,
        width,
        height,
        output.current_scale().fractional_scale(),
        &output.name(),
        spec.window,
        kind,
    ))
}

pub fn output_signature(output: &Output) -> comp_model::capture::OutputSignature {
    comp_model::capture::OutputSignature {
        mode: output
            .current_mode()
            .map(|m| (m.size.w, m.size.h, m.refresh)),
        scale_bits: output.current_scale().fractional_scale().to_bits(),
        transform: format!("{:?}", output.current_transform()),
    }
}

fn crop_region<'a>(
    spec: &CaptureFrameSpec,
    width: u32,
    height: u32,
    pixels: &'a [u8],
    scale: f64,
) -> Result<(u32, u32, std::borrow::Cow<'a, [u8]>), String> {
    use smithay::utils::{Logical, Rectangle};
    if pixels.len() != width as usize * height as usize * 3 {
        return Err("capture RGB layout mismatch".into());
    }
    let Some(region) = spec.region else {
        return Ok((width, height, pixels.into()));
    };
    if !scale.is_finite() || scale <= 0.0 {
        return Err("capture scale invalid".into());
    }
    let logical = Rectangle::<i32, Logical>::new(
        (region.x, region.y).into(),
        (region.width, region.height).into(),
    );
    let rect = crate::scaled_region(
        logical,
        scale,
        Rectangle::from_size((width as i32, height as i32).into()),
    )
    .ok_or("capture region outside output")?;
    let mut cropped = Vec::with_capacity(rect.size.w as usize * rect.size.h as usize * 3);
    for y in rect.loc.y..rect.loc.y + rect.size.h {
        let start = (y as usize * width as usize + rect.loc.x as usize) * 3;
        cropped.extend_from_slice(&pixels[start..start + rect.size.w as usize * 3]);
    }
    Ok((rect.size.w as u32, rect.size.h as u32, cropped.into()))
}

fn rgb(
    bytes: &[u8],
    width: usize,
    height: usize,
    rgba: bool,
    bottom_up: bool,
) -> Result<Vec<u8>, String> {
    let mut pixels = Vec::new();
    rgb_into(bytes, width, height, rgba, bottom_up, &mut pixels)?;
    Ok(pixels)
}

fn rgb_into(
    bytes: &[u8],
    width: usize,
    height: usize,
    rgba: bool,
    bottom_up: bool,
    pixels: &mut Vec<u8>,
) -> Result<(), String> {
    let row = width.checked_mul(4).ok_or("capture size overflow")?;
    let len = row.checked_mul(height).ok_or("capture size overflow")?;
    if bytes.len() < len {
        return Err("capture readback is shorter than its layout".into());
    }
    if len == 0 {
        pixels.clear();
        return Ok(());
    }
    pixels.resize(len / 4 * 3, 0);
    for (y, out) in pixels.chunks_exact_mut(width * 3).enumerate() {
        let at = source_row(y, height, bottom_up) * row;
        let mut packed = 0;
        #[cfg(target_arch = "x86_64")]
        if std::arch::is_x86_feature_detected!("ssse3") {
            // SAFETY: runtime feature check; input and output are complete
            // rows with respectively four and three bytes per pixel.
            packed = unsafe { pack_rgb_ssse3(&bytes[at..at + row], out, rgba) };
        }
        let at = at + packed * 4;
        let out = &mut out[packed * 3..];
        let width = width - packed;
        // Four pixels expose a fixed byte shuffle to LLVM, rather than a
        // succession of three-byte copies with loop-carried bounds checks.
        let bulk = width / 4;
        for (px, rgb) in bytes[at..at + bulk * 16]
            .chunks_exact(16)
            .zip(out[..bulk * 12].chunks_exact_mut(12))
        {
            if rgba {
                rgb.copy_from_slice(&[
                    px[0], px[1], px[2], px[4], px[5], px[6], px[8], px[9], px[10], px[12], px[13],
                    px[14],
                ]);
            } else {
                rgb.copy_from_slice(&[
                    px[2], px[1], px[0], px[6], px[5], px[4], px[10], px[9], px[8], px[14], px[13],
                    px[12],
                ]);
            }
        }
        let out = &mut out[bulk * 12..];
        let at = at + bulk * 16;
        let row = width % 4 * 4;
        if rgba {
            for (px, rgb) in bytes[at..at + row]
                .chunks_exact(4)
                .zip(out.chunks_exact_mut(3))
            {
                rgb.copy_from_slice(&px[..3]);
            }
        } else {
            for (px, rgb) in bytes[at..at + row]
                .chunks_exact(4)
                .zip(out.chunks_exact_mut(3))
            {
                rgb.copy_from_slice(&[px[2], px[1], px[0]]);
            }
        }
    }
    Ok(())
}

/// Pack sixteen pixels with four loads and three stores, including BGRA
/// channel reversal. Other CPUs retain the portable fixed-size shuffle.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
unsafe fn pack_rgb_ssse3(input: &[u8], output: &mut [u8], rgba: bool) -> usize {
    use std::arch::x86_64::*;
    let mask = if rgba {
        _mm_setr_epi8(0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, -1, -1, -1, -1)
    } else {
        _mm_setr_epi8(2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1)
    };
    let groups = (input.len() / 64).min(output.len() / 48);
    for i in 0..groups {
        // SAFETY: each iteration reads exactly 64 available bytes and writes
        // exactly 48 available bytes. Loads and stores permit unaligned rows.
        unsafe {
            let src = input.as_ptr().add(i * 64);
            let dst = output.as_mut_ptr().add(i * 48);
            let a = _mm_shuffle_epi8(_mm_loadu_si128(src.cast()), mask);
            let b = _mm_shuffle_epi8(_mm_loadu_si128(src.add(16).cast()), mask);
            let c = _mm_shuffle_epi8(_mm_loadu_si128(src.add(32).cast()), mask);
            let d = _mm_shuffle_epi8(_mm_loadu_si128(src.add(48).cast()), mask);
            _mm_storeu_si128(dst.cast(), _mm_or_si128(a, _mm_slli_si128::<12>(b)));
            _mm_storeu_si128(
                dst.add(16).cast(),
                _mm_or_si128(_mm_srli_si128::<4>(b), _mm_slli_si128::<8>(c)),
            );
            _mm_storeu_si128(
                dst.add(32).cast(),
                _mm_or_si128(_mm_srli_si128::<8>(c), _mm_slli_si128::<4>(d)),
            );
        }
    }
    groups * 16
}

fn write(spec: &CaptureFrameSpec, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    match spec.format {
        CaptureFormat::Png => image::save_buffer_with_format(
            &spec.path,
            pixels,
            width,
            height,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )
        .map_err(|err| format!("write PNG: {err}")),
        CaptureFormat::Ppm => {
            let mut file =
                std::fs::File::create(&spec.path).map_err(|err| format!("create PPM: {err}"))?;
            write!(file, "P6\n{width} {height}\n255\n")
                .and_then(|_| file.write_all(pixels))
                .and_then(|_| file.flush())
                .map_err(|err| format!("write PPM: {err}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fence_budget_counts_serviced_time_and_caps_unpolled_gaps() {
        let start = Instant::now();
        let mut budget = PollBudget::new(start);
        let resumed = start + Duration::from_secs(30);
        assert!(!budget.unsignalled(resumed));
        assert_eq!(budget.spent, MAX_POLL_GAP);
        for ms in 1..950 {
            assert!(!budget.unsignalled(resumed + Duration::from_millis(ms)));
        }
        assert!(budget.unsignalled(resumed + Duration::from_millis(950)));
    }

    #[test]
    fn fractional_regions_round_outwards_and_clip() {
        let spec = comp_model::capture::parse(&serde_json::json!({"path":"/tmp/a",
            "region":{"x":1,"y":1,"width":2,"height":1}}))
        .unwrap();
        let pixels: Vec<u8> = (0..5 * 4 * 3).collect();
        let (w, h, p) = crop_region(&spec, 5, 4, &pixels, 1.5).unwrap();
        assert_eq!((w, h), (4, 2));
        assert_eq!(&p[..12], &pixels[18..30]);
        assert_eq!(&p[12..], &pixels[33..45]);
        assert!(crop_region(&spec, 1, 1, &[0, 0, 0], 2.5).is_err());
    }
    #[test]
    fn topology_fence_is_rechecked_before_file_creation() {
        use smithay::output::{PhysicalProperties, Subpixel};
        let output = Output::new(
            "capture-generation-test".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "test".into(),
                model: "test".into(),
                serial_number: "test".into(),
            },
        );
        output.user_data().insert_if_missing(|| {
            comp_model::capture::OutputGeneration(
                std::sync::atomic::AtomicU64::new(8),
                std::sync::Mutex::new(output_signature(&output)),
            )
        });
        let spec=comp_model::capture::parse(&serde_json::json!({"path":"/tmp/never-created-cap-generation.png","output":"capture-generation-test","output_generation":7})).unwrap();
        let result = complete(
            &spec,
            Readback {
                size: (1, 1).into(),
                origin_bottom_left: false,
            },
            &[0, 0, 0],
            &output,
            CaptureSource::Offscreen,
        );
        assert_eq!(
            result.unwrap_err(),
            "output changed after capture admission"
        );
        output
            .user_data()
            .get::<comp_model::capture::OutputGeneration>()
            .unwrap()
            .0
            .store(7, std::sync::atomic::Ordering::Relaxed);
        output.change_current_state(
            None,
            None,
            Some(smithay::output::Scale::Fractional(2.0)),
            None,
        );
        let result = complete(
            &spec,
            Readback {
                size: (1, 1).into(),
                origin_bottom_left: false,
            },
            &[0, 0, 0],
            &output,
            CaptureSource::Offscreen,
        );
        assert_eq!(
            result.unwrap_err(),
            "output topology changed after capture admission"
        );
    }
    #[test]
    fn readback_channel_order_and_orientation() {
        let bgra = [3, 2, 1, 255, 6, 5, 4, 255];
        assert_eq!(rgb(&bgra, 1, 2, false, false).unwrap(), [1, 2, 3, 4, 5, 6]);
        assert_eq!(rgb(&bgra, 1, 2, false, true).unwrap(), [4, 5, 6, 1, 2, 3]);
        assert_eq!(rgb(&bgra, 1, 2, true, false).unwrap(), [3, 2, 1, 6, 5, 4]);
        assert!(rgb(&bgra[..3], 1, 1, false, false).is_err());
        for width in 1..=65 {
            let bytes: Vec<u8> = (0..width * 3 * 4).map(|i| i as u8).collect();
            for rgba in [false, true] {
                for flipped in [false, true] {
                    let expected: Vec<u8> = (0..3)
                        .flat_map(|y| {
                            let at = source_row(y, 3, flipped) * width * 4;
                            bytes[at..at + width * 4].chunks_exact(4).flat_map(|px| {
                                if rgba {
                                    [px[0], px[1], px[2]]
                                } else {
                                    [px[2], px[1], px[0]]
                                }
                            })
                        })
                        .collect();
                    assert_eq!(rgb(&bytes, width, 3, rgba, flipped).unwrap(), expected);
                }
            }
        }
    }

    #[test]
    fn stitched_pixels_do_not_warm_the_source_pool() {
        PIXEL_POOL.with_borrow_mut(Vec::clear);
        prepare_raw_pool(&std::sync::Arc::new(vec![1; 24]));
        assert_eq!(PIXEL_POOL.with_borrow(Vec::len), 0);
    }

    #[test]
    fn pooled_pixels_never_overwrite_a_live_writer_frame() {
        PIXEL_POOL.with_borrow_mut(Vec::clear);
        let first = pooled_rgb(&[3, 2, 1, 255], 1, 1, false, false).unwrap();
        let address = first.as_ptr();
        prepare_raw_pool(&first);
        assert_eq!(PIXEL_POOL.with_borrow(Vec::len), 3);
        let second = pooled_rgb(&[6, 5, 4, 255], 1, 1, false, false).unwrap();
        assert_eq!(&**first, &[1, 2, 3]);
        assert_eq!(&**second, &[4, 5, 6]);
        drop(first);
        let third = pooled_rgb(&[9, 8, 7, 255], 1, 1, false, false).unwrap();
        assert_eq!(third.as_ptr(), address);
        assert_eq!(&**second, &[4, 5, 6]);
        assert_eq!(&**third, &[7, 8, 9]);
        let _fourth = pooled_rgb(&[0; 4], 1, 1, true, false).unwrap();
        let _fifth = pooled_rgb(&[0; 4], 1, 1, true, false).unwrap();
        assert_eq!(PIXEL_POOL.with_borrow(Vec::len), 3);
        PIXEL_POOL.with_borrow_mut(Vec::clear);
    }

    #[test]
    fn shared_pool_slot_is_replaced_and_stop_reclaims_pool() {
        PIXEL_POOL.with_borrow_mut(Vec::clear);
        let held: Vec<_> = (0..3)
            .map(|i| pooled_rgb(&[i, 2, 3, 255], 1, 1, true, false).unwrap())
            .collect();
        let replacement = pooled_rgb(&[9, 8, 7, 255], 1, 1, true, false).unwrap();
        let address = replacement.as_ptr();
        drop(replacement);
        let reused = pooled_rgb(&[6, 5, 4, 255], 1, 1, true, false).unwrap();
        assert_eq!(address, reused.as_ptr());
        assert_eq!(&**held[0], &[0, 2, 3]);
        release_raw("/pool-test-");
        assert_eq!(PIXEL_POOL.with_borrow(Vec::len), 0);
        assert_eq!(&**reused, &[6, 5, 4]);
    }

    #[test]
    fn cancelled_raw_answer_never_becomes_a_file_capture() {
        use smithay::output::{PhysicalProperties, Subpixel};
        let output = Output::new(
            "cancelled".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "test".into(),
                model: "test".into(),
                serial_number: "test".into(),
            },
        );
        let path = std::env::temp_dir().join(format!(
            "cancelled-raw-{}-{}.ppm",
            std::process::id(),
            ledger::frame_trace::monotonic_us()
        ));
        let spec =
            comp_model::capture::parse(&serde_json::json!({"path":path,"format":"ppm"})).unwrap();
        let result = complete_timed(
            &spec,
            Readback {
                size: (1, 1).into(),
                origin_bottom_left: false,
            },
            &std::sync::Arc::new(vec![0; 3]),
            &output,
            CaptureSource::Offscreen,
            0,
            true,
        );
        assert!(result.is_err());
        assert!(
            !path.exists(),
            "a retired raw token must never be opened as a path"
        );
    }

    #[test]
    fn window_service_preserves_output_requests_and_target_refusals() {
        use smithay::output::{PhysicalProperties, Subpixel};
        use std::rc::Rc;
        let output = Output::new(
            "capture-test".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "test".into(),
                model: "test".into(),
                serial_number: "test".into(),
            },
        );
        let answers = Rc::new(RefCell::new(Vec::new()));
        let got = answers.clone();
        let spec = comp_model::capture::parse(&serde_json::json!({
            "path": "/tmp/window.png", "window": {"id": 7, "generation": 3}
        }))
        .unwrap();
        let window_id = enqueue(&output, spec, move |answer| got.borrow_mut().push(answer));
        assert!(pending(&output));
        assert!(pending_windows());
        assert!(!pending_output(&output));
        let got = answers.clone();
        let spec =
            comp_model::capture::parse(&serde_json::json!({"path": "/tmp/output.png"})).unwrap();
        let output_id = enqueue(&output, spec, move |answer| got.borrow_mut().push(answer));
        service_windows(&output, |target, _| {
            assert_eq!(
                target,
                CaptureWindow {
                    id: 7,
                    generation: 3
                }
            );
            Err(ControlReply::refused(
                "stale_target",
                serde_json::json!({
                    "id": 7, "generation": 3, "current": 4
                }),
            ))
        });
        assert!(pending_output(&output));
        assert!(!pending_windows());
        let reply = answers.borrow_mut().pop().unwrap().unwrap_err().into_wire();
        let body: Value = serde_json::from_str(&reply.1).unwrap();
        assert_eq!(reply.0, 10);
        assert_eq!(body["error"], "stale_target");
        assert_eq!(body["current"], 4);
        fail(window_id, "deadline after completion");
        assert!(answers.borrow().is_empty());
        fail(output_id, "output deadline");
        assert_eq!(answers.borrow().len(), 1);
        assert!(!pending(&output));
    }

    #[test]
    fn failed_or_expired_requests_are_answered_once() {
        use smithay::output::{PhysicalProperties, Subpixel};
        use std::rc::Rc;
        let output = Output::new(
            "DP-1".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "test".into(),
                model: "test".into(),
                serial_number: "test".into(),
            },
        );
        let answers = Rc::new(RefCell::new(Vec::new()));
        let got = answers.clone();
        let spec =
            comp_model::capture::parse(&serde_json::json!({"path": "/tmp/frame.png"})).unwrap();
        let id = enqueue(&output, spec, move |answer| got.borrow_mut().push(answer));
        assert!(pending(&output));
        fail(id, "deadline");
        fail_output(&output, "render failed");
        fail(id, "deadline again");
        assert!(!pending(&output));
        let answers = answers.borrow();
        assert_eq!(answers.len(), 1);
        let Err(ControlReply::Refused { error, detail }) = &answers[0] else {
            panic!("expected refusal")
        };
        assert_eq!(*error, "capture_failed");
        assert_eq!(detail["message"], "deadline");
    }
}
