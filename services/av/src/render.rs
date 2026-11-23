use media_av1::{Encoder, Settings, Target, reference::ReferenceTracker};
use media_frame::Bgra;
use serde::Deserialize;
use std::{ffi::OsString, path::{Path, PathBuf}, io::{BufWriter, Read}, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Timeline { version: u32, duration_s: f64, video: Video, audio: Audio, encode: Encode }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Audio { sample_rate: u32, channels: u16, sources: Vec<media_audio::timeline::Source>, master: media_audio::timeline::Master }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Video { width: usize, height: usize, fps: u32, segments: Vec<Segment>, #[serde(default)] crossfades: Vec<Fade> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Segment { path: PathBuf, from_slot: u64, to_slot: u64, at_s: f64 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fade { at_s: f64, duration_s: f64 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Encode { speed: u8, qp: u8, tile_cols: usize, tile_rows: usize, threads: usize, #[serde(default = "parallelism")] chunks: usize, keyint_s: f64 }
fn parallelism() -> usize { std::thread::available_parallelism().map_or(1, |n| (n.get() / 8).max(1)) }
fn frame(s: f64) -> u64 { (s * 30.0).round() as u64 }
fn time(s: f64) -> bool { s.is_finite() && (0.0..=10800.0).contains(&s) }
fn refuse<T>(name: &str) -> Result<T> { Err(format!("render: {name}").into()) }
impl Timeline {
    fn validate(&self, base: &Path) -> Result<Vec<u64>> {
        let v = &self.video;
        if self.version != 1 { return refuse("version"); }
        if !time(self.duration_s) || frame(self.duration_s) == 0 { return refuse("duration_s"); }
        if v.fps != 30 { return refuse("fps: only 30 supported"); }
        if !(16..=7680).contains(&v.width) || !(16..=4320).contains(&v.height) { return refuse("dimensions"); }
        if v.segments.is_empty() || v.segments.len() > 10000 || v.crossfades.len() > 10000 { return refuse("segments_limit"); }
        let e = &self.encode;
        if e.speed > 10 || e.chunks == 0 || e.chunks > 64 || !time(e.keyint_s) || frame(e.keyint_s) == 0 { return refuse("encode_settings"); }
        let mut starts = vec![0]; let mut previous = None;
        for s in &v.segments {
            if !time(s.at_s) || s.from_slot >= s.to_slot || frame(s.at_s) >= frame(self.duration_s) { return refuse("segment_range"); }
            if s.path.as_os_str().is_empty() || s.path.components().any(|c| !matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir)) { return refuse("segment_path_relative"); }
            let r = media_vfl::reader::Reader::open(base.join(&s.path))?;
            if r.fps() != 30 || !r.complete() { return refuse("segment_complete_30fps_required"); }
            let start = frame(s.at_s);
            if previous.is_none() && start != 0 { return refuse("gap_before_first_segment"); }
            if previous.is_some_and(|p| start <= p) { return refuse("segment_order"); }
            previous = Some(start); starts.push(start);
        }
        for (i, f) in v.crossfades.iter().enumerate() {
            if !time(f.at_s) || !time(f.duration_s) || frame(f.duration_s) == 0 || frame(f.at_s + f.duration_s) > frame(self.duration_s) { return refuse("crossfade_range"); }
            let Some(j) = v.segments.iter().position(|s| frame(s.at_s) == frame(f.at_s)) else { return refuse("crossfade_segment_start"); };
            if j == 0 || frame(v.segments[j-1].at_s) + v.segments[j-1].to_slot - v.segments[j-1].from_slot < frame(f.at_s + f.duration_s) { return refuse("crossfade_requires_overlap"); }
            if i > 0 && frame(v.crossfades[i-1].at_s + v.crossfades[i-1].duration_s) > frame(f.at_s) { return refuse("crossfade_order"); }
        }
        let n = frame(self.duration_s); let step = frame(e.keyint_s);
        starts.extend((0..n).step_by(step as usize)); starts.push(n); starts.sort_unstable(); starts.dedup();
        Ok(starts)
    }
}
struct Source { reader: media_vfl::reader::Reader, next: u64, current: Option<Bgra> }
impl Source {
    fn new(base: &Path, s: &Segment) -> Result<Self> { Ok(Self { reader: media_vfl::reader::Reader::open(base.join(&s.path))?, next: u64::MAX, current: None }) }
    fn get(&mut self, slot: u64, width: usize, height: usize) -> Result<(Bgra, bool)> {
        if self.next == slot + 1 { return Ok((self.current.as_ref().ok_or("missing_frame")?.clone(), true)); }
        let w = self.reader.width() as usize; let h = self.reader.height() as usize;
        let f = if self.next == slot { self.reader.read()?.ok_or("segment_trim_outside_vfl")? } else { self.reader.seek(slot)? };
        let repeat = f.repeat && self.current.is_some();
        if !repeat {
            let b = Bgra::new(w, h, f.pixels.to_vec())?;
            let ratio = (width as f64 / w as f64).min(height as f64 / h as f64);
            let sw = ((w as f64 * ratio).round() as usize).clamp(1, width);
            let sh = ((h as f64 * ratio).round() as usize).clamp(1, height);
            self.current = Some(if w == width && h == height { b } else { b.scale(sw, sh)?.pad(width, height, (width-sw)/2, (height-sh)/2, [0,0,0,255])? });
        }
        self.next = slot + 1;
        Ok((self.current.as_ref().ok_or("missing_frame")?.clone(), repeat))
    }
}
struct Chunk { packets: Vec<media_av1::Packet>, config: Vec<u8>, encoded: usize }
fn chunk(t: &Timeline, base: &Path, start: u64, end: u64) -> Result<Chunk> {
    let v = &t.video; let e = &t.encode;
    let mut enc = Encoder::new(Settings { width: v.width, height: v.height, fps_num: 30, fps_den: 1, speed: e.speed, target: Target::Quality(e.qp), key_interval: end-start+1, tile_cols: e.tile_cols, tile_rows: e.tile_rows, threads: e.threads })?;
    let mut sources: Vec<_> = v.segments.iter().map(|s| Source::new(base, s)).collect::<Result<_>>()?;
    let mut encoded = Vec::new(); let mut positions = Vec::new();
    for k in start..end {
        let j = v.segments.partition_point(|s| frame(s.at_s) <= k)-1;
        let s = &v.segments[j]; let slot = (s.from_slot + k-frame(s.at_s)).min(s.to_slot-1);
        let (mut b, mut repeat) = sources[j].get(slot, v.width, v.height)?;
        if let Some(f) = v.crossfades.iter().find(|f| k >= frame(f.at_s) && k < frame(f.at_s + f.duration_s)) {
            let old = &v.segments[j-1];
            let old_slot = (old.from_slot + k-frame(old.at_s)).min(old.to_slot-1);
            let (a, _) = sources[j-1].get(old_slot, v.width, v.height)?;
            let len = frame(f.duration_s);
            let weight = if len == 1 { 255 } else { ((k-frame(f.at_s))*255/(len-1)) as u8 };
            b = a.crossfade(&b, weight)?; repeat = false;
        }
        // The displayed keyframe is not showable. Submit its first successor too.
        if !repeat || k < start+2 || k == frame(s.at_s) {
            let p = enc.prepare(&b.to_yuv420())?;
            encoded.extend(enc.submit(&p)?); positions.push(Some(k));
        } else { positions.push(None); }
    }
    encoded.extend(enc.finish()?);
    let count = positions.iter().filter(|p| p.is_some()).count();
    if count != encoded.len() { return refuse("encoder_packet_count"); }
    let config = media_webm::av1_config(&encoded.first().ok_or("empty_chunk")?.data)?;
    let mut tracker = ReferenceTracker::default(); let mut input = encoded.into_iter(); let mut packets = Vec::new();
    for (i, position) in positions.into_iter().enumerate() {
        let k = start+i as u64;
        let packet = if position.is_some() {
            let mut p = input.next().ok_or("encoder_packet_count")?;
            let key = tracker.observe(&p.data)?;
            if key != p.key_frame || (i == 0) != key { return refuse("unexpected_keyframe"); }
            p.pts = k; p
        } else { media_av1::Packet { data: tracker.repeat()?, pts: k, duration: 1, key_frame: false } };
        packets.push(packet);
    }
    Ok(Chunk { packets, config, encoded: count })
}
fn cpu() -> f64 {
    let ticks = std::fs::read("/proc/self/auxv").ok().and_then(|b| b.chunks_exact(16).find_map(|x| {
        (u64::from_ne_bytes(x[..8].try_into().ok()?) == 17).then(|| u64::from_ne_bytes(x[8..].try_into().unwrap()))
    })).unwrap_or(100) as f64;
    std::fs::read_to_string("/proc/self/stat").ok().and_then(|s| {
        let fields: Vec<_> = s.rsplit_once(')')?.1.split_whitespace().collect();
        Some((fields.get(11)?.parse::<f64>().ok()? + fields.get(12)?.parse::<f64>().ok()?) / ticks)
    }).unwrap_or(0.0)
}
pub fn run(args: impl Iterator<Item=OsString>) -> Result<u8> {
    let args: Vec<_> = args.collect();
    if args.len() != 3 || args[1] != "--out" { return refuse("usage: av render TIMELINE.json --out FILE.webm"); }
    let path = Path::new(&args[0]); let out = Path::new(&args[2]);
    let mut bytes = Vec::new(); std::fs::File::open(path)?.take(4*1024*1024+1).read_to_end(&mut bytes)?;
    if bytes.len() > 4*1024*1024 { return refuse("timeline_size_limit"); }
    let t: Timeline = serde_json::from_slice(&bytes)?; let base = path.parent().unwrap_or(Path::new("."));
    let starts = t.validate(base)?;
    let audio = media_audio::timeline::Timeline { version: t.version, duration_s: t.duration_s, sample_rate: t.audio.sample_rate, channels: t.audio.channels, sources: t.audio.sources, master: t.audio.master };
    // Preserve the validated video executor while borrowing the audio specification.
    let wall = Instant::now(); let c = cpu();
    let a = media_audio::timeline::prepare(&audio, base)?;
    let audio_wall = wall.elapsed().as_secs_f64(); let audio_cpu = cpu()-c;
    if a.receipt.refused || !a.receipt.target_reached { println!("{}", serde_json::to_string(&a.receipt)?); return Ok(3); }
    let t = Timeline { audio: Audio { sample_rate: 48000, channels: 2, sources: Vec::new(), master: audio.master }, ..t };
    let video_start = Instant::now(); let c = cpu(); let mut chunks = Vec::new();
    for ranges in starts.windows(2).collect::<Vec<_>>().chunks(t.encode.chunks) {
        let batch = std::thread::scope(|scope| {
            let handles: Vec<_> = ranges.iter().map(|r| { let t = &t; scope.spawn(move || chunk(t, base, r[0], r[1]).map_err(|e| e.to_string())) }).collect();
            handles.into_iter().map(|h| h.join().map_err(|_| "chunk_worker_panicked".to_string())?).collect::<std::result::Result<Vec<_>, String>>()
        })?;
        chunks.extend(batch);
    }
    let video_wall = video_start.elapsed().as_secs_f64(); let video_cpu = cpu()-c;
    if chunks.iter().any(|x| x.config != chunks[0].config) { return refuse("sequence_header_mismatch"); }
    let mux_start = Instant::now(); let c = cpu();
    let mut pending = media_audio::pending::Pending::new(out)?;
    let mut w = media_webm::Writer::new(BufWriter::new(&mut pending.file), media_webm::Video { width: t.video.width as u32, height: t.video.height as u32, codec_private: chunks[0].config.clone() }, Some(media_webm::Opus { channels: 2, pre_skip: a.head.pre_skip, input_sample_rate: 48000 }))?;
    let mut ai = 0; let mut encoded = 0; let mut keyframes = 0;
    for chunk in &chunks {
        encoded += chunk.encoded;
        for p in &chunk.packets {
            let ms = p.pts*1000/30;
            while ai < a.packets.len() && ai as u64*20 <= ms {
                let packet = &a.packets[ai]; w.opus_packet(ai as u64*20, &packet.data, packet.discard_padding)?; ai += 1;
            }
            keyframes += usize::from(p.key_frame);
            w.packet(1, ms, (p.pts+1)*1000/30-ms, p.key_frame, &p.data)?;
        }
    }
    for (i, p) in a.packets.iter().enumerate().skip(ai) { w.opus_packet(i as u64*20, &p.data, p.discard_padding)?; }
    w.set_duration_samples(a.receipt.sample_count as u64); drop(w.finish()?);
    let size = pending.file.metadata()?.len(); pending.publish()?; pending.commit();
    let n = frame(t.duration_s);
    println!("{}", serde_json::json!({"frames_out": n, "frames_encoded": encoded, "repeats": n-encoded as u64, "keyframes": keyframes, "chunks": chunks.len(), "parallel_chunks": t.encode.chunks, "stages": {"audio": {"wall_s": audio_wall, "cpu_s": audio_cpu}, "video": {"wall_s": video_wall, "cpu_s": video_cpu}, "mux": {"wall_s": mux_start.elapsed().as_secs_f64(), "cpu_s": cpu()-c}}, "wall_s": wall.elapsed().as_secs_f64(), "effective_fps": n as f64/wall.elapsed().as_secs_f64(), "bytes": size, "audio": a.receipt}));
    Ok(0)
}
