# Reusable time-series widget

`libraries/viu/src/time_series.rs` adapts the renderer from MixOS
`8779f33284b7d3f1178c3d57a699efb9b0867b22`,
`apps/monitor/src/chart.rs`. Original file SHA256:
`508eb838b0c0b9b22e7b27ea64c2d339b63a7d5a5985d3db352b44421a19175e`.
Original MIT OR Apache-2.0 licence and copyright are retained.

The bounded JSON adapter supplies names, legends, series, units, ranges,
interval, size and context. Resource selection, sampling, seeding, history
alignment and page composition remain Mix policy. Native range navigation,
series visibility, readouts and axis layout reuse the gold implementation.
The former node reset is a generic context reset. The English age readout
matches the pinned original catalogue; additional localisation is unaccepted.

Dependency inspection: the existing workspace pin `egui_plot =0.38.0`, with
default features disabled, is MIT OR Apache-2.0, has `build = false` and
denies unsafe code. Its enabled dependencies are ahash (no-rng/std), egui,
emath and nohash-hasher; there is no C build or new service. Cargo.lock
changes must be returned by CBC. This is lab implementation evidence,
not Monitor or widget acceptance; gate receipts belong in `last.md`.
