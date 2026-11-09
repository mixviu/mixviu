// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Read the framebuffer a CRTC is scanning out RIGHT NOW, whoever put it there
//! (viud, KWin, anything), at its full depth: what goes to the display, not a
//! compositor's own capture. Built for HDR work, where a 10-bit PQ buffer is the
//! evidence and ffmpeg's kmsgrab does not know the 10-bit formats.
//!
//! The buffer is exported as a dmabuf (GETFB2 + PRIME; needs CAP_SYS_ADMIN, so
//! run it as root), imported into GLES on the render node, drawn 1:1 into a
//! 10-bit offscreen target and read back. Output: a 16-bit PPM (maxval 1023,
//! values are the 10-bit codes) plus the codes at any `x,y` points given.
//!
//! usage: testkit-scanout-grab CARD RENDER_NODE OUT.ppm [x,y ...]

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};

use smithay::backend::allocator::dmabuf::{Dmabuf, DmabufFlags};
use smithay::backend::allocator::gbm::GbmDevice;
use smithay::backend::allocator::{Fourcc, Modifier};
use smithay::backend::egl::{EGLContext, EGLDisplay};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::renderer::{
    Bind, Color32F, ExportMem, Frame, ImportDma, Offscreen, Renderer,
};
use smithay::reexports::drm::Device as DrmDevice;
use smithay::reexports::drm::control::Device as ControlDevice;
use smithay::utils::{DeviceFd, Physical, Rectangle, Size, Transform};

struct Card(File);
impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
impl DrmDevice for Card {}
impl ControlDevice for Card {}

fn open(path: &str) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|e| panic!("open {path}: {e}"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: {} CARD RENDER_NODE OUT.ppm [x,y ...]", args[0]);
        std::process::exit(2);
    }
    let card = Card(open(&args[1]));

    // The first CRTC with a framebuffer attached.
    let res = card.resource_handles().expect("resource handles");
    let fb = res
        .crtcs()
        .iter()
        .find_map(|c| card.get_crtc(*c).ok().and_then(|i| i.framebuffer()))
        .expect("no CRTC is scanning out a framebuffer");
    let info = card
        .get_planar_framebuffer(fb)
        .expect("GETFB2 (run as root: buffer handles need CAP_SYS_ADMIN)");
    let (w, h) = info.size();
    let format = info.pixel_format();
    let modifier = info.modifier().unwrap_or(Modifier::Invalid);
    eprintln!("framebuffer {w}x{h} {format:?} modifier {modifier:?}");

    let mut builder = Dmabuf::builder((w as i32, h as i32), format, modifier, DmabufFlags::empty());
    let (pitches, offsets) = (info.pitches(), info.offsets());
    for (i, handle) in info.buffers().iter().enumerate() {
        let Some(handle) = handle else { continue };
        let fd: OwnedFd = card
            .buffer_to_prime_fd(*handle, libc::O_CLOEXEC as u32)
            .expect("PRIME export");
        builder.add_plane(fd, offsets[i], pitches[i]);
    }
    let dmabuf = builder.build().expect("dmabuf");

    let gbm = GbmDevice::new(DeviceFd::from(OwnedFd::from(open(&args[2])))).expect("gbm");
    let display = unsafe { EGLDisplay::new(gbm) }.expect("EGL display");
    let context = EGLContext::new(&display).expect("EGL context");
    let mut gles = unsafe { GlesRenderer::new(context) }.expect("GLES renderer");

    let src = gles
        .import_dmabuf(&dmabuf, None)
        .expect("import scanout dmabuf");
    let size: Size<i32, Physical> = (w as i32, h as i32).into();
    let full = Rectangle::from_size(size);
    let mut target = Offscreen::<GlesTexture>::create_buffer(
        &mut gles,
        Fourcc::Abgr2101010,
        (w as i32, h as i32).into(),
    )
    .expect("10-bit offscreen target");
    {
        let mut fbuf = gles.bind(&mut target).expect("bind target");
        let mut frame = gles
            .render(&mut fbuf, size, Transform::Normal)
            .expect("frame");
        frame
            .clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[full])
            .expect("clear");
        Frame::render_texture_from_to(
            &mut frame,
            &src,
            Rectangle::from_size((w as f64, h as f64).into()),
            full,
            &[full],
            &[],
            Transform::Normal,
            1.0,
        )
        .expect("copy");
        frame.finish().expect("finish").wait().expect("GPU wait");
    }
    let fbuf = gles.bind(&mut target).expect("rebind");
    let mapping = gles
        .copy_framebuffer(
            &fbuf,
            Rectangle::from_size((w as i32, h as i32).into()),
            Fourcc::Abgr2101010,
        )
        .expect("readback");
    drop(fbuf);
    let bytes = gles.map_texture(&mapping).expect("map").to_vec();

    // ABGR2101010 little-endian word: R bits 0-9, G 10-19, B 20-29.
    let code = |x: u32, y: u32| -> [u16; 3] {
        let i = ((y * w + x) * 4) as usize;
        let v = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        [
            (v & 0x3ff) as u16,
            ((v >> 10) & 0x3ff) as u16,
            ((v >> 20) & 0x3ff) as u16,
        ]
    };

    // Run as root: create a NEW file only, so it can neither overwrite an
    // existing file nor write through a planted symlink.
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])
        .unwrap_or_else(|e| panic!("create {} (must not exist): {e}", args[3]));
    write!(out, "P6\n{w} {h}\n1023\n").unwrap();
    let mut row = Vec::with_capacity((w * 6) as usize);
    for y in 0..h {
        row.clear();
        for x in 0..w {
            for c in code(x, y) {
                row.extend_from_slice(&c.to_be_bytes());
            }
        }
        out.write_all(&row).unwrap();
    }
    for p in &args[4..] {
        let mut it = p.split(',').map(|v| v.parse::<u32>().expect("x,y"));
        let (x, y) = (it.next().unwrap(), it.next().unwrap());
        println!("{x},{y} {:?}", code(x.min(w - 1), y.min(h - 1)));
    }
}
