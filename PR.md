# Derivation plan: one image builder (Phase 2d)

`lab/image-builder` is a discovery lab and never lands. This file is the
plan the production change derives from, plus what the lab measured.

## Summary

One Mix builder (`tools/image.mix`), one pinned package snapshot
(`etc/image/pacman.conf`, a dated Arch Linux Archive day) and one explicit
package list (`etc/image/packages.conf.mix`) produce the MixViu images. The
lab built the BUILD image on a CBC worker, packed it as an OCI image layout
in Mix, unpacked that layout again and ran `cargo build --locked -p mix`
inside it under systemd-nspawn. Nothing was published.

## What the lab built (evidence)

Final run: CBC job `20261107-211449-3de6d7d9` on a heavy CBC worker, source
`a1480914b256`, `mix tools/image.mix all --kind build --work ... --repro --purge`,
BUILD_RC=0.

| Measure | Value |
|---|---|
| Snapshot | Arch Linux Archive 2026-11-06 (core, extra) |
| Packages asked / installed | 29 / 214 |
| Toolchain | rustup 1.97.1 (from `rust-toolchain.toml`), clippy + rustfmt, minimal profile |
| Rootfs | 2,352,290,157 bytes (2.35 GB) |
| Layer | tar 2,378,772,480 bytes, gzip 761,137,642 bytes |
| OCI tarball | 761,149,440 bytes, sha256 `c3e048ab6af0...ceaf1` |
| Manifest digest | `sha256:b1c4e051a093283444e466937e5edc994c7fec1c558f127addb5d5dd40131ac5` |
| Config digest | `sha256:2ee81bf067e4f84bd9d89a84abd12036b24f71952c09ca5e9cd533897aa9098a` |
| Layer digest / diff_id | `sha256:259c1e6d...02ed` / `sha256:b3c70b4c...f4b2` |
| Tag (lab, local only) | `lab-build-20261106-a148091` |
| Proof | `cargo build --locked -p mix --bin mix` in the tree unpacked from the packed layer: 33.7 s, `mix 0.119.0 (a1480914b256)`, rustc 1.97.1 |
| Reproducible | second build + pack from the same snapshot: identical diff_id and manifest digest, 0 differing files |
| Timings | pacman 16-81 s (cache cold/warm), rustup 6 s, tar 2.3 s, gzip 76 s |

Runtime survey (same job, debug builds, default features): `mix` and `mixd`
link only glibc and libgcc; `viu` adds libxkbcommon; `viud` links libinput,
libdrm, mesa (gbm), seatd, pixman, systemd-libs, dbus, libpulse, libxcb,
libxkbcommon and lua54, plus their dependencies. The runtime list in
`etc/image/packages.conf.mix` is those direct owners plus run-time-opened
data and drivers (vulkan-icd-loader, xkeyboard-config, ca-certificates).

## Survey of `desk_image.mix` (read-only)

What carries over:

- An explicit package list resolved by pacman, never a cloned machine.
- A pinned `pacman.conf` with `SigLevel = Required` and `NoExtract` for
  locale, docs, man pages (and for lean kinds, headers, GIR and Python).
- Lean checks after install: forbidden packages (python, perl, pacman,
  gnupg) and directories that must be empty.
- Packages stay out of the image (`pacstrap -c` there, an external
  `--cachedir` here; sync databases removed).

What changes:

- The desk builder pins a live mirror (`geo.mirror.pkgbuild.com`), so two
  builds a day apart differ. The image builder pins a dated archive day and
  refuses a live mirror or mixed days.
- `pacstrap` is not on the workers (no `arch-install-scripts`). The builder
  does what pacstrap does with `pacman --root` plus the proc/sys/dev/run/tmp
  mounts its scriptlets need, all unmounted in `finally` and checked before
  any removal (`rm --one-file-system`). No extra host package needed.
- Desk-only layers (ZFS dataset, host gid alignment, the `mixos` account,
  units, session env, profile, Firefox) are not image concerns. They become
  the desk output's overlay step (below), not part of the base tree.

## Derivation (the production change)

1. **Builder in `tools/image.mix`, three outputs from one tree.**
   - `runtime`: the runtime list only, lean `NoExtract`, forbid list
     enforced. Packed as OCI (`ghcr.io/mixviu/runtime`).
   - `build`: runtime + toolchain + headers, rustup toolchain from
     `rust-toolchain.toml` preinstalled in `/usr/local/rustup`. Packed as OCI
     (`ghcr.io/mixviu/build`). CI's `container.image` pins it by digest and
     the per-run `pacman -Syu` step goes away.
   - `desk`: the runtime tree plus a Mix overlay (account, units, session,
     profile) ported from `desk_image.mix`, written as an nspawn rootfs
     directory; never an OCI. The hub `desk_image.mix` stays for gold desks
     until the desk output replaces it in its own PR.
