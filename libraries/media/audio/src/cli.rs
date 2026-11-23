use crate::timeline::{self, Timeline};
use std::{ffi::OsString, path::PathBuf, io::Read};
const MAX_JSON: u64 = 16 * 1024 * 1024;
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
    let file = std::fs::File::open(&input)?;
    if file.metadata()?.len() > MAX_JSON { return Err("timeline JSON exceeds 16 MiB".into()); }
    let mut bytes = Vec::new();
    // The sentinel catches growth after the metadata check. Reserve fallibly
    // before read_to_end so the bounded read cannot trigger an allocation.
    bytes.try_reserve_exact(MAX_JSON as usize + 1).map_err(|_| crate::Error::SizeOverflow)?;
    file.take(MAX_JSON + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_JSON { return Err("timeline JSON exceeds 16 MiB".into()); }
    let t: Timeline = serde_json::from_slice(&bytes)?;
    let receipt = timeline::render(
        &t,
        input.parent().unwrap_or(std::path::Path::new(".")),
        &out,
        wav.as_deref(),
    )?;
    println!("{}", serde_json::to_string(&receipt)?);
    if receipt.refused {
        eprintln!("av: mastering refused: maximum gain reduction {:.3} dB; shortfall {:.3} LU", receipt.max_gain_reduction_db, receipt.shortfall_lu);
        return Ok(3);
    }
    if !receipt.target_reached {
        eprintln!("av: loudness target missed by {:.3} LU", (t.master.integrated_lufs - receipt.encoded.integrated_lufs).abs());
        return Ok(3);
    }
    Ok(0)
}
