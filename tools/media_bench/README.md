# Media benchmark

Run from the repository root, on the oracle host:

```
mix tools/media_bench/corpus/generate.mix --corpus /var/tmp/media-bench/corpus
mix tools/media_bench/run.mix --suite av1enc --engine ffmpeg --corpus /var/tmp/media-bench/corpus --out /var/tmp/media-bench/results/av1enc --runs 3
mix tools/media_bench/run.mix --suite av1enc --engine rust --corpus /var/tmp/media-bench/corpus --out /var/tmp/media-bench/results/rust-av1enc --compare /var/tmp/media-bench/results/av1enc
mix tools/media_bench/test.mix
```

Keep all output outside this repository. The generator runs twice and refuses
byte instability. `manifest.conf.mix` pins the generated files; `spec.mix` owns
the complete argv. Font: DejaVu Sans at the explicit system path in the argv.
The generator and ffmpeg versions affect hashes: a mismatch requires investigation,
not automatic acceptance or overwriting the expected manifest.

The result has exactly seven top-level fields: `suite`, `engine`, `variant`,
`tool_versions`, `host`, `cpu_model`, `items`. Every item has `id` and `metrics`.
Metric maps carry units in their names. Each speed has separate `wall_s`,
`cpu_s` (user plus system), and `peak_rss_kib`, each with median and nearest-rank
p95. CPU/RSS come from GNU time around the tool **inside** its systemd scope;
scope startup is excluded. Commands, counters, stdout and stderr are retained.
FPS includes demux, codec startup and output I/O. FPS p95 is the percentile of
the FPS samples, rather than the inverse of wall-time p95.

The default video workload is the first **two seconds / 60 frames at 4K30**
of the generated two-minute motion source. It measures a bounded codec
specimen, including startup, and cannot establish episode throughput. The
AV1 matrix includes SVT presets 10/12, libaom 6/8 and librav1e 10 at four
quality points each. BD-rate is the exact integral of piecewise-linear
log(bitrate) versus VMAF over their common quality interval, relative to SVT
preset 10. No overlap is reported explicitly. This is a linear BD-rate
variant, not a cubic polynomial fit. IVF envelope bytes count towards bitrate.

The decode suite prepares its own SVT stream, uses libdav1d and saves all 60
frame checksums. Audio uses the full generated music bed and the first 20
sorted narration WAVs; every discovered narration WAV is hashed in the private
`tts-inventory.json`. Files are resampled to 48 kHz stereo before PCM quality
comparison. Opus is encoded at 64/96/128 kb/s. PCM SNR uses reference and
error RMS. Spectral magnitude distance is RMS dBFS of the difference between
two magnitude-only inverse STFTs (Hann 1024, overlap 75%); it is an explicitly
defined distance, not a perceptual parity claim. ViSQOL availability is
reported; it is not installed by the harness. Loudness runs ebur128 and
both loudnorm passes, retaining the measured linear-mode parameters and
normalised WAV. Check raw loudnorm output for dynamic fallback.

`encode` invokes the **current private `assemble.mix`** on a synthetic
two-second stills take, with generated narration, then separately makes an
AV1/Opus motion WebM. It refuses a missing pipeline. This exercises the real
stills route; VFL export and full episode/timeline assembly await S1/S2.
Rust suites currently return `not_built`: the existing AV1 example accepts
only its own generated eight-frame scene, not corpus input. No Rust build or
new binary is introduced here. Only the `default` variant is defined.

`--compare DIR` requires a result of the same suite from the opposite engine.
Tables show ffmpeg/Rust for every paired numeric metric, including medians
and p95s. A missing metric or zero Rust denominator has an unavailable ratio.
The item IDs and metric paths are the adapter contract for future Rust runners.

Every measured command rechecks the nightly window and process guard. The
guard refuses 23:30–03:00 local time, an active headless Chrome, or an episode
`media/pipeline/build.mix`. It never kills an episode. Each command runs under
`systemd-run --user --scope -p MemoryMax=24G -p CPUWeight=20 nice -n 10`.
GNU time and a working user systemd manager are required. A command failure
stops the run and points to retained raw evidence; incomplete runs have no
success `results.json`.

Deferred suites refuse with their owning row: `deck` D1–D3, `record` R3,
`sync` S1/P1, `assemble` S2, `playback` P1. Synthetic UI and music are stand-ins;
pending desk/episode corpus items are never silently substituted.
