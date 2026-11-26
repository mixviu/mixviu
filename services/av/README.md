# av

Offline media executor. `av audio` executes the audio timeline documented in
`libraries/media/audio/README.md`. `av render TIMELINE.json --out FILE.webm`
executes an explicitly planned video/audio timeline. Mix owns scene planning.

```json
{
  "version": 1,
  "duration_s": 4,
  "video": {
    "width": 3840, "height": 2160, "fps": 30,
    "segments": [
      {"path": "take.vfl", "from_slot": 0, "to_slot": 120, "at_s": 0}
    ],
    "crossfades": [],
    "fade_in_s": 0.6, "fade_out_s": 1.0
  },
  "audio": {
    "sample_rate": 48000, "channels": 2,
    "sources": [{"path": "voice.wav", "at_s": 0}],
    "master": {"integrated_lufs": -14, "true_peak_dbtp": -1}
  },
  "encode": {
    "speed": 10, "qp": 100, "tile_cols": 4, "tile_rows": 4,
    "threads": 8, "chunks": 2, "keyint_s": 60
  }
}
```

Times round to the nearest 30 fps slot. Trims are half-open. Segments start in
strictly increasing order; the most recently started segment supplies the frame.
After its trim ends, its final frame is held until the next segment or output
end. A gap before the first segment is refused (`gap_before_first_segment`).
The output contains exactly `round(duration_s * 30)` video blocks. Audio length
is `round(duration_s * 48000)` samples, within half a video frame of that grid.

Three segment kinds are accepted, with exactly one source selector:

```json
{"solid": "black", "at_s": 0, "duration_s": 0.6}
{"solid": [30, 60, 90], "at_s": 1, "duration_s": 1}
{"still": "steps/step-01.png", "at_s": 2, "duration_s": 3}
```

Solid colours are `black` or three integer RGB channels in 0..255. They access
no files. Solids and stills require a positive duration that rounds to at least
one slot, and refuse VFL trim fields; VFL segments refuse `duration_s`.
Their slot count is `round((at_s + duration_s) * 30) - round(at_s * 30)`.
They hold their last frame using the same rule as VFL trims and participate in
the same frame-count, memory, chunk and crossfade rules.
Repeated static frames use `show_existing_frame`, with the existing keyframe
successor exception. Fade frames encode normally.

PNG input uses the already locked pure-Rust `png 0.18.1` decoder (foreign-policy
`allow`, forbidden to the shell by Tier 2). Named bounds are
`STILL_FILE_BYTES_LIMIT` (64 MiB of input, including a bounded decoder read),
`STILL_DIMENSION_LIMIT` (7680 on either axis), `STILL_PIXELS_LIMIT` (7680*4320)
and `STILL_DECODE_BYTES_LIMIT` (256 MiB decoder allocation budget and output
buffer cap). Dimensions are checked from the header before allocating pixels. The full
256 MiB decoder allowance is also charged per concurrent chunk in the
render memory admission estimate whenever the timeline includes stills.
Animated PNG is refused. Palette, greyscale, RGB, alpha and 16-bit PNGs are
expanded to 8-bit BGRA; transparency is composited onto black. The existing
scaler and padding run only when the source differs from the output canvas
(normally 3840x2160). No alternate image scaler is used.

Still sources use lexical relative-path checks and descriptor-resolved
confinement beneath the opened timeline directory, require regular files and
share the 256 video-source-name cap with VFL sources. Repeated still names
share a retained descriptor and one decoded, scaled canvas per output size,
across all segments and chunks. Admission counts each retained canvas once,
in addition to each chunk's working allowance. Decode reads reopen the descriptor through
`/proc/self/fd` for independent offsets, never the original path. Refusals
include `segment_kind`, `segment_kind_fields`, `solid_colour`,
`segment_duration_s`, `still_regular_file`, `still_file_bytes_limit`,
`still_dimensions_limit`, `still_decode_bytes_limit`, `still_colour_type` and
`still_animation`. `still_colour_type` is a defensive refusal for unexpected
decoder output after palette expansion and 16-bit stripping.

