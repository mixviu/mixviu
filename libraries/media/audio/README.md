# Narration audio

`media-audio` holds in-memory primitives for S1/S2. It is not connected to
`mix`, `mix-shell` or any engine. All offsets and lengths are sample **frames**.
Samples are finite interleaved f64, mono or stereo. Mixing preserves headroom.

`Audio::read_wav` / `write_wav` support PCM16/24/32 and float32 at nonzero
rates, including canonical 44-byte LINEAR16 replies. RIFF sizes, chunk bounds,
frame alignment and byte rates are checked before decoding. Unsupported encodings,
truncated files and non-finite samples return typed errors. RIFF/WAVE only:
RF64, RIFX, more than two channels and compressed WAV are not supported.
Integer output rounds without dither and refuses clipping; float32 retains
headroom. Files are held in memory, so the caller must bound input size.

`resample_48k` uses rubato 5.1.0's public positional sinc interpolator: 512
taps, squared Blackman-Harris window, automatic cutoff (about 0.9734 of the
lower Nyquist frequency), 256 table phases, cubic interpolation and f64.
The long window trades CPU for alias rejection and passband flatness. Zero
padding at both ends and positions `n * input_rate / 48000` avoid fractional
startup delay. Output is `ceil(input_frames * 48000 / input_rate)` frames.
The 48 kHz path copies unchanged. This is an offline API, not realtime streaming.

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
actual gain, peak-limited flag and LU shortfall. There is no dynamics processor.
Silence cannot be normalised and returns `Unmeasurable`. Absolute gating may
change after gain; the reported output is measured rather than inferred.
The true-peak ceiling refers to this meter, with oracle agreement assessed
separately. Quantisation or later sample-rate conversion can change peaks.

## Existing filter ownership

| Existing operation | L5 primitive / S2 responsibility |
| --- | --- |
| `aresample=48000` | L5 sinc resample; S2 chooses where it runs |
| stereo `aformat` | L5 mono duplication/stereo identity; S2 format negotiation |
| `adelay` | L5 frame placement; S2 marker/hold timing and ms rounding |
| `amix normalize=0` | L5 unnormalised sum; S2 input selection/duration policy |
| `loudnorm I=-14:TP=-1` | L5 measured linear gain cap with shortfall; S2 policy for shortfall; dynamic mode is outside L5 |
| `apad`, end `atrim` | L5 resize; S2 final duration |
| ranged `atrim`, `asetpts` | S2 range selection and timestamp reset |
| audio `concat`, `acrossfade` | S2 timeline execution/fade curves |

## Verification

`mix libraries/media/audio/worker.mix` runs worker fmt, all test targets
(including a required ffmpeg oracle), clippy and the foreign gate, then builds
the bench-only `audio_probe` example. It prints and restores worker formatting
and lockfile diffs for application by the caller. No oracle silently skips.
Generated fixtures cover reference tones, analytically levelled octave pink
spectrum, seeded pink noise, LRA/gates, true vs sample peak, gain shortfall,
resampling timing/alias rejection and malformed WAVs. They are EBU-style
signals, not copies of the EBU conformance media or a claim of full certification.
The oracle asserts ffmpeg really selected two-pass linear normalisation.

Private corpus, SNR and speed results belong in the maintainer's hub.
