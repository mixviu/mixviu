# Rio VT local patch

Source: `raphamorim/rio` at
`932c1a7d9e07b4db5924f7a0dd689e823c3a1442`, `rio-vt/` only, MIT.
The root licence is included. Per-file source hashes and Git blobs are in
`provenance/rio-vt.json`.

The manifest materialises upstream workspace inheritance; sibling Rio
dependencies retain the same Git revision, versions and feature flags.
Viu still enables only `pty`. No new dependency or foreign-code exemption
is introduced; simdutf remains Viu debt under Fable D6.

Functional correction under Fable C1: `ClearMode::Above` must clear the
first row when the cursor is on the second row. Rio's zero-based row guard
was `cursor.row > 1`; it is now `cursor.row > 0`.
Viu's real-parser ED0/1/2 conformance assertions are preserved.
The native Gold and unpatched port both retain the first row after ED1.

Second C1 correction: widening a wrapped line must calculate its cursor
delta before clamping to the live viewport. `grow_columns` uses unbounded
temporary positions computed directly from the new column count. Grid row
storage is temporarily taken during reflow, so the position helper cannot
supply bounds at that point. Clamping early loses
the column when the merge target is in history and leaves the rejoined
text hidden above a blank live screen. Native Gold and the unmodified port
reproduce this after a 98→48→98 column resize with identical real Mix output.
The regression retains exact text, selection and final cursor assertions.
All other imported Rust source is unchanged.

Exit path: upstream the correction, review a fixed Rio revision, remove
this local patch and repeat conformance and matched native captures.
