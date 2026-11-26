use media_av1::{Encoder, Settings, Target, reference::ReferenceTracker};
use media_frame::Bgra;
use media_vfl::reader::{Directory, Reader};
use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
type Directories = Vec<Input>;
#[derive(Clone, Debug)]
enum Input {
    Vfl(Arc<Directory>),
    Still(Arc<std::fs::File>),
    Solid([u8; 3]),
}
const STILL_FILE_BYTES_LIMIT: u64 = 64 * 1024 * 1024;
const STILL_DIMENSION_LIMIT: u32 = 7680;
const STILL_PIXELS_LIMIT: u64 = 7680 * 4320;
const STILL_DECODE_BYTES_LIMIT: usize = 256 * 1024 * 1024;

fn png_reader(
    file: &std::fs::File,
) -> Result<png::Reader<io::BufReader<std::io::Take<std::fs::File>>>> {
    use std::io::{Seek, SeekFrom};
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(0))?;
    let mut decoder = png::Decoder::new_with_limits(
        io::BufReader::new(file.take(STILL_FILE_BYTES_LIMIT)),
        png::Limits {
            bytes: STILL_DECODE_BYTES_LIMIT,
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    // Check IHDR before read_info can initialise row/image decoder buffers.
    let info = decoder.read_header_info()?;
    if info.width == 0
        || info.height == 0
        || info.width > STILL_DIMENSION_LIMIT
        || info.height > STILL_DIMENSION_LIMIT
        || u64::from(info.width) * u64::from(info.height) > STILL_PIXELS_LIMIT
    {
        return refuse("still_dimensions_limit");
    }
    let reader = decoder.read_info()?;
    if reader.info().animation_control.is_some() {
        return refuse("still_animation");
    }
    Ok(reader)
}
fn open_still(base: &Path, relative: &Path) -> Result<(Input, u64, u64)> {
    let file = std::fs::File::options()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(base.join(relative))?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return refuse("still_regular_file");
    }
    let resolved = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    if !resolved.starts_with(base) {
        return refuse("segment_path_escape");
    }
    if meta.len() > STILL_FILE_BYTES_LIMIT {
        return refuse("still_file_bytes_limit");
    }
    let reader = png_reader(&file)?;
    let (w, h) = (reader.info().width, reader.info().height);
    Ok((Input::Still(Arc::new(file)), u64::from(w), u64::from(h)))
}
const SOURCE_LIMIT: usize = 256;
use std::collections::{HashMap, HashSet};
struct VflSource {
    directory: Arc<Directory>,
    count: u64,
    width: u64,
    height: u64,
}
#[derive(Default)]
struct SourceCatalog {
    paths: HashMap<PathBuf, Arc<VflSource>>,
    resolved: HashMap<PathBuf, Arc<VflSource>>,
    stills: HashMap<PathBuf, (Input, u64, u64)>,
    #[cfg(test)]
    opens: usize,
    #[cfg(test)]
    scans: usize,
}
impl SourceCatalog {
    fn get(&mut self, base: &Path, relative: &Path) -> Result<Arc<VflSource>> {
        if let Some(source) = self.paths.get(relative) {
            return Ok(source.clone());
        }
        let directory = Arc::new(
            Directory::open(base.join(relative))
                .map_err(|e| format!("render: segment_directory: {e}"))?,
        );
        #[cfg(test)]
        {
            self.opens += 1;
        }
        let resolved = directory.resolved()?;
        if !resolved.starts_with(base) {
            return refuse("segment_path_escape");
        }
        let source = if let Some(source) = self.resolved.get(&resolved) {
            source.clone()
        } else {
            let mut r = Reader::from_directory(directory.clone())?;
            if r.fps() != 30 || !r.complete() {
                return refuse("segment_complete_30fps_required");
            }
            #[cfg(test)]
            {
                self.scans += 1;
            }
            let source = Arc::new(VflSource {
                count: r.frame_count()?,
                width: u64::from(r.width()),
                height: u64::from(r.height()),
                directory,
            });
            self.resolved.insert(resolved, source.clone());
            source
        };
        self.paths.insert(relative.to_path_buf(), source.clone());
        Ok(source)
    }
}
fn source_paths(segments: &[Segment]) -> Result<Vec<PathBuf>> {
    let mut distinct = HashSet::new();
    let mut paths = Vec::with_capacity(segments.len());
    for s in segments {
        if s.solid.is_some() {
            paths.push(PathBuf::new());
            continue;
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
        distinct.insert(relative.clone());
        if distinct.len() > SOURCE_LIMIT {
            return refuse("too_many_sources");
        }
        paths.push(relative);
    }
    Ok(paths)
}
fn source_canvas_bytes(w: u64, h: u64) -> Result<u64> {
    w.checked_mul(h)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "render: encode_memory_budget".into())
}
fn working_bytes(source: u64, scale: u64, width: u64, height: u64) -> Result<u64> {
    source
        .checked_mul(8)
        .and_then(|n| n.checked_add(scale))
        .and_then(|n| width.checked_mul(height)?.checked_mul(448)?.checked_add(n))
        .and_then(|n| n.checked_add(CHUNK_BYTES_LIMIT as u64))
        .and_then(|n| n.checked_add(64 * 1024 * 1024))
        .ok_or_else(|| "render: encode_memory_budget".into())
}
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
    #[serde(skip)]
    limits: Limits,
}
#[derive(Clone, Copy, Default)]
struct Limits {
    cpu: Option<usize>,
    mem: Option<u64>,
}
fn clamp_cpu(chunks: &mut usize, threads: &mut usize, cpu: usize) {
    *chunks = (*chunks).min((cpu / *threads).max(1));
    *threads = (*threads).min(cpu / *chunks);
}
fn memory_limit(value: &std::ffi::OsStr) -> Result<u64> {
    media_audio::cli::memory_limit(value)
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
    #[serde(default)]
    fade_in_s: f64,
    #[serde(default)]
    fade_out_s: f64,
}

