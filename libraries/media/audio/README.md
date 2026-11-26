# Narration audio

`media-audio` holds in-memory primitives for S1/S2. The `av audio` command uses its flat timeline module;
`mix` and `mix-shell` remain free of media dependencies. All offsets and lengths are sample **frames**.
Samples are finite interleaved f64, mono or stereo. Mixing preserves headroom.

`Audio::read_wav` / `write_wav` support PCM16/24/32 and float32 at nonzero
rates, including canonical 44-byte LINEAR16 replies. RIFF sizes, chunk bounds,
frame alignment and byte rates are checked before decoding. Unsupported encodings,
truncated files and non-finite samples return typed errors. RIFF/WAVE only:
RF64, RIFX, more than two channels and compressed WAV are not supported.
Integer output rounds without dither and refuses clipping; float32 retains
headroom. NaN and infinity are refused, including decoded float32 samples.
Output requires an empty stream positioned at zero (also a hound requirement).

Reads are bounded internally: `read_wav` uses `MAX_WAV_BYTES` (4 GiB + 7 bytes,
the standard RIFF maximum); `read_wav_limited` accepts a smaller caller limit.
At most the limit plus one overflow sentinel byte is read, and oversized input
returns `InputTooLarge`. Two hours of stereo float32 at 48 kHz needs about
2.76 GB, below the default. Whole-file bytes and decoded f64 samples coexist
while decoding, so callers should use a smaller limit where possible.
Resampling checks `MAX_RESAMPLED_FRAMES` (518,400,000 frames, three hours at
48 kHz) before creating filters or output buffers, including the identity path.
This bounds rate amplification without imposing an arbitrary rate floor.
Excess returns `OutputTooLong`; the cap leaves 50% duration headroom over two hours.

RIFF parsing is deliberately strict: the RIFF size must equal the entire file
length, every chunk must fit, and final odd chunks must include their pad byte.
Exactly one fmt chunk must precede exactly one data chunk. Unknown chunks are
accepted only within those boundaries; total non-audio RIFF bytes (including
chunk headers and padding) are limited to 64 KiB to bound header scanning.
Pipe WAVs with placeholder sizes (including
ffmpeg pipe output) and lax writers without final padding are refused. PCM,
IEEE float32 and their extensible GUIDs are accepted; extensible valid bits must
equal container bits. Narrow valid bits in wider containers are refused with
`UnsupportedFormat`, avoiding ambiguous scaling in hound.

`resample_48k` uses rubato 5.1.0's public positional sinc interpolator: 512
taps, squared Blackman-Harris window, automatic cutoff (about 0.9734 of the
lower Nyquist frequency), 256 table phases, cubic interpolation and f64.
The long window trades CPU for alias rejection and passband flatness. Zero
padding at both ends and positions `n * input_rate / 48000` avoid fractional
startup delay. Output is `ceil(input_frames * 48000 / input_rate)` frames.
The 48 kHz path copies unchanged after the output-length check. This is an offline
API, not realtime streaming. It uses rubato's lower-level interpolator items,
protected by the exact `=5.1.0` pin. That version explicitly requires
`index + nbr_points < wave.len()` in `get_sinc_interpolated`, so the guard
rejects equality (`>=`), despite the mathematical slice permitting equality.
An unexpected index/phase violation returns `InterpolationBounds`.

`map_channels` duplicates mono or takes the stereo mean. `place` sums without
normalisation at a frame offset, extending with silence. `resize` pads or trims
the end. Callers convert timeline times to integer frames and choose ordering.

`measure` reports integrated LUFS with BS.1770 gating, Tech 3342 LRA, sample
peak dBFS and FIR true peak dBTP through ebur128 0.1.11. True peak is 4x below
96 kHz, 2x below 192 kHz and unchanged above that; measure the 48 kHz output
for the narration chain. The FIR tail is flushed without adding loudness blocks.
Silence/insufficient gated duration returns negative infinity for integrated
loudness. LRA needs at least three seconds and enough programme variation.

`normalise(-14.0, -1.0)` measures, applies static gain capped by the measured
true-peak ceiling, then measures the output again. `GainReport` includes the
actual gain, peak-limited flag and LU shortfall. This entry point remains static.
Silence cannot be normalised and returns `Unmeasurable`. Absolute gating may
change after gain; the reported output is measured rather than inferred.
The true-peak ceiling refers to this meter, with oracle agreement assessed
separately. Quantisation or later sample-rate conversion can change peaks.

