# Media benchmark

Run from any working directory using the entry point's absolute path:

```
mix tools/media_bench/corpus/generate.mix --corpus /var/tmp/media-bench/corpus
mix tools/media_bench/run.mix --suite av1enc --engine ffmpeg --corpus /var/tmp/media-bench/corpus --out /var/tmp/media-bench/bounded/av1enc --runs 3
mix tools/media_bench/run.mix --suite av1dec --engine ffmpeg --variant sustained --corpus /var/tmp/media-bench/corpus --out /var/tmp/media-bench/sustained/av1dec --runs 3
mix tools/media_bench/run.mix --suite episode --engine ffmpeg --variant sustained --budget unrestricted --episode-run DIR --corpus /var/tmp/media-bench/corpus --out /var/tmp/media-bench/sustained/episode
mix tools/media_bench/test.mix
```

The supervisor runs the full sustained pass once from the merged head. Do not
use the bounded workload to rule on sustained throughput. Round-1 BD-rate
figures are void: their degenerate curves did not meet the overlap rules.

Each result has exactly `suite`, `engine`, `variant`, `tool_versions`, `host`,
`cpu_model`, and `items`. Items have `id` and `metrics`. Tool versions include
the source commit and explicit VMAF model name/version. CPU model is the model
name. Raw argv, stdout, stderr and GNU-time counters remain beside results.
CPU seconds are user+system; peak RSS is KiB. Wall/CPU/RSS each have median and
nearest-rank p95 over the repetitions. FPS has median and **p5**, the slow tail.
Any speed whose median wall time is below one second is `below_resolution`;
its speed/FPS ratios are excluded. `--compare DIR` reports ffmpeg/Rust for
paired numeric metrics; dynamic loudnorm gain is also excluded. Comparison
validation precedes writing successful results.

All measured processes run in a user systemd scope with MemoryMax=24G,
CPUWeight=20 and nice 10. `--budget 4` is the default: FFmpeg requests four
threads and tools inherit affinity to four allowed CPUs. `--budget unrestricted`
adds no affinity restriction and requests automatic FFmpeg/filter threading
(thread count 0). The budget is recorded in every result's tool_versions and
item metrics, and in every speed distribution. It is a resource policy, not
a count of every OS thread. The sustained production point and episode suite
must use `--budget unrestricted`; future Rust adapters must match that budget.
The host guard refuses 23:30–03:00 local time, headless Chrome (including
chrome-headless-shell), and build.mix/episode.mix invocations, including
relative paths and invocations with flags. Each measured command rechecks it.

The default video workload is 2 s / 60 frames, 3840x2160 at 30 fps. Sustained
uses the full 120 s / 3600 frames. SVT p8/p9 use native CRF 56/61/66/70 through svtav1-params (the top-level
FFmpeg CRF flag stops at 63); libaom cpu-used 6 uses CRF 56/59/61/63 with
qmin=qmax at each point; librav1e speed 10 uses QP 200/220/240/255.
Sustained adds librav1e speed 8. Higher SVT presets clamp at 4K; libaom 8
duplicated 6 and is omitted. BD-rate is calculated on both PSNR and VMAF,
using exact piecewise-linear log-rate integration over the common interval.
Curves must be strictly monotonic in quality and rate, with at least 3 dB or
3 VMAF overlap. Otherwise the metric is `unavailable` with reason and overlap
bounds; unavailable is not parity. IVF envelope bytes count towards bitrate.

Quality uses explicit `vmaf_4k_v0.6.1`, falling back to explicitly recorded
`vmaf_v0.6.1` only if the former is unavailable. Every input must have the
workload geometry and enough decoded frames. Both branches are trimmed and
aligned to a shared clock, scaled identically, converted to yuv420p with
limited range and BT.709 matrix/primaries/transfer, then checked through
showinfo. Unknown source colour metadata is interpreted using this declared
content contract. Source and distorted stream probes are retained. A short
stream is refused rather than padded by the metric filter. The held-source
reference uses explicit BT.709 RGB-to-YUV conversion.

Corpus generators run twice and require byte identity. The manifest pins
the complete argv and SHA-256; generated media never enters git. Strict
hashes depend on FFmpeg and the named DejaVu Sans font. Keep corpus and output
outside the repository. Paths must use only ASCII letters/digits, underscore,
dot, slash and hyphen, with no parent traversal or leading hyphen. Option
values cannot start with `-`. These constraints protect filtergraph and concat
interpolation. External tools use argv lists and path option terminators.

Narration selection is pinned on first use in private `tts-selection.json`
under the corpus root (20 sorted cached demo WAVs); subsequent runs require
the same hashes. Every discovered cache WAV is hashed in the private
inventory. Selected files are materialised under `tts-ref/`; IDs derive from
their relative corpus paths (`/` encoded as `__`), and duplicates/ambiguous
separators are refused. The programme item concatenates every selected
narration file after resampling to 48 kHz stereo. Loudness measures individual
files, the music bed and that programme. It retains actual loudnorm mode,
output and `linear_shortfall_lu = max(0, (-14-input_i)-(-1-input_tp))`.
Dynamic fallback is oracle behaviour, never a gain-parity target.
`linear_shortfall_lu == 0` does not mean loudnorm ran linear; `loudnorm_mode`
is the record of what actually ran.

Opus uses 64/96/128 kb/s libopus encode and explicitly selects libopus before
each decode input, checking stderr for every repeat and quality decode. PCM
SNR and spectral magnitude error are relative to reference RMS. The spectral
metric uses magnitude-only inverse STFT differences (Hann 1024, 75% overlap),
not a perceptual score. ViSQOL availability is recorded. Sustained speed uses
only music plus programme narration, repeated ten times to get process times
of seconds; the duration/repetition count is recorded. Quality compares one
original programme duration, with proper codec-delay handling.

`encode` invokes the private current assemble.mix stills route and separately
makes motion AV1/Opus WebM. `episode --variant sustained --episode-run DIR`
dereferences/copies an existing run into a new /var/tmp/media-bench directory,
rebases metadata against both the real source and the argument path, with path
boundaries so sibling prefixes remain intact, then runs assemble.mix on copied
takes and build.mix on the copy, one repeat each. Before/after SHA-256 snapshots
of the original's file list, sizes and nanosecond mtimes determine
`original_untouched`; a change fails the run. The copy is removed on success
or failure unless `--keep` is given; verification and timing files stay in
`--out`. It refuses runs
requiring fresh native desk recordings or lacking an assembly take. The
two-second synthetic episode-copy path is covered by a separate local smoke
receipt; it does not replace the commissioned full episode run.

No clip-only default `--episode-run` is available: the 2026-11-21 inventory
found 18 episode plans under `{home}/.viu/media/runs`, all containing deck/sting
parts and/or native parts. No default is invented. Non-native mixed plans
with assembly takes exist (including `2026-11-15-bus-sniffer` and `2026-11-20`),
but they are not clip-only; an operator must choose a run explicitly.

Rust AV1 stays `not_built` for L3b; other adapters await their owning rows.
Deferred suites identify their rows: deck D1–D3, record R3, sync S1/P1,
assemble S2, playback P1. The manifest resolves relative to this repository,
not the caller's working directory. Tests run alongside existing tool tests;
the real quality oracle is in `tools/ci.mix all`. Missing FFmpeg/capabilities
print visible SKIP unless MIX_TEST_REQUIRE_FFMPEG=1, when they fail. Focused
CBC must set that variable; tests without FFmpeg remain unconditional.
