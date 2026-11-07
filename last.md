# lab/image-builder (Phase 2d lab, never lands)

One Mix builder (`tools/image.mix`), one pinned snapshot
(`etc/image/pacman.conf`: Arch Linux Archive 2026-11-06), one package list
(`etc/image/packages.conf.mix`). Derivation plan and full evidence: `PR.md`.

- Build image: 214 packages, rootfs 2.35 GB, OCI tarball 761 MB, manifest
  `sha256:b1c4e051a093283444e466937e5edc994c7fec1c558f127addb5d5dd40131ac5`;
  `cargo build --locked -p mix` ran inside the tree unpacked from the packed
  layer (33.7 s, rustc 1.97.1); a second build gave the same digest.
- Runtime image: 125 packages, rootfs 465 MB, OCI tarball 169 MB, lean.
- Runtime list derived from ldd of mix, mixd, viu, viud built in the image.
- systemd-nspawn cannot run in the LXC CBC workers; the builder chroots with
  explicit mounts instead. Scratch removed on both workers; nothing published.

Branch lab/image-builder; head: see `git log -1` (this commit's parent is the evidence head 02984883); gate: GATE_PLACEHOLDER; CBC jobs 20261107-205007-640e065c, 20261107-205619-db9bcc18, 20261107-205945-7fcf9ef8, 20261107-210646-d7118b48, 20261107-211449-3de6d7d9, 20261107-212349-b8813fc4; needs Mark: nspawn-vs-chroot on CBC, snapshot security cadence ("official release distro"), lua54 in viud.
