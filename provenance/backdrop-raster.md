# Bounded raster decoder

Source: MixOS 8779f33284b7d3f1178c3d57a699efb9b0867b22, apps/backdrop/crates/wallpaper/src/decode.rs.

Immutable upstream Git blob: 5671e1ceaac7d70dce4e01f608e8e6d274c6631b. SPDX MIT OR Apache-2.0 and copyright retained. Adapted SHA256: 367d6cade74d7e1ae6cffe6c21d20986eac2b3762f51ab31bffa0e2ebad9a5eb.

Adaptation: move into the shared Viu library for retained raster facilities; replace the owning app schema constant with a local MAX_FILES=4096. Decode, bounded read, folder discovery, downscale and tests otherwise retained. This primitive contains no slideshow policy or Bus transport.
