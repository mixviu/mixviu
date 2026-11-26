//! Owned, tightly packed I420 output from the pure Rust rav1d decoder.
use crate::Error;
use rav1d::include::dav1d::{
    data::Dav1dData,
    dav1d::{Dav1dContext, Dav1dSettings},
    picture::Dav1dPicture,
};
use rav1d::*;
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
    if code < 0 {
        Err(Error(format!("rav1d error {code}")))
    } else {
        Ok(())
    }
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
        // SAFETY: the pinned Dav1dLogger is two nullable pointer Options;
        // all-zero sets both cookie and callback to None, disabling stderr logs.
        settings.logger = unsafe { std::mem::zeroed() };
        let mut ctx = None;
        // SAFETY: both outputs are valid and settings retains its default allocator.
        check(
            unsafe {
                dav1d_open(
                    Some(NonNull::from(&mut ctx)),
                    Some(NonNull::from(&mut settings)),
                )
            }
            .0,
        )?;
        Ok(Self {
            ctx,
            frames: 0,
            limit: max_frames,
            closed: false,
        })
    }
    pub fn submit(
        &mut self,
        bytes: &[u8],
        timestamp_ns: i64,
        mut emit: impl FnMut(DecodedFrame) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if self.closed || bytes.is_empty() || bytes.len() > 64 * 1024 * 1024 {
            return Err(Error("closed decoder, empty or oversized AV1 input".into()));
        }
        let result = (|| {
            validate_obus(bytes)?;
            let mut data = Data(Dav1dData::default());
            // SAFETY: data is valid writable storage; allocation size is bounded.
            let ptr = unsafe { dav1d_data_create(Some(NonNull::from(&mut data.0)), bytes.len()) };
            if ptr.is_null() {
                return Err(Error("rav1d allocation failed".into()));
            }
            // SAFETY: create returned a writable buffer of exactly bytes.len().
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len()) };
            data.0.m.timestamp = timestamp_ns;
            while data.0.sz != 0 {
                // SAFETY: context and owned input are live for the call.
                let code = unsafe { dav1d_send_data(self.ctx, Some(NonNull::from(&mut data.0))) }.0;
                if code != -libc::EAGAIN {
                    check(code)?;
                }
                let drained = self.drain(&mut emit)?;
                if code == -libc::EAGAIN && drained == 0 {
                    return Err(Error("rav1d made no progress".into()));
                }
            }
            Ok(())
        })();
        if result.is_err() {
            self.closed = true;
        }
        result
    }
    pub fn finish(
        &mut self,
        mut emit: impl FnMut(DecodedFrame) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if self.closed {
            return Err(Error("decoder is closed".into()));
        }
        self.closed = true;
        self.drain(&mut emit).map(|_| ())
    }
    fn drain(
        &mut self,
        emit: &mut impl FnMut(DecodedFrame) -> Result<(), Error>,
    ) -> Result<u64, Error> {
        let start = self.frames;
        loop {
            let mut p = Picture(Dav1dPicture::default());
            // SAFETY: context is live and p is an empty, writable picture.
            let code = unsafe { dav1d_get_picture(self.ctx, Some(NonNull::from(&mut p.0))) }.0;
            if code == -libc::EAGAIN {
                break;
            }
            check(code)?;
            let p = &p.0;
            let (w, h) = (p.p.w as usize, p.p.h as usize);
            if p.p.bpc != 8 || p.p.layout != 1 {
                return Err(Error(
                    "UNSUPPORTED_PIXEL_FORMAT: expected 8-bit I420".into(),
                ));
            }
            if w == 0
                || h == 0
                || w > 8192
                || h > 4320
                || p.p.bpc != 8
                || p.p.layout != 1
                || self.frames >= self.limit
            {
                return Err(Error("unsupported picture or frame limit exceeded".into()));
            }
            let mut planes = [Vec::new(), Vec::new(), Vec::new()];
            for (i, plane) in planes.iter_mut().enumerate() {
                let (width, height) = if i == 0 {
                    (w, h)
                } else {
                    (w.div_ceil(2), h.div_ceil(2))
                };
                let stride = p.stride[usize::from(i != 0)];
                let ptr = p.data[i].ok_or_else(|| Error("missing plane".into()))?;
                if stride < width as isize {
                    return Err(Error("invalid picture stride".into()));
                }
                plane.reserve_exact(width * height);
                for row in 0..height {
                    // SAFETY: rav1d's default allocator owns positive-stride planes
                    // with p.p dimensions. Picture stays referenced while copying.
                    let data = unsafe {
                        std::slice::from_raw_parts(
                            ptr.as_ptr().cast::<u8>().offset(row as isize * stride),
                            width,
                        )
                    };
                    plane.extend_from_slice(data);
                }
            }
            self.frames += 1;
            emit(DecodedFrame {
                width: w,
                height: h,
                timestamp_ns: p.m.timestamp,
                planes,
            })?;
        }
        Ok(self.frames - start)
    }
}
// Inspect every sequence header before rav1d can allocate reference pictures.
// Low-overhead OBU streams need explicit sizes, as in WebM temporal units.
fn validate_obus(bytes: &[u8]) -> Result<(), Error> {
    let mut at = 0usize;
    while at < bytes.len() {
        let start = at;
        let header = bytes[at];
        at += 1;
        if header & 0x83 != 2 {
            return Err(Error("invalid or unsized OBU".into()));
        }
        if header & 4 != 0 {
            at = at
                .checked_add(1)
                .ok_or_else(|| Error("OBU overflow".into()))?;
        }
        let mut size = 0u64;
        let mut done = false;
        for shift in (0..56).step_by(7) {
            let byte = *bytes
                .get(at)
                .ok_or_else(|| Error("truncated OBU size".into()))?;
            at += 1;
            size |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                done = true;
                break;
            }
        }
        let end = at
            .checked_add(usize::try_from(size).map_err(|_| Error("OBU size".into()))?)
            .filter(|end| *end <= bytes.len());
        if !done || end.is_none() {
            return Err(Error("truncated OBU".into()));
        }
        at = end.ok_or_else(|| Error("OBU size".into()))?;
        if (header >> 3) & 15 == 1 {
            let mut seq = std::mem::MaybeUninit::<
                rav1d::include::dav1d::headers::Dav1dSequenceHeader,
            >::uninit();
            // SAFETY: parser reads this bounded OBU and initialises seq on success.
            let code = unsafe {
                dav1d_parse_sequence_header(
                    Some(NonNull::from(&mut seq).cast()),
                    NonNull::new(bytes[start..at].as_ptr().cast_mut()),
                    at - start,
                )
            }
            .0;
            check(code)?;
            // SAFETY: successful parse initialised seq.
            let seq = unsafe { seq.assume_init() };
            if seq.max_width <= 0
                || seq.max_height <= 0
                || seq.max_width > 8192
                || seq.max_height > 4320
                || seq.hbd != 0
                || seq.layout != 1
            {
                return Err(Error(
                    "UNSUPPORTED_AV1_SEQUENCE: exceeds 8192x4320 8-bit I420 limits".into(),
                ));
            }
        }
    }
    Ok(())
}
impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: this uniquely owns the context and closes it exactly once.
        unsafe { dav1d_close(Some(NonNull::from(&mut self.ctx))) }
    }
}