`normalise_to_target(target_lufs, ceiling_dbtp, LimiterConfig::default())`
preserves the static path bit for bit whenever its measured peak permits the
requested gain. Otherwise it renders target gain through `limit_true_peak`,
remeasures and makes at most eight attempts (`MAX_LIMITER_PASSES`), each from the original.
`TargetReport` carries before/after LUFS, LRA and true peak, gain, pass count,
maximum attempted and final applied reduction, refusal, shortfall and an explicit `target_reached`
flag (inclusive ±0.1 LU). Callers must check that flag: bounded corrections can exhaust.
Normalisation is transactional: on any error, input samples are unchanged.
Silence, near-silence below the absolute gate, or an unmeasurable rendered
attempt returns typed `Error::Unmeasurable` without applying gain.
While passes remain, the PCM loop aims for ±0.03 LU to leave codec margin;
`target_reached` means ±0.1 LU. If an attempt exceeds 6 dB, `refused=true`
records the refused attempt, with its reduction in `max_gain_reduction_db`.
The best earlier in-bound attempt (smallest absolute target error) is kept;
its actual reduction is `applied_gain_reduction_db`. Capped static output is
used only when no limiting attempt is in bounds. Exhaustion also keeps the best attempt,
but refuses it if outside ±0.1 LU. A reduction refusal always clears success.

The offline limiter uses four-phase 24-tap Hann-windowed sinc peak detection,
linked across channels. `lookahead_ms` specifies a unity-to-zero slope,
not a fixed window or delay: `attack_step = 1 / (rate * lookahead_ms / 1000)`.
A reduction to gain g starts approximately `(1-g)*lookahead_ms` before the
peak, plus FIR support. The default slope duration is 5 ms (valid 0.1–100 ms); release
is 100 ms (valid 1–1000 ms). A backwards linear attack and forward exponential
release anticipate peaks without moving any samples or changing duration.
100 ms gives speech peaks time to recover between words without rapid envelope
modulation. A release tail below 0.0001 linear reduction (0.00087 dB) snaps
to exact unity so distant quiet beds remain bit-identical. The detector checks zero-padded and reflected file boundaries. A final constant
trim, verified by the detector and meter, catches reconstruction overshoot
from gain modulation; its reduction counts towards the 6 dB bound. A 0.02 dB
final margin protects float32 export rounding; detection uses a 0.04 dB
margin to leave room for modulation before a global safety trim is needed. This is peak limiting, not loudness AGC.
The direct limiter returns ReductionLimit without changing samples above 6 dB.
Processing order is fixed and deterministic for a given binary and platform;
cross-platform floating-point bit identity is not promised. The limiter uses
one f64 envelope value per frame in addition to the audio; target normalisation
also retains a transactional work copy, original, current candidate and best
in-bound attempt. Callers must bound input duration/memory accordingly.

## Existing filter ownership

| Existing operation | L5 primitive / S2 responsibility |
| --- | --- |
| `aresample=48000` | L5 sinc resample; S2 chooses where it runs |
| stereo `aformat` | L5 mono duplication/stereo identity; S2 format negotiation |
| `adelay` | L5 frame placement; S2 marker/hold timing and ms rounding |
| `amix normalize=0` | L5 unnormalised sum; S2 input selection/duration policy |
| `loudnorm I=-14:TP=-1` | L5 target gain with bounded limiting; S2 checks target_reached/refusal/shortfall |
| `apad`, end `atrim` | L5 resize; S2 final duration |
| ranged `atrim`, `asetpts` | S2 range selection and timestamp reset |
| audio `concat`, `acrossfade` | S2 timeline execution/fade curves |

## Verification

`mix libraries/media/audio/worker.mix` runs worker fmt, all test targets
(including a required ffmpeg oracle), clippy and the foreign gate, then builds
the bench-only `audio_probe` example. It prints and restores worker formatting
and lockfile diffs for application by the caller. Cleanup requires clean initial
state, unchanged HEAD and a diff limited to audio Rust files/Cargo.lock; it
restores only that explicit path list. No oracle silently skips.
Generated fixtures cover reference tones, analytically levelled octave pink
spectrum, seeded pink noise, LRA/gates, true vs sample peak, gain shortfall,
resampling timing/alias rejection and malformed WAVs. They are EBU-style
signals, not copies of the EBU conformance media or a claim of full certification.
The oracle asserts ffmpeg really selected two-pass linear normalisation.

Private corpus, SNR and speed results belong in the maintainer's hub.

`mix libraries/media/audio/mutations.mix` is a separate CBC-only check. It
requires a clean checkout, swaps global gain, no look-ahead, instant release
and sample-only detection into the limiter one at a time, and requires the
named locality/attack/release/4x test to fail. It restores the exact source
bytes in a finally block. The normal tests must pass on the unmodified source.

## Episode timelines

