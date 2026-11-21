//! Bench adapter: progressive 8-bit 420 Y4M stdin to streaming AV1 IVF.
use media_av1::{Encoder, Packet, Settings, Target};
use media_frame::Yuv420;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_PIXELS: usize = 8192 * 4320;
const USAGE: &str = "Usage: av1enc [--speed 0..10] [--qp Q | --quantizer Q] [--threads 1..64] [--tile-cols C] [--tile-rows R] [--keyint K] [--out FILE]\nReads progressive 8-bit 420 Y4M stdin; writes IVF. Tile rows/columns are power-of-two counts (1..64), not logarithms. --help prints this usage.";

#[derive(Debug)]
struct Options {
    speed: u8,
    qp: u8,
    threads: usize,
    cols: usize,
    rows: usize,
    keyint: u64,
    out: Option<String>,
    help: bool,
}
impl Options {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self> {
        let mut o = Self {
            speed: 10,
            qp: 100,
            threads: 1,
            cols: 1,
            rows: 1,
            keyint: 240,
            out: None,
            help: false,
        };
        let mut args = args;
        while let Some(flag) = args.next() {
            if flag == "--help" {
                o.help = true;
                continue;
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--speed" => o.speed = value.parse()?,
                "--qp" | "--quantizer" => o.qp = value.parse()?,
                "--threads" => o.threads = value.parse()?,
                "--tile-cols" => o.cols = value.parse()?,
                "--tile-rows" => o.rows = value.parse()?,
                "--keyint" => o.keyint = value.parse()?,
                "--out" => o.out = Some(value),
                _ => return Err(format!("unknown option {flag}").into()),
            }
        }
        if o.speed > 10
            || !(1..=64).contains(&o.threads)
            || o.keyint == 0
            || !o.cols.is_power_of_two()
            || !o.rows.is_power_of_two()
            || o.cols > 64
            || o.rows > 64
        {
            return Err("invalid speed, threads, tile counts or key interval".into());
        }
        Ok(o)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Header {
    width: usize,
    height: usize,
    num: u32,
    den: u32,
}

// Bound metadata reads, including when the input never supplies a newline.
fn line(reader: &mut impl BufRead) -> Result<Option<String>> {
    let mut bytes = Vec::new();
    let n = reader.take(4097).read_until(b'\n', &mut bytes)?;
    if n == 0 {
        return Ok(None);
    }
    if n > 4096 || bytes.last() != Some(&b'\n') {
        return Err("truncated or oversized Y4M header".into());
    }
    bytes.pop();
    Ok(Some(String::from_utf8(bytes)?))
}

impl Header {
    fn parse(text: &str) -> Result<Self> {
        let mut fields = text.split_whitespace();
        if fields.next() != Some("YUV4MPEG2") {
            return Err("expected YUV4MPEG2 header".into());
        }
        let (mut w, mut h, mut rate, mut chroma, mut interlace) = (None, None, None, None, None);
        let mut aspect = None;
        for field in fields {
            if !field.is_ascii() {
                return Err("non-ASCII Y4M field".into());
            }
            let (key, value) = field.split_at(1);
            if matches!(key, "W" | "H" | "F")
                && !value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (key == "F" && b == b':'))
            {
                return Err("Y4M sizes and frame rate must be unsigned decimal digits".into());
            }
            match key {
                "W" if w.is_none() => w = Some(value.parse::<usize>()?),
                "H" if h.is_none() => h = Some(value.parse::<usize>()?),
                "F" if rate.is_none() => {
                    let (a, b) = value.split_once(':').ok_or("invalid Y4M frame rate")?;
                    rate = Some((a.parse::<u32>()?, b.parse::<u32>()?));
                }
                "C" if chroma.is_none() => chroma = Some(value),
                "I" if interlace.is_none() => interlace = Some(value),
                "A" if aspect.is_none() && matches!(value, "1:1" | "0:0") => aspect = Some(value),
                "X" if field.starts_with("XBITDEPTH=") && field != "XBITDEPTH=8" => {
                    return Err("Y4M requires 8-bit samples".into());
                }
                "X" if field == "XCOLORRANGE=LIMITED" => {}
                "X" if !field.starts_with("XCOLORRANGE=") => {}
                _ => return Err(format!("unsupported or duplicate Y4M field {field}").into()),
            }
        }
        if !matches!(chroma, Some("420" | "420jpeg" | "420mpeg2" | "420paldv")) {
            return Err("Y4M requires explicit 4:2:0 8-bit chroma".into());
        }
        if interlace != Some("p") {
            return Err("Y4M requires progressive frames (Ip)".into());
        }
        let (num, den) = rate.ok_or("missing Y4M frame rate")?;
        let s = Self {
            width: w.ok_or("missing Y4M width")?,
            height: h.ok_or("missing Y4M height")?,
            num,
            den,
        };
        if !(16..=65535).contains(&s.width)
            || !(16..=65535).contains(&s.height)
            || num == 0
            || den == 0
            || s.width.checked_mul(s.height).is_none_or(|n| n > MAX_PIXELS)
        {
            return Err("unsupported Y4M dimensions or frame rate (pixel cap: 8192*4320)".into());
        }
        Ok(s)
    }
    fn frame(self, reader: &mut impl BufRead) -> Result<Option<Yuv420>> {
        let Some(marker) = line(reader)? else {
            return Ok(None);
        };
        if marker != "FRAME" {
            // No per-frame size/rate/interlace overrides: CFR and fixed size only.
            return Err("expected FRAME; per-frame overrides, size/rate changes and concatenated streams are unsupported".into());
        }
        fn plane(reader: &mut impl Read, len: usize) -> Result<Vec<u8>> {
            let mut data = Vec::new();
            reader
                .take(len as u64)
                .read_to_end(&mut data)
                .map_err(|e| format!("truncated Y4M frame: {e}"))?;
            if data.len() != len {
                return Err("truncated Y4M frame".into());
            }
            Ok(data)
        }
        let y = plane(reader, self.width * self.height)?;
        let c = self.width.div_ceil(2) * self.height.div_ceil(2);
        Ok(Some(Yuv420 {
            width: self.width,
            height: self.height,
            y,
            u: plane(reader, c)?,
            v: plane(reader, c)?,
        }))
    }
}

fn ivf_header(out: &mut impl Write, h: Header) -> Result<()> {
    out.write_all(b"DKIF\0\0\x20\0AV01")?;
    out.write_all(&(h.width as u16).to_le_bytes())?;
    out.write_all(&(h.height as u16).to_le_bytes())?;
    for n in [h.num, h.den, 0, 0] {
        out.write_all(&n.to_le_bytes())?;
    }
    Ok(())
}
fn packets(out: &mut impl Write, packets: Vec<Packet>) -> Result<(u64, u32)> {
    let mut bytes = 0;
    let mut count = 0u32;
    for p in packets {
        out.write_all(&u32::try_from(p.data.len())?.to_le_bytes())?;
        out.write_all(&p.pts.to_le_bytes())?;
        out.write_all(&p.data)?;
        bytes += 12 + p.data.len() as u64;
        count = count.checked_add(1).ok_or("IVF packet count overflow")?;
    }
    Ok((bytes, count))
}

struct Stats {
    frames: u32,
    bytes: u64,
    seconds: f64,
    header: Header,
    tiles: (usize, usize),
}
fn encode(reader: &mut impl BufRead, out: &mut impl Write, o: &Options) -> Result<Stats> {
    let h = Header::parse(&line(reader)?.ok_or("empty Y4M input")?)?;
    let mut encoder = Encoder::new(Settings {
        width: h.width,
        height: h.height,
        fps_num: h.num,
        fps_den: h.den,
        speed: o.speed,
        target: Target::Quality(o.qp),
        threads: o.threads,
        tile_cols: o.cols,
        tile_rows: o.rows,
        key_interval: o.keyint,
    })?;
    let mut frame = h.frame(reader)?.ok_or("empty Y4M input (no frames)")?;
    ivf_header(out, h)?;
    let mut stats = Stats {
        frames: 0,
        bytes: 32,
        seconds: 0.0,
        header: h,
        tiles: encoder.tiles(),
    };
    let mut packet_count = 0u32;
    loop {
        let start = Instant::now();
        let prepared = encoder.prepare(&frame)?;
        let ready = encoder.submit(&prepared)?;
        stats.seconds += start.elapsed().as_secs_f64();
        let (bytes, count) = packets(out, ready)?;
        stats.bytes += bytes;
        packet_count = packet_count
            .checked_add(count)
            .ok_or("IVF packet count overflow")?;
        stats.frames = stats
            .frames
            .checked_add(1)
            .ok_or("IVF frame count overflow")?;
        match h.frame(reader)? {
            Some(next) => frame = next,
            None => break,
        }
    }
    let start = Instant::now();
    let ready = encoder.finish()?;
    stats.seconds += start.elapsed().as_secs_f64();
    let (bytes, count) = packets(out, ready)?;
    stats.bytes += bytes;
    packet_count = packet_count
        .checked_add(count)
        .ok_or("IVF packet count overflow")?;
    if packet_count != stats.frames {
        return Err("encoded packet count differs from input frame count".into());
    }
    out.flush()?;
    Ok(stats)
}

fn run() -> Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(|s| s.into_string().map_err(|_| "non-UTF-8 argument"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let o = Options::parse(args.into_iter())?;
    if o.help {
        println!("{USAGE}");
        return Ok(());
    }
    let input = io::stdin();
    let s = if let Some(path) = &o.out {
        let mut output = io::BufWriter::new(std::fs::File::create(path)?);
        let stats = encode(&mut input.lock(), &mut output, &o)?;
        output.seek(SeekFrom::Start(24))?;
        output.write_all(&stats.frames.to_le_bytes())?;
        output.flush()?;
        stats
    } else {
        encode(
            &mut input.lock(),
            &mut io::BufWriter::new(io::stdout().lock()),
            &o,
        )?
    };
    eprintln!(
        "{{\"frames\":{},\"bytes\":{},\"encode_seconds\":{:.9},\"settings\":{{\"width\":{},\"height\":{},\"fps_num\":{},\"fps_den\":{},\"speed\":{},\"qp\":{},\"threads\":{},\"tile_cols\":{},\"tile_rows\":{},\"actual_tile_cols\":{},\"actual_tile_rows\":{},\"keyint\":{},\"asm\":{},\"low_latency\":true,\"min_keyint\":{},\"max_keyint\":{},\"actual_threads\":{},\"rav1e_version\":\"0.8.1\"}}}}",
        s.frames,
        s.bytes,
        s.seconds,
        s.header.width,
        s.header.height,
        s.header.num,
        s.header.den,
        o.speed,
        o.qp,
        o.threads,
        o.cols,
        o.rows,
        s.tiles.0,
        s.tiles.1,
        o.keyint,
        cfg!(feature = "asm"),
        o.keyint,
        o.keyint,
        o.threads
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("av1enc: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[path = "av1enc/tests.rs"]
mod tests;
