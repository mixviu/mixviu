# Resolved dependency provenance

Worker cargo metadata/tree inspection, 2026-11-15. Crates.io archives are
pinned by Cargo.lock checksums. Same-name versions elsewhere in the workspace
may also appear in this inventory; it is a conservative licence inventory.
Only rav1e builds foreign code in this encoder closure (admitted nasm asm).
No decoder test, C API, binary CLI or git2 features are selected.

```text
aligned 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-embedded-community/aligned
aligned-vec 0.6.4 | MIT | https://github.com/sarah-ek/aligned-vec/
anyhow 1.0.104 | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow
arg_enum_proc_macro 0.3.4 | MIT | https://github.com/lu-zero/arg_enum_proc_macro
arrayvec 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec
as-slice 0.2.1 | MIT OR Apache-2.0 | https://github.com/japaric/as-slice
autocfg 1.5.1 | Apache-2.0 OR MIT | https://github.com/cuviper/autocfg
av-scenechange 0.14.1 | MIT | https://github.com/rust-av/av-scenechange
av1-grain 0.2.5 | BSD-2-Clause | https://github.com/rust-av/av1-grain
bitstream-io 4.10.0 | MIT/Apache-2.0 | https://github.com/tuffy/bitstream-io
built 0.8.1 | MIT | https://github.com/lukaslueg/built
cc 1.11.0 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs
cfg-if 1.0.4 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if
crossbeam-deque 0.8.8 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam
crossbeam-epoch 0.9.21 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam
crossbeam-utils 0.8.22 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam
either 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either
equator 0.4.2 | MIT | https://github.com/sarah-ek/equator/
equator-macro 0.4.2 | MIT | https://github.com/sarah-ek/equator/
find-msvc-tools 0.1.15 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs
itertools 0.13.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools
itertools 0.14.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools
itertools 0.15.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools
jobserver 0.1.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/jobserver-rs
libc 0.2.189 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc
log 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log
maybe-rayon 0.1.1 | MIT | https://github.com/shssoichiro/maybe-rayon
media-av1 0.1.2 | MIT OR Apache-2.0 | workspace
media-frame 0.1.2 | MIT OR Apache-2.0 | workspace
memchr 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr
nasm-rs 0.3.2 | MIT OR Apache-2.0 | https://github.com/medek/nasm-rs
new_debug_unreachable 1.0.6 | MIT | https://github.com/mbrubeck/rust-debug-unreachable
no_std_io2 0.9.4 | Apache-2.0 OR MIT | https://github.com/wcampbell0x2a/no-std-io2
nom 7.1.3 | MIT | https://github.com/Geal/nom
nom 8.0.0 | MIT | https://github.com/rust-bakery/nom
noop_proc_macro 0.3.0 | MIT | https://github.com/lu-zero/noop_proc_macro
num-bigint 0.4.8 | MIT OR Apache-2.0 | https://github.com/rust-num/num-bigint
num-derive 0.4.2 | MIT OR Apache-2.0 | https://github.com/rust-num/num-derive
num-integer 0.1.47 | MIT OR Apache-2.0 | https://github.com/rust-num/num-integer
num-rational 0.4.2 | MIT OR Apache-2.0 | https://github.com/rust-num/num-rational
num-traits 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits
paste 1.0.15 | MIT OR Apache-2.0 | https://github.com/dtolnay/paste
pastey 0.1.1 | MIT OR Apache-2.0 | https://github.com/as1100k/pastey
pastey 0.2.3 | MIT OR Apache-2.0 | https://github.com/as1100k/pastey
proc-macro2 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2
profiling 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling
profiling-procmacros 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling
quote 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote
rav1e 0.8.1 | BSD-2-Clause | https://github.com/xiph/rav1e/
rayon 1.12.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/rayon
rayon-core 1.13.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/rayon
shlex 1.3.0 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex
shlex 2.0.1 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex
simd_helpers 0.1.0 | MIT | https://github.com/lu-zero/simd_helpers
stable_deref_trait 1.2.1 | MIT OR Apache-2.0 | https://github.com/storyyeller/stable_deref_trait
syn 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn
syn 3.0.4 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn
thiserror 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror
thiserror 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror
thiserror-impl 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror
thiserror-impl 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror
unicode-ident 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident
v_frame 0.3.9 | BSD-2-Clause | https://github.com/rust-av/v_frame
y4m 0.8.0 | MIT | https://github.com/image-rs/y4m.git
```
\nAssembly is opt-in (--features asm); the default and --no-default-features\nbuilds are pure Rust. focused.mix tests both and oracle.mix builds and measures\nboth explicitly. dependencies.mix includes the asm closure for this inventory.\n\nThe extra pastey 0.1.1 comes from rav1e -> av-scenechange 0.14.1, whose\nmanifest requires ^0.1.0. Existing pastey 0.2.3 is semver-incompatible with\nthat requirement. Cargo cannot unify them without an upstream dependency\nchange or a fork; neither is justified for this scoped wrapper fix.\n