//! Headless `mixd vfl` commands. Binary output never shares stdout with diagnostics.
use crate::reader::{Markers, Reader, verify_with_options};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
};
#[derive(Debug)]
pub struct Usage;
impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(USAGE) }
}
impl std::error::Error for Usage {}
const USAGE: &str = "usage: mixd vfl verify TAKE [--require-complete] | markers TAKE | export TAKE [--from FRAME] [--to FRAME] [--format bgra|y4m] [--fps N (1-60)]\nExit codes: 0 success or streaming stdout BrokenPipe; 1 data or I/O error; 2 usage error.";
pub fn exit_code(error: &(dyn std::error::Error + 'static)) -> u8 {
    if error.is::<Usage>() { 2 } else { 1 }
}
pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = args.into_iter().collect();
    let usage = || Usage;
    let command = args.first().and_then(|v| v.to_str()).ok_or_else(usage)?;
    let require_complete = command == "verify" && args.iter().any(|a| a == "--require-complete");
    let path_arg = if command == "verify" && args.get(1).is_some_and(|a| a == "--require-complete") { 2 } else { 1 };
    let path = PathBuf::from(args.get(path_arg).ok_or_else(usage)?);
    let mut out = io::BufWriter::new(io::stdout().lock());
    if command == "verify" && (args.len() == 2 && !require_complete || args.len() == 3 && require_complete) {
        let result = verify_with_options(&path, require_complete);
        writeln!(out, "{result}")?;
        out.flush()?;
        if result["ok"] != true {
            return Err("VFL verification failed".into());
        }
        return Ok(());
    }
    // Streaming commands stop when their consumer leaves. Discard buffered bytes
    // as well: BufWriter's Drop would otherwise attempt another write.
    macro_rules! stdout_or_end {
        ($write:expr) => {
            if let Err(error) = $write {
                if error.kind() == io::ErrorKind::BrokenPipe {
                    let _ = out.into_parts();
                    return Ok(());
                }
                return Err(error.into());
            }
        };
    }
    if command == "markers" && args.len() == 2 {
        let mut markers = Markers::open(&path)?;
        while let Some(v) = markers.read()? {
            stdout_or_end!(writeln!(out, "{v}"));
        }
        stdout_or_end!(out.flush());
        return Ok(());
    }
    if command != "export" {
        return Err(Usage.into());
    }
    let mut from = 0u64;
    let mut to = None;
    let mut format = "bgra";
    let mut fps = None;
    let mut opts = args[2..].chunks_exact(2);
    for pair in &mut opts {
        let value = pair[1].to_str().ok_or_else(usage)?;
        match pair[0].to_str() {
            Some("--from") => from = value.parse().map_err(|_| Usage)?,
            Some("--to") => to = Some(value.parse::<u64>().map_err(|_| Usage)?),
            Some("--fps") => fps = Some(value.parse::<u32>().map_err(|_| Usage)?),
            Some("--format") => format = value,
            _ => return Err(Usage.into()),
        }
    }
    if !opts.remainder().is_empty()
        || !["bgra", "y4m"].contains(&format)
        || to.is_some_and(|n| n < from)
        || fps.is_some_and(|n| !(1..=60).contains(&n))
    {
        return Err(Usage.into());
    }
    let mut r = Reader::open(&path)?;
    let (w, h) = (r.width() as usize, r.height() as usize);
    let rate = fps.unwrap_or(r.fps());
    if !r.complete() {
        eprintln!(
            "VFL: interrupted take ({:?}); exporting valid prefix",
            r.completion_file()
        );
    }
    if format == "y4m" {
        stdout_or_end!(writeln!(
            out,
            "YUV4MPEG2 W{w} H{h} F{rate}:1 Ip A1:1 C420jpeg XCOLORRANGE=LIMITED"
        ));
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
            stdout_or_end!(out.write_all(frame.pixels));
        } else {
            let yuv = media_frame::Bgra::new(w, h, frame.pixels.to_vec())?.to_yuv420();
            stdout_or_end!(out.write_all(b"FRAME\n"));
            stdout_or_end!(out.write_all(&yuv.y));
            stdout_or_end!(out.write_all(&yuv.u));
            stdout_or_end!(out.write_all(&yuv.v));
        }
    }
    stdout_or_end!(out.flush());
    Ok(())
}
