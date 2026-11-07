# Workshop parity adaptations

Source: MixOS commit `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Licence: MIT OR Apache-2.0, retained in adapted source headers.

Workshop application policy in `apps/workshop/*.mix` follows the pinned
`apps/workshop/crates/inspector/src/{engine,model,workshop,mesh,agents,bus}.rs`
and application `src/{view,agents,mesh,saved,shell,workshop,main}.rs`.

The measured-label radial layout, separate directional Bézier paths, dashed
unobserved links and arc-length sampling in `libraries/viu/src/network_map.rs`
adapt the pure geometry routines in pinned `apps/workshop/src/mesh.rs`.
The reusable canvas receives owner-authored topology, styling and interaction
labels. Application discovery, traffic tally, observation, probes, selection
and command outcomes remain in Mix.

`libraries/viu/src/data_panels.rs` supplies data-authored text blocks,
collapsible content, forms, cell grids and small history lines using existing
egui/toolkit controls. Workshop's corresponding Mix projections retain all
session, permission and transcript policy.

The reusable dock `reference_extent` placement input follows the pinned
Workshop view's full-window fraction sizing, including its narrow-window
available-size rule. Explicit root dock `clip:false` preserves native panel
paint margins across the shared frame boundary. Default frame clipping is
unchanged. Native paired captures cover both expanded Traffic placements.
