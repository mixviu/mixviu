//! Worker/bench-only command; never a service or production transport.
use media_audio::{Audio, Loudness, WavFormat};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};
fn read(p: impl AsRef<Path>) -> Result<Audio, Box<dyn std::error::Error>> {
    Ok(Audio::read_wav(BufReader::new(File::open(p)?))?)
}
fn write(p: impl AsRef<Path>, a: &Audio) -> Result<(), Box<dyn std::error::Error>> {
    Ok(a.write_wav(BufWriter::new(File::create(p)?), WavFormat::Float32)?)
}
fn value(m: Loudness) -> Value {
    json!({"i":m.integrated_lufs,"lra":m.range_lu,"sp":m.sample_peak_dbfs,"tp":m.true_peak_dbtp})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let valid = match args.get(1).map(String::as_str) {
        Some("measure" | "snr") => args.len() == 3,
        Some("target" | "limit" | "normalise" | "resample" | "sweep") => args.len() == 4,
        Some("mix" | "render") => (4..=5).contains(&args.len()),
        _ => false,
    };
    if !valid {
        return Err("usage: audio_probe measure|snr INPUT; target|limit|normalise|resample INPUT OUTPUT; mix|render MANIFEST OUTPUT [FRAMES]; sweep OUTPUT RATE".into());
    }
    match args.get(1).map(String::as_str) {
        Some("target" | "limit") => {
            let mut a = read(&args[2])?;
            if args[1] == "target" {
                let r = a.normalise_to_target(-14.0, -1.0, Default::default())?;
                println!(
                    "{}",
                    json!({"before":value(r.before), "after":value(r.after), "gain":r.gain_db, "passes":r.passes, "max_gr":r.max_gain_reduction_db, "applied_gr":r.applied_gain_reduction_db, "refused":r.refused, "target_reached":r.target_reached, "shortfall_lu":r.shortfall_lu})
                );
            } else {
                let r = a.limit_true_peak(-1.0, Default::default())?;
                println!(
                    "{}",
                    json!({"after":value(r.after), "max_gr":r.max_gain_reduction_db})
                );
            }
            write(&args[3], &a)?;
        }
        Some("measure") => println!("{}", value(read(&args[2])?.measure()?)),
        Some("normalise") => {
            let mut a = read(&args[2])?.resample_48k()?.map_channels(2)?;
            let r = a.normalise(-14.0, -1.0)?;
            write(&args[3], &a)?;
            println!(
                "{}",
                json!({"before":value(r.before),"after":value(r.after),"gain":r.gain_db,"shortfall":r.shortfall_lu,"capped":r.peak_limited})
            );
        }
        Some("resample") => write(&args[3], &read(&args[2])?.resample_48k()?)?,
        Some("mix") | Some("render") => {
            // JSON [{path,offset}] frame offsets at 48k; caller owns timeline policy.
            let clips: Value = serde_json::from_reader(File::open(&args[2])?)?;
            let mut a = Audio::silence(48000, 2, 0)?;
            for c in clips.as_array().ok_or("expected array")? {
                let clip = read(c["path"].as_str().ok_or("path")?)?
                    .resample_48k()?
                    .map_channels(2)?;
                a.place(&clip, c["offset"].as_u64().ok_or("offset")?.try_into()?)?;
            }
            if let Some(n) = args.get(4) {
                a.resize(n.parse()?)?;
            }
            if args[1] == "render" {
                let r = a.normalise(-14.0, -1.0)?;
                println!(
                    "{}",
                    json!({"before":value(r.before),"after":value(r.after),"gain":r.gain_db,"shortfall":r.shortfall_lu,"capped":r.peak_limited})
                );
            }
            write(&args[3], &a)?;
            println!("{}", value(a.measure()?));
        }
        Some("sweep") => {
            // 10-second linear 20 Hz..18 kHz chirp, analytic phase at either rate.
            let rate: u32 = args[3].parse()?;
            let a = Audio::new(
                rate,
                1,
                (0..rate * 10)
                    .map(|n| {
                        let t = n as f64 / rate as f64;
                        0.5 * (std::f64::consts::TAU * (20.0 * t + 0.5 * 1798.0 * t * t)).sin()
                    })
                    .collect(),
            )?;
            write(&args[2], &a)?;
        }
        Some("snr") => {
            let a = read(&args[2])?;
            let mut signal = 0.0;
            let mut noise = 0.0;
            for n in 4800..a.frames().saturating_sub(4800) {
                let t = n as f64 / 48000.0;
                let s = 0.5 * (std::f64::consts::TAU * (20.0 * t + 0.5 * 1798.0 * t * t)).sin();
                signal += s * s;
                noise += (s - a.samples()[n]).powi(2);
            }
            println!(
                "{}",
                json!({"snr_db":10.0_f64*(signal/noise).log10(),"frames":a.frames()})
            );
        }
        _ => return Err("usage: audio_probe measure|normalise|resample|mix|sweep|snr ...".into()),
    }
    Ok(())
}
