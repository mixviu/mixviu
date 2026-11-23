use media_av1::{Encoder, Settings, Target, reference::ReferenceTracker};
use media_frame::Bgra;
use media_vfl::reader::{Directory, Reader};
use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
type Directories = Vec<Arc<Directory>>;
const CHUNK_BYTES_LIMIT: usize = 64 * 1024 * 1024;
fn collect_packets(
    out: &mut Vec<media_av1::Packet>,
    bytes: &mut usize,
    packets: Vec<media_av1::Packet>,
) -> Result<()> {
    for p in packets {
        *bytes = bytes
            .checked_add(p.data.len())
            .filter(|n| *n <= CHUNK_BYTES_LIMIT)
            .ok_or("render: chunk_bytes_limit")?;
        out.push(p);
    }
    Ok(())
}
fn opened_timeline(path: &Path) -> Result<(std::fs::File, PathBuf)> {
    let file = std::fs::File::options()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return refuse("timeline_regular_file");
    }
    let resolved = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    let base = std::fs::canonicalize(resolved.parent().ok_or("timeline_directory")?)?;
    Ok((file, base))
}
fn slot(s: &Segment, k: u64) -> Result<u64> {
    let delta = k
        .checked_sub(frame(s.at_s))
        .ok_or("render: slot_arithmetic")?;
    let last = s.to_slot.checked_sub(1).ok_or("render: slot_arithmetic")?;
    Ok(s.from_slot
        .checked_add(delta)
        .ok_or("render: slot_arithmetic")?
        .min(last))
}
fn fade_end(f: &Fade) -> u64 {
    frame(f.at_s + f.duration_s)
}
fn fade_weight(f: &Fade, k: u64) -> Result<u8> {
    let start = frame(f.at_s);
    let len = fade_end(f)
        .checked_sub(start)
        .filter(|n| *n > 0)
        .ok_or("render: crossfade_range")?;
    let delta = k
        .checked_sub(start)
        .filter(|n| *n < len)
        .ok_or("render: crossfade_range")?;
    Ok(if len == 1 {
        255
    } else {
        (delta
            .checked_mul(255)
            .ok_or("render: crossfade_arithmetic")?
            / (len - 1)) as u8
    })
}
use serde::Deserialize;
use std::sync::Arc;
use std::{
    ffi::OsString,
    io::{self, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Timeline {
    version: u32,
    duration_s: f64,
    video: Video,
    audio: Audio,
    encode: Encode,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Audio {
    sample_rate: u32,
    channels: u16,
    sources: Vec<media_audio::timeline::Source>,
    master: media_audio::timeline::Master,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Video {
    width: usize,
    height: usize,
    fps: u32,
    segments: Vec<Segment>,
    #[serde(default)]
    crossfades: Vec<Fade>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Segment {
    path: PathBuf,
    from_slot: u64,
    to_slot: u64,
    at_s: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fade {
    at_s: f64,
    duration_s: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Encode {
    speed: u8,
    qp: u8,
    tile_cols: usize,
    tile_rows: usize,
    threads: usize,
    #[serde(default = "parallelism")]
    chunks: usize,
    keyint_s: f64,
}
fn parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, |n| (n.get() / 8).max(1))
}
fn frame(s: f64) -> u64 {
    (s * 30.0).round() as u64
}
fn time(s: f64) -> bool {
    s.is_finite() && (0.0..=10800.0).contains(&s)
}
fn refuse<T>(name: &str) -> Result<T> {
    Err(format!("render: {name}").into())
}
impl Timeline {
    fn validate(&mut self, base: &Path) -> Result<(Vec<u64>, Directories)> {
        self.video
            .crossfades
            .sort_by(|a, b| a.at_s.total_cmp(&b.at_s));
        let v = &self.video;
        if self.version != 1 {
            return refuse("version");
        }
        if !time(self.duration_s) || frame(self.duration_s) == 0 {
            return refuse("duration_s");
        }
        if v.fps != 30 {
            return refuse("fps: only 30 supported");
        }
        if !(16..=7680).contains(&v.width) || !(16..=4320).contains(&v.height) {
            return refuse("dimensions");
        }
        if v.segments.is_empty() || v.segments.len() > 10000 || v.crossfades.len() > 10000 {
            return refuse("segments_limit");
        }
        if self.audio.sample_rate != 48000 {
            return refuse("audio_sample_rate");
        }
        if self.audio.channels != 2 {
            return refuse("audio_channels");
        }
        let e = &self.encode;
        if e.threads == 0
            || e.threads > 64
            || e.tile_cols > 64
            || e.tile_rows > 64
            || !e.tile_cols.is_power_of_two()
            || !e.tile_rows.is_power_of_two()
        {
            return refuse("encode_tiles_threads");
        }
        if e.speed > 10
            || e.chunks == 0
            || e.chunks > 64
            || !time(e.keyint_s)
            || frame(e.keyint_s) == 0
        {
            return refuse("encode_settings");
        }
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
        if e.chunks
            .checked_mul(e.threads)
            .is_none_or(|n| n > cores.saturating_mul(2))
        {
            return refuse("encode_thread_budget");
        }
        let mut directories = Vec::new();
        let mut source_bytes = 0u64;
        let mut scale_bytes = 0u64;
        let mut starts = vec![0];
        let mut previous = None;
        for s in &v.segments {
            if !time(s.at_s) || s.from_slot >= s.to_slot || frame(s.at_s) >= frame(self.duration_s)
            {
                return refuse("segment_range");
            }
            if s.path.as_os_str().is_empty()
                || s.path.components().any(|c| {
                    !matches!(
                        c,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )
                })
            {
                return refuse("segment_path_relative");
            }
            let relative: PathBuf = s
                .path
                .components()
                .filter_map(|c| {
                    if let std::path::Component::Normal(n) = c {
                        Some(n)
                    } else {
                        None
                    }
                })
                .collect();
            if relative.as_os_str().is_empty() {
                return refuse("segment_path_relative");
            }
            let directory = Arc::new(
                Directory::open(base.join(relative))
                    .map_err(|e| format!("render: segment_directory: {e}"))?,
            );
            if !directory.resolved()?.starts_with(base) {
                return refuse("segment_path_escape");
            }
            let mut r = Reader::from_directory(directory.clone())?;
            if r.fps() != 30 || !r.complete() {
                return refuse("segment_complete_30fps_required");
            }
            let count = r.frame_count()?;
            if s.from_slot >= count || s.to_slot > count {
                return refuse("segment_trim_outside_vfl");
            }
            let (w, h) = (u64::from(r.width()), u64::from(r.height()));
            source_bytes = source_bytes.max(w * h * 4);
            let ratio = (v.width as f64 / w as f64).min(v.height as f64 / h as f64);
            let sw = ((w as f64 * ratio).round() as u64).clamp(1, v.width as u64);
            scale_bytes = scale_bytes.max(
                sw.checked_mul(h)
                    .and_then(|n| n.checked_mul(16))
                    .ok_or("render: encode_memory_budget")?,
            );
            directories.push(directory);
            let start = frame(s.at_s);
            start
                .checked_add(
                    s.to_slot
                        .checked_sub(s.from_slot)
                        .ok_or("render: coverage_arithmetic")?,
                )
                .ok_or("render: coverage_arithmetic")?;
            if previous.is_none() && start != 0 {
                return refuse("gap_before_first_segment");
            }
            if previous.is_some_and(|p| start <= p) {
                return refuse("segment_order");
            }
            previous = Some(start);
            starts.push(start);
        }
        for (i, f) in v.crossfades.iter().enumerate() {
            if !time(f.at_s)
                || !time(f.duration_s)
                || !time(f.at_s + f.duration_s)
                || fade_end(f) <= frame(f.at_s)
                || frame(f.at_s + f.duration_s) > frame(self.duration_s)
            {
                return refuse("crossfade_range");
            }
            let Ok(j) = v
                .segments
                .binary_search_by_key(&frame(f.at_s), |s| frame(s.at_s))
            else {
                return refuse("crossfade_segment_start");
            };
            // The outgoing trim may end earlier: compose holds its last frame.
            if j == 0 {
                return refuse("crossfade_requires_overlap");
            }
            if v.segments
                .get(j + 1)
                .is_some_and(|s| frame(s.at_s) < frame(f.at_s + f.duration_s))
            {
                return refuse("crossfade_third_segment");
            }
            if i > 0
                && frame(v.crossfades[i - 1].at_s + v.crossfades[i - 1].duration_s) > frame(f.at_s)
            {
                return refuse("crossfade_order");
            }
        }
        let n = frame(self.duration_s);
        // Conservative admission estimate, including packet/position metadata,
        // two VFL canvases with payload/scratch, scaler and encoder frame storage.
        let working = source_bytes * 6
            + scale_bytes
            + (v.width as u64 * v.height as u64) * 96
            + CHUNK_BYTES_LIMIT as u64
            + 64 * 1024 * 1024;
        if working
            .checked_mul(e.chunks as u64)
            .is_none_or(|n| n > 8 * 1024 * 1024 * 1024)
        {
            return refuse("encode_memory_budget");
        }
        let step = frame(e.keyint_s);
        starts.extend((0..n).step_by(step as usize));
        starts.push(n);
        starts.sort_unstable();
        starts.dedup();
        Ok((starts, directories))
    }
}
#[derive(Default)]
struct ScaleStats {
    frames: usize,
    wall_s: f64,
}
struct Source {
    reader: media_vfl::reader::Reader,
    next: u64,
    current: Option<Arc<Bgra>>,
}
impl Source {
    fn new(directory: Arc<Directory>) -> Result<Self> {
        Ok(Self {
            reader: Reader::from_directory(directory)?,
            next: u64::MAX,
            current: None,
        })
    }
    fn get(
        &mut self,
        slot: u64,
        width: usize,
        height: usize,
        stats: &mut ScaleStats,
    ) -> Result<(Arc<Bgra>, bool)> {
        let after = slot.checked_add(1).ok_or("render: slot_arithmetic")?;
        if self.next == after {
            return Ok((self.current.as_ref().ok_or("missing_frame")?.clone(), true));
        }
        let w = self.reader.width() as usize;
        let h = self.reader.height() as usize;
        let f = if self.next == slot {
            self.reader.read()?.ok_or("segment_trim_outside_vfl")?
        } else {
            self.reader.seek(slot)?
        };
        let repeat = f.repeat && self.current.is_some();
        if !repeat {
            let b = Bgra::new(w, h, f.pixels.to_vec())?;
            let ratio = (width as f64 / w as f64).min(height as f64 / h as f64);
            let sw = ((w as f64 * ratio).round() as usize).clamp(1, width);
            let sh = ((h as f64 * ratio).round() as usize).clamp(1, height);
            self.current = Some(Arc::new(if w == width && h == height {
                b
            } else {
                {
                    let start = Instant::now();
                    let scaled = b.resample(sw, sh)?;
                    stats.frames += 1;
                    stats.wall_s += start.elapsed().as_secs_f64();
                    scaled
                }
                .pad(
                    width,
                    height,
                    (width - sw) / 2,
                    (height - sh) / 2,
                    [0, 0, 0, 255],
                )?
            }));
        }
        self.next = after;
        Ok((
            self.current.as_ref().ok_or("missing_frame")?.clone(),
            repeat,
        ))
    }
}
struct Chunk {
    packets: Vec<media_av1::Packet>,
    config: Vec<u8>,
    encoded: usize,
    scaling: ScaleStats,
}
fn compose(
    v: &Video,
    directories: &[Arc<Directory>],
    sources: &mut [Option<Source>],
    fade_index: &mut usize,
    k: u64,
    stats: &mut ScaleStats,
) -> Result<(Arc<Bgra>, bool)> {
    let j = v.segments.partition_point(|s| frame(s.at_s) <= k) - 1;
    // Execution walks forward. Retain only the current and outgoing segment;
    // opening every reader up front would allocate one canvas per segment.
    if j >= 2 {
        sources[j - 2] = None;
    }
    let s = &v.segments[j];
    if sources[j].is_none() {
        sources[j] = Some(Source::new(directories[j].clone())?);
    }
    let current_slot = slot(s, k)?;
    let (mut b, mut repeat) = sources[j].as_mut().ok_or("missing_segment_reader")?.get(
        current_slot,
        v.width,
        v.height,
        stats,
    )?;
    while v
        .crossfades
        .get(*fade_index)
        .is_some_and(|f| k >= fade_end(f))
    {
        *fade_index += 1;
    }
    if let Some(f) = v.crossfades.get(*fade_index).filter(|f| k >= frame(f.at_s)) {
        let old = &v.segments[j - 1];
        if sources[j - 1].is_none() {
            sources[j - 1] = Some(Source::new(directories[j - 1].clone())?);
        }
        let old_slot = slot(old, k)?;
        let (a, _) = sources[j - 1]
            .as_mut()
            .ok_or("missing_segment_reader")?
            .get(old_slot, v.width, v.height, stats)?;
        let weight = fade_weight(f, k)?;
        b = Arc::new(a.crossfade(&b, weight)?);
        repeat = false;
    }
    Ok((b, repeat && k != frame(s.at_s)))
}
/// Lossless oracle export: exactly the executor's scaling, padding and fades.
/// Used by the benchmark example, never by render's output path.
pub fn reference(path: &Path, out: impl std::io::Write) -> Result<()> {
    use std::io::Write;
    let (file, base) = opened_timeline(path)?;
    let mut t: Timeline = serde_json::from_reader(file.take(16 * 1024 * 1024 + 1))?;
    let (_, directories) = t.validate(&base)?;
    let v = &t.video;
    let mut scaling = ScaleStats::default();
    let mut fade_index = 0;
    let mut sources: Vec<_> = (0..v.segments.len()).map(|_| None).collect();
    let mut out = BufWriter::new(out);
    writeln!(
        out,
        "YUV4MPEG2 W{} H{} F30:1 Ip A1:1 C420jpeg",
        v.width, v.height
    )?;
    let mut yuv = None;
    for k in 0..frame(t.duration_s) {
        let (b, repeat) = compose(
            v,
            &directories,
            &mut sources,
            &mut fade_index,
            k,
            &mut scaling,
        )?;
        if !repeat {
            yuv = Some(b.to_yuv420());
        }
        let yuv = yuv.as_ref().ok_or("missing_reference_frame")?;
        out.write_all(b"FRAME\n")?;
        out.write_all(&yuv.y)?;
        out.write_all(&yuv.u)?;
        out.write_all(&yuv.v)?;
    }
    out.flush()?;
    Ok(())
}
fn chunk(t: &Timeline, directories: &[Arc<Directory>], start: u64, end: u64) -> Result<Chunk> {
    let v = &t.video;
    let e = &t.encode;
    let mut enc = Encoder::new_repeats(Settings {
        width: v.width,
        height: v.height,
        fps_num: 30,
        fps_den: 1,
        speed: e.speed,
        target: Target::Quality(e.qp),
        key_interval: end - start + 1,
        tile_cols: e.tile_cols,
        tile_rows: e.tile_rows,
        threads: e.threads,
    })?;
    let mut scaling = ScaleStats::default();
    let mut fade_index = v.crossfades.partition_point(|f| fade_end(f) <= start);
    let mut sources: Vec<_> = (0..v.segments.len()).map(|_| None).collect();
    let mut encoded = Vec::new();
    let mut encoded_bytes = 0usize;
    let mut positions = Vec::new();
    for k in start..end {
        let (b, repeat) = compose(
            v,
            &directories,
            &mut sources,
            &mut fade_index,
            k,
            &mut scaling,
        )?;
        // The displayed keyframe is not showable. Submit its first successor too.
        if !repeat || k < start + 2 {
            let p = enc.prepare(&b.to_yuv420())?;
            collect_packets(&mut encoded, &mut encoded_bytes, enc.submit(&p)?)?;
            positions.push(Some(k));
        } else {
            positions.push(None);
        }
    }
    collect_packets(&mut encoded, &mut encoded_bytes, enc.finish()?)?;
    let count = positions.iter().filter(|p| p.is_some()).count();
    if count != encoded.len() {
        return refuse("encoder_packet_count");
    }
    let config = media_webm::av1_config(&encoded.first().ok_or("empty_chunk")?.data)?;
    let mut tracker = ReferenceTracker::default();
    let mut input = encoded.into_iter();
    let mut input_index = 0;
    let mut packets = Vec::new();
    for (i, position) in positions.into_iter().enumerate() {
        let k = start + i as u64;
        let packet = if position.is_some() {
            let mut p = input.next().ok_or("encoder_packet_count")?;
            if p.pts != input_index || p.duration != 1 {
                return refuse("reordered_encoder_packets");
            }
            input_index += 1;
            let key = tracker.observe(&p.data)?;
            if key != p.key_frame || (i == 0) != key {
                return refuse("unexpected_keyframe");
            }
            p.pts = k;
            p
        } else {
            media_av1::Packet {
                data: tracker.repeat()?,
                pts: k,
                duration: 1,
                key_frame: false,
            }
        };
        packets.push(packet);
    }
    Ok(Chunk {
        packets,
        config,
        encoded: count,
        scaling,
    })
}
fn cpu() -> f64 {
    let ticks = std::fs::read("/proc/self/auxv")
        .ok()
        .and_then(|b| {
            b.chunks_exact(16).find_map(|x| {
                (u64::from_ne_bytes(x[..8].try_into().ok()?) == 17)
                    .then(|| u64::from_ne_bytes(x[8..].try_into().unwrap()))
            })
        })
        .unwrap_or(100) as f64;
    std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| {
            let fields: Vec<_> = s.rsplit_once(')')?.1.split_whitespace().collect();
            Some(
                (fields.get(11)?.parse::<f64>().ok()? + fields.get(12)?.parse::<f64>().ok()?)
                    / ticks,
            )
        })
        .unwrap_or(0.0)
}
pub fn run(args: impl Iterator<Item = OsString>) -> Result<u8> {
    let overall = Instant::now();
    let total_cpu = cpu();
    let args: Vec<_> = args.collect();
    if args.len() != 3 || args[1] != "--out" {
        return refuse("usage: av render TIMELINE.json --out FILE.webm");
    }
    let path = Path::new(&args[0]);
    let out = Path::new(&args[2]);
    let mut bytes = Vec::new();
    let (file, base) = opened_timeline(path)?;
    file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return refuse("timeline_size_limit");
    }
    let mut t: Timeline = serde_json::from_slice(&bytes)?;
    let (starts, directories) = t.validate(&base)?;
    let audio = media_audio::timeline::Timeline {
        version: t.version,
        duration_s: t.duration_s,
        sample_rate: t.audio.sample_rate,
        channels: t.audio.channels,
        sources: t.audio.sources,
        master: t.audio.master,
    };
    // Preserve the validated video executor while borrowing the audio specification.
    let input_wall = overall.elapsed().as_secs_f64();
    let input_cpu = cpu() - total_cpu;
    let wall = Instant::now();
    let c = cpu();
    let a = media_audio::timeline::prepare(&audio, &base)?;
    let audio_wall = wall.elapsed().as_secs_f64();
    let audio_cpu = cpu() - c;
    if a.receipt.refused || !a.receipt.target_reached {
        let _ = media_audio::cli::diagnostic(format_args!(
            "mastering refused or loudness target missed"
        ));
        return deliver_receipt(&serde_json::to_string(&a.receipt)?, 3);
    }
    drop(a.audio);
    let t = Timeline {
        audio: Audio {
            sample_rate: 48000,
            channels: 2,
            sources: Vec::new(),
            master: audio.master,
        },
        ..t
    };
    let mut pending = media_audio::pending::Pending::new(out)?;
    let mut writer = None;
    let mut config: Option<Vec<u8>> = None;
    let mut ai = 0;
    let mut encoded = 0;
    let mut keyframes = 0;
    let mut scaled_frames = 0;
    let mut scale_wall_s = 0.0;
    let mut video_wall = 0.0;
    let mut video_cpu = 0.0;
    let mut mux_wall = 0.0;
    let mut mux_cpu = 0.0;
    let chunk_count = starts.len() - 1;
    for ranges in starts
        .windows(2)
        .collect::<Vec<_>>()
        .chunks(t.encode.chunks)
    {
        let wall = Instant::now();
        let c = cpu();
        let batch = std::thread::scope(|scope| {
            let handles: Vec<_> = ranges
                .iter()
                .map(|r| {
                    let t = &t;
                    let directories = &directories;
                    scope
                        .spawn(move || chunk(t, directories, r[0], r[1]).map_err(|e| e.to_string()))
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().map_err(|_| "chunk_worker_panicked".to_string())?)
                .collect::<std::result::Result<Vec<_>, String>>()
        })?;
        video_wall += wall.elapsed().as_secs_f64();
        video_cpu += cpu() - c;
        let wall = Instant::now();
        let c = cpu();
        for chunk in batch {
            if let Some(previous) = &config {
                if *previous != chunk.config {
                    return refuse("sequence_header_mismatch");
                }
            } else {
                config = Some(chunk.config.clone());
                writer = Some(media_webm::Writer::new(
                    BufWriter::new(pending.file.try_clone()?),
                    media_webm::Video {
                        width: t.video.width as u32,
                        height: t.video.height as u32,
                        codec_private: chunk.config.clone(),
                    },
                    Some(media_webm::Opus {
                        channels: a.head.channels,
                        pre_skip: a.head.pre_skip,
                        input_sample_rate: a.head.input_sample_rate,
                    }),
                )?);
            }
            let w = writer.as_mut().ok_or("missing_muxer")?;
            encoded += chunk.encoded;
            scaled_frames += chunk.scaling.frames;
            scale_wall_s += chunk.scaling.wall_s;
            for p in chunk.packets {
                let ms = p.pts * 1000 / 30;
                while ai < a.packets.len() && ai as u64 * 20 <= ms {
                    let packet = &a.packets[ai];
                    w.opus_packet(ai as u64 * 20, &packet.data, packet.discard_padding)?;
                    ai += 1;
                }
                keyframes += usize::from(p.key_frame);
                w.packet(1, ms, (p.pts + 1) * 1000 / 30 - ms, p.key_frame, &p.data)?;
            }
        }
        mux_wall += wall.elapsed().as_secs_f64();
        mux_cpu += cpu() - c;
    }
    let mux_start = Instant::now();
    let c = cpu();
    let mut w = writer.ok_or("missing_muxer")?;
    for (i, p) in a.packets.iter().enumerate().skip(ai) {
        w.opus_packet(i as u64 * 20, &p.data, p.discard_padding)?;
    }
    w.set_duration_samples(a.receipt.sample_count as u64);
    w.finish()?.flush()?;
    let size = pending.file.metadata()?.len();
    mux_wall += mux_start.elapsed().as_secs_f64();
    mux_cpu += cpu() - c;
    let n = frame(t.duration_s);
    let receipt = serde_json::to_string(
        &serde_json::json!({"scaling": {"filter":"bicubic_catmull_rom_gamma","frames":scaled_frames,"sum_wall_s":scale_wall_s,"mean_ms":scale_wall_s*1000.0/scaled_frames.max(1) as f64}, "frames_out": n, "frames_encoded": encoded, "repeats": n-encoded as u64, "keyframes": keyframes, "chunks": chunk_count, "parallel_chunks": t.encode.chunks, "stages": {"input": {"wall_s": input_wall, "cpu_s": input_cpu}, "audio": {"wall_s": audio_wall, "cpu_s": audio_cpu}, "video": {"wall_s": video_wall, "cpu_s": video_cpu}, "mux": {"wall_s": mux_wall, "cpu_s": mux_cpu}}, "wall_s": overall.elapsed().as_secs_f64(), "cpu_s": cpu()-total_cpu, "effective_fps": n as f64/overall.elapsed().as_secs_f64(), "bytes": size, "audio": a.receipt}),
    )?;
    pending.publish()?;
    pending.commit();
    deliver_receipt(&receipt, 0)
}

fn deliver_receipt(receipt: &str, outcome: u8) -> Result<u8> {
    let mut stdout = io::stdout().lock();
    if let Err(e) = writeln!(stdout, "{receipt}").and_then(|()| stdout.flush())
        && outcome == 0
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        let _ =
            media_audio::cli::diagnostic(format_args!("published but receipt not written: {e}"));
        return Ok(4);
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture(root: &Path) -> Timeline {
        let mut w = media_vfl::Writer::new(root.join("a.vfl"), 32, 32, 30).unwrap();
        w.origin(0).unwrap();
        w.frame(media_vfl::Frame {
            slot: 0,
            pixels: media_vfl::Pixels::Bgra(vec![70; 4096].into()),
        })
        .unwrap();
        w.finish(60, 0, 0, None, true).unwrap();
        serde_json::from_value(json!({"version":1,"duration_s":4,
            "video":{"width":32,"height":32,"fps":30,"segments":[
                {"path":"a.vfl","from_slot":0,"to_slot":30,"at_s":0},
                {"path":"a.vfl","from_slot":0,"to_slot":30,"at_s":1.51},
                {"path":"a.vfl","from_slot":0,"to_slot":30,"at_s":2.51}],
                "crossfades":[{"at_s":2.51,"duration_s":0.041},{"at_s":1.51,"duration_s":0.041}]},
            "audio":{"sample_rate":48000,"channels":2,"sources":[],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}},
            "encode":{"speed":10,"qp":100,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":1,"keyint_s":1}})).unwrap()
    }
    #[test]
    fn confined_directory_survives_validation_path_swap() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        let (_, directories) = t.validate(root.path()).unwrap();
        std::fs::rename(root.path().join("a.vfl"), root.path().join("original.vfl")).unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("a.vfl")).unwrap();
        let mut source = Source::new(directories[0].clone()).unwrap();
        assert_eq!(
            source
                .get(0, 32, 32, &mut ScaleStats::default())
                .unwrap()
                .0
                .pixels()[0],
            70
        );
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("segment_directory")
        );
        // An intermediate symlink is followed only during the one directory
        // open; descriptor resolution must still confine the actual result.
        std::os::unix::fs::symlink(outside.path(), root.path().join("parent")).unwrap();
        std::fs::create_dir(outside.path().join("escape.vfl")).unwrap();
        t.video.segments[0].path = "parent/escape.vfl".into();
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("segment_path_escape")
        );
    }
    #[test]
    fn symlink_chunk_and_non_directory_refused() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        let p = root.path().join("a.vfl/chunk-000000.bin");
        std::fs::rename(&p, root.path().join("chunk")).unwrap();
        std::os::unix::fs::symlink(root.path().join("chunk"), &p).unwrap();
        assert!(t.validate(root.path()).is_err());
        std::fs::write(root.path().join("file"), b"not a directory").unwrap();
        t.video.segments[0].path = "file".into();
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("segment_directory")
        );
    }
    #[test]
    fn rounding_hold_sorting_budgets_and_checked_slots() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        t.validate(root.path()).unwrap(); // previous trim ends before fade: held frame covers it
        assert_eq!(t.video.crossfades[0].at_s, 1.51);
        for (at, duration) in [(1.51, 0.041), (1.52, 0.05)] {
            let f = Fade {
                at_s: at,
                duration_s: duration,
            };
            assert_eq!(fade_weight(&f, fade_end(&f) - 1).unwrap(), 255);
            if fade_end(&f) - frame(at) > 1 {
                assert_eq!(fade_weight(&f, frame(at)).unwrap(), 0);
            }
        }
        t.video.crossfades.push(Fade {
            at_s: 1.51,
            duration_s: 0.02,
        });
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("crossfade_order")
        );
        t.video.crossfades.pop();
        t.audio.channels = 1;
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("audio_channels")
        );
        t.audio.channels = 2;
        t.audio.sample_rate = 44100;
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("audio_sample_rate")
        );
        t.audio.sample_rate = 48000;
        t.encode.chunks = 64;
        t.encode.threads = 64;
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("encode_thread_budget")
        );
        t.encode.chunks = 1;
        t.encode.threads = 1;
        if std::thread::available_parallelism().map_or(1, |n| n.get()) >= 2 {
            t.video.crossfades.clear();
            t.video.width = 7680;
            t.video.height = 4320;
            t.encode.chunks = 3;
            assert!(
                t.validate(root.path())
                    .unwrap_err()
                    .to_string()
                    .contains("encode_memory_budget")
            );
            t.video.width = 32;
            t.video.height = 32;
            t.encode.chunks = 1;
        }
        t.video.segments[0].to_slot = u64::MAX;
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("segment_trim_outside_vfl")
        );
        let s = Segment {
            path: "a.vfl".into(),
            from_slot: u64::MAX - 1,
            to_slot: u64::MAX,
            at_s: 0.0,
        };
        assert!(
            slot(&s, 2)
                .unwrap_err()
                .to_string()
                .contains("slot_arithmetic")
        );
        let mut bytes = CHUNK_BYTES_LIMIT;
        assert!(
            collect_packets(
                &mut Vec::new(),
                &mut bytes,
                vec![media_av1::Packet {
                    data: vec![1],
                    pts: 0,
                    duration: 1,
                    key_frame: false
                }]
            )
            .unwrap_err()
            .to_string()
            .contains("chunk_bytes_limit")
        );
    }
}