// Inclusive endpoints: the first/last envelope pictures are exactly black.
fn programme_gain(v: &Video, k: u64, n: u64) -> Option<(u64, u64)> {
    let incoming = frame(v.fade_in_s);
    let outgoing = frame(v.fade_out_s);
    if incoming > 0 && k < incoming {
        Some((k, incoming - 1))
    } else if outgoing > 0 && k >= n - outgoing {
        Some((n - 1 - k, outgoing - 1))
    } else {
        None
    }
}

fn programme_picture(v: &Video, b: &Bgra, k: u64, n: u64) -> media_frame::Yuv420 {
    let mut yuv = b.to_yuv420();
    if let Some((gain, denominator)) = programme_gain(v, k, n) {
        // BT.709 limited-range black. Work in the composed delivery planes,
        // matching ffmpeg fade on yuv420p, with nearest-integer rounding.
        for (plane, black) in [(&mut yuv.y, 16i64), (&mut yuv.u, 128), (&mut yuv.v, 128)] {
            for value in plane {
                let delta = (i64::from(*value) - black) * gain as i64;
                let rounded = (delta.abs() + denominator as i64 / 2) / denominator as i64;
                *value = (black + rounded * delta.signum()).clamp(0, 255) as u8;
            }
        }
    }
    yuv
}
#[derive(Deserialize, Default)]
#[serde(try_from = "SegmentSpec")]
struct Segment {
    path: PathBuf,
    from_slot: u64,
    to_slot: u64,
    at_s: f64,
    solid: Option<[u8; 3]>,
    still: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SegmentSpec {
    path: Option<PathBuf>,
    still: Option<PathBuf>,
    solid: Option<serde_json::Value>,
    from_slot: Option<u64>,
    to_slot: Option<u64>,
    duration_s: Option<f64>,
    at_s: f64,
}
impl TryFrom<SegmentSpec> for Segment {
    type Error = String;
    fn try_from(s: SegmentSpec) -> std::result::Result<Self, String> {
        if usize::from(s.path.is_some())
            + usize::from(s.still.is_some())
            + usize::from(s.solid.is_some())
            != 1
        {
            return Err("render: segment_kind".into());
        }
        let mut out = Self {
            at_s: s.at_s,
            ..Self::default()
        };
        if let Some(path) = s.path {
            if s.duration_s.is_some() {
                return Err("render: segment_kind_fields".into());
            }
            out.path = path;
            out.from_slot = s.from_slot.ok_or("render: segment_from_slot")?;
            out.to_slot = s.to_slot.ok_or("render: segment_to_slot")?;
        } else {
            if s.from_slot.is_some() || s.to_slot.is_some() {
                return Err("render: segment_kind_fields".into());
            }
            let duration = s.duration_s.ok_or("render: segment_duration_s")?;
            if !time(duration) || frame(duration) == 0 || !time(s.at_s + duration) {
                return Err("render: segment_duration_s".into());
            }
            out.to_slot = frame(s.at_s + duration)
                .checked_sub(frame(s.at_s))
                .filter(|n| *n > 0)
                .ok_or("render: segment_duration_s")?;
            if let Some(path) = s.still {
                out.path = path;
                out.still = true;
            }
            if let Some(colour) = s.solid {
                out.solid = Some(if colour == "black" {
                    [0, 0, 0]
                } else {
                    serde_json::from_value::<[u8; 3]>(colour).map_err(|_| "render: solid_colour")?
                });
            }
        }
        Ok(out)
    }
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
        self.validate_sources(base, &mut SourceCatalog::default())
    }
    fn validate_sources(
        &mut self,
        base: &Path,
        catalog: &mut SourceCatalog,
    ) -> Result<(Vec<u64>, Directories)> {
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
        for duration in [v.fade_in_s, v.fade_out_s] {
            if !time(duration)
                || duration > self.duration_s
                || (duration > 0.0 && frame(duration) < 2)
            {
                return refuse("programme_fade_range");
            }
        }
        if v.fade_in_s + v.fade_out_s > self.duration_s
            || frame(v.fade_in_s) + frame(v.fade_out_s) > frame(self.duration_s)
        {
            return refuse("programme_fade_overlap");
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
        // Pure lexical preflight: reject excess source names before any VFL open.
        let paths = source_paths(&v.segments)?;
        if self.audio.sample_rate != 48000 {
            return refuse("audio_sample_rate");
        }
        if self.audio.channels != 2 {
            return refuse("audio_channels");
        }
        let e = &mut self.encode;
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
        if let Some(cpu) = self.limits.cpu {
            if cpu == 0 || cpu > cores {
                return refuse("cpu_limit");
            }
            clamp_cpu(&mut e.chunks, &mut e.threads, cpu.saturating_sub(1).max(1));
        }
        if self.limits.mem.is_none()
            && e.chunks
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
        for (s, relative) in v.segments.iter().zip(paths) {
            if !time(s.at_s) || s.from_slot >= s.to_slot || frame(s.at_s) >= frame(self.duration_s)
            {
                return refuse("segment_range");
            }
            let (input, w, h) = if let Some(rgb) = s.solid {
                (Input::Solid(rgb), v.width as u64, v.height as u64)
            } else if s.still {
                if let Some(source) = catalog.stills.get(&relative) {
                    source.clone()
                } else {
                    let source = open_still(base, &relative)?;
                    catalog.stills.insert(relative, source.clone());
                    source
                }
            } else {
                let source = catalog.get(base, &relative)?;
                if s.from_slot >= source.count || s.to_slot > source.count {
                    return refuse("segment_trim_outside_vfl");
                }
                (
                    Input::Vfl(source.directory.clone()),
                    source.width,
                    source.height,
                )
            };
            source_bytes = source_bytes.max(source_canvas_bytes(w, h)?);
            let ratio = (v.width as f64 / w as f64).min(v.height as f64 / h as f64);
            let sw = ((w as f64 * ratio).round() as u64).clamp(1, v.width as u64);
            scale_bytes = scale_bytes.max(
                sw.checked_mul(h)
                    .and_then(|n| n.checked_mul(16))
                    .ok_or("render: encode_memory_budget")?,
            );
            directories.push(input);
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
        if !catalog.stills.is_empty() {
            // PNG ancillary chunks and inflate scratch have a separate bounded
            // allowance, even for a tiny image with large metadata chunks.
            scale_bytes = scale_bytes
                .checked_add(STILL_DECODE_BYTES_LIMIT as u64)
                .ok_or("render: encode_memory_budget")?;
        }
        // Conservative admission estimate, including packet/position metadata,
        // two VFL canvases with payload/scratch, scaler and encoder frame storage.
        let working = working_bytes(source_bytes, scale_bytes, v.width as u64, v.height as u64)?;
        let audio_peak = media_audio::timeline::peak_memory_bytes(
            self.duration_s, &self.audio.sources, base,
        )?;
        let budget = self.limits.mem.unwrap_or(8 * 1024 * 1024 * 1024)
            .checked_sub(audio_peak).ok_or("render: encode_memory_budget")?;
        if let Some(mem) = self.limits.mem {
            if mem < audio_peak || budget < working {
                return refuse("encode_memory_budget");
            }
            e.chunks = e.chunks.min((budget / working).min(64) as usize);
        }
        if working
            .checked_mul(e.chunks as u64)
            .is_none_or(|n| n > budget)
        {
            return refuse("encode_memory_budget");
        }
        if e.chunks * e.threads > cores.saturating_mul(2) {
            return refuse("encode_thread_budget");
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
    reader: Option<media_vfl::reader::Reader>,
    next: u64,
    current: Option<Arc<Bgra>>,
}
fn scale_frame(b: Bgra, width: usize, height: usize, stats: &mut ScaleStats) -> Result<Bgra> {
    let (w, h) = (b.width(), b.height());
    if w == width && h == height {
        return Ok(b);
    }
    let ratio = (width as f64 / w as f64).min(height as f64 / h as f64);
    let sw = ((w as f64 * ratio).round() as usize).clamp(1, width);
    let sh = ((h as f64 * ratio).round() as usize).clamp(1, height);
    let start = Instant::now();
    let scaled = b.resample(sw, sh)?;
    stats.frames += 1;
    stats.wall_s += start.elapsed().as_secs_f64();
    Ok(scaled.pad(
        width,
        height,
        (width - sw) / 2,
        (height - sh) / 2,
        [0, 0, 0, 255],
    )?)
}
impl Source {
    fn new(input: Input, width: usize, height: usize) -> Result<Self> {
        let (reader, current) = match input {
            Input::Vfl(directory) => (Some(Reader::from_directory(directory)?), None),
            Input::Solid(rgb) => (
                None,
                Some(Arc::new(Bgra::solid(
                    width,
                    height,
                    [rgb[2], rgb[1], rgb[0], 255],
                )?)),
            ),
            Input::Still(file) => {
                // Each chunk has an independent open file description: cloned FDs share offsets.
                let independent =
                    std::fs::File::open(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
                let mut r = png_reader(&independent)?;
                let size = r
                    .output_buffer_size()
                    .filter(|n| *n <= STILL_DECODE_BYTES_LIMIT)
                    .ok_or("render: still_decode_bytes_limit")?;
                let mut pixels = vec![0; size];
                let info = r.next_frame(&mut pixels)?;
                let mut bgra = Vec::with_capacity(info.width as usize * info.height as usize * 4);
                let channels = info.color_type.samples();
                for p in pixels[..info.buffer_size()].chunks_exact(channels) {
                    let (red, green, blue, alpha) = match info.color_type {
                        png::ColorType::Rgb => (p[0], p[1], p[2], 255),
                        png::ColorType::Rgba => (p[0], p[1], p[2], p[3]),
                        png::ColorType::Grayscale => (p[0], p[0], p[0], 255),
                        png::ColorType::GrayscaleAlpha => (p[0], p[0], p[0], p[1]),
                        _ => return refuse("still_colour_type"),
                    };
                    // Composite transparency onto the programme's black canvas.
                    for c in [blue, green, red] {
                        bgra.push(((u16::from(c) * u16::from(alpha) + 127) / 255) as u8);
                    }
                    bgra.push(255);
                }
                (
                    None,
                    Some(Arc::new(Bgra::new(
                        info.width as usize,
                        info.height as usize,
                        bgra,
                    )?)),
                )
            }
        };
        Ok(Self {
            reader,
            next: u64::MAX,
            current,
        })
    }
    fn get(
        &mut self,
        slot: u64,
        width: usize,
        height: usize,
        stats: &mut ScaleStats,
    ) -> Result<(Arc<Bgra>, bool)> {
        if self.reader.is_none() {
            let repeat = self.next != u64::MAX;
            if !repeat {
                let b = Arc::unwrap_or_clone(self.current.take().ok_or("missing_frame")?);
                self.current = Some(Arc::new(scale_frame(b, width, height, stats)?));
                self.next = 0;
            }
            return Ok((
                self.current.as_ref().ok_or("missing_frame")?.clone(),
                repeat,
            ));
        }
        let reader = self.reader.as_mut().ok_or("missing_reader")?;
        let after = slot.checked_add(1).ok_or("render: slot_arithmetic")?;
        if self.next == after {
            return Ok((self.current.as_ref().ok_or("missing_frame")?.clone(), true));
        }
        let w = reader.width() as usize;
        let h = reader.height() as usize;
        let f = if self.next == slot {
            reader.read()?.ok_or("segment_trim_outside_vfl")?
        } else {
            reader.seek(slot)?
        };
        let repeat = f.repeat && self.current.is_some();
        if !repeat {
            let b = Bgra::new(w, h, f.pixels.to_vec())?;
            self.current = Some(Arc::new(scale_frame(b, width, height, stats)?));
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
    directories: &[Input],
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
        sources[j] = Some(Source::new(directories[j].clone(), v.width, v.height)?);
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
            sources[j - 1] = Some(Source::new(directories[j - 1].clone(), v.width, v.height)?);
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
        if !repeat || programme_gain(v, k, frame(t.duration_s)).is_some() {
            yuv = Some(programme_picture(v, &b, k, frame(t.duration_s)));
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
fn chunk(
    t: &Timeline,
    directories: &[Input],
    start: u64,
    end: u64,
    progress: &media_audio::progress::Progress,
) -> Result<Chunk> {
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
            directories,
            &mut sources,
            &mut fade_index,
            k,
            &mut scaling,
        )?;
        // The displayed keyframe is not showable. Submit its first successor too.
        if !repeat || k < start + 2 || programme_gain(v, k, frame(t.duration_s)).is_some() {
            let p = enc.prepare(&programme_picture(v, &b, k, frame(t.duration_s)))?;
            let packets = enc.submit(&p)?;
            progress.frames(0, packets.len() as u64);
            collect_packets(&mut encoded, &mut encoded_bytes, packets)?;
            positions.push(Some(k));
        } else {
            positions.push(None);
        }
        progress.frames(1, 0);
    }
    let packets = enc.finish()?;
    progress.frames(0, packets.len() as u64);
    collect_packets(&mut encoded, &mut encoded_bytes, packets)?;
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
struct VideoResult {
    config: Vec<u8>,
    packets: u64,
    encoded: usize,
    scaling: ScaleStats,
    wall_s: f64,
}

// Return both outcomes only after both stages have ended, including failures.
fn join_stages<A: Send, V>(
    serial: bool,
    audio: impl FnOnce() -> std::result::Result<A, String> + Send,
    video: impl FnOnce() -> Result<V>,
) -> (std::result::Result<A, String>, Result<V>) {
    if serial {
        let a = audio();
        (a, video())
    } else {
        std::thread::scope(|scope| {
            let audio = scope.spawn(audio);
            let video = video();
            let audio = audio.join().map_err(|_| "audio_worker_panicked".to_string())
                .and_then(|r| r);
            (audio, video)
        })
    }
}

fn encode_video(
    t: &Timeline,
    directories: &[Input],
    starts: &[u64],
    progress: &media_audio::progress::Progress,
    spool: &mut std::fs::File,
) -> Result<VideoResult> {
    let wall = Instant::now();
    let chunk_count = starts.len() - 1;
    progress.event("video", "start", serde_json::json!({"frames_total":frame(t.duration_s),"chunks_total":chunk_count}));
    let mut result = VideoResult {
        config: Vec::new(), packets: 0, encoded: 0,
        scaling: ScaleStats::default(), wall_s: 0.0,
    };
    let mut spool = BufWriter::new(spool);
    for ranges in starts.windows(2).collect::<Vec<_>>().chunks(t.encode.chunks) {
        let batch = std::thread::scope(|scope| {
            let handles: Vec<_> = ranges.iter().map(|r| {
                let index = starts.binary_search(&r[0]).unwrap();
                scope.spawn(move || {
                    progress.event("video", "start", serde_json::json!({"chunk":index,"chunks_total":chunk_count}));
                    let result = chunk(t, directories, r[0], r[1], progress).map_err(|e| e.to_string());
                    progress.event("video", "end", serde_json::json!({"chunk":index,"chunks_total":chunk_count}));
                    result
                })
            }).collect();
            handles.into_iter().map(|h| h.join().map_err(|_| "chunk_worker_panicked".to_string())?)
                .collect::<std::result::Result<Vec<_>, String>>()
        })?;
        for chunk in batch {
            if result.config.is_empty() {
                result.config = chunk.config;
            } else if result.config != chunk.config {
                return refuse("sequence_header_mismatch");
            }
            result.encoded += chunk.encoded;
            result.scaling.frames += chunk.scaling.frames;
            result.scaling.wall_s += chunk.scaling.wall_s;
            for p in chunk.packets {
                spool.write_all(&p.pts.to_le_bytes())?;
                spool.write_all(&(p.data.len() as u64).to_le_bytes())?;
                spool.write_all(&[u8::from(p.key_frame)])?;
                spool.write_all(&p.data)?;
                result.packets += 1;
            }
        }
    }
    spool.flush()?;
    result.wall_s = wall.elapsed().as_secs_f64();
    progress.event("video", "end", serde_json::json!({}));
    Ok(result)
}

fn thread_cpu() -> f64 {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: valid writable timespec, read only after successful clock call.
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut time) } != 0 {
        return 0.0;
    }
    time.tv_sec as f64 + time.tv_nsec as f64 * 1e-9
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
fn peak_rss_kib() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}
pub fn run(args: impl Iterator<Item = OsString>) -> Result<u8> {
    let (args, progress) = media_audio::progress::arguments(args);
    let result = run_inner(args, &progress);
    progress.done(&result);
    result
}
fn run_inner(args: Vec<OsString>, progress: &media_audio::progress::Progress) -> Result<u8> {
    let overall = Instant::now();
    let total_cpu = cpu();
    progress.event("input", "start", serde_json::json!({}));
    let path = Path::new(args.first().ok_or("render: missing timeline")?);
    let mut out = None;
    let mut limits = Limits::default();
    let mut i = 1;
    while i < args.len() {
        let value = args.get(i + 1).ok_or("missing_option_value")?;
        match args[i].to_str() {
            Some("--out") if out.is_none() => out = Some(Path::new(value)),
            Some("--cpu") if limits.cpu.is_none() => {
                limits.cpu = Some(media_audio::cli::cpu_limit(value)?)
            }
            Some("--mem") if limits.mem.is_none() => limits.mem = Some(memory_limit(value)?),
            _ => return Err("unknown_or_duplicate_option".into()),
        }
        i += 2;
    }
    let out = out.ok_or("render: missing --out")?;
    let mut bytes = Vec::new();
    let (file, base) = opened_timeline(path)?;
    file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return refuse("timeline_size_limit");
    }
    let mut t: Timeline = serde_json::from_slice(&bytes)?;
    t.limits = limits;
    let requested = serde_json::json!({"chunks":t.encode.chunks,"threads":t.encode.threads,"tile_cols":t.encode.tile_cols,"tile_rows":t.encode.tile_rows});
    let (starts, directories) = t.validate(&base)?;
    let audio = media_audio::timeline::Timeline {
        version: t.version,
        duration_s: t.duration_s,
        sample_rate: t.audio.sample_rate,
        channels: t.audio.channels,
        sources: std::mem::take(&mut t.audio.sources),
        master: t.audio.master,
    };
    // Preserve the validated video executor while borrowing the audio specification.
    let input_wall = overall.elapsed().as_secs_f64();
    let input_cpu = cpu() - total_cpu;
    progress.event("input", "end", serde_json::json!({}));
    progress.event("audio", "start", serde_json::json!({"audio_pass":0}));
    let mut pending = media_audio::pending::Pending::new(out)?;
    // Never published: the same pending-file discipline removes the packet spool
    // on success, refusal, I/O error or panic.
    let mut spool = media_audio::pending::Pending::new(out)?;
    let prepare_audio = || {
        let wall = Instant::now();
        let c = thread_cpu();
        let a = media_audio::timeline::prepare(&audio, &base).map_err(|e| e.to_string())?;
        progress.event("audio", "end", serde_json::json!({"audio_pass":a.receipt.codec_passes}));
        Ok::<_, String>((a, wall.elapsed().as_secs_f64(), thread_cpu() - c))
    };
    let (a, video) = join_stages(limits.cpu == Some(1), prepare_audio, || {
        encode_video(&t, &directories, &starts, progress, &mut spool.file)
    });
    let (a, audio_wall, audio_cpu) = a?;
    let video = video?;
    if a.receipt.refused {
        let _ = media_audio::cli::diagnostic(format_args!(
            "mastering refused: encoded {:.3} LUFS, {:.3} dBTP; target {:.3} LUFS, ceiling {:.3} dBTP",
            a.receipt.encoded.integrated_lufs, a.receipt.encoded.true_peak_dbtp,
            audio.master.integrated_lufs, audio.master.true_peak_dbtp
        ));
        return deliver_receipt(&serde_json::to_string(&a.receipt)?, 3);
    }
    drop(a.audio);
    let video_wall = video.wall_s;
    // Process CPU over both joined stages, less the serial audio thread's CPU.
    let video_cpu = (cpu() - total_cpu - input_cpu - audio_cpu).max(0.0);
    let encoded = video.encoded;
    let scaled_frames = video.scaling.frames;
    let scale_wall_s = video.scaling.wall_s;
    let chunk_count = starts.len() - 1;
    let mut keyframes = 0;
    let mut ai = 0;
    let mut mux_wall = 0.0;
    let mut mux_cpu = 0.0;
    use std::io::{Seek, SeekFrom};
    spool.file.seek(SeekFrom::Start(0))?;
    let mut spool_reader = io::BufReader::new(&mut spool.file);
    progress.event("mux", "start", serde_json::json!({}));
    let wall = Instant::now();
    let c = cpu();
    let mut writer = Some(media_webm::Writer::new(
        BufWriter::new(pending.file.try_clone()?),
        media_webm::Video {
            width: t.video.width as u32, height: t.video.height as u32,
            codec_private: video.config,
        },
        Some(media_webm::Opus {
            channels: a.head.channels, pre_skip: a.head.pre_skip,
            input_sample_rate: a.head.input_sample_rate,
        }),
    )?);
    for _ in 0..video.packets {
        let mut header = [0u8; 17];
        spool_reader.read_exact(&mut header)?;
        let pts = u64::from_le_bytes(header[..8].try_into().unwrap());
        let len = u64::from_le_bytes(header[8..16].try_into().unwrap()) as usize;
        if len > CHUNK_BYTES_LIMIT { return refuse("spool_packet_limit"); }
        let mut data = vec![0; len];
        spool_reader.read_exact(&mut data)?;
        let ms = pts * 1000 / 30;
        let w = writer.as_mut().ok_or("missing_muxer")?;
        while ai < a.packets.len() && ai as u64 * 20 <= ms {
            let packet = &a.packets[ai];
            w.opus_packet(ai as u64 * 20, &packet.data, packet.discard_padding)?;
            ai += 1;
        }
        let key = header[16] != 0;
        keyframes += usize::from(key);
        w.packet(1, ms, (pts + 1) * 1000 / 30 - ms, key, &data)?;
    }
    mux_wall += wall.elapsed().as_secs_f64();
    mux_cpu += cpu() - c;
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
    progress.event("mux", "end", serde_json::json!({"bytes":size}));
    let n = frame(t.duration_s);
    let mut receipt = serde_json::json!({"peak_rss_kib":peak_rss_kib(), "scaling": {"filter":"bicubic_catmull_rom_gamma","frames":scaled_frames,"sum_wall_s":scale_wall_s,"mean_ms":scale_wall_s*1000.0/scaled_frames.max(1) as f64}, "frames_out": n, "frames_encoded": encoded, "repeats": n-encoded as u64, "keyframes": keyframes, "chunks": chunk_count, "parallel_chunks": t.encode.chunks, "stages": {"input": {"wall_s": input_wall, "cpu_s": input_cpu}, "audio": {"wall_s": audio_wall, "cpu_s": audio_cpu}, "video": {"wall_s": video_wall, "cpu_s": video_cpu}, "mux": {"wall_s": mux_wall, "cpu_s": mux_cpu}}, "wall_s": overall.elapsed().as_secs_f64(), "cpu_s": cpu()-total_cpu, "effective_fps": n as f64/overall.elapsed().as_secs_f64(), "bytes": size, "audio": a.receipt});
    receipt["limits"] = serde_json::json!({"cpu":limits.cpu,"mem_bytes":limits.mem.unwrap_or(8 * 1024 * 1024 * 1024)});
    receipt["encode_requested"] = requested;
    receipt["encode_effective"] = serde_json::json!({"chunks":t.encode.chunks,"threads":t.encode.threads,"tile_cols":t.encode.tile_cols,"tile_rows":t.encode.tile_rows});
    let receipt = serde_json::to_string(&receipt)?;
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
    #[test]
    fn either_failure_joins_the_other_stage_and_removes_staging() {
        use std::sync::{Barrier, atomic::{AtomicBool, Ordering}};
        for audio_fails in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let out = root.path().join("out.webm");
            let ended = AtomicBool::new(false);
            let barrier = Barrier::new(2);
            {
                let _pending = media_audio::pending::Pending::new(&out).unwrap();
                let _spool = media_audio::pending::Pending::new(&out).unwrap();
                let (a, v) = join_stages(false, || {
                    barrier.wait();
                    if audio_fails { Err("audio failure".to_string()) }
                    else { ended.store(true, Ordering::SeqCst); Ok(()) }
                }, || {
                    barrier.wait();
                    if audio_fails { ended.store(true, Ordering::SeqCst); Ok(()) }
                    else { Err("video failure".into()) }
                });
                assert_eq!(a.is_err(), audio_fails);
                assert_eq!(v.is_err(), !audio_fails);
                assert!(ended.load(Ordering::SeqCst));
            }
            assert!(!out.exists());
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        }
    }
    #[test]
    fn caller_limits_clamp_and_refuse() {
        let (mut chunks, mut threads) = (4, 8);
        clamp_cpu(&mut chunks, &mut threads, 16);
        assert_eq!((chunks, threads), (2, 8));
        clamp_cpu(&mut chunks, &mut threads, 1);
        assert_eq!((chunks, threads), (1, 1));
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        t.encode.chunks = 2;
        t.encode.threads = 8;
        t.limits.cpu = Some(1);
        t.validate(root.path()).unwrap();
        assert_eq!((t.encode.chunks, t.encode.threads), (1, 1));
        t.encode.chunks = 2;
        let working = working_bytes(4096, 16384, 32, 32).unwrap();
        t.limits.cpu = None;
        t.limits.mem = Some(working + working / 2);
        t.validate(root.path()).unwrap();
        assert_eq!(t.encode.chunks, 1);
        t.limits.mem = Some(working - 1);
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("encode_memory_budget")
        );
        let audio_peak = media_audio::timeline::peak_memory_bytes(t.duration_s, &t.audio.sources, root.path()).unwrap();
        t.limits.mem = Some(working + audio_peak - 1);
        assert!(t.validate(root.path()).is_err());
        t.limits.mem = Some(working + audio_peak);
        t.validate(root.path()).unwrap();
        for n in [0, std::thread::available_parallelism().unwrap().get() + 1] {
            assert!(media_audio::cli::cpu_limit(std::ffi::OsStr::new(&n.to_string())).is_err());
        }
        for (s, n) in [
            ("10", 10),
            ("2K", 2048),
            ("3M", 3 * 1024 * 1024),
            ("8G", 8 * 1024 * 1024 * 1024),
        ] {
            assert_eq!(memory_limit(std::ffi::OsStr::new(s)).unwrap(), n);
        }
        for s in ["", "-1", "x", "G", "18446744073709551615G"] {
            assert!(memory_limit(std::ffi::OsStr::new(s)).is_err());
        }
    }
    #[test]
    fn programme_envelope_validation_and_endpoints() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        for (incoming, outgoing, refusal) in [
            (-0.1, 0.0, "programme_fade_range"),
            (0.0, -0.1, "programme_fade_range"),
            (4.1, 0.0, "programme_fade_range"),
            (0.0, 4.1, "programme_fade_range"),
            (0.01, 0.0, "programme_fade_range"),
            (f64::NAN, 0.0, "programme_fade_range"),
            (0.0, f64::INFINITY, "programme_fade_range"),
            (2.1, 2.0, "programme_fade_overlap"),
        ] {
            t.video.fade_in_s = incoming;
            t.video.fade_out_s = outgoing;
            assert!(
                t.validate(root.path())
                    .unwrap_err()
                    .to_string()
                    .contains(refusal)
            );
        }
        t.video.fade_in_s = 2.0;
        t.video.fade_out_s = 2.0;
        t.validate(root.path()).unwrap(); // touching is allowed
        t.duration_s = 4.016;
        t.video.fade_in_s = 2.017;
        t.video.fade_out_s = 1.999;
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("programme_fade_overlap")
        );
        t.duration_s = 4.0;
        t.video.fade_in_s = 0.6;
        t.video.fade_out_s = 1.0;
        t.validate(root.path()).unwrap(); // ordinary crossfades remain valid too
        assert_eq!(programme_gain(&t.video, 0, 120), Some((0, 17)));
        assert_eq!(programme_gain(&t.video, 17, 120), Some((17, 17)));
        assert_eq!(programme_gain(&t.video, 18, 120), None);
        assert_eq!(programme_gain(&t.video, 90, 120), Some((29, 29)));
        assert_eq!(programme_gain(&t.video, 119, 120), Some((0, 29)));
        t.video.fade_in_s = 0.0;
        t.video.fade_out_s = 0.0;
        for k in 0..120 {
            assert_eq!(programme_gain(&t.video, k, 120), None);
        }
    }
    fn png_file(path: &Path, width: u32, height: u32) {
        let file = std::fs::File::create(path).unwrap();
        let mut e = png::Encoder::new(file, width, height);
        e.set_color(png::ColorType::Rgba);
        let mut w = e.write_header().unwrap();
        if u64::from(width) * u64::from(height) <= STILL_PIXELS_LIMIT {
            w.write_image_data(&[100, 80, 60, 128].repeat(width as usize * height as usize))
                .unwrap();
        } else {
            w.write_chunk(png::chunk::IDAT, &[]).unwrap();
        }
    }
    #[test]
    fn segment_kinds_refuse_ambiguous_fields_colours_and_durations() {
        for value in [
            json!({"solid":"red","at_s":0,"duration_s":1}),
            json!({"solid":[256,0,0],"at_s":0,"duration_s":1}),
            json!({"solid":[0,0],"at_s":0,"duration_s":1}),
            json!({"solid":"black","at_s":0,"duration_s":0}),
            json!({"solid":"black","at_s":0,"duration_s":-1}),
            json!({"solid":"black","at_s":0,"duration_s":10801}),
            json!({"still":"a.png","at_s":0}),
            json!({"still":"a.png","at_s":0,"duration_s":0.001}),
            json!({"still":"a.png","solid":"black","at_s":0,"duration_s":1}),
            json!({"solid":"black","path":"a.vfl","at_s":0,"duration_s":1}),
            json!({"still":"a.png","from_slot":0,"at_s":0,"duration_s":1}),
            json!({"solid":"black","to_slot":1,"at_s":0,"duration_s":1}),
        ] {
            assert!(
                serde_json::from_value::<Segment>(value.clone()).is_err(),
                "{value}"
            );
        }
    }
    #[test]
    fn solid_needs_no_files_and_repeats_without_scaling() {
        let mut t = fixture(tempfile::tempdir().unwrap().path());
        t.video.crossfades.clear();
        t.video.segments = vec![
            serde_json::from_value(json!({"solid":[10,20,30],"at_s":0,"duration_s":0.6})).unwrap(),
        ];
        let (_, inputs) = t.validate(Path::new("/does-not-exist")).unwrap();
        assert_eq!(t.video.segments[0].to_slot, 18);
        let mut source = Source::new(inputs[0].clone(), 32, 32).unwrap();
        let mut stats = ScaleStats::default();
        let (first, repeat) = source.get(0, 32, 32, &mut stats).unwrap();
        assert!(!repeat);
        assert_eq!(&first.pixels()[..4], &[30, 20, 10, 255]);
        let (held, repeat) = source.get(17, 32, 32, &mut stats).unwrap();
        assert!(repeat && Arc::ptr_eq(&first, &held));
        assert_eq!(stats.frames, 0);
    }
    #[test]
    fn still_confinement_limits_retained_descriptor_and_scaler() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        png_file(&root.path().join("a.png"), 16, 8);
        t.video.crossfades.clear();
        t.video.segments =
            vec![serde_json::from_value(json!({"still":"a.png","at_s":0,"duration_s":1})).unwrap()];
        let (_, inputs) = t.validate(root.path()).unwrap();
        std::fs::rename(root.path().join("a.png"), root.path().join("saved.png")).unwrap();
        std::os::unix::fs::symlink("/dev/zero", root.path().join("a.png")).unwrap();
        let mut source = Source::new(inputs[0].clone(), 32, 32).unwrap();
        let mut stats = ScaleStats::default();
        let (b, repeat) = source.get(0, 32, 32, &mut stats).unwrap();
        assert!(!repeat);
        assert_eq!(b.width(), 32);
        assert_eq!(&b.pixels()[..4], &[0, 0, 0, 255]); // aspect padding
        assert_eq!(
            &b.pixels()[(16 * 32 + 16) * 4..(16 * 32 + 16) * 4 + 4],
            &[30, 40, 50, 255]
        );
        assert!(source.get(1, 32, 32, &mut stats).unwrap().1);
        assert_eq!(stats.frames, 1);
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("still_regular_file")
        );
        for path in ["../escape.png", "/absolute.png", ""] {
            t.video.segments[0].path = path.into();
            assert!(
                t.validate(root.path())
                    .unwrap_err()
                    .to_string()
                    .contains("segment_path_relative")
            );
        }
        let outside = tempfile::tempdir().unwrap();
        png_file(&outside.path().join("outside.png"), 16, 16);
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
        t.video.segments[0].path = "escape/outside.png".into();
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("segment_path_escape")
        );
        let huge = std::fs::File::create(root.path().join("huge.png")).unwrap();
        huge.set_len(STILL_FILE_BYTES_LIMIT + 1).unwrap();
        t.video.segments[0].path = "huge.png".into();
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("still_file_bytes_limit")
        );
        png_file(
            &root.path().join("dimensions.png"),
            STILL_DIMENSION_LIMIT + 1,
            4320,
        );
        t.video.segments[0].path = "dimensions.png".into();
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("still_dimensions_limit")
        );
        t.video.segments = (0..=SOURCE_LIMIT)
            .map(|i| {
                serde_json::from_value(
                    json!({"still":format!("missing-{i}.png"),"at_s":i as f64/30.0,"duration_s":1}),
                )
                .unwrap()
            })
            .collect();
        assert!(
            t.validate(root.path())
                .unwrap_err()
                .to_string()
                .contains("too_many_sources")
        );
    }
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
    fn repeated_vfl_opens_and_scans_once() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        t.duration_s = 334.0;
        t.video.crossfades.clear();
        t.video.segments = (0..10000)
            .map(|i| Segment {
                path: if i % 2 == 0 { "a.vfl" } else { "./a.vfl/" }.into(),
                from_slot: 0,
                to_slot: 1,
                at_s: f64::from(i) / 30.0,
                ..Segment::default()
            })
            .collect();
        let mut catalog = SourceCatalog::default();
        let (_, directories) = t.validate_sources(root.path(), &mut catalog).unwrap();
        assert_eq!(catalog.opens, 1);
        assert_eq!(catalog.scans, 1);
        assert_eq!(directories.len(), 10000);
        let Input::Vfl(first) = &directories[0] else {
            panic!("VFL expected")
        };
        assert!(
            directories
                .iter()
                .all(|d| matches!(d, Input::Vfl(d) if Arc::ptr_eq(d, first)))
        );
        // A new alias still needs a confined open, but shares the retained
        // descriptor and scan after descriptor resolution.
        std::os::unix::fs::symlink(root.path(), root.path().join("alias")).unwrap();
        let alias = catalog.get(root.path(), Path::new("alias/a.vfl")).unwrap();
        assert!(Arc::ptr_eq(&alias.directory, first));
        assert_eq!(catalog.resolved.len(), 1);
        assert_eq!(catalog.scans, 1);
    }
    #[test]
    fn source_limit_precedes_all_directory_opens() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        t.video.segments = (0..=SOURCE_LIMIT)
            .map(|i| Segment {
                path: format!("missing-{i}.vfl").into(),
                from_slot: 0,
                to_slot: 1,
                at_s: i as f64 / 30.0,
                ..Segment::default()
            })
            .collect();
        let mut catalog = SourceCatalog::default();
        assert!(
            t.validate_sources(root.path(), &mut catalog)
                .unwrap_err()
                .to_string()
                .contains("too_many_sources")
        );
        assert_eq!(catalog.opens, 0);
        assert_eq!(catalog.scans, 0);
    }
    #[test]
    fn memory_budget_overflow_is_refused() {
        for result in [
            source_canvas_bytes(u64::MAX, 2),
            source_canvas_bytes(u64::from(u32::MAX), u64::from(u32::MAX)),
            working_bytes(u64::MAX / 8 + 1, 0, 32, 32),
            working_bytes(1, u64::MAX, 32, 32),
            working_bytes(0, 0, u64::MAX, 2),
        ] {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("encode_memory_budget")
            );
        }
        assert_eq!(source_canvas_bytes(32, 32).unwrap(), 4096);
    }
    #[test]
    fn confined_directory_survives_validation_path_swap() {
        let root = tempfile::tempdir().unwrap();
        let mut t = fixture(root.path());
        let (_, directories) = t.validate(root.path()).unwrap();
        std::fs::rename(root.path().join("a.vfl"), root.path().join("original.vfl")).unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("a.vfl")).unwrap();
        let mut source = Source::new(directories[0].clone(), 32, 32).unwrap();
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
            ..Segment::default()
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
