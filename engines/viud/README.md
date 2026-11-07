# viud

The MixViu Wayland compositor: the MixOS compd compositor without the
in-process egui widget toolkit. Furniture and applications run out of process
in `viu`. `cargo tree --locked -p viud -e normal` must show no `egui`,
`egui-wgpu`, `egui-winit` or `toolkit`.

Its private crates live in `crates/`. Bus service names, configuration paths
and environment variables keep their compd names for now (`comp`,
`~/.config/compd`, `COMPD_*`); renaming them is a runtime change of its own.
