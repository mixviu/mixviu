# media-opus

48 kHz mono/stereo Opus packets, with one trait for encoding and one for
decoding. Encoding accepts exactly 960 interleaved samples per channel and a
6000–512000 bit/s setting. Decode returns f32 PCM; `None` explicitly requests
concealment of a lost 20 ms frame and sets `packet_lost`. Empty or corrupt input
returns an error rather than silently requesting concealment.

Use the encoder's `OpusHead` fields for container initialisation. Call `finish`
to flush lookahead, and apply the final packet's `discard_padding` in samples
per channel. Together with `pre_skip`, this preserves the input duration.

The implementation is opus-pure 0.2.3 (BSD-3-Clause). The optional
`bench-cranpose` feature selects cranpose-opus-pure 0.2.2 (BSD-3-Clause), whose
spike results were identical. Both are pure Rust with no dependency build step.
Neither crate is reachable from the shell engines.

Worker test setup: run `mix tools/opus_test_setup.mix /tmp/opus-vectors`, then
set `OPUS_VECTOR_ROOT=/tmp/opus-vectors` for `cargo test -p media-opus
--release --all-targets`. Repeat with `--features bench-cranpose` for the bench
implementation. All tests require their tools and fixtures; none silently skip.

The setup verifies SHA256 for the original RFC 6716 and updated RFC 8251
archives and the external C comparator source. Vectors use the IETF Trust BSD
terms and are downloaded, not committed. The updated set is the conformance
quality gate, as RFC 8251 permits; original streams also receive full decode
and output-length checks. The spike separately recorded every original and
updated vector's comparison result and calibrated the oracle with libopus.

Interop tests require worker ffmpeg with libopus, exercise both directions at
64/96/128 kb/s in mono and stereo, and check pre-skip/end-padding duration.
They use generated tones only. The lab's broader sweep and noise-burst quality,
speed and memory measurements remain separate from these correctness tests.
