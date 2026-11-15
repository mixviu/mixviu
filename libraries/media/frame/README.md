# Frame primitives

Dependency-free Rust. Packed, tightly-strided **straight-alpha** BGRA; callers
must unpremultiply premultiplied capture buffers before blending. Conversion
ignores alpha: composite first. Scale is nearest-neighbour; pad rejects crops;
blend is source-over; crossfade interpolates encoded bytes (not linear light).
Still-hold clones an Arc, not pixels. These are primitives, not timeline policy.

BT.709 Y′CbCr uses 8-bit limited range (Y 16–235, Cb/Cr 16–240).
Chroma is centred at (0.5, 0.5) in each 2×2 luma block, box filtered before
rounding, with replicated odd edges. This avoids phase-biased coloured UI text
and limits aliasing, but cannot preserve single-pixel colour detail in 4:2:0.
AV1 has no explicit centred enum: signal Unknown and carry centre siting in the
container. A fair quality comparison must use identical siting/filter/matrix.

`oracle.mix` is an opt-in focused worker job, never part of crate tests. It
compares generated bars and high-frequency colour patterns with ffmpeg area
conversion and also exercises the reverse conversion. Per-plane tolerance:
max 2 code values, mean 0.6, allowing fixed-point and rounding differences
without allowing a matrix/range mismatch. First run: Y max 1 / mean 0.003784,
U 1 / 0.001465, V 1 / 0.002441. This is a small correctness fixture, not VMAF
or a corpus benchmark. Normal tests require no ffmpeg.

Original implementation; no imported code or third-party dependencies.
