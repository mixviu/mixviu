# rav1d decode dependency

Source: https://github.com/memorysafety/rav1d
Revision: `2e7216c20ac2a75de204d6c8d7268b75a45959b4` (unreleased 1.1.1).
Licence: BSD-2-Clause. Source is consumed as a pinned Cargo git dependency,
without copied or modified upstream code.

Inspected source SHA-256:

| File | SHA-256 |
| --- | --- |
| build.rs | 845bf00d771807a1e38fefffd57f21e640e74a9d7a8e61ef8cc7a7ac526a60ea |
| Cargo.toml | 8a4fb87441577a64773549c0d59686c1c0e03dca37694d72591aee2da6688123 |
| COPYING | 6477f5c0674c89dc9251708a442343449bcb9c14b969b09286eb61d0640b284a |

The released 1.1.0 aborts during corrupt-frame error cleanup at decode.rs:4997
(frame_hdr unwrap). This revision checks the optional header at decode.rs:4901
instead. The wrapper's mutation regression exercises the failure path.

x86 assembly admission follows Mark's 2026-11-14 decision. The x86 build uses
nasm to assemble explicit `.asm` inputs, then cc only to archive objects. No
C sources are compiled. ARM builds select `.S` inputs; these are not the x86
assembly covered by this admission. Current validation targets x86-64 only.
Pure Rust fallback: disable media-av1's default features. Transitive additions
are pure Rust (`av-data`, its numerical traits, and strum proc macros); no
additional foreign-code admission or system-library dependency is introduced.
