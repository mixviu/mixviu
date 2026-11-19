//! Headless `mixd vfl` commands. Binary output never shares stdout with diagnostics.
use crate::reader::{Markers, Reader, verify};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
};
pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = args.into_iter().collect();
    let usage = "usage: mixd vfl verify|markers|export TAKE [--from FRAME] [--to FRAME] [--format bgra|y4m] [--fps N]";
    let command = args.first().and_then(|v| v.to_str()).ok_or(usage)?;
    let path = PathBuf::from(args.get(1).ok_or(usage)?);
    let mut out = io::BufWriter::new(io::stdout().lock());
    if command == "verify" && args.len() == 2 {
        let result = verify(&path);
        writeln!(out, "{result}")?;
        out.flush()?;
        if result["ok"] != true {
            return Err("VFL verification failed".into());
        }
        return Ok(());
    }
    if command == "markers" && args.len() == 2 {
        let mut markers = Markers::open(&path)?;
        while let Some(v) = markers.read()? {
            writeln!(out, "{v}")?;
        }
        out.flush()?;
        return Ok(());
    }
    if command != "export" {
        return Err(usage.into());
    }
    let mut from = 0u64;
    let mut to = None;
    let mut format = "bgra";
    let mut fps = None;
    let mut opts = args[2..].chunks_exact(2);
    for pair in &mut opts {
        let value = pair[1].to_str().ok_or(usage)?;
        match pair[0].to_str() {
            Some("--from") => from = value.parse()?,
            Some("--to") => to = Some(value.parse::<u64>()?),
            Some("--fps") => fps = Some(value.parse::<u32>()?),
            Some("--format") => format = value,
            _ => return Err(usage.into()),
        }
    }
    if !opts.remainder().is_empty()
        || !["bgra", "y4m"].contains(&format)
        || to.is_some_and(|n| n < from)
        || fps.is_some_and(|n| !(1..=60).contains(&n))
    {
        return Err(usage.into());
    }
    let mut r = Reader::open(&path)?;
    let (w, h) = (r.width as usize, r.height as usize);
    let rate = fps.unwrap_or(r.fps);
    if !r.complete {
        eprintln!(
            "VFL: interrupted take ({:?}); exporting valid prefix",
            r.completion_file
        );
    }
    if format == "y4m" {
        writeln!(
            out,
            "YUV4MPEG2 W{w} H{h} F{rate}:1 Ip A1:1 C420jpeg XCOLORRANGE=LIMITED"
        )?;
    }
    // --to is inclusive; --fps declares the output rate and never resamples slots.
    let mut next = 0u64;
    while to.is_none_or(|n| next <= n) {
        let Some(frame) = r.read()? else { break };
        next = frame.slot + 1;
        if frame.slot < from {
            continue;
        }
        if to.is_some_and(|n| frame.slot > n) {
            break;
        }
        if format == "bgra" {
            out.write_all(frame.pixels)?;
        } else {
            let yuv = media_frame::Bgra::new(w, h, frame.pixels.to_vec())?.to_yuv420();
            out.write_all(b"FRAME\n")?;
            out.write_all(&yuv.y)?;
            out.write_all(&yuv.u)?;
            out.write_all(&yuv.v)?;
        }
    }
    out.flush()?;
    Ok(())
}
