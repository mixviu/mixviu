use crate::timeline::{self, Timeline};
use std::{
    ffi::OsString,
    io::{self, Read, Write},
    path::PathBuf,
};
const MAX_JSON: u64 = 16 * 1024 * 1024;
/// Shared byte ceiling syntax; audio validates it but has no memory admission.
pub fn memory_limit(value: &std::ffi::OsStr) -> timeline::Result<u64> {
    let value = value.to_str().ok_or("mem_limit")?;
    let (digits, multiplier) = match value.as_bytes().last() {
        Some(b'K') => (&value[..value.len() - 1], 1024u64),
        Some(b'M') => (&value[..value.len() - 1], 1024u64.pow(2)),
        Some(b'G') => (&value[..value.len() - 1], 1024u64.pow(3)),
        _ => (value, 1),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err("mem_limit".into());
    }
    digits.parse::<u64>().ok().and_then(|n| n.checked_mul(multiplier))
        .ok_or_else(|| "mem_limit".into())
}
/// Audio execution is single-threaded; validate this no-op ceiling consistently.
pub fn cpu_limit(value: &std::ffi::OsStr) -> timeline::Result<usize> {
    let value = value.to_str().ok_or("cpu_limit")?;
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("cpu_limit".into());
    }
    let n: usize = value.parse().map_err(|_| "cpu_limit")?;
    if n == 0 || n > std::thread::available_parallelism().map_or(1, |n| n.get()) {
        return Err("cpu_limit".into());
    }
    Ok(n)
}
/// Diagnostics must never panic, including when the stderr reader has left.
pub fn diagnostic(message: std::fmt::Arguments<'_>) -> io::Result<()> {
    let mut out = io::stderr().lock();
    match writeln!(out, "av: {message}").and_then(|()| out.flush()) {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}
pub fn run(args: impl IntoIterator<Item = OsString>) -> timeline::Result<u8> {
    let (a, progress) = crate::progress::arguments(args);
    let result = run_inner(a, &progress);
    progress.done(&result);
    result
}
fn run_inner(a: Vec<OsString>, progress: &crate::progress::Progress) -> timeline::Result<u8> {
    progress.event("input", "start", serde_json::json!({}));
    if a.len() < 3 {
        return Err("usage: av audio TIMELINE.json --out FILE.webm [--wav FILE.wav] [--cpu N] [--mem BYTES] [--progress]".into());
    }
    let input = PathBuf::from(&a[0]);
    let mut out = None;
    let mut wav = None;
    let mut cpu = None;
    let mut mem = None;
    let mut i = 1;
    while i < a.len() {
        let value = a.get(i + 1).ok_or("missing_option_value")?;
        if a[i] == "--out" && out.is_none() {
            out = Some(PathBuf::from(value));
        } else if a[i] == "--wav" && wav.is_none() {
            wav = Some(PathBuf::from(value));
        } else if a[i] == "--cpu" && cpu.is_none() {
            cpu = Some(cpu_limit(value)?);
        } else if a[i] == "--mem" && mem.is_none() {
            mem = Some(memory_limit(value)?);
        } else {
            return Err("unknown_or_duplicate_option".into());
        }
        i += 2;
    }
    let out = out.ok_or("missing --out")?;
    if out == input || wav.as_ref().is_some_and(|w| w == &out || w == &input) {
        return Err("input and output paths must differ".into());
    }
    let file = crate::source::regular(&input)?;
    if file.metadata()?.len() > MAX_JSON {
        return Err("timeline JSON exceeds 16 MiB".into());
    }
    let resolved = crate::source::resolved(&file)?;
    let base = resolved
        .parent()
        .ok_or("opened timeline has no parent directory")?;
    let mut bytes = Vec::new();
    // The sentinel catches growth after the metadata check. Reserve fallibly
    // before read_to_end so the bounded read cannot trigger an allocation.
    bytes
        .try_reserve_exact(MAX_JSON as usize + 1)
        .map_err(|_| crate::Error::SizeOverflow)?;
    file.take(MAX_JSON + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_JSON {
        return Err("timeline JSON exceeds 16 MiB".into());
    }
    let t: Timeline = serde_json::from_slice(&bytes)?;
    progress.event("input", "end", serde_json::json!({}));
    progress.event("audio", "start", serde_json::json!({"audio_pass":0}));
    let prepared = timeline::prepare(&t, base)?;
    progress.event(
        "audio",
        "end",
        serde_json::json!({"audio_pass":prepared.receipt.codec_passes}),
    );
    progress.event("mux", "start", serde_json::json!({}));
    let receipt = timeline::render_prepared(prepared, &out, wav.as_deref())?;
    let outcome = if receipt.refused || !receipt.target_reached {
        3
    } else {
        0
    };
    progress.event(
        "mux",
        "end",
        serde_json::json!({"bytes":if outcome == 0 { std::fs::metadata(&out).map_or(0, |m| m.len()) } else { 0 }}),
    );
    let mut stdout = io::stdout().lock();
    if let Err(e) =
        writeln!(stdout, "{}", serde_json::to_string(&receipt)?).and_then(|()| stdout.flush())
        && outcome == 0
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        let _ = diagnostic(format_args!("published but receipt not written: {e}"));
        return Ok(4);
    }
    if receipt.refused {
        let _ = diagnostic(format_args!(
            "mastering refused: encoded {:.3} LUFS, {:.3} dBTP; target {:.3} LUFS, ceiling {:.3} dBTP; maximum gain reduction {:.3} dB; shortfall {:.3} LU",
            receipt.encoded.integrated_lufs,
            receipt.encoded.true_peak_dbtp,
            t.master.integrated_lufs,
            t.master.true_peak_dbtp,
            receipt.max_gain_reduction_db,
            receipt.shortfall_lu
        ));
    }
    Ok(outcome)
}
