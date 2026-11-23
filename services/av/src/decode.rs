use media_opus::{Channels, Decoder as _, OpusDecoder};
use media_webm::Demuxer;
use serde_json::json;
use std::{ffi::OsString, fs::File, io::{Read, Write, BufWriter}};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(probe: bool, mut args: impl Iterator<Item=OsString>) -> Result<()> {
    let input = args.next().ok_or("expected WebM input")?;
    let mut y4m = None;
    let mut wav = None;
    while let Some(arg) = args.next() {
        if probe { return Err("probe takes only one input".into()); }
        if arg == "--y4m" && y4m.is_none() { y4m = Some(args.next().ok_or("missing Y4M path")?); }
        else if arg == "--wav" && wav.is_none() { wav = Some(args.next().ok_or("missing WAV path")?); }
        else { return Err("usage: av decode FILE.webm --y4m OUT.y4m [--wav OUT.wav]".into()); }
    }
    let mut bytes = Vec::new();
    File::open(&input)?.take(256 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let d = Demuxer::new(&bytes)?;
    if probe {
        let tracks: Vec<_> = d.tracks.iter().map(|t| json!({
            "number": t.number, "codec":t.codec, "width":t.width, "height":t.height,
            "fps": if t.default_duration_ns > 0 { Some(1e9 / t.default_duration_ns as f64) } else { None },
            "sample_rate": t.sample_rate, "channels":t.channels,
            "packets": d.packets().iter().filter(|p| p.track == t.number).count()
        })).collect();
        let mut frames = 0;
        if let Some(t) = d.tracks.iter().find(|t| t.codec == "V_AV1") {
            let mut decoder = media_av1::Decoder::new(1, 500_000)?;
            for p in d.packets().iter().filter(|p| p.track == t.number) {
                decoder.submit(p.data, 0, |_| { frames += 1; Ok(()) })?;
            }
            decoder.finish(|_| { frames += 1; Ok(()) })?;
        }
        println!("{}", json!({"duration":d.duration.map(|v| v * d.scale_ns as f64 / 1e9), "tracks":tracks,
            "frames":frames, "packets":d.packets().len(),
            "keyframes":d.packets().iter().filter(|p| p.key_frame && d.tracks.iter().any(|t| t.number == p.track && t.kind == 1)).count(), "cues":d.cues.len()}));
        return Ok(());
    }
    let out = y4m.ok_or("--y4m is required")?;
    if out == input || wav.as_ref().is_some_and(|w| w == &input || w == &out) { return Err("input and output paths must differ".into()); }
    let t = d.tracks.iter().find(|t| t.codec == "V_AV1").ok_or("no AV1 track")?;
    let mut video = BufWriter::new(File::create(out)?);
    let mut decoder = media_av1::Decoder::new(4, 500_000)?;
    let mut size = None;
    let mut emit = |f: media_av1::DecodedFrame| {
        let write = (|| -> std::io::Result<()> {
            if size.is_none() {
                let duration = t.default_duration_ns;
                // VFR timing is preserved in per-frame XPTS tags (nanoseconds).
                let rate = if duration > 0 { format!("1000000000:{duration}") } else { "0:0".into() };
                writeln!(video, "YUV4MPEG2 W{} H{} F{} Ip A1:1 C420jpeg", f.width, f.height, rate)?;
                size = Some((f.width, f.height));
            }
            if size != Some((f.width, f.height)) { return Err(std::io::Error::other("Y4M size changed")); }
            writeln!(video, "FRAME XPTS={}", f.timestamp_ns)?;
            for plane in &f.planes { video.write_all(plane)?; }
            Ok(())
        })();
        write.map_err(|e| media_av1::Error(e.to_string()))
    };
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        let ns = (p.timestamp as i128 * d.scale_ns as i128).try_into()?;
        decoder.submit(p.data, ns, &mut emit)?;
    }
    decoder.finish(&mut emit)?;
    video.flush()?;
    if let Some(path) = wav { audio(&d, path)?; }
    Ok(())
}

fn audio(d: &Demuxer<'_>, path: OsString) -> Result<()> {
    let t = d.tracks.iter().find(|t| t.codec == "A_OPUS").ok_or("no Opus track")?;
    let h = t.codec_private;
    if h.len() != 19 || &h[..8] != b"OpusHead" || h[8] > 15 || h[18] != 0 {
        return Err("unsupported OpusHead".into());
    }
    let channels = match h[9] { 1 => Channels::Mono, 2 => Channels::Stereo, _ => return Err("unsupported Opus channels".into()) };
    let n = channels.count();
    let mut skip = usize::from(u16::from_le_bytes([h[10], h[11]]));
    let gain = 10f32.powf(i16::from_le_bytes([h[16], h[17]]) as f32 / (256.0 * 20.0));
    let mut decoder = OpusDecoder::new(channels)?;
    let mut writer = hound::WavWriter::create(path, hound::WavSpec { channels:n as u16, sample_rate:48_000, bits_per_sample:32, sample_format:hound::SampleFormat::Float })?;
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        let pcm = decoder.decode(Some(p.data))?.pcm;
        let padding = ((p.discard_padding_ns.unsigned_abs() as u128 * 48_000 + 500_000_000) / 1_000_000_000) as usize;
        let len = pcm.len() / n;
        if padding > len { return Err("Opus discard padding exceeds packet".into()); }
        let start_pad = if p.discard_padding_ns < 0 { padding } else { 0 };
        let end_pad = if p.discard_padding_ns > 0 { padding } else { 0 };
        let available = len - start_pad - end_pad;
        let skipped = skip.min(available);
        skip -= skipped;
        for sample in &pcm[(start_pad + skipped)*n..(len-end_pad)*n] { writer.write_sample(*sample * gain)?; }
    }
    writer.finalize()?;
    Ok(())
}