2. **Publish from `main` only.** A workflow on push to `main` (and
   `workflow_dispatch`) builds both OCI images in a privileged runner job,
   signs them with the engine artefacts' key, and pushes to
   `ghcr.io/mixviu/{build,runtime}` with an immutable `X.Y.Z-<sha7>` tag
   (`X.Y.Z` from `[workspace.package] version`) plus the snapshot day as an
   annotation. Tags are never moved; there is no `latest`.
3. **CI pins by digest.** `ci.yml` names `ghcr.io/mixviu/build@sha256:...`.
   Changing the digest is a Tier 2 PR (CODEOWNERS), usually the same PR
   that bumps the snapshot day.
4. **Snapshot cadence.** A snapshot bump is one commit changing the day in
   every `Server` line; the builder refuses mixed days. Edge moves the day;
   stable promotes after soak. Owning that security cadence is Mark's call
   (below).
5. **aarch64** later on `ubuntu-24.04-arm` with the same builder and
   `Architecture` from the runner.
6. **Runtime list from evidence.** `tools/image.mix survey` maps each
   engine's `ldd` to owning packages; a CI check fails when an engine links a
   library whose package is not in the runtime list.

## What the lab learned

1. **systemd-nspawn does not run on the CBC workers.** They are LXC
   containers. From an ssh session nspawn fails resetting the audit login
   UID; from a `systemd-run` unit the kernel refuses its fresh proc mount.
   The builder runs scriptlets and the proof in a chroot with explicit
   proc/sys/dev/run/tmp mounts and binds, undone in `finally` and checked
   before any removal. Production on GitHub runners can use the same
   chroot path or a container step; the desk output keeps nspawn on hosts.
2. **pacstrap is not needed.** `pacman --root` plus the mounts does the
   same job; no `arch-install-scripts` on the workers.
3. **Reproducibility needs four normalisations**, measured: pacman's
   `%INSTALLDATE%` in every `local/*/desc` (set to the snapshot day), the
   pacman log, ldconfig's aux-cache and fontconfig caches (regenerated at
   run time), and p11-kit's Java trust store (unused). With tar `--sort=name`,
   numeric owners, mtimes clamped to the snapshot day, no atime/ctime and
   `gzip -n`, two builds give the same manifest digest.
4. **The runtime set is smaller than CI's pacman line.** No engine links
   wayland, fontconfig, freetype2, pipewire or alsa-lib in the default
   features; those stay in the build list because CI installs them (crate
   build scripts and feature-gated code). viud's `backend-all` (desk) build
   is not surveyed; it may add pipewire and more. lua54 linked into viud is
   worth a look under the foreign-code tiers.
5. **Signatures checked against the worker's keyring** (archlinux-keyring
   20260909). A snapshot newer than a worker's keyring can fail on a new
   packager key; the derived builder should seed its own gpgdir from the
   snapshot's archlinux-keyring.
6. **Scriptlet noise.** Install order runs a few scriptlets before their
   tools exist (fontconfig's `vercmp`, pipewire's `systemctl`); pacstrap
   has the same behaviour. Harmless for an image; the run that counted had
   none.
7. **Mix:** `type()` spells `list` on mix 0.117 (the workers) and `List`
   on 0.119; `step` is a keyword. The builder lowercases the type.
8. **Cost:** a full build image run (build, pack, prove, survey, repro) is
   about 8 minutes on one heavy worker and holds its mixviu checkout lock
   for that time.

## Risk tier

Tier 2 when derived (release/signing, CI image). The lab itself touches no
shipped path.

## How tested

- Run: CBC jobs `20261107-205007-640e065c` (rc 1: nspawn), `20261107-205619-db9bcc18`
  (rc 1: capture option), `20261107-205945-7fcf9ef8` (rc 0, survey
  ownership bug), `20261107-210646-d7118b48` (rc 0, repro found 218
  differing files), `20261107-211449-3de6d7d9` (rc 0, reproducible, final
  evidence). Lint/env gate: see `last.md`.
- Observed: the table above; scratch `/build/image-lab-*` absent on both
  workers afterwards.
- Not tested: the runtime kind (list and lean checks written, not built);
  the desk output; publishing, signing and CI pinning (none of it exists);
  aarch64; a build on a GitHub runner; viud `backend-all` linkage.

## How to undo

Lab: delete the branch. Derived change: revert the PR; CI falls back to the
`archlinux:base-devel` digest and its pacman line.

## Authorship and review

- Author: Claude Opus 5.5 (lab worker).
- Agent reviewers: none yet (lab; review applies to the derived PR).
- Explainer: this file.
