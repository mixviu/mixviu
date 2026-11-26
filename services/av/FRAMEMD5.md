# Decoded frame checksums

`av decode FILE.webm [--y4m OUT.y4m] [--framemd5 OUT.txt] [--wav OUT.wav] [--max-output-bytes N]`
requires at least one video output. Both video outputs may be requested together.
All outputs stage and publish together; failures leave no partial outputs.
The byte budget counts all output bytes, including checksum text and WAV.

The version 1 checksum format is UTF-8 text, with one header followed by one
line per decoded frame in output order:

```text
#av-framemd5 v1 W3840 H2160 yuv420p
0 d41d8cd98f00b204e9800998ecf8427e 12441600 0
```

The illustrative digest above is a placeholder. Columns are zero-based frame
index, lowercase MD5, packed frame byte count and signed timestamp in nanoseconds.
Only 8-bit 4:2:0 is supported. Each digest hashes tightly packed Y, U, V planes
in that order, excluding Y4M headers and FRAME tags. Compare the digest column
with the final comma-separated column from:

```console
ffmpeg -c:v libdav1d -i FILE.webm -map 0:v:0 -fps_mode passthrough -f framemd5 -
```

The JSON receipt reports `framemd5` and `framemd5_lines` (data lines only).
`frames` remains the decoded frame count; `output_bytes` is the total bytes written.
Y4M uses whole-stream inferred frame rate when available and retains per-frame
`XPTS` nanoseconds for variable-rate streams, falling back to the first duration.
