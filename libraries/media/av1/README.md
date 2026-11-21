# AV1 packet encoder

## Bench example

`cargo build -p media-av1 --example av1enc --release` builds an example,
not an installed binary. Add `--features asm` for the assembly variant;
the default has assembly disabled. Feed progressive 8-bit 4:2:0 Y4M on stdin.
`--out encoded.ivf` selects a file; otherwise IVF goes to stdout.
Options are `--speed` (default 10), `--qp` / `--quantizer` (100),
`--threads` (1), `--tile-cols` / `--tile-rows` (1 each, counts rather
than logarithms), and `--keyint` (240).

The stream declares its fixed size and rational frame rate in the Y4M
header. Missing/duplicate format fields, interlacing, high bit depth,
full range, truncated frames, frame overrides and concatenated streams
are refused. No frame-rate conversion or scaling is performed. Metadata
lines are limited to 4096 bytes. Discard partial output after any error.
The IVF header uses an unknown frame count (zero) on stdout; `--out`
patches the exact count after success. Timestamps use the Y4M timebase.

One JSON line on stderr reports frames, total IVF bytes (including headers),
encode seconds, requested settings, actual tiles and assembly status.
Encode seconds includes frame preparation, submission and flush; it excludes
input reads, encoder construction and output writes. Bench wall/CPU/RSS
measurements must cover the whole process separately.

`adapter_check.mix` runs focused worker clippy, all test targets for both
assembly variants and the foreign gate. Its `--fmt` mode prints the worker's
format diff and restores its checkout. The example tests generate Y4M,
encode IVF, decode with ffmpeg/libdav1d, require five frames and PSNR above
35 dB, and check refusal cases and IVF fields. Missing oracle tools fail.
No library API or dependency changes are needed: `prepare`, `submit` and
`finish` already provide external frames in and packets out.

`media-av1` pins rav1e **0.8.1**, default features disabled. Only threading and
optional asm are enabled. `cargo test -p media-av1 --no-default-features` tests
the pure-Rust default; --features asm opts into nasm-built x86 kernels.
Shipping binaries must enable asm explicitly when approved. No C encoder,
ffmpeg, container, transport or application policy is linked here.

`Settings` chooses dimensions, rational fps, preset 0–10, quantizer 0–255
(lower is better) or positive bits/second target, fixed key interval,
power-of-two tile columns/rows (1–64), and threads (1–64). The wrapper validates
frame sizes (16–65535) before tiling, plus rate and key interval. Extremely large frames
can still exhaust memory; rav1e's pool creation can panic on OS thread failure.
Use a worker admission budget. Requested tiles are adjusted by rav1e's tiler;
`Encoder::tiles()` reports the actual grid. Threads bound the encoder Rayon
pool; they are not a promise of proportional speedup. At 4K 2×2 tiles are
available; even one 3840×2160 tile fits AV1's 4096-pixel width and area limit.

The API accepts validated L6 `Yuv420`, copies/pads once into `Prepared`, and
permits cheap clones of that immutable frame for repeated submissions. That
saves repeated BGRA-to-YUV conversion and copying/padding into rav1e input
planes. Pinned rav1e uses Arc::get_mut (not make_mut) in internal send_frame:
a shared frame is checked for padding and queued without a deep copy. This
does not save lookahead or encoding. Every CFR input
still produces a displayed frame. `submit` drains available packets; `finish`
flushes once. Packets contain temporal-unit bytes, zero-based presentation
ticks (`fps_den / fps_num` seconds), duration 1 and a key-frame flag. Low
latency disables reordering. The caller muxes packets and carries the timebase
and centred chroma siting; AV1 signals Unknown because it has no centred enum.

## API probe, before wrapper design

Examined the pinned upstream `src/api/config/{encoder,speedsettings,mod}.rs`,
`src/api/{context,util,color}.rs`, `src/encoder.rs`, `src/tiling/tiler.rs` and
`Cargo.toml`; focused tests exercise the real API, not a remembered CLI.

- Speeds 0–10; upstream clamps larger values to 10, this API rejects them.
  Presets alter partition sizes, lookahead, motion/prediction searches,
  segmentation and filters. At 8 LRF turns off; at 10 partitions are 32×32.
- Public speed settings expose prediction complexity, directional intra,
  reference search, SATD subpel, CDEF/LRF, segmentation and transform search.
- There is **no working video palette toggle**: encoder writes palette false.
  Video force_screen_content_tools is zero; intra-block-copy is false and
  inter integer-motion forcing is zero. No screen-content preset can be
  honestly exposed or compared on/off here. Still-picture is not a video
  workaround. A future upstream change needs a fresh probe and benchmark.
- `Config::with_threads`, `tile_cols`, `tile_rows`, desired total `tiles`,
  and `tiling_info()` are public. Shared pools and parallel GOP slots are
  unstable; this wrapper uses the stable context API and explicit grid.
- `send_frame` takes `Arc<Frame<u8>>`; sharing requires padding first because
  upstream can only automatically pad uniquely owned frames. No repeat opcode.

The opt-in `oracle.mix` runs generated static/blinking-cursor, scrolling-list
and window-drag sequences at 1080p and 4K, presets 6/8/10, asm on/off, then
checks decode, frame counts and dimensions with ffmpeg on a worker.
It prints PSNR for the record; it does not enforce a quality floor. The
single-run measurements do not yet establish per-case acceptance floors.
Eight frames per case, q100, 30fps, two threads, requested 2×2 tiles.
FPS times only submissions plus flush; excludes generation, conversion,
preparation and output I/O. Bitrate counts AV1 payload bytes over 8/30 seconds.
These are **first numbers, not the av1enc suite**: no repetitions, p95, VMAF,
BD-rate, long static stretches, oracle encoder comparison or cut-over claim.

First measurements and every case: [FIRST_NUMBERS.md](FIRST_NUMBERS.md).
The published [current API](https://docs.rs/rav1e/0.8.1/rav1e/config/struct.EncoderConfig.html)
also identifies 0.8.1; the local pinned source was the design authority.

## Provenance

Wrapper and generated fixtures are original. Upstream source is crates.io
rav1e 0.8.1, https://github.com/xiph/rav1e, BSD-2-Clause plus its AOM patent
licence (`LICENSE`, `PATENTS` in the published crate); lockfile checksums pin
the exact source archive. `dependencies.mix` prints licences/repositories of
the resolved normal/build closure for audit and runs the foreign gate.
No upstream code is copied. Assembly admission is restricted to media-av1;
strict shell crates explicitly forbid media-av1 and rav1e. The `cc` build
helper is used for assembly linking; on x86 the enabled rav1e build compiles
nasm assembly, not C. Optional binary/C-API/decoder-test/git-version features
are off. nasm is a build tool, not a product runtime dependency.