`av probe FILE` detects RIFF WAV as well as WebM. WAV uses the audio reader's
validation and reports format, PCM sample format, rate, channels, sample-frame
count and duration. PCM16/24/32 and float32 mono/stereo are supported, including
ancillary RIFF chunks. Streaming WAVs with unbounded chunk lengths are refused
by the same reader used for render inputs.

A crossfade starts at an incoming segment's start. The preceding segment's held
last frame can cover a fade after its trim ends. Fades are sorted once and must
not overlap; an advancing cursor replaces per-frame searches. The frame count is
`round((at_s + duration_s) * 30) - round(at_s * 30)`. The weight runs from 0 to
255 inclusive over that interval (255 for a one-frame fade), using media-frame's
integer encoded-BGRA interpolation.
Crossfades always encode. Overlaps without a crossfade are cuts.

Optional `video.fade_in_s` and `video.fade_out_s` apply a programme envelope
after segment selection, scaling, padding, ordinary crossfades and conversion
to BT.709 limited-range I420. They fade the composed picture against black
(Y=16, U=V=128), including across any still, solid or VFL cuts. Mix supplies
the durations; source trims, holds, cut times and audio remain unchanged.
Omitted or zero durations disable the envelope and preserve existing output.

Each nonzero duration rounds to N slots and must cover at least two slots.
Fade-in occupies [0,N); fade-out occupies [programme_slots-N,programme_slots).
Their first/last weights include both endpoints: fade-in runs from black to
the composed picture over N-1 steps; fade-out runs from the composed picture
to black. This matches ffmpeg `fade=t=in:s=0:n=N-1` and
`fade=t=out:s=programme_slots-N:n=N-1` on the same composed yuv420p source.
Signed plane deltas from black use nearest-integer rounding.

Negative, non-finite, sub-two-slot and longer-than-programme nonzero durations
refuse as `programme_fade_range`. Envelopes may touch but cannot overlap
on either the seconds or rounded-slot grid (`programme_fade_overlap`).
Every slot inside either envelope encodes, even when the source repeats or
the envelope endpoint matches a neighbour; no `show_existing_frame` is used
there. Those slots count in `frames_encoded`, not `repeats`; output frame
count, timestamps, chunk boundaries and the 64 MiB chunk payload cap remain
unchanged. The plane transform runs in place on the existing output YUV
frame, using constant scratch space and the existing memory allowance.
The CBC oracle test compares decoded AV1 to ffmpeg's fade on that same
composed source with maximum absolute error of 3 in each 8-bit Y/U/V plane.

Scaling uses media-frame's separable Catmull-Rom bicubic filter (a=-0.5), with
pixel-centre alignment, clamped edges and an antialias kernel for reduction.
It maintains aspect ratio and centres on opaque black padding. Conversion is
BT.709 limited-range I420 with centred 2x2 box chroma. Filtering stays in
gamma-encoded BGRA space to match the current UI/video and
crossfade contract; it does not introduce a separate linear-light workflow.

Each chunk starts at a segment boundary or at most `keyint_s` apart. `chunks`
limits concurrent encoders; omitted, it is available cores / 8, minimum one.
`chunks * threads` must not exceed twice the available cores
(`encode_thread_budget`). A conservative live-video admission estimate must
also fit 8 GiB (`encode_memory_budget`): per concurrent chunk, eight times the
largest source BGRA canvas, the largest horizontal scaler intermediate
(`scaled_width * source_height * 16`), 448 bytes per output pixel for frame and
encoder working storage, 64 MiB for compressed payloads and 64 MiB for metadata.
The encoder allowance was raised after a 4K episode measured 6.61 GiB process
high-water RSS. This is an admission estimate, not an operating-system RSS limit; rav1e's internal
allocator and audio mastering add their own storage. Real episode peak RSS is
reported with the worker receipt. Audio has the bounded three-hour contract
documented by media-audio; its mastering storage is released before video work.
Each render encoder uses pinned rav1e 0.8.1, Main 8-bit 420, low latency,
error-resilient frames, constant QP,
no reordering and one GOP. Actual sequence OBUs must match across chunks.
The bitstream parser reads refresh masks to choose the last shown frame's slot.
It accepts only shown key/inter frames, no frame IDs, timing models or OBU
extensions. Size overrides and non-error-resilient frames are refused before
reading refresh flags; OrderHintBits follows enable_order_hint per the AV1 spec.
Unsupported headers return `unknown_reference_state`; differing
sequence headers return `sequence_header_mismatch`. No slot-index formula is used.
The first successor of each keyframe is encoded because a displayed keyframe
cannot be re-shown. Later VFL repeats become five-byte show-existing units.