`av audio TIMELINE.json --out FILE.webm [--wav FILE.wav]` executes
a strict version-1 flat timeline. Paths are relative to the JSON file.
Times round to 48 kHz frames; trims refer to the resampled source. Source
tails at the episode boundary are trimmed. A source starting exactly at the
end contributes no samples. Mono maps to stereo at 1/sqrt(2) per channel,
matching ffmpeg aformat. Gains are -120..60 dB. Each fade includes both
endpoints; a single-frame fade is zero. Overlapping fades multiply.
The mastering receipt reports PCM measurements before Opus encoding.
Silence and programmes too short to meter return an unmeasurable error.

Memory trade-off: this is an offline mastering executor, using the existing
L5/L5b whole-programme API to retain its tested gating and limiter behaviour.
The unnormalised accumulator is f32 (576 MB for 25 minutes). It is converted
once to f64 for mastering (1.15 GB). The limiter can retain approximately
six f64 programme buffers plus an envelope: budget 8 GB for 25 minutes,
plus the largest decoded source. Three hours requires approximately 55 GB
and a correspondingly sized worker. Static gain uses fewer buffers.
This favours the existing verified mastering path over a new streaming
limiter; long programmes need a memory-sized worker. No new foreign crate
is introduced. `episode_fixture.mix` generates private test inputs from a
read-only run; scenes are placed back to back without crossfades.

Opus safety: the executor decodes only its own encoded packets and meters
the exact presentation samples. If codec reconstruction exceeds the ceiling,
it lowers the PCM limiter ceiling by the measured overshoot plus a 0.02 dB
guard, and adjusts the PCM loudness target by the decoded loudness error.
Every correction re-masters the original mix. `MAX_CODEC_ATTEMPTS` is six
(one initial attempt and at most five corrections); each master uses at most
eight limiter attempts, so the total is bounded by 48 limiter renders and six
encode/decode/meter cycles. The existing envelope and 6 dB reduction cap remain.

The limiter adds its measured loudness deficit to input gain. For speech with
sparse peaks, most samples remain outside limiting, so a gain correction
recovers most of that deficit; repeated residual corrections shrink the error.
The outer loop compensates codec loss and tightens the ceiling for overshoot.
Eight and six are convergence budgets, not a guarantee for arbitrary gated or
heavily limited audio: every final encoded result must independently meet
inclusive ±0.1 LU and the true-peak ceiling. Reduction refusal or budget
exhaustion returns measured values with `refused=true`, `target_reached=false`
and publishes neither output. No limiter redesign or relaxation of the cap
is hidden in the correction loop.

An individual timeline may explicitly set `master.accept_ceiling_limited: true`
to publish a ceiling-limited shortfall. It defaults to false and must never be
enabled by configuration defaults or timeline generators. There is no duration
exception. Acceptance requires a PCM limiter refusal with a shortfall above
0.1 LU, a finite decoded loudness below target, and decoded true peak at or below
the requested ceiling. The existing limiter/reduction bound still applies;
this permission accepts its retained in-bound output without changing DSP or
trying extra passes. Codec-only misses, overshoots and unmeasurable audio remain
refused. An accepted shortfall reports `refused: false`, `target_reached: false`
and the decoded `shortfall_lu`. Callers use `refused` to decide publication.

The receipt reports `codec_trim_db` (cumulative PCM ceiling adjustment),
`codec_passes` (corrections), `codec_attempts` (including the initial cycle), and encoded loudness. `passes` is the total
limiter attempts over all masters. `mastering_cost` gives thread CPU seconds
for mastering and encode/decode/meter separately on every codec attempt,
plus its limiter attempt count and individual `limiter_cpu_s` costs (including
gain application and candidate copying). Each entry also gives `mastering_wall_s`
and `codec_wall_s`. These observational timings never affect
iteration or encoded output. Shortfall and success use decoded audio.
Encoded packets, decisions and metered values are deterministic for identical
inputs and encoder builds. Receipts as a whole are **not byte-reproducible**:
`MasteringCost` CPU times vary between runs and must be excluded from receipt
comparisons.

Worst-case work is 48 limiter renders plus six codec cycles. Scaling the
optimised CBC fixture costs (about 3.0 CPU seconds per limiter render and
0.27 seconds per codec cycle per 30 seconds of audio) to a ten-minute
programme gives 2,880 CPU seconds of limiting plus about 33 seconds of codec
work. Six static-master setup/metering costs add roughly 15 seconds: about
49 CPU minutes, rounded to **50 CPU minutes**, before mixing and output I/O.
This is a conservative fixture-based estimate, not a measured ten-minute
worst-case run or a time limit; hardware and programme content affect CPU cost.
Optional WAV is the final mastered PCM. Packet storage is about 24 MB per 25 minutes;
decoded metering adds one f64 programme buffer, below the limiter budget.
The original mix is retained across codec attempts: one additional stereo f64
programme buffer (about 1.15 GB for 25 minutes at 48 kHz).

