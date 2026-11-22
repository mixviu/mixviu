# media-quality

Pure Rust 8-bit planar YUV420 SSIM and PSNR over `media_frame::Yuv420`.
No new third-party dependencies. Package 0.1.2, MIT OR Apache-2.0.

The numerical specification is FFmpeg **n9.1**:
[vf_ssim.c](https://github.com/FFmpeg/FFmpeg/blob/n9.1/libavfilter/vf_ssim.c)
and [vf_psnr.c](https://github.com/FFmpeg/FFmpeg/blob/n9.1/libavfilter/vf_psnr.c).
This is an independent implementation of the formulas, without vendored C.
SSIM uses 4x4 integer moments, overlapping 8x8 windows at stride 4,
rounded constants 416 and 235963, and f32 products/division per window.
Trailing partial blocks are discarded. Plane dimensions use ceil chroma
division; All uses actual plane area, including samples outside SSIM windows.
PSNR uses full-plane squared error and 255 as peak, including limited range.
The combined PSNR is computed from area-weighted MSE, not weighted dB.
`Decibels::Infinite` represents zero error; no NaN is returned.

`ssim` and `psnr` validate public frame storage before indexing. Mismatched
sizes and malformed storage have distinct errors. SSIM refuses frames with
any plane smaller than 8x8 (`NoSsimWindow`), where FFmpeg's window denominator
is zero; PSNR accepts every valid size. `Sequence::update` returns per-frame
metrics. `summary` returns componentwise minima and means in constant memory;
PSNR mean is PSNR of mean frame MSE, matching FFmpeg's sequence summary.
An empty sequence has no summary. Failed updates do not change the sequence.

The `quality` example compares two streaming Y4M files, or stdin and one file:

```text
cargo run --release -p media-quality --example quality -- a.y4m b.y4m
cargo run --release -p media-quality --example quality -- reference.y4m < input.y4m
```

Output combines SSIM and PSNR stats-file fields in one line per frame,
followed by SSIM and PSNR summary lines. The reader supports progressive
8-bit C420/jpeg/mpeg2/paldv only, does no resampling, refuses malformed or
truncated streams and unequal frame counts. Input frame rate/tags are ignored:
frames are paired by index. Explicitly align time and chroma siting upstream.

Focused CBC driver: `mix libraries/media/quality/worker.mix`. It formats,
runs all-target clippy, all test targets for this crate and logging, requires
the ffmpeg oracle, runs the foreign gate, and measures 120 frames of 4K SSIM
in release-fast. No Rust command runs on a developer desktop.
The unconditional tests pin identity, analytic offset, error distinctions,
sequence aggregation, odd-edge cropping/area weights and Y4M streaming.
The oracle generates moving detail, odd-sized gradients and deck-like panels
using ffmpeg alone; it needs no provisioned fixture. It compares 18 frames,
all four metric components. Missing ffmpeg/filters prints SKIP unless
`MIX_TEST_REQUIRE_FFMPEG=1`. SSIM tolerance is 1e-6; PSNR must agree within
1e-6 dB or exactly at ffmpeg's two printed decimals.
Real private deck/corpus captures are separate evidence, not CI fixtures.
