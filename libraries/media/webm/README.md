# media-webm

In-tree EBML writer and finite-file, zero-copy reader. No runtime dependencies,
Bus surface or application policy. Version and licence follow the workspace
(0.1.2, MIT OR Apache-2.0). Tests use the existing media-av1/media-frame crates.

The writer requires an empty `Write + Seek` target. Feed AV1 temporal units
and optional Opus packets in nondecreasing integer millisecond order, with
packet durations. Round timestamps once at the call boundary. Packets go
straight to the target; compact cue positions take O(keyframes) memory.
Each video keyframe or five-second gap starts a cluster. Finish patches finite
cluster/Segment sizes, Duration and SeekHead and emits Cues. Discard the file
after any write error. For a pipe, spool to a seekable temporary file first.

`av1_config` extracts the sequence OBU and constructs av1C for the media-av1
contract (Main, eight-bit, non-monochrome 4:2:0). Sequence timing/decoder model
streams are refused explicitly. The container colour metadata is BT.709,
limited range, matching media-frame. Opus supports mapping-family-zero mono
and stereo, 48 kHz decode rate, CodecDelay from pre-skip and 80 ms SeekPreRoll.

The reader borrows payloads, indexes packet headers and validates cue targets.
Timestamps are signed container ticks before CodecDelay; `scale_ns` gives their
unit. `presentation_timestamp` subtracts CodecDelay in container ticks. Seek takes container ticks and selects a video keyframe through Cues.
Audio preroll and subtracting CodecDelay are playback responsibilities.
SimpleBlock and unlaced BlockGroup are supported; all lacing is refused.
Unknown element IDs are skipped. Unknown-sized Segment is supported, but
unknown-sized child masters (live clusters) are explicitly refused.
Metadata/index allocation is O(input length); packet payloads are never copied.

Implementation references: [RFC 9559](https://www.rfc-editor.org/rfc/rfc9559),
[Matroska codec mappings](https://www.matroska.org/technical/codec_specs.html),
and [AV1 configuration record](https://aomediacodec.github.io/av1-isobmff/).
No filmcraft-matroska or matroska-demuxer code was read or copied; no dependency
or associated foreign licence is introduced.

`tests/fixtures/sine.webm` is generated demo audio, owned by this project and
released under the workspace licence. Generated with ffmpeg n9.1/libopus:

```
ffmpeg -v error -f lavfi -i sine=frequency=440:sample_rate=48000:duration=0.3 -c:a libopus -frame_duration 20 sine.webm
```

`cargo test -p media-webm --all-targets` requires ffmpeg/ffprobe and AV1/Opus
decoders. Missing tools/decoders fail. The ignored `sample_4k` test writes a
10-second 3840x2160 25 fps sample and probe JSON when `MEDIA_WEBM_SAMPLE` is set.
Run builds/tests only through the repository's worker build service.

The foreign gate includes media-webm in the existing rav1e admission because
its test-only media-av1 dependency supplies generated packets, including the
permitted assembly feature. The library has no runtime dependencies.