## Validation and exit status

Source paths are relative to the opened timeline's real directory: its
`/proc/self/fd` location supplies the parent, rather than the lexical parent
of the CLI argument. A timeline reached through a symlink therefore uses and
confines sources to the real timeline directory. Absolute source paths and any
`..` component are refused by source index before opening any WAV. Each source
is opened nonblocking, then its descriptor metadata must identify a regular
file. FIFOs, devices and directories are refused. On Linux, `/proc/self/fd`
provides the opened file's resolved path: it must be beneath the canonical
timeline directory. Symlinks within that directory are accepted; escaping
symlinks are refused. Validation and decoding use the same descriptor, without
reopening the source path. Other platforms fail closed on this operation.

The opened WAV's size is capped at **16,588,865,536 bytes**: three hours of
48 kHz, eight-channel, 32-bit audio plus 64 KiB of RIFF metadata. Standard
RIFF's 32-bit length imposes a tighter effective limit (4,294,967,303 bytes);
the supported input remains mono/stereo. Before sample allocation or decoding,
the header sample count is capped at 4,147,200,000 interleaved values and its
resampled duration at three hours. Hound seeks directly to the required input
window and stops decoding at the trim or placed end. At rates other than
48 kHz, the window includes up to 512 input frames on each side for the sinc
filter; buffered I/O can read ahead by at most 8 KiB. No whole-file PCM or
byte buffer is allocated by the timeline path.

Summed placed source frames must not exceed **16 times the output frames**.
This permits sixteen full-length overlapping layers while bounding decoding,
resampling and summing work independently of the 10,000-source limit. Trimmed
tails and sources at the timeline end contribute zero extra placed work. The
running total is checked from each header before decoding that source; no
source that would cross the limit is decoded. RIFF headers are also bounded.

Timeline WAV sample rates must be **8,000–384,000 Hz inclusive**, covering
telephony through high-resolution masters. Unsupported rates are refused by
source index before decoding. An additional aggregate native-input budget is
**16 × output frames × 2 channels × (384000 / 48000)** scalar samples.
This allows sixteen stereo layers at the maximum supported input rate while
bounding actual decoding, rather than only output placement. Each source is
charged its exact decoded native window frames times its channel count,
including sinc support and clipping at the file edges. The decoder and budget
share the window calculation. The running total is checked before each decode;
equality is accepted, excess is refused. Very short or heavily fragmented
timelines may hit this budget through filter support alone. Empty placed
windows cost zero. These limits apply to timeline execution; the lower-level
Audio API retains its existing nonzero-rate contract.

Timeline
JSON is limited to 16 MiB: the opened file is checked, then read through a
bounded reader with a growth sentinel. Programme copies/conversions, limiter
envelopes and decoded metering storage reserve fallibly; allocation failure
is a normal error.

Timeline JSON uses the same nonblocking open and descriptor regular-file
check as WAV inputs, before applying its 16 MiB descriptor-size limit.

Exit 2 reports usage, validation, pre-publication I/O, allocation or codec
errors as `av: ...`. Once rendering has an outcome, that outcome takes
precedence over receipt delivery and refusal diagnostics:

| Outcome | Receipt delivered | Closed stdout (EPIPE) | Other receipt-write error |
| --- | ---: | ---: | ---: |
| Accepted and published | 0 | 0 | 4 |
| Mastering refused, unpublished | 3 | 3 | 3 |
| Decoded target missed, unpublished | 3 | 3 | 3 |

EPIPE is quiet and never changes the outcome code. Refusal/miss diagnostics
are best-effort; stderr failure never changes exit 3. Exit 4 prints
`av: published but receipt not written: …` best-effort: outputs exist but the
receipt was not delivered, so do not retry blindly. Diagnostic writes never
panic on EPIPE.

WAV and WebM are staged in temporary files in their destination directories.
Each buffer is explicitly flushed and each file fsynced before publication;
its directory is fsynced after publication. A flush or sync error exits 2 and
rolls back any outputs from this run. Only an accepted master publishes them.
The primary WebM is published before the optional WAV. Atomic no-clobber hard links provide
rename-like publication while preserving existing destinations; an error
rolls back any file published by this transaction. Refused/missed renders
leave no final outputs or temporary files. The receipt bytes field describes
the staged WebM even when that master is discarded.
Killing a run can leave recognisable `.av-<pid>-N.tmp` files beside the
destinations, or a published WebM without its optional WAV. There is no sweep.
After linking, destination lstat device/inode must match the synced temporary
descriptor's fstat. A swapped temporary fails publication; cleanup checks the
entry's identity before removing the mismatching destination link.
