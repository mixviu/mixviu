use crate::timeline::{self, Timeline};
use std::{ffi::OsString, path::PathBuf};
pub fn run(args: impl IntoIterator<Item=OsString>) -> timeline::Result<()> {
    let a: Vec<_> = args.into_iter().collect();
    if a.len()<4 || a[0]!="audio" { return Err("usage: mixd av audio TIMELINE.json --out FILE.webm [--wav FILE.wav]".into()); }
    let input = PathBuf::from(&a[1]);
    let mut out = None;
    let mut wav = None;
    let mut i = 2;
    while i<a.len() {
        let value = a.get(i+1).ok_or("missing option value")?;
        if a[i]=="--out" && out.is_none() {out=Some(PathBuf::from(value));}
        else if a[i]=="--wav" && wav.is_none() {wav=Some(PathBuf::from(value));}
        else {return Err("unknown or duplicate option".into());}
        i+=2;
    }
    let out = out.ok_or("missing --out")?;
    if out==input || wav.as_ref().is_some_and(|w| w==&out || w==&input) {return Err("input and output paths must differ".into());}
    let t: Timeline = serde_json::from_reader(std::fs::File::open(&input)?)?;
    let receipt = timeline::render(&t, input.parent().unwrap_or(std::path::Path::new(".")), &out, wav.as_deref())?;
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}
