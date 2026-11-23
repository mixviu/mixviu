AV1 decoding lives in `media-av1` beside the encoder: the codec boundary is
independent of containers, CLI I/O and the future player. rav1d is BSD-2-Clause,
pinned to upstream revision `2e7216c20ac2a75de204d6c8d7268b75a45959b4`
(https://github.com/memorysafety/rav1d/tree/2e7216c20ac2a75de204d6c8d7268b75a45959b4).
Its x86 build script runs
nasm on `.asm` files and uses cc to archive those objects, with no C sources.
The default `decode-asm` feature enables rav1d assembly; `asm` also enables the
encoder's existing optional assembly. Build
with `--no-default-features` to measure the pure Rust paths.
The released 1.1.0 aborts on corrupted input during error teardown and cannot
meet this API's error contract. The pinned upstream fixes that teardown; the
mutation regression remains required. Both upstream bit-depth features are built;
the wrapper still refuses high-bit-depth input before picture allocation.

`Decoder::submit` accepts sized, low-overhead OBUs and a caller timestamp in
nanoseconds. The callback receives tightly packed, owned Y/U/V planes, including
show-existing repeats. `finish` drains delayed frames. A decoding or callback
error closes the decoder. Frame count is caller-limited; sequence dimensions
are checked before picture allocation (8192x4320, 8-bit I420), and compressed
input is capped at 64 MiB per submit. Output is streamed rather than collected.

WebM uses the existing reader rather than another dependency: it remains below
800 lines, borrows payloads, and needs only a bounded EBML subset. Limits are
256 MiB input, 64 MiB leaf/master metadata elements, depth 8, 2 million elements,
64 tracks, 500,000 packets and 100,000 cues. Unknown IDs are skipped. Lacing and
unknown-sized Clusters are refused explicitly. Segment unknown size is allowed.
This is a finite-file API, not yet a streaming reader for long recordings.

`av probe` reports duration in seconds, actual decoded video frame count, packet
counts, video keyframes and validated cues. FPS is nominal (DefaultDuration,
BlockDuration, then the first positive packet delta). `av decode` writes Y4M
with nanosecond XPTS frame tags and float32 WAV, applying OpusHead pre-skip,
output gain and signed DiscardPadding. Y4M size changes are refused.

Decode stages both files using the same public `media_audio::pending::Pending`
module as audio: same-directory temporary, checked flush and sync, no-clobber
hard link, inode verification, and rollback until every output is published.
Existing destinations (including symlinks) refuse. Zero video frames refuse.
Success prints one JSON receipt; exit codes are 0 for success (also broken receipt
pipe), 2 for errors before publication and 4 for other receipt I/O errors after
publication, matching audio.

The shared Y4M/WAV output budget defaults to 64 GiB, including headers/tags;
`--max-output-bytes N` sets a positive u64 byte cap. Before each video frame the
frame count times its packed plane size and the total output bytes are checked.
WAV header and PCM bytes share the same budget. Limit errors publish nothing.
If Segment Duration and a nominal frame interval are known, output is limited to
ceil(duration / interval) plus two frames (also capped at 500,000). The interval
uses DefaultDuration, BlockDuration, then the first positive timestamp delta.
Without duration or an interval, there is no independent duration/rate bound;
the byte cap and 500,000-frame limit still apply. Timestamp arithmetic errors
refuse cleanly. Probe checks timestamp inference but does not publish outputs.

Worker checks: `mix services/av/focused.mix`; informational benchmark:
`mix services/av/bench.mix`. Neither invokes the full CI suite.
