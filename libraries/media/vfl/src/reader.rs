//! Bounded streaming VFL decoder. A corrupt record terminates the delta chain.
use crate::{KEY, RECT_DELTA, REPEAT};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Metadata,
    Structure,
    Crc,
    Compression,
    Limit,
    TornTail,
    Incomplete,
}
#[derive(Debug)]
pub struct Error {
    pub file: PathBuf,
    pub chunk: Option<u64>,
    pub offset: u64,
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file.display())?;
        if let Some(chunk) = self.chunk {
            write!(f, " chunk {chunk}")?;
        }
        write!(
            f,
            " offset {}: {}: {}",
            self.offset,
            self.kind.name(),
            self.message
        )
    }
}
impl std::error::Error for Error {}
impl ErrorKind {
    fn name(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Metadata => "metadata",
            Self::Structure => "structure",
            Self::Crc => "crc",
            Self::Compression => "compression",
            Self::Limit => "limit",
            Self::TornTail => "torn_tail",
            Self::Incomplete => "incomplete",
        }
    }
}
impl Error {
    fn json(&self) -> Value {
        json!({"kind": self.kind.name(), "file": self.file.to_string_lossy(),
            "chunk": self.chunk, "offset": self.offset, "message": self.message})
    }
}
fn completion(path: &Path) -> Result<Option<Value>, Error> {
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(err(path, None, 0, ErrorKind::Io, e)),
        Ok(_) => match json_file(path) {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.kind == ErrorKind::Metadata => Ok(None),
            Err(e) => Err(e),
        },
    }
}
fn err(
    path: &Path,
    chunk: Option<u64>,
    offset: u64,
    kind: ErrorKind,
    message: impl ToString,
) -> Error {
    Error {
        file: path.into(),
        chunk,
        offset,
        kind,
        message: message.to_string(),
    }
}
fn open(path: &Path) -> Result<File, Error> {
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|e| err(path, None, 0, ErrorKind::Io, e))?;
    if !f
        .metadata()
        .map_err(|e| err(path, None, 0, ErrorKind::Io, e))?
        .is_file()
    {
        return Err(err(
            path,
            None,
            0,
            ErrorKind::Structure,
            "expected regular file",
        ));
    }
    Ok(f)
}
fn json_file(path: &Path) -> Result<Value, Error> {
    let mut data = Vec::new();
    open(path)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|e| err(path, None, 0, ErrorKind::Io, e))?;
    if data.len() > 1024 * 1024 {
        return Err(err(
            path,
            None,
            0,
            ErrorKind::Limit,
            "metadata exceeds 1 MiB",
        ));
    }
    serde_json::from_slice(&data).map_err(|e| err(path, None, 0, ErrorKind::Metadata, e))
}
fn number(v: &Value, key: &str, path: &Path) -> Result<u64, Error> {
    v[key]
        .as_u64()
        .ok_or_else(|| err(path, None, 0, ErrorKind::Metadata, format!("invalid {key}")))
}
/// Read one JSON line with a hard limit, including when there is no newline.
fn line(input: &mut BufReader<File>, path: &Path, recovery: bool) -> Result<Option<Value>, Error> {
    let at = input.stream_position().unwrap_or(0);
    let mut b = Vec::new();
    (&mut *input)
        .take(1024 * 1024 + 1)
        .read_until(b'\n', &mut b)
        .map_err(|e| err(path, None, at, ErrorKind::Io, e))?;
    if b.is_empty() {
        return Ok(None);
    }
    if b.len() > 1024 * 1024 {
        return Err(err(path, None, at, ErrorKind::Limit, "line exceeds 1 MiB"));
    }
    match serde_json::from_slice(&b) {
        Ok(v) => Ok(Some(v)),
        Err(_)
            if recovery
                && !b.ends_with(b"\n")
                && input
                    .fill_buf()
                    .map_err(|e| err(path, None, at, ErrorKind::Io, e))?
                    .is_empty() =>
        {
            Ok(None)
        }
        Err(e) => Err(err(path, None, at, ErrorKind::Metadata, e)),
    }
}
/// Present rows are always checked; only an absent/torn final row is advisory
/// during recovery. Used by sequential decoding and the seek sidecar skip.
fn frame_row(
    input: &mut BufReader<File>,
    path: &Path,
    slot: u64,
    fps: u32,
    complete: bool,
) -> Result<(), Error> {
    let at = input.stream_position().unwrap_or(0);
    let row = line(input, path, !complete)?;
    let valid = match row {
        Some(v) => {
            v["frame"] == slot
                && v["t_us"] == (u128::from(slot) * 1_000_000 / u128::from(fps)) as u64
        }
        None => !complete,
    };
    if !valid {
        return Err(err(
            path,
            None,
            at,
            ErrorKind::Structure,
            "frame timeline mismatch",
        ));
    }
    Ok(())
}
pub struct Markers {
    input: Option<BufReader<File>>,
    complete: bool,
    path: PathBuf,
    seq: u64,
    last_time: u64,
    last_frame: u64,
    row_offset: u64,
}
impl Markers {
    pub fn open(take: &Path) -> Result<Self, Error> {
        json_file(&take.join("manifest.json"))?;
        let path = take.join("markers.jsonl");
        let complete =
            completion(&take.join("complete.json"))?.is_some_and(|v| v["complete"] == true);
        let input = match open(&path) {
            Ok(f) => Some(BufReader::new(f)),
            Err(_) if !complete && !path.exists() => None,
            Err(e) => return Err(e),
        };
        Ok(Self {
            input,
            complete,
            path,
            seq: 0,
            last_time: 0,
            last_frame: 0,
            row_offset: 0,
        })
    }
    pub fn read(&mut self) -> Result<Option<Value>, Error> {
        let Some(input) = &mut self.input else {
            return Ok(None);
        };
        let at = input.stream_position().unwrap_or(0);
        self.row_offset = at;
        let Some(v) = line(input, &self.path, !self.complete)? else {
            return Ok(None);
        };
        let numeric = |key| {
            number(&v, key, &self.path).map_err(|mut e| {
                e.offset = at;
                e
            })
        };
        let t = numeric("t_us")?;
        let frame = numeric("frame")?;
        if numeric("seq")? != self.seq
            || t < self.last_time
            || frame < self.last_frame
            || !v["label"].is_string()
            || v.get("data").is_none()
        {
            return Err(err(
                &self.path,
                None,
                at,
                ErrorKind::Structure,
                "invalid marker or order",
            ));
        }
        self.seq += 1;
        self.last_time = t;
        self.last_frame = frame;
        Ok(Some(v))
    }
}
pub struct Decoded<'a> {
    pub slot: u64,
    pub t_us: u64,
    pub pixels: &'a [u8],
}
#[derive(Clone)]
struct KeyPosition {
    chunk: u64,
    offset: u64,
    slot: u64,
    crc: crc32fast::Hasher,
    frame_offset: Option<u64>,
}
pub struct Reader {
    path: PathBuf,
    width: u32,
    height: u32,
    fps: u32,
    version: u64,
    /// Completion claimed by metadata; only `verify` reports validated completion.
    complete: bool,
    completion_file: Option<&'static str>,
    origin: Option<Value>,
    receipt: Option<Value>,
    raw: usize,
    pixels: Vec<u8>,
    payload: Vec<u8>,
    scratch: Vec<u8>,
    chunk: u64,
    chunks: u64,
    file: Option<BufReader<File>>,
    offset: u64,
    crc: crc32fast::Hasher,
    next: u64,
    keyed: bool,
    stopped: bool,
    frames: Option<BufReader<File>>,
    keys: Vec<KeyPosition>,
    scanned: u64,
    poisoned: bool,
    torn_tail: bool,
    known_frames: Option<u64>,
    index_keys: bool,
    decoded_from_start: bool,
    #[cfg(test)]
    chunk_reads: Vec<u64>,
}
impl Reader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        let mp = path.join("manifest.json");
        let m = json_file(&mp)?;
        let version = number(&m, "version", &mp)?;
        let width = u32::try_from(number(&m, "width", &mp)?).unwrap_or(0);
        let height = u32::try_from(number(&m, "height", &mp)?).unwrap_or(0);
        let fps = u32::try_from(number(&m, "fps", &mp)?).unwrap_or(0);
        // Reader-side format limits, before allocating, independent of writer helpers.
        let raw = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|n| n.checked_mul(4))
            .filter(|n| *n > 0 && *n <= 256 * 1024 * 1024)
            .filter(|_| (1..=60).contains(&fps))
            .ok_or_else(|| err(&mp, None, 0, ErrorKind::Limit, "invalid canvas or fps"))?
            as usize;
        if ![1, 2].contains(&version)
            || m["format"] != "bgra8"
            || m["encoder"] != "vfl-lz4"
            || m["key_interval"] != u64::from(fps) * 10
        {
            return Err(err(
                &mp,
                None,
                0,
                ErrorKind::Metadata,
                "unsupported manifest",
            ));
        }
        let complete_path = path.join("complete.json");
        let completion_value = completion(&complete_path)?;
        let (receipt, completion_file, complete) =
            if std::fs::symlink_metadata(&complete_path).is_ok() {
                match completion_value {
                    Some(v) if v["complete"] == true => (Some(v), Some("complete.json"), true),
                    _ => (None, Some("complete.json"), false),
                }
            } else if path.join("interrupted.json").exists() {
                (
                    json_file(&path.join("interrupted.json")).ok(),
                    Some("interrupted.json"),
                    false,
                )
            } else {
                (None, None, false)
            };
        let mut origin = match json_file(&path.join("origin.json")) {
            Ok(v) => Some(v),
            Err(e) if complete => return Err(e),
            Err(_) => None,
        };
        if let Some(o) = &origin
            && (o["t0_us"].as_u64().is_none() || o["clock"] != "monotonic")
        {
            if complete {
                return Err(err(
                    &path.join("origin.json"),
                    None,
                    0,
                    ErrorKind::Metadata,
                    "invalid origin",
                ));
            }
            origin = None;
        }
        let mut chunks = 0;
        for entry in std::fs::read_dir(&path).map_err(|e| err(&path, None, 0, ErrorKind::Io, e))? {
            let entry = entry.map_err(|e| err(&path, None, 0, ErrorKind::Io, e))?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(n) = name
                .strip_prefix("chunk-")
                .and_then(|n| n.strip_suffix(".bin"))
            {
                let n: u64 = n.parse().map_err(|_| {
                    err(
                        &entry.path(),
                        None,
                        0,
                        ErrorKind::Structure,
                        "invalid chunk name",
                    )
                })?;
                if name != format!("chunk-{n:06}.bin") || n == u64::MAX {
                    return Err(err(
                        &entry.path(),
                        None,
                        0,
                        ErrorKind::Structure,
                        "invalid chunk number",
                    ));
                }
                chunks = chunks.max(n + 1);
            }
        }
        // Count entries as well as the maximum, so a hostile huge index never causes a huge loop.
        let count = std::fs::read_dir(&path)
            .map_err(|e| err(&path, None, 0, ErrorKind::Io, e))?
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".bin"))
            .count() as u64;
        if (complete && chunks != count) || chunks == 0 {
            return Err(err(
                &path,
                None,
                0,
                ErrorKind::Structure,
                "missing or non-contiguous chunks",
            ));
        }
        let frames = match open(&path.join("frames.jsonl")) {
            Ok(f) => Some(BufReader::new(f)),
            Err(_) if !complete && !path.join("frames.jsonl").exists() => None,
            Err(e) => return Err(e),
        };
        if complete {
            open(&path.join("markers.jsonl"))?;
        }
        Ok(Self {
            path,
            width,
            height,
            fps,
            version,
            complete,
            completion_file,
            origin,
            receipt,
            raw,
            pixels: vec![0; raw],
            payload: Vec::new(),
            scratch: Vec::new(),
            chunk: 0,
            chunks,
            file: None,
            offset: 0,
            crc: crc32fast::Hasher::new(),
            next: 0,
            keyed: false,
            stopped: false,
            frames,
            keys: Vec::new(),
            scanned: 0,
            poisoned: false,
            torn_tail: false,
            known_frames: None,
            index_keys: true,
            decoded_from_start: true,
            #[cfg(test)]
            chunk_reads: vec![0; chunks as usize],
        })
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn fps(&self) -> u32 {
        self.fps
    }
    pub fn complete(&self) -> bool {
        self.complete
    }
    pub fn completion_file(&self) -> Option<&'static str> {
        self.completion_file
    }
    pub fn origin(&self) -> Option<&Value> {
        self.origin.as_ref()
    }
    pub fn recoverable(&self) -> bool {
        self.torn_tail
    }
    pub(crate) fn without_seek_index(mut self) -> Self {
        self.index_keys = false;
        self
    }
    /// Exact decodable frame count, cached after EOF. A first call scans using
    /// a separate decoder, preserving playback position even if the scan fails.
    pub fn frame_count(&mut self) -> Result<u64, Error> {
        if self.poisoned {
            return Err(self.error(ErrorKind::Structure, "reader poisoned"));
        }
        if let Some(n) = self.known_frames {
            return Ok(n);
        }
        let mut scan = Self::open(&self.path)?;
        scan.index_keys = false;
        while scan.read()?.is_some() {}
        self.known_frames = Some(scan.next);
        Ok(scan.next)
    }
    fn chunk_path(&self) -> PathBuf {
        self.path.join(format!("chunk-{:06}.bin", self.chunk))
    }
    fn error(&self, kind: ErrorKind, message: impl ToString) -> Error {
        err(
            &self.chunk_path(),
            Some(self.chunk),
            self.offset,
            kind,
            message,
        )
    }
    fn seal(&self, torn: bool) -> Result<(), Error> {
        let p = self.path.join(format!("chunk-{:06}.json", self.chunk));
        let present = match std::fs::symlink_metadata(&p) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(err(&p, Some(self.chunk), self.offset, ErrorKind::Io, e)),
        };
        if self.complete || present {
            let final_recovery = !self.complete && self.chunk + 1 == self.chunks;
            let r = match json_file(&p) {
                Ok(r) => Some(r),
                Err(e) if final_recovery && e.kind == ErrorKind::Metadata => None,
                Err(mut e) => {
                    e.chunk = Some(self.chunk);
                    e.offset = self.offset;
                    return Err(e);
                }
            };
            // Only this chunk receipt is advisory. In particular, never bypass
            // finish(), which checks the remaining frame sidecar independently.
            if let Some(r) = r.filter(|r| {
                !final_recovery
                    || ["bytes", "crc32", "next_frame"]
                        .iter()
                        .all(|k| r[*k].as_u64().is_some())
            }) && (torn
                || r["bytes"] != self.offset
                || r["crc32"] != self.crc.clone().finalize()
                || r["next_frame"] != self.next)
            {
                return Err(err(
                    &p,
                    Some(self.chunk),
                    self.offset,
                    ErrorKind::Crc,
                    "chunk receipt mismatch",
                ));
            }
        }
        Ok(())
    }
    fn finish(&mut self) -> Result<(), Error> {
        // Only complete.json promises a trustworthy dropped count. Recovery
        // never reads dropped from interrupted.json or from chunk receipts;
        // verify represents the unknown count as JSON null.
        if self.complete {
            let p = self.path.join("complete.json");
            let r = self
                .receipt
                .as_ref()
                .ok_or_else(|| err(&p, None, 0, ErrorKind::Metadata, "missing receipt"))?;
            if r["frames"] != self.next
                || r["chunks"] != self.chunks
                || r["version"] != self.version
                || !r["error"].is_null()
                || self
                    .origin
                    .as_ref()
                    .is_none_or(|o| o["t0_us"] != r["t0_us"])
            {
                return Err(err(
                    &p,
                    None,
                    0,
                    ErrorKind::Structure,
                    "completion counts or origin mismatch",
                ));
            }
            number(r, "dropped", &p)?;
        }
        if let Some(f) = &mut self.frames
            && line(f, &self.path.join("frames.jsonl"), !self.complete)?.is_some()
        {
            return Err(err(
                &self.path.join("frames.jsonl"),
                None,
                0,
                ErrorKind::Structure,
                "extra frame sidecar rows",
            ));
        }
        Ok(())
    }
    fn advance(&mut self, decode: bool) -> Result<bool, Error> {
        loop {
            if self.chunk == self.chunks {
                self.finish()?;
                return Ok(false);
            }
            if self.file.is_none() {
                self.file = Some(BufReader::new(
                    open(&self.chunk_path()).map_err(|e| self.error(e.kind, e.message))?,
                ));
            }
            let mut h = [0u8; 17];
            #[cfg(test)]
            {
                self.chunk_reads[self.chunk as usize] += 1;
            }
            let file = self
                .file
                .as_mut()
                .ok_or_else(|| err(&self.path, None, 0, ErrorKind::Io, "missing chunk"))?;
            let n = file
                .read(&mut h[..1])
                .map_err(|e| self.error(ErrorKind::Io, e))?;
            if n == 0 {
                self.seal(false)?;
                self.chunk += 1;
                self.file = None;
                self.offset = 0;
                self.crc = crc32fast::Hasher::new();
                continue;
            }
            self.file
                .as_mut()
                .unwrap()
                .read_exact(&mut h[1..])
                .map_err(|e| self.record_eof(e))?;
            let len = u32::from_le_bytes(h[..4].try_into().unwrap()) as usize;
            let slot = u64::from_le_bytes(h[8..16].try_into().unwrap());
            let kind = h[16];
            if len > self.raw + self.raw / 255 + 32 {
                return Err(self.error(ErrorKind::Limit, "payload exceeds canvas bound"));
            }
            if slot != self.next
                || kind
                    > if self.version == 1 {
                        REPEAT
                    } else {
                        RECT_DELTA
                    }
                || (!self.keyed && kind != KEY)
                || (kind == REPEAT && len != 0)
            {
                return Err(self.error(ErrorKind::Structure, "invalid slot, kind or key chain"));
            }
            self.payload.resize(len, 0);
            self.file
                .as_mut()
                .unwrap()
                .read_exact(&mut self.payload)
                .map_err(|e| self.record_eof(e))?;
            let mut crc = crc32fast::Hasher::new();
            crc.update(&h[8..]);
            crc.update(&self.payload);
            if crc.finalize() != u32::from_le_bytes(h[4..8].try_into().unwrap()) {
                return Err(self.error(ErrorKind::Crc, "record CRC mismatch"));
            }
            if kind != REPEAT {
                let (x, y, w, ht, start) = if kind == RECT_DELTA {
                    if len < 16 {
                        return Err(self.error(ErrorKind::Structure, "short rectangle"));
                    }
                    let mut c = [0usize; 4];
                    for (i, b) in self.payload[..16].chunks_exact(4).enumerate() {
                        c[i] = u32::from_le_bytes(b.try_into().unwrap()) as usize;
                    }
                    let [x, y, w, ht] = c;
                    if w == 0
                        || ht == 0
                        || x.checked_add(w).is_none_or(|n| n > self.width as usize)
                        || y.checked_add(ht).is_none_or(|n| n > self.height as usize)
                    {
                        return Err(self.error(ErrorKind::Limit, "rectangle exceeds canvas"));
                    }
                    (x, y, w, ht, 16)
                } else {
                    (0, 0, self.width as usize, self.height as usize, 0)
                };
                if decode {
                    self.scratch.resize(w * ht * 4, 0);
                    let decoded =
                        lz4_flex::block::decompress_into(&self.payload[start..], &mut self.scratch)
                            .map_err(|e| self.error(ErrorKind::Compression, e))?;
                    if decoded != self.scratch.len() {
                        return Err(self.error(ErrorKind::Compression, "decoded size mismatch"));
                    }
                    if kind == KEY {
                        self.pixels.copy_from_slice(&self.scratch);
                        self.keyed = true;
                    } else {
                        for row in 0..ht {
                            let dst = ((y + row) * self.width as usize + x) * 4;
                            for (p, d) in self.pixels[dst..dst + w * 4]
                                .iter_mut()
                                .zip(&self.scratch[row * w * 4..(row + 1) * w * 4])
                            {
                                *p ^= d;
                            }
                        }
                    }
                }
                if kind == KEY {
                    self.keyed = true;
                }
            }
            let frame_offset = self
                .frames
                .as_mut()
                .map(|f| f.stream_position())
                .transpose()
                .map_err(|e| self.error(ErrorKind::Io, e))?;
            if let Some(f) = &mut self.frames {
                let p = self.path.join("frames.jsonl");
                frame_row(f, &p, slot, self.fps, self.complete).map_err(|mut e| {
                    e.chunk = Some(self.chunk);
                    e
                })?;
            }
            if self.index_keys && kind == KEY && self.keys.last().is_none_or(|k| k.slot < slot) {
                if self.keys.len() == 65_536 {
                    return Err(self.error(ErrorKind::Limit, "seek index exceeds 65536 keys"));
                }
                self.keys.push(KeyPosition {
                    chunk: self.chunk,
                    offset: self.offset,
                    slot,
                    crc: self.crc.clone(),
                    frame_offset,
                });
            }
            self.scanned = self.scanned.max(slot + 1);
            self.crc.update(&h);
            self.crc.update(&self.payload);
            self.offset += 17 + len as u64;
            self.next = self
                .next
                .checked_add(1)
                .ok_or_else(|| self.error(ErrorKind::Limit, "slot overflow"))?;
            return Ok(true);
        }
    }
    /// Returns one borrowed frame; no frame history is retained. After an error the reader stops.
    pub fn read(&mut self) -> Result<Option<Decoded<'_>>, Error> {
        if self.poisoned {
            return Err(self.error(ErrorKind::Structure, "reader poisoned"));
        }
        if self.stopped {
            return Ok(None);
        }
        match self.advance(true) {
            Ok(true) => {}
            Ok(false) => {
                self.stopped = true;
                if self.decoded_from_start {
                    self.known_frames = Some(self.next);
                }
                return Ok(None);
            }
            Err(e) => {
                self.stopped = true;
                if e.kind == ErrorKind::TornTail && !self.complete {
                    // A durable seal or an extra sidecar row disproves the
                    // unsealed crash shape, even for an incomplete take.
                    if let Err(e) = self.seal(true).and_then(|()| self.finish()) {
                        self.poisoned = true;
                        return Err(e);
                    }
                    self.torn_tail = true;
                    if self.decoded_from_start {
                        self.known_frames = Some(self.next);
                    }
                    return Ok(None);
                }
                self.poisoned = true;
                return Err(e);
            }
        }
        let slot = self.next - 1;
        let t_us = u64::try_from(u128::from(slot) * 1_000_000 / u128::from(self.fps))
            .map_err(|_| self.error(ErrorKind::Limit, "timestamp overflow"))?;
        Ok(Some(Decoded {
            slot,
            t_us,
            pixels: &self.pixels,
        }))
    }
    fn record_eof(&self, e: std::io::Error) -> Error {
        self.error(
            if e.kind() == std::io::ErrorKind::UnexpectedEof && self.chunk + 1 == self.chunks {
                ErrorKind::TornTail
            } else {
                ErrorKind::Io
            },
            e,
        )
    }
    /// Lazily index keys and their checksum/sidecar positions, then decode forward.
    /// Any failed seek poisons this reader; reopen it to try again.
    pub fn seek(&mut self, target: u64) -> Result<Decoded<'_>, Error> {
        if self.poisoned {
            return Err(self.error(ErrorKind::Structure, "reader poisoned"));
        }
        self.poisoned = true;
        self.seek_inner(target)?;
        self.poisoned = false;
        Ok(Decoded {
            slot: target,
            t_us: (u128::from(target) * 1_000_000 / u128::from(self.fps)) as u64,
            pixels: &self.pixels,
        })
    }
    fn position(&mut self, key: KeyPosition) -> Result<(), Error> {
        self.decoded_from_start = false;
        self.chunk = key.chunk;
        self.offset = key.offset;
        self.next = key.slot;
        self.crc = key.crc;
        let mut f = BufReader::new(open(&self.chunk_path())?);
        f.seek(std::io::SeekFrom::Start(key.offset))
            .map_err(|e| self.error(ErrorKind::Io, e))?;
        self.file = Some(f);
        if let (Some(f), Some(at)) = (&mut self.frames, key.frame_offset) {
            f.seek(std::io::SeekFrom::Start(at))
                .map_err(|e| err(&self.path.join("frames.jsonl"), None, at, ErrorKind::Io, e))?;
        }
        self.stopped = false;
        self.keyed = false;
        Ok(())
    }
    fn seek_inner(&mut self, target: u64) -> Result<(), Error> {
        if target >= self.scanned {
            if let Some(key) = self.keys.iter().max_by_key(|k| k.slot).cloned() {
                self.position(key)?;
            }
            while self.next <= target {
                if !self.advance(false)? {
                    return Err(self.error(ErrorKind::Structure, "seek beyond take"));
                }
            }
        }
        let key = self
            .keys
            .iter()
            .filter(|k| k.slot <= target)
            .max_by_key(|k| k.slot)
            .cloned()
            .ok_or_else(|| self.error(ErrorKind::Structure, "seek beyond take"))?;
        self.position(key)?;
        while self.next <= target {
            if !self.advance(true)? {
                return Err(self.error(ErrorKind::Structure, "unexpected EOF during seek replay"));
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn replay_to(&mut self, target: u64) -> Result<(), Error> {
        while self.next <= target {
            if self.read()?.is_none() {
                return Err(self.error(ErrorKind::Structure, "unexpected EOF during seek replay"));
            }
        }
        Ok(())
    }
}

pub fn verify(path: &Path) -> Value {
    verify_with_options(path, false)
}
pub fn verify_with_options(path: &Path, require_complete: bool) -> Value {
    let mut frames = 0u64;
    let mut markers = 0u64;
    let mut errors = Vec::new();
    let mut complete = false;
    let mut dropped = Value::Null;
    let mut recoverable = false;
    let mut tail = Value::Null;
    match Reader::open(path) {
        Err(e) => errors.push(e.json()),
        Ok(mut r) => {
            r.index_keys = false;
            loop {
                match r.read() {
                    Ok(Some(_)) => frames += 1,
                    Ok(None) => break,
                    Err(e) => {
                        errors.push(e.json());
                        break;
                    }
                }
            }
            match Markers::open(path) {
                Err(e) => errors.push(e.json()),
                Ok(mut m) => loop {
                    match m.read() {
                        Ok(Some(v)) => {
                            if v["frame"].as_u64().is_some_and(|n| n >= frames) {
                                errors.push(
                                    err(
                                        &m.path,
                                        None,
                                        m.row_offset,
                                        ErrorKind::Structure,
                                        "marker beyond take",
                                    )
                                    .json(),
                                );
                                break;
                            }
                            markers += 1;
                        }
                        Ok(None) => break,
                        Err(e) => {
                            errors.push(e.json());
                            break;
                        }
                    }
                },
            }
            recoverable = r.torn_tail && errors.is_empty();
            if recoverable {
                tail = json!({"chunk":r.chunk,"offset":r.offset});
            }
            complete = r.complete && errors.is_empty();
            if require_complete && !complete {
                errors.push(
                    err(
                        &path.join("complete.json"),
                        None,
                        0,
                        ErrorKind::Incomplete,
                        "complete valid take required",
                    )
                    .json(),
                );
            }
            if complete {
                dropped = r
                    .receipt
                    .as_ref()
                    .map(|v| v["dropped"].clone())
                    .unwrap_or(Value::Null);
            }
        }
    }
    json!({"ok":errors.is_empty(),"recoverable":recoverable,"tail":tail,"valid_frames":frames,"frames":frames,"dropped":dropped,"markers":markers,"complete":complete,"errors":errors})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_seeks_do_not_read_chunk_zero() {
        eprintln!(
            "VFL_KEY_POSITION_BYTES={}",
            std::mem::size_of::<KeyPosition>()
        );
        let path = std::env::temp_dir().join(format!("vfl-index-{}", std::process::id()));
        let mut w = crate::Writer::new(path.clone(), 2, 2, 1).unwrap();
        w.origin(0).unwrap();
        w.frame(crate::Frame {
            slot: 0,
            pixels: crate::Pixels::Bgra(vec![7; 16].into()),
        })
        .unwrap();
        w.finish(25, 0, 0, None, true).unwrap();
        let p = path.join("chunk-000000.bin");
        let b = std::fs::read(&p).unwrap();
        let mut cut = 0;
        for _ in 0..10 {
            cut += 17 + u32::from_le_bytes(b[cut..cut + 4].try_into().unwrap()) as usize;
        }
        for (n, bytes, next) in [(0, &b[..cut], 10), (1, &b[cut..], 25)] {
            std::fs::write(path.join(format!("chunk-{n:06}.bin")), bytes).unwrap();
            std::fs::write(
                path.join(format!("chunk-{n:06}.json")),
                json!({"bytes":bytes.len(),"crc32":crc32fast::hash(bytes),"next_frame":next})
                    .to_string(),
            )
            .unwrap();
        }
        let cp = path.join("complete.json");
        let mut c = json_file(&cp).unwrap();
        c["chunks"] = json!(2);
        std::fs::write(cp, c.to_string()).unwrap();
        let mut r = Reader::open(&path).unwrap();
        r.seek(24).unwrap();
        let reads = r.chunk_reads[0];
        for target in (10..25).rev().chain(10..25) {
            assert_eq!(r.seek(target).unwrap().pixels, [7; 16]);
        }
        assert_eq!(r.chunk_reads[0], reads);
        for target in 0..25 {
            assert_eq!(r.seek(target).unwrap().pixels, [7; 16]);
        }
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn seek_replay_reports_unexpected_eof() {
        let path = std::env::temp_dir().join(format!("vfl-seek-eof-{}", std::process::id()));
        let w = crate::Writer::new(path.clone(), 2, 2, 30).unwrap();
        drop(w);
        // Simulate a take truncated between scan and replay: an empty unsealed
        // chunk reaches clean EOF without advancing next.
        let mut r = Reader::open(&path).unwrap();
        let e = r.replay_to(0).unwrap_err();
        assert_eq!(e.kind, ErrorKind::Structure);
        assert!(e.message.contains("unexpected EOF"));
        std::fs::remove_dir_all(path).unwrap();
    }
}