WebM includes a block for every output frame, BlockDuration on video blocks and
cues for each keyframe. Millisecond timestamps alternate 33/34 ms; the segment
duration retains the exact audio sample duration. Sparse VFR playback and upload
behaviour remain outside this CFR contract. Audio uses the same mastering,
Opus reconstruction ceiling checks, pre-skip and terminal padding as `av audio`.

Unknown JSON fields, non-relative or parent-traversing paths, oversized timelines,
more than 10,000 segments/fades, duration over three hours, non-30-fps/incomplete
VFL, out-of-range trims and invalid encoder settings are refused. Exit 2 means
input/execution error; exit 3 means refused mastering; exit 4 means published
output with an undelivered receipt (EPIPE preserves the outcome). Output stages beside the
destination and publishes atomically without replacing an existing file.

The JSON receipt reports frame/encode/repeat/keyframe/chunk counts, stage wall
and process CPU seconds, effective fps, bytes and the complete audio receipt.
`peak_rss_kib` reads the Linux process high-water RSS (`VmHWM`) at receipt time,
covering validation, audio and all video workers, rather than summing per-thread
resident pages. It is null if Linux process metrics are unavailable.
`repeats` counts injected units; keyframe successors are included in encoded
frames. CPU accounting uses Linux process counters, including encoder threads.
The nested audio byte count is Opus payload bytes; the outer byte count is the
complete WebM. Only the active/outgoing segment pair retains decoded canvases per
chunk. A completed batch is muxed in order and released before another starts;
at most `chunks` results are retained. Each chunk's coded payload is capped at
64 MiB (`chunk_bytes_limit`), plus at most 324,000 five-byte repeats and bounded
frame/position metadata. Whole-episode compressed video is never retained.

At most 256 distinct normalised relative VFL and still path names are accepted
(`too_many_sources`); this conservative source-name cap is checked before any
VFL directory is opened, even if some names would resolve to the same source.
Repeated names share one open and one frame-count scan. Distinct names are opened
with O_DIRECTORY and O_NOFOLLOW, confined, then deduplicated by descriptor-resolved
canonical path: aliases share the retained descriptor and cached frame count,
and their temporary duplicate descriptor is immediately closed. At most 256
source directory descriptors are retained, plus one temporary validation open.
Their descriptor-resolved paths must lie beneath the canonical directory of the
opened timeline file. Every reader, scan and lazy chunk read uses that retained
descriptor via `/proc/self/fd`; original source paths are never reopened. Files
inside the VFL must be regular and cannot be symlinks. Segment slots are bounded
by a verified frame scan and all subsequent slot/coverage arithmetic is checked.
Source canvas sizes and all memory-estimate products and sums use checked
arithmetic, refusing overflow as `encode_memory_budget`.
Audio must explicitly request 48 kHz stereo (`audio_sample_rate`, `audio_channels`).

Worker-only benchmark reference: `cargo build --release -p av --example reference`,
then `reference TIMELINE.json` writes lossless Y4M through the exact same pixel
composition path. Captures and benchmark outputs must stay outside the repository.

The example QP, keyframe interval and chunk count are provisional measurement
settings. Choosing the YouTube master settings requires quality/performance
results; this command does not upload or install anything.
