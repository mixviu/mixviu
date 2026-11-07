# Six-app integration: validation in progress

Tested runtime source: `2894a603a903d05f5a53f734dc5c591a11e69a91`,
branch `lab/gold-parity`. This integrates the accepted Monitor, Ced, Prefs,
Screenshot and Backdrop work, Workshop's shared editor, and the bulk
primitives from `4c1d715`. Overall acceptance is **OPEN**.

## Exact-source gate and artifacts

CBC `gold-six-full-20261107-r6-a1` passed on cbc2, rc 0. The complete saved
log was read: 1,120,645 bytes, 19,531 lines, SHA256
`18f376917419e7d02dcff85e61412e93aec2d07cfdc528363fae10c6566f8931`.
The gate covers formatting, both Clippy profiles, shell/workspace tests,
app policy suites using the source-built Mix interpreter, bulk tests,
P2 furniture, Track A (`SMOKE_FAILS=0`) and four release builds.

All four engines are together on cbc2 at
`/build/es-artifacts/2894a603a903d05f5a53f734dc5c591a11e69a91/`:

| Engine | SHA256 |
|---|---|
| mix | `35afa7c406a2c8d767ab68d5a41726ebb17c0ef36466f787955c36cd294211d2` |
| mixd | `b1638d8d7459c195cebd72a2ca1d1aa3642040dd689838f707a538344c7df97d` |
| viud | `5f50a834e2abd1020787160d8ea0ac9fb6deca8464faa417a4249b8dc9d0cf80` |
| viu | `b15c1272d91a3ef2b9f4e43562854750fd4583558c1ea4057b9d2874ac1944f0` |

SHA256SUMS SHA256:
`368f59d81cd6ddc0e4b7e7931cacfb8898e45649018bc6ba78404f08d1a2e7ea`.
The fetched release viu was hash-verified before native tests. These saved
artifacts are not yet an accepted desk handoff; no installation is authorised.

## Open acceptance work

The exact-release six-app suites, deferred-reply witnesses, paired captures
and same-worker comparison are running. The concurrent Workshop editor
witness passed New macro, Unicode typing, undo/redo and Save, then received
`BUSY/Full` from `workshop.info` after launching Ced. That failed run is
preserved. An isolated repetition on identical release bytes will distinguish
load-dependent admission from a reproducible editor launch failure; it does
not erase the failure or grant acceptance automatically.

The preceding source `73bda564` passed CBC but failed visual acceptance:
background appearance retention leaked into ordinary windows after authority
loss, and the generic gold dialog path ignored shortcut grids. Its worker
handoff explicitly says NOT_ACCEPTED. The successor restores ordinary-window
file-palette fallback, keeps Backdrop retention opt-in, and uses the shared
grid renderer for grid-bearing dialogs. Three focused debug capture pairs
are pixel-exact; final release captures must establish acceptance separately.

The native Ced fixture now uses a stable broker-attested caller for its own
edits and undo, with a second-caller refusal assertion. Gold demonstrates
this identity contract. Workshop's persistence fixture seeds deterministic
membership before scrolling, retaining the strict 70px restoration assertion
for both implementations. No scroll waiver or weakened assertion is claimed.

The [22 inherited import-hash mismatches](import-hash-debt.md) remain
provenance debt. That record names each file, both hashes, first divergence
and cause. Behavioural acceptance does not repair import provenance;
`verify_runtime` remains failing until reconciliation before production
derivation.

Workshop's installed native Dopus interoperability follows gold. Moving
Dopus itself to the generic host remains a separate open port.
