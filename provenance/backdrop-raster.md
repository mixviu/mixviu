# Bounded raster decoder

Source: MixOS 8779f33284b7d3f1178c3d57a699efb9b0867b22, apps/backdrop/crates/wallpaper/src/decode.rs.

Immutable upstream Git blob: 5671e1ceaac7d70dce4e01f608e8e6d274c6631b. SPDX MIT OR Apache-2.0 and copyright retained. Adapted SHA256: 367d6cade74d7e1ae6cffe6c21d20986eac2b3762f51ab31bffa0e2ebad9a5eb.

Adaptation: move into the shared Viu library for retained raster facilities; replace the owning app schema constant with a local MAX_FILES=4096. Decode, bounded read, folder discovery, downscale and tests otherwise retained. This primitive contains no slideshow policy or Bus transport.

## Round-two shared primitives

Pinned source revision remains 8779f33284b7d3f1178c3d57a699efb9b0867b22. Existing egui_plot, wgpu and egui-wgpu dependencies are reused; no dependency or licence closure was added. All source copyright and SPDX declarations are retained.

- apps/backdrop/crates/wallpaper/src/fit.rs (Git blob 0bfa4161677290995f34ecb4554aa74de88eb7d0) -> libraries/viu/src/raster_fit.rs.
- apps/backdrop/src/program.rs (Git blob 5d3f31a72b8f086a27aefa60afaa07c2e6a110b6) -> libraries/viu/src/raster_fill.rs.
- apps/backdrop/src/views/train_gpu.rs (Git blob f6a9d00bbb2a409c2025da2636fa54bb5a6ae4ff) -> libraries/viu/src/cuboids.rs.
- apps/backdrop/src/views/train.wgsl (Git blob 1e44062c4869a54452b8c02b9f82dfb5f20945d1) -> libraries/viu/src/cuboids.wgsl.

Placement remains unchanged. Raster fill adapts upload, placement and bars/crossfade into a retained-handle widget; app scheduling remains Mix. Cuboids retains the bounded GPU pipelines and shader, replacing application-specific car movement and label eligibility with explicit per-block motion vectors and label_visible data. It shares the native raster texture budget. The composition widget is an application-neutral bounded tree of native frames, columns, labels, progress, plots and worlds; Mix supplies every node and service interpretation.
