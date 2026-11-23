# Narration audio

`media-audio` holds in-memory primitives for S1/S2. The `mixd av audio` command uses its flat timeline module;
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
accepted only within those boundaries. Pipe WAVs with placeholder sizes (including
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
remeasures and makes at most two corrective attempts, each from the original.
`TargetReport` carries before/after LUFS, LRA and true peak, gain, pass count,
maximum attempted and final applied reduction, refusal, shortfall and an explicit `target_reached`
flag (±0.5 LU). Callers must check that flag: bounded corrections can exhaust.
Normalisation is transactional: on any error, input samples are unchanged.
Silence, near-silence below the absolute gate, or an unmeasurable rendered
attempt returns typed `Error::Unmeasurable` without applying gain.
While passes remain, acceptance requires ±0.2 LU to leave meter-parity margin;
`target_reached` still means ±0.5 LU. If an attempt exceeds 6 dB, `refused=true`
records the refused attempt, with its reduction in `max_gain_reduction_db`.
The best earlier in-bound attempt (smallest absolute target error) is kept;
its actual reduction is `applied_gain_reduction_db`. Capped static output is
used only when no limiting attempt is in bounds. Exhaustion also keeps the best attempt.

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

`mixd av audio TIMELINE.json --out FILE.webm [--wav FILE.wav]` executes
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
