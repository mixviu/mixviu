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
    "crossfades": []
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

A crossfade starts at an incoming segment's start. The preceding segment must
cover the complete overlap. The weight runs from 0 to 255 inclusive over the
rounded duration, using media-frame's integer encoded-BGRA interpolation.
Crossfades always encode. Overlaps without a crossfade are cuts.

Scaling uses media-frame's separable Catmull-Rom bicubic filter (a=-0.5), with
pixel-centre alignment, clamped edges and an antialias kernel for reduction.
It maintains aspect ratio and centres on opaque black padding. Conversion is
BT.709 limited-range I420 with centred 2x2 box chroma. Filtering stays in
gamma-encoded BGRA space to match the current UI/video and
crossfade contract; it does not introduce a separate linear-light workflow.

Each chunk starts at a segment boundary or at most `keyint_s` apart. `chunks`
limits concurrent encoders; omitted, it is available cores / 8, minimum one.
Each encoder uses pinned rav1e 0.8.1, Main 8-bit 420, low latency, constant QP,
no reordering and one GOP. Actual sequence OBUs must match across chunks.
The bitstream parser reads refresh masks to choose the last shown frame's slot.
It accepts only shown key/inter frames, no frame IDs, timing models or OBU
extensions. Unsupported headers return `unknown_reference_state`; differing
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
input/execution error; exit 3 means refused mastering. Output stages beside the
destination and publishes atomically without replacing an existing file.

The JSON receipt reports frame/encode/repeat/keyframe/chunk counts, stage wall
and process CPU seconds, effective fps, bytes and the complete audio receipt.
`repeats` counts injected units; keyframe successors are included in encoded
frames. CPU accounting uses Linux process counters, including encoder threads.
The nested audio byte count is Opus payload bytes; the outer byte count is the
complete WebM. Only the active/outgoing VFL pair retains decoded canvases per
chunk. Completed compressed chunks remain in memory until muxing.

Worker-only benchmark reference: `cargo build --release -p av --example reference`,
then `reference TIMELINE.json` writes lossless Y4M through the exact same pixel
composition path. Captures and benchmark outputs must stay outside the repository.

The example QP, keyframe interval and chunk count are provisional measurement
settings. Choosing the YouTube master settings requires quality/performance
results; this command does not upload or install anything.
