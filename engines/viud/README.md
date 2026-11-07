# viud

The MixViu Wayland compositor: the MixOS compositor without the in-process
egui widget toolkit. Furniture and applications run out of process in `viu`.
`cargo tree --locked -p viud -e normal` must show no `egui`, `egui-wgpu`,
`egui-winit` or `toolkit`.

Its private crates live in `crates/`, its units in `units/` (`viud@`,
`viud-headless@`). Configuration lives in `~/.config/viud`, environment
variables are `VIUD_*` (and `MIXOS_VIUD_CONFIG`). The Bus service is still
`comp` with its `comp.*` commands: that is the protocol name.
