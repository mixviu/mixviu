# First AV1 numbers

Generated fixtures only, eight frames each at 30fps; rav1e 0.8.1, q100,
low-latency, two threads, actual 2x2 tiles. Intel Core i9-13900H worker,
release-fast profile (no LTO), ffmpeg 9.0.2/libdav1d decoder. All 36 streams
decoded to exactly eight frames at the requested dimensions. PSNR compares
decoded YUV420 against the L6 YUV420 source, not the original BGRA.

FPS is encoder-only (submit plus flush), excluding generation, conversion,
preparation, context creation and file I/O. Bitrate is AV1 payload over 8/30s.
Single run per case; key-frame overhead dominates. These are first numbers,
not the av1enc suite, steady-state throughput or a cut-over verdict.

| Resolution | Scene | Preset | asm | fps | kb/s | PSNR dB |
|---|---|---:|---|---:|---:|---:|
| 1920x1080 | static | 6 | true | 4.4492 | 3586.7100 | 44.621208 |
| 1920x1080 | scroll | 6 | true | 3.0573 | 3625.5900 | 44.578753 |
| 1920x1080 | drag | 6 | true | 4.6373 | 2633.5200 | 45.305597 |
| 1920x1080 | static | 8 | true | 8.1283 | 3559.4400 | 43.492288 |
| 1920x1080 | scroll | 8 | true | 5.8925 | 3726.1200 | 43.324161 |
| 1920x1080 | drag | 8 | true | 8.5004 | 2632.2000 | 44.354154 |
| 1920x1080 | static | 10 | true | 15.4394 | 4734.0600 | 43.070760 |
| 1920x1080 | scroll | 10 | true | 10.9736 | 5330.4900 | 42.936251 |
| 1920x1080 | drag | 10 | true | 16.2528 | 3799.2900 | 43.821595 |
| 3840x2160 | static | 6 | true | 1.2607 | 7291.5000 | 45.459631 |
| 3840x2160 | scroll | 6 | true | 0.9968 | 7358.5500 | 45.475192 |
| 3840x2160 | drag | 6 | true | 1.2696 | 6280.4700 | 45.604722 |
| 3840x2160 | static | 8 | true | 2.5659 | 7203.1200 | 44.704501 |
| 3840x2160 | scroll | 8 | true | 1.9659 | 7420.7400 | 44.629201 |
| 3840x2160 | drag | 8 | true | 2.3663 | 6277.6200 | 45.006705 |
| 3840x2160 | static | 10 | true | 3.9539 | 9518.9400 | 44.411665 |
| 3840x2160 | scroll | 10 | true | 3.1318 | 9814.2600 | 44.408601 |
| 3840x2160 | drag | 10 | true | 4.1164 | 8634.6300 | 44.566290 |
| 1920x1080 | static | 6 | false | 1.3534 | 3586.7100 | 44.621208 |
| 1920x1080 | scroll | 6 | false | 0.9908 | 3625.5900 | 44.578753 |
| 1920x1080 | drag | 6 | false | 1.4016 | 2633.5200 | 45.305597 |
| 1920x1080 | static | 8 | false | 2.4653 | 3559.4400 | 43.492288 |
| 1920x1080 | scroll | 8 | false | 1.7400 | 3726.1200 | 43.324161 |
| 1920x1080 | drag | 8 | false | 2.5630 | 2632.2000 | 44.354154 |
| 1920x1080 | static | 10 | false | 4.5213 | 4734.0600 | 43.070760 |
| 1920x1080 | scroll | 10 | false | 3.2679 | 5330.4900 | 42.936251 |
| 1920x1080 | drag | 10 | false | 4.8318 | 3799.2900 | 43.821595 |
| 3840x2160 | static | 6 | false | 0.3638 | 7291.5000 | 45.459631 |
| 3840x2160 | scroll | 6 | false | 0.2888 | 7358.5500 | 45.475192 |
| 3840x2160 | drag | 6 | false | 0.3611 | 6280.4700 | 45.604722 |
| 3840x2160 | static | 8 | false | 0.7063 | 7203.1200 | 44.704501 |
| 3840x2160 | scroll | 8 | false | 0.5346 | 7420.7400 | 44.629201 |
| 3840x2160 | drag | 8 | false | 0.6702 | 6277.6200 | 45.006705 |
| 3840x2160 | static | 10 | false | 1.1708 | 9518.9400 | 44.411665 |
| 3840x2160 | scroll | 10 | false | 0.9234 | 9814.2600 | 44.408601 |
| 3840x2160 | drag | 10 | false | 1.2142 | 8634.6300 | 44.566290 |
