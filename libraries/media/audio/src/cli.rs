use crate::timeline::{self, Timeline};
use std::{
    ffi::OsString,
    io::{self, Read, Write},
    path::PathBuf,
};
const MAX_JSON: u64 = 16 * 1024 * 1024;
/// Diagnostics must never panic, including when the stderr reader has left.
pub fn diagnostic(message: std::fmt::Arguments<'_>) -> io::Result<()> {
    let mut out = io::stderr().lock();
    match writeln!(out, "av: {message}").and_then(|()| out.flush()) {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}
pub fn run(args: impl IntoIterator<Item = OsString>) -> timeline::Result<u8> {
    let a: Vec<_> = args.into_iter().collect();
    if a.len() < 3 {
        return Err("usage: av audio TIMELINE.json --out FILE.webm [--wav FILE.wav]".into());
    }
    let input = PathBuf::from(&a[0]);
    let mut out = None;
    let mut wav = None;
    let mut i = 1;
    while i < a.len() {
        let value = a.get(i + 1).ok_or("missing option value")?;
        if a[i] == "--out" && out.is_none() {
            out = Some(PathBuf::from(value));
        } else if a[i] == "--wav" && wav.is_none() {
            wav = Some(PathBuf::from(value));
        } else {
            return Err("unknown or duplicate option".into());
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
    let receipt = timeline::render(&t, base, &out, wav.as_deref())?;
    let outcome = if receipt.refused || !receipt.target_reached {
        3
    } else {
        0
    };
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
