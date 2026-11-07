# Six-app integration: validation in progress

Latest repair source: `0124e5c58c889f1d93c730b92993e2e74b707464`.
Integration acceptance remains **OPEN**. The source-5bd release set passed
full CBC, all original native suites and two 510-sample performance runs,
but an additional native keyboard witness found the shared gold dialog
reclaiming Tab focus from Cancel. Its worker HANDOFF explicitly says
NOT_ACCEPTED. The reusable renderer now follows gold: request field focus
only when no widget owns keyboard focus. All seven text-input tests pass,
including title/path Tab transfer and Enter cancellation. CBC debug build
`gold-six-focus-debug-r8b-a1` passes. Its native editor witness preserves all
existing creation/edit/undo/redo/Save and window-effect assertions, and adds
explicit Tab focus assertions. The tabbed New macro capture is pixel-exact
and inspected. Full gate `gold-six-full-20261107-r8` is running on that
frozen source; release-byte acceptance is still pending.

The original source-5bd Ced run retained two clean-checkout preparation
refusals while documentation was being edited. The three cases they stopped
were then executed unchanged against the same frozen source and binary from
a clean checkout. All eleven Ced suites and their 35 pairs are accounted for;
the original refusals remain in the private archive. Screenshot's Open-dialog
caret-phase difference was closed with a paired left/right cursor reset:
the additional whole-window capture has SSIM/RMSE distortion 0.

The sections below retain the earlier integration checkpoints.

Tested runtime source: `2894a603a903d05f5a53f734dc5c591a11e69a91`,
branch `lab/gold-parity`. This integrates the accepted Monitor, Ced, Prefs,
Screenshot and Backdrop work, Workshop's shared editor, and the bulk
primitives from `4c1d715`. Overall acceptance is **OPEN**. Source `2894a603`
is **NOT_ACCEPTED**: final inspection found Workshop retaining client chrome
after authority loss where gold removes it. Its saved worker handoff says
NOT_ACCEPTED. Successor bundle repair `5bdc85c3a833e471fa5b5d7a5b666bb32ada8202`
sets the existing `retain_window_on_loss` policy to false for Workshop.
That paired repair capture is pixel-exact; detached full gate
`gold-six-full-20261107-r7` and successor release-byte repetition are pending.

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

The exact-release six-app suites and deferred-reply witnesses executed.
Monitor has 46/46 replies and ten exact pairs; Ced has eleven suites and
35 pairs (31 exact); Prefs has 69 functional checks, 42 pages and 91 pairs
(88 exact); Screenshot has 55 Bus comparisons, eleven suites and 47 image
comparisons; Backdrop has its four native groups and 44 pairs. Workshop
matches 99/99 protocol replies and 25/25 macro replies, with native UI,
editor, panels, persistence and live Prefs runs. These results do not close
the remaining visual mismatch or grant acceptance to the set.

The concurrent Workshop editor
witness passed New macro, Unicode typing, undo/redo and Save, then received
`BUSY/Full` from `workshop.info` after launching Ced. That failed run is
preserved. An isolated repetition on identical release bytes will distinguish
load-dependent admission from a reproducible editor launch failure. That
unchanged isolated witness passed, including both window effects and cleanup
of all thirteen owned PIDs. It does not erase the concurrent failure. The
successor recipe runs the strict editor witness before the other app groups,
without retrying or ignoring refused requests.

Expanded Traffic regions were inspected separately from the collapsed
layout. Bottom/right headers, static content, controls and status all have
SSIM/RMSE distortion 0. Populated traffic data has whole distortion
0.0184629/0.00517524 and data-region distortion 0.082597/0.0218041;
timestamps, observations and message sizes are truthful S2 data. Clearing
traffic produces whole-window and every-region distortion 0 in both docks.

The persistence witness initially compared selection reveal at different
initial viewport sizes. Setting both saved window sizes to 1040×680 before
restart gives equal restored offsets (809.5px) and a pixel-exact restored
capture; the raw 70px restart capture is also exact. The original strict
offset and passive-state assertions remain intact. Startup sizing and
in-progress wheel captures are retained as diagnostics, rather than being
presented as matched settled visual states. This fixture correction changes
no application behaviour and claims no scroll waiver.

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

## Same-worker 2,000-row comparison

CBC `gold-six-perf-2894a603a903-a1` passed on cbc2 against
`4c1d7153f6e13da0e1f67317af9c3b4518ff5e09`. The complete log was read.
All four engine hashes verify; 510 native samples pass row-count, sorting,
selection, refresh-witness and applied-model checks. No compiler was active.
Thirty Latin-rotated rounds use 2,000 actual files and alternating Email
order. ACK timing is separate from applied-model readback and does not
measure physical presentation.

| Operation | Median before → after, ms | p95 before → after, ms |
|---|---:|---:|
| view.sort-size | 112.847 → 114.924 | 124.971 → 121.356 |
| view.sort-name | 110.874 → 110.420 | 119.234 → 116.129 |
| view.refresh | 101.629 → 102.429 | 117.606 → 108.119 |
| selection.first | 63.192 → 64.241 | 74.526 → 67.123 |
| selection.last | 64.335 → 64.797 | 68.835 → 71.276 |
| email.list | 115.287 → 117.390 | 135.131 → 140.187 |

Bulk correctness and hot paths are retained. Residual timing shifts remain
visible, including Email's 1.8% median increase; these are not identical
measurements or a claim to have reached a new latency target. Full raw
distributions, model readbacks, original gold timings and cleanup receipts
remain in the private evidence.
