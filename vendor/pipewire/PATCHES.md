# vendor/pipewire: local patches

pipewire 0.10.1 from crates.io, plus one addition. Wired through
`[patch.crates-io] pipewire = { path = "vendor/pipewire" }`.

## Upstream base: crates.io `pipewire` 0.10.1

- Archive: `https://static.crates.io/crates/pipewire/pipewire-0.10.1.crate`
- SHA-256: `bde71084c4e25959d68f1ea54daa75e5ecdb338e5caf0b5510143b79baa32d5c`
  (matches the `checksum` in `Cargo.lock`).

Method: download the archive, check its SHA-256, untar, copy over this
directory, then apply `mixviu.patch` from the crate root
(`git apply -p1 mixviu.patch`).

## Local edits

The whole local delta is `mixviu.patch`: the diff of this directory against
the unpacked archive, `PATCHES.md` and `mixviu.patch` themselves excepted.

- `src/buffer.rs`: `Buffer::find_meta_mut<T>(&mut self) -> Option<&mut T>`,
  the mutable twin of `find_meta`. It is needed so a producer can stamp
  `SPA_META_Header` (PTS and sequence) on a buffer it dequeued. Upstream only
  exposes a shared reference. The buffer is borrowed exclusively, so the
  metadata is too.

Checksums (SHA-256) of the one patched file:

| File | Upstream | Patched |
|---|---|---|
| `src/buffer.rs` | `d829b2540192a39c43bbc1a1964573b8054812a67ee9cac2efbf14225230857f` | `cdea98fbd33988359b41efec09f86a11fe40ded2c4c6e63f77867d47559a4fdb` |

Every other file is byte-identical to the archive.

## Guard

The portald build asserts, at compile time, that its mirror of
`spa_meta_header` matches the C size
(`services/portald/src/screencast/stream.rs`, `MetaHeader`). Re-vendoring
must re-apply the edit above; the build fails if `find_meta_mut` is missing.
