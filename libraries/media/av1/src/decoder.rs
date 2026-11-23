//! Owned, tightly packed I420 output from the pure Rust rav1d decoder.
use crate::Error;
use rav1d::include::dav1d::{data::Dav1dData, dav1d::{Dav1dContext, Dav1dSettings}, picture::Dav1dPicture};
use rav1d::src::lib::*;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct DecodedFrame {
    pub width: usize,
    pub height: usize,
    /// Caller supplied nanoseconds, preserved through frame reordering/repeats.
    pub timestamp_ns: i64,
    pub planes: [Vec<u8>; 3],
}
/// Stateful bounded decoder. Stream output through the callback: a single input
/// may contain many temporal units without retaining all their decoded pixels.
pub struct Decoder {
    ctx: Option<Dav1dContext>,
    frames: u64,
    limit: u64,
    closed: bool,
}
fn check(code: i32) -> Result<(), Error> {
    if code < 0 { Err(Error(format!("rav1d error {code}"))) } else { Ok(()) }
}
struct Data(Dav1dData);
impl Drop for Data {
    fn drop(&mut self) {
        // SAFETY: this owns a data reference created by dav1d_data_create.
        unsafe { dav1d_data_unref(Some(NonNull::from(&mut self.0))) }
    }
}
struct Picture(Dav1dPicture);
impl Drop for Picture {
    fn drop(&mut self) {
        // SAFETY: the picture reference was returned by get_picture, or is empty.
        unsafe { dav1d_picture_unref(Some(NonNull::from(&mut self.0))) }
    }
}
impl Decoder {
    pub fn new(threads: u8, max_frames: u64) -> Result<Self, Error> {
        if threads == 0 || threads > 64 || max_frames == 0 {
            return Err(Error("invalid decoder threads or frame limit".into()));
        }
        let mut settings = std::mem::MaybeUninit::<Dav1dSettings>::uninit();
        // SAFETY: default_settings initialises the complete settings object.
        let mut settings = unsafe {
            dav1d_default_settings(NonNull::from(&mut settings).cast());
            settings.assume_init()
        };
        settings.n_threads = threads.into();
        settings.max_frame_delay = 1;
        settings.frame_size_limit = 8192 * 4320;
        settings.strict_std_compliance = 1;
        let mut ctx = None;
        // SAFETY: both outputs are valid and settings retains its default allocator.
        check(unsafe { dav1d_open(Some(NonNull::from(&mut ctx)), Some(NonNull::from(&mut settings))) }.0)?;
        Ok(Self { ctx, frames: 0, limit: max_frames, closed: false })
    }
    pub fn submit(&mut self, bytes: &[u8], timestamp_ns: i64, mut emit: impl FnMut(DecodedFrame) -> Result<(), Error>) -> Result<(), Error> {
        if self.closed || bytes.is_empty() || bytes.len() > 64 * 1024 * 1024 {
            return Err(Error("closed decoder, empty or oversized AV1 input".into()));
        }
        let result = (|| {
            let mut data = Data(Dav1dData::default());
            // SAFETY: data is valid writable storage; allocation size is bounded.
            let ptr = unsafe { dav1d_data_create(Some(NonNull::from(&mut data.0)), bytes.len()) };
            if ptr.is_null() { return Err(Error("rav1d allocation failed".into())); }
            // SAFETY: create returned a writable buffer of exactly bytes.len().
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()) };
            data.0.m.timestamp = timestamp_ns;
            while data.0.sz != 0 {
                // SAFETY: context and owned input are live for the call.
                let code = unsafe { dav1d_send_data(self.ctx, Some(NonNull::from(&mut data.0))) }.0;
                if code != -libc::EAGAIN { check(code)?; }
                let drained = self.drain(&mut emit)?;
                if code == -libc::EAGAIN && drained == 0 { return Err(Error("rav1d made no progress".into())); }
            }
            Ok(())
        })();
        if result.is_err() { self.closed = true; }
        result
    }
    pub fn finish(&mut self, mut emit: impl FnMut(DecodedFrame) -> Result<(), Error>) -> Result<(), Error> {
        if self.closed { return Err(Error("decoder is closed".into())); }
        self.closed = true;
        self.drain(&mut emit).map(|_| ())
    }
    fn drain(&mut self, emit: &mut impl FnMut(DecodedFrame) -> Result<(), Error>) -> Result<u64, Error> {
        let start = self.frames;
        loop {
            let mut p = Picture(Dav1dPicture::default());
            // SAFETY: context is live and p is an empty, writable picture.
            let code = unsafe { dav1d_get_picture(self.ctx, Some(NonNull::from(&mut p.0))) }.0;
            if code == -libc::EAGAIN { break; }
            check(code)?;
            let p = &p.0;
            let (w, h) = (p.p.w as usize, p.p.h as usize);
            if w == 0 || h == 0 || w > 8192 || h > 4320 || p.p.bpc != 8 || p.p.layout != 1 || self.frames >= self.limit {
                return Err(Error("unsupported picture or frame limit exceeded".into()));
            }
            let mut planes = [Vec::new(), Vec::new(), Vec::new()];
            for (i, plane) in planes.iter_mut().enumerate() {
                let (width, height) = if i == 0 { (w, h) } else { (w.div_ceil(2), h.div_ceil(2)) };
                let stride = p.stride[usize::from(i != 0)];
                let ptr = p.data[i].ok_or_else(|| Error("missing plane".into()))?;
                if stride < width as isize { return Err(Error("invalid picture stride".into())); }
                plane.reserve_exact(width * height);
                for row in 0..height {
                    // SAFETY: rav1d's default allocator owns positive-stride planes
                    // with p.p dimensions. Picture stays referenced while copying.
                    let data = unsafe { std::slice::from_raw_parts(ptr.as_ptr().cast::<u8>().offset(row as isize * stride), width) };
                    plane.extend_from_slice(data);
                }
            }
            self.frames += 1;
            emit(DecodedFrame { width: w, height: h, timestamp_ns: p.m.timestamp, planes })?;
        }
        Ok(self.frames - start)
    }
}
impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: this uniquely owns the context and closes it exactly once.
        unsafe { dav1d_close(Some(NonNull::from(&mut self.ctx))) }
    }
}
