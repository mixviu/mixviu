Current integrated acceptance: **ACCEPTED** at runtime `e1bbea99071c2fa8793a7e0e52aed381707d536f` on `lab/gold-parity`. All eight apps passed exact-release revalidation; see [eight-app integration acceptance](eight-app-integration-acceptance.md) for the complete gate, native/Bus/live-Prefs/deadline results, all 405 raw comparisons and matched supplements, performance and the four-engine hashes. Historical records below retain their original sources and results.

# Dopus gold parity

## Selection feedback and explicit pane actions (2026-11-13)

The tutorial reported an unchanged idle line and pane totals after selecting
a file. This matches the pinned MixOS oracle
`8779f33284b7d3f1178c3d57a699efb9b0867b22`:

- `apps/dopus/src/view.rs:367` draws `app.info()`;
  `apps/dopus/src/app.rs:199` returns the transient status override or
  `core.info()`. `crates/dopus-core/src/model.rs:38,425,528` initialises and
  returns `Select a file or folder`.
- `model.rs:912-930,935-986` changes selection and emits notifications without
  assigning info text. Metadata is displayed separately in Properties.
- `view.rs:734-736` draws `visible_summary` or `footer_summary`.
  `model.rs:157-164,350-360` counts the listing, without filtering by selection.

Selection feedback is therefore unchanged under the requested parity rule.
No selection count/name/size was added to the info line or pane footer.
`nav.pane-left` and `nav.pane-right` extend the Bus action catalogue with
idempotent active-pane selection; existing toggle and targeted-open semantics
are preserved. Bundle tests exercise discovery, repeated execution, selection
and dialog actions; Mix tests verify headless operation and mkdir targeting.

Overall verdict: **ACCEPTED** at the tested source below.

## Acceptance: 2026-11-07 round five

Verdict: **ACCEPTED**, following [parity-method.md](parity-method.md).
Tested implementation: `4bccf5913312900b504054fc3c2d277078071c4a` on
`lab/parity-dopus`. Subsequent commits contain receipts only. Gold is MixOS
`8779f33284b7d3f1178c3d57a699efb9b0867b22`, binary SHA256
`590cfad14856518ba668154e987d0282f85963f5c708bfd53c202e5d48a9e359`.
The exact CBC release Viu executable has SHA256
`9e34bf227d469fc858be297e7a6a6af4f0f31c169f15a978cdeed9e7ae5c1689`.
Its source/profile/hash receipt and executable are saved under
`/build/es-artifacts/4bccf5913312900b504054fc3c2d277078071c4a/`.
Run that executable with the tested bundle through `viu run apps/dopus`.
No per-app Rust host remains; `viu-dopus` and its comparator are retired.
Nothing was installed or merged by this acceptance round.

### Gate diagnosis and repeat

The round-four command log ending after the empty DOPUS_PATCH block was
complete, not a failed build. Its own cbc2 worker receipt
`/build/.cbc-logs/dopus-r4-accept-framing-a1.result` records stage done,
rc/exit_code 0, exact head/commit, no timeout, no signal and untruncated
output. The native build service independently records rc 0. CBC's saved
file contains command output; its queue-runner path deliberately suppresses
the outer `BUILD_RC` print. No application or gate repair was warranted.
Both worker receipts are retained privately. The private follow-on wrapper
now supplies explicit long deadlines rather than a short default deadline.

The requested detached repeat, **`dopus-r5-accept-a1` on cbc2**, passed
`/opt/mixos/bin/mix apps/dopus/gate.mix` at the exact source above:
72 scene tests, 22 focused tests, 66 library tests, one release bulk profile,
format check, all-target checks, Clippy and release build. There was no
worker diff. The complete 2,553-line saved log was read:
`20261107-175255-cbc2-4bccf5913312-dopus-r5-accept-a1-16988546.log`.
Existing Clippy warnings remain warnings. The worker result again proves
rc 0 with no timeout or signal. Round-four `dopus-r4-accept-framing-a1`
also passed; neither job needed an rc 75/143 requeue.

### Features, Bus and live Prefs

The fetched release hash was verified before repeating all twelve isolated
native groups: full visual/Bus/Prefs, features, input, preview, chooser,
hosting, authority recovery, launch, drag, service edges, window close and
release performance/refusal. Each group passed and every owned PID exited.
The private archive is `parity-dopus/accept-4bccf5913312-final`;
`final-summary.json`, `audit-index.json`, result files, fixture/script hashes,
native requests/replies, complete logs, original captures, figures and
cleanup receipts identify the exact evidence. Earlier round-four results
are preserved with the `r4-` prefix. Native service binary hashes are retained.

| Gold inventory | Release-native closure |
| --- | --- |
| Shared frame, panes, sidebars, status | Gold frame, split/width restore, active outline, location fields and pane summaries; matched baseline and narrow captures |
| Tree and navigation | Expansion/collapse, native location submission, keyboard and range selection, context menus, directory-first sort, Places/bookmarks and pane targets |
| Metadata and previews | Gold formatting and wrapping, raw names/symlinks, native image preview and scroll, asynchronous completion fences |
| Files and dialogs | Native open, mkdir/rename/delete, copy/move bytes, drag copy/move/cancel, pinned modal Enter/Escape and Tab focus |
| Mail and launch | Email success/BUSY attachment fallback/DENIED, native launch receipts, singleton path activation, headless and offline mapped windows |
| Furniture and persistence | Search, shortcuts/scrollbar, About, macro CRUD/run/output/stop, config validation/atomic persistence and reply-fenced close/quit |
| Chooser | Gold frame, Places/tree, save stem, filters/choices/extensions, replacement, save-files ordering, independent windows, stale/ambiguous refusal and idempotent cancel; no file writes |

Coverage is **44/44 gold command descriptors and 33/33 core actions**,
including real invocation, refusals, all twelve macro commands and native
executed/titlebar/macro-completion topics. The audit additionally records
one diagnostic command and seven UI actions; these do not replace the gold
inventory. Ninety paired request records match in rc and stable reply fields;
differences are truthful PID/build stamps and native UI tree hierarchy,
whose native labels/input/geometry are exercised separately. Headless
typed operations and window overrides retain the pinned gold contract.

All **17 main Prefs states** pass on one unchanged PID, and all **17 chooser
states** pass on one continuing request: modes, schemes/styles, accent,
density/roundness, font families/weight/size, mono font, scale, icon stroke,
app overrides, daylight, tunables, decorations, caption/menu/action/title
alignment. Live keymap changes, invalid-setting refusal, authority loss and
restart/recovery pass. Gold's chooser ignores live Prefs; the port follows
them under Mark's standing mandate.

### Visual inspection

Both clients use owned native Bus/compositor/settingsd instances, identical
filesystem fixtures, scale 1 and matched effective settings. Main clients
are 1040×680; chooser clients are 820×600 and 600×300 inside 1100×800
compositor captures. PNGs are never rescaled. ImageMagick 7.1.2-35 reports
normalised **distortion**, zero identical. Originals, side-by-side/difference
figures and region crops were inspected. No arbitrary threshold is used.

Baseline gold and candidate PNG SHA256 are identical:
`b7915c3afe67b25af3cd8c65462eaae075706e9b971c2db2cf85a89ada9f820e`.

| Baseline region | x,y,w,h | SSIM / RMSE distortion |
| --- | --- | --- |
| Full | 0,0,1040,680 | 0 / 0 |
| Header | 0,0,1040,38 | 0 / 0 |
| Panes | 147,38,677,612 | 0 / 0 |
| Places | 0,38,147,612 | 0 / 0 |
| Properties | 824,38,216,612 | 0 / 0 |
| Status | 0,650,1040,30 | 0 / 0 |

The main audit retains 61 raw comparison sets. Two input/preview setup
baselines contain fixtures created between clients and are explicitly
excluded; their actual expanded/narrow/preview state pairs match. Four
untuned chooser pairs are retained but superseded by matched effective
legacy radii in the chooser add-on below. Other inspected differences:

- Compact-square and fonts-scale: whole SSIM 0.000177575/0.000170964,
  header 0.00317766/0.00305935; content regions zero. Native bounds differ
  by less than 0.1 logical pixel, leaving fractional rasterisation only.
- About: whole SSIM 0.00094159, RMSE 0.0102443; truthful source stamp only.
- Hosted mode result: whole SSIM 0.339583, RMSE 0.752958. Gold publishes
  `view.mode` but refuses its execution; the port fixes that broken function
  under standing Fable C1. Both replies/captures are retained. Ordinary
  hosted client/whole, native pointer mode-toggle and macro-popup pairs match.
- Menus, search, shortcuts, light-mode icon, selection, file dialogs and the
  remaining matched live Prefs geometry have zero distortion.

Final chooser receipt:
`matched-chooser-4bccf5913312-r7/round2.gRt1V3`. Both clients receive the
same effective legacy radii described in the historical checkpoint. Six
comparison sets retain focused and passive captures; no images are masked.
All header/navigation/pane/form crops have **SSIM 0 and RMSE 0**. The only
non-zero pixels are truthful caller connection text at the same position:

| Chooser state | Whole SSIM | Whole RMSE |
| --- | ---: | ---: |
| Save | 0.000274071 | 0.00721406 |
| Replacement | 0.000251181 | 0.00876238 |
| Options, focused and passive | 0.000262402 | 0.00894236 |
| Short, focused and passive | 0.000236446 | 0.00867926 |

Earlier caret-phase and capture-harness failures remain retained. The final
focused and passive pairs close that ambiguity. No unresolved visual
deviation or deferred capability remains; no new Fable ruling is needed.

### Release performance and atomic scan refusal

Bulk primitives and row-cache paths are retained. These are 21 measured
samples after three warmups, actual 2,000-file directories; milliseconds:

| Action / stage | Median | p95 | p99 |
| --- | ---: | ---: | ---: |
| Sort callback | 67.138 | 68.418 | 70.425 |
| Refresh callback | 67.651 | 68.971 | 69.841 |
| Sort native Bus acknowledgement | 111.892 | 129.111 | 129.673 |
| Refresh native Bus acknowledgement | 113.293 | 129.114 | 130.187 |
| Sort acknowledgement + applied-model readback | 128.478 | 146.233 | 147.279 |
| Refresh acknowledgement + applied-model readback | 129.898 | 146.307 | 147.273 |

Fresh shared-frame adapter medians are 15.164/15.037 ms and cold draw
10.596/10.587 ms (sort/refresh). Callback work meets the standing ~100 ms
bulk target. Native Bus/readback measurements include transport and host
application; they do not measure physical presentation. Release probes ran
alone, before image comparisons.

The pane scan explicitly budgets **65,536 total enumerated names and depth
64**, including hidden/filtered names, child counts and expanded subtrees;
links are not followed. A real 65,537-name two-pane open returns rc 10 and
`SCAN_BUDGET`, preserves the complete prior state of both panes and keeps
model serial **56 → 56**. The chooser repeats the real 65,537-name refusal,
preserving folder, complete rows and serial **0 → 0**, with its request
still open. Enumeration/depth regression tests and the two-pane copied
candidate test pass. There is no successful truncated result or partial
state. This closes the 2026-11-06 scan-budget defect.

## Historical migration records

The checkpoints below describe earlier states. Their OPEN/pending statements
are superseded by the exact release acceptance above.

## Current acceptance checkpoint: 2026-11-07 round four

Implementation: `4bccf5913312900b504054fc3c2d277078071c4a`, branch
`lab/parity-dopus`. Later checkpoint commits are receipts only. The native
chooser frame is now complete: Mix composes the shared frame, Places rail,
tree table, save controls, filter/choice fields and replacement dialog. The
generic host initialises native window framing and honours the chooser's
explicit client-side frame. No per-app Rust or `viu-dopus` remains.

CBC `20261107-165508-fcc5458e` passed twenty focused tests and the debug host
build with no worker diff; its complete saved log was read. Debug binary
SHA256: `d3896dd7369d42082347b36ff63774745b5b079624bd63d94d0c8d493d87741f`.
This is diagnostic evidence, not the required final release receipt.
Pinned gold Dopus SHA256:
`590cfad14856518ba668154e987d0282f85963f5c708bfd53c202e5d48a9e359`.

The private `matched-chooser-4bccf5913312-d/round2.oDweAV` receipt retains
gold and candidate whole-compositor PNGs, originals, side-by-side/difference
figures, region maps, native replies and cleanup. Both run on the same owned
native broker/compositor/settingsd, at 820×600 client pixels (short case:
600×300), scale 1. They wait for mapping and complete the same native
resize/repaint sequence; no capture is rescaled. Gold's chooser loads the
file theme and ignores live Prefs. The port follows live Prefs as Mark
requires. Paired captures therefore set identical effective legacy radii:
small 6, large 12, field 4 and menu highlight 6, preserving the port's live
follower. The untuned pairs remain retained separately.

ImageMagick 7.1.2-35 reports normalised **distortion**, zero identical.
Header, navigation, pane and form regions all have **SSIM 0 and RMSE 0**
for save, replacement, options and short-window pairs. The inspected
non-zero pixels are truthful caller connection text, in its gold format
and position:

| Chooser capture | Whole SSIM distortion | Whole RMSE distortion |
| --- | ---: | ---: |
| Save | 0.000254806 | 0.00698644 |
| Replacement | 0.000210692 | 0.00799713 |
| Options | 0.000259444 | 0.00889131 |
| Short window | 0.000224497 | 0.0086332 |

The earlier outer-corner residue was a real missing native frame facility,
not merely debug raster marks. The shared initialisation repair above closes
it; the complete region figures establish that result.

Native save-stem/Enter, replacement clicks, filtered extensions, caller
choices, independent windows, stale/ambiguous addressing and Escape with
idempotent cancellation pass. Seventeen chooser live Prefs states pass on
one continuing request. A real 65,537-name navigation probe refuses with
`SCAN_BUDGET`/rc 10, preserves the prior folder, complete rows and serial,
and leaves the request open. Every owned process exited.

Prior-source release `955bd3f2b76bedd3c551317844bc15e57b596ff3` passed
heavy CBC `dopus-r4-native-final-a1` and all twelve native groups. Its
61-figure audit exercises all **44 gold descriptors and 33 core actions**;
the extra diagnostic command and UI actions do not replace those inventories.
The baseline and search palette match exactly. Main Prefs, authority recovery,
hosted chrome, native input, headless mutations, operations, macros and
scan refusal pass there. Those receipts do not attest the final source.

The final source and complete recipe are frozen as
`accept-4bccf5913312-final`. Heavy CBC `dopus-r4-accept-framing` is detached
and queued. Its owned follow-on verifies the exact release executable's
source/profile/hash, runs all twelve native groups, repeats the matched
chooser and chooser budget probe, compares structured replies and generates
the region audit. Final complete-log/image inspection, release 2,000-row
numbers and the acceptance verdict remain **OPEN** until that run finishes.
No Fable decision or installation is requested.

The inventories and older checkpoints below retain the migration history;
their pending statements are superseded only by the explicit evidence above.

## Gold inventory

Gold source owns the contract, including the window's overrides of the
headless command dispatcher. In particular `App::serve` handles `dopus.dialog`
before `commands::serve_command`; the latter's UNAVAILABLE reply is not the
window contract.

| Area | Required gold behaviour | Current migration evidence |
| --- | --- | --- |
| Frame | Folder icon, active path followed by ` — MixOS DOpus`, File/View/Go/Help menus, sidebar actions, toolkit tail, status | Shared gold frame mounted; native visual comparison pending |
| Panes | Two locations, active outline, resizable split, summaries, name/size/modified, directory-first sorting | Existing Mix reducer and bulk projections retained |
| Tree | Expansion, collapse, double-click, keyboard navigation, selection, Ctrl/Shift ranges, context menus | Existing Mix/native widgets retained; complete gold witness pending |
| Navigation | Back/forward/parent/home, location edit/submit/cancel, Places and bookmarks, pane target | Existing Mix policy; targeted open now preserves active pane |
| Sidebars | Resizable Places/Properties; lend space below 640px and restore choices | Existing scene policy retained; gold geometry pending |
| Metadata | Names, kind, counts/size, three dates, permissions, owner/group, symlinks, raw path display | Existing Mix projection retained |
| Previews | Local image preview, cache, refusal, toggle, stale completion fence, remote policy | Generic `viu.job.completed` now enters existing reducer |
| Files | Open, mail selection and busy-compose fallback, mkdir, rename, copy/move, delete, drag/drop choice | Existing owned Mix jobs retained; native acceptance pending |
| Dialogs | Pinned prompts, queued confirmations, native Enter/Escape, old Bus dialog envelope | Existing Mix forms retained |
| App furniture | Search, shortcuts, About, macros, native/hosted menu and action placement | Generic macros and frame wiring; full gold restoration pending |
| Persistence | Config migration, poison-pill refusal, atomic write, panel/split/location state, orderly quit | Existing Mix persistence retained; exit fences pending |
| Appearance | All gold themes/styles, window overrides, live settings/keymap, authority recovery | Generic session/theme/keymap events wired; live matrix pending |
| Launch | Canonical identity, service/comp/URL overrides, path arguments and single instance, chooser service | Generic overrides available; legacy profile argument retained; gold path/chooser migration pending |

## Bus inventory

Gold app commands: `dopus.ping`, `dopus.describe`, `dopus.info`,
`dopus.state`, `dopus.dialog`, `dopus.execute`, `dopus.actions`,
`dopus.theme.set`, `dopus.open`, `dopus.show`, `dopus.quit`, plus `HELP`
and `app.describe`. Add the full toolkit drive/window/capture/menu surface,
debug/debug.set/debug.recent, all twelve macro commands and the executed,
titlebar and macro completion topics. The chooser service has its own
contract and lifecycle and remains part of the migration.

Ping/state/info/actions/open/theme/dialog/quit handlers now exist in Mix.
Gold open accepts an empty paths list, reports the number supplied, applies
at most two paths without a target, and requires exactly one with a pane
target. Describe/HELP/activation, complete action replies, topic delivery,
launch and shutdown still need matched native acceptance. No catalogue-only
claim counts as Bus parity.

## Scan work budget

The integration baseline already introduced a default budget of 65,536
enumerated names and depth 64. Every root enumeration, visible directory
child-count enumeration and expanded subtree spends the same remaining work
allowance. Hidden entries also spend work. Symlinks are not followed. The
depth bound is inclusive; expansion beyond it refuses. Caller budgets are
validated (rows 1..65,536; depth 0..64). Exhaustion raises `SCAN_BUDGET`, never
a truncated successful listing. `dp_request` reduces a copied candidate, so
failure in the second pane discards the entire navigation request.

CBC `20261107-112139-11c64519-a1` at `c10e6f725a39` passed both scan tests
and the 2,000-row profile (three tests total). The complete saved log was
read. Debug callback medians were 328.02 ms sort and 327.90 ms refresh;
these are debug measurements, not the release performance gate. Bulk
`stat_many`/`rows_project` and row-cache paths remain in place. A new full
bundle test checks frame mounting and refusal across both panes; its gate
is pending. Release remeasurement and native applied-model witnesses remain
required before acceptance.

## Frame checkpoint

Code `c10e6f725a39` adopts the shared frame. Follow-up `8fd7120` adds
generic event/Bus wiring and the bundle refusal test. No per-app Rust was
added to the runtime; retained comparator sources are not the generic app's
execution path. Private diagnostic captures, scripts and cleanup receipts
live outside this tree. Exact-source release hashes, paired visual figures,
SSIM/region inspection and live Prefs acceptance are pending. No Fable
deviation is proposed at this checkpoint.

## Diagnostic visual pair (not acceptance)

An isolated native Bus/compositor run captured clean pinned gold and the
generic-host specimen sequentially at 1040×680. Gold and candidate use the
same real filesystem fixture. The candidate used the saved integration
executable; later shared-widget changes have not been captured. This run
did not include the required private settingsd matrix, so it cannot attest
Prefs or exact-source release parity. All owned processes were confirmed
stopped. Earlier failed diagnostic runs remain retained.

The whole original, candidate, side-by-side and difference images were
inspected, along with header/Places/status crops. ImageMagick 7.1.2-35 reports
normalised **distortion**, zero identical:

| Region | x,y,w,h | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| Full | 0,0,1040,680 | 0.0466528 | 0.0870047 |
| Header | 0,0,1040,38 | 0.00241008 | 0.017971 |
| Places | 0,38,147,612 | 0.0278773 | 0.0657722 |
| Panes | 147,38,677,612 | 0.053698 | 0.0830627 |
| Properties | 824,38,216,612 | 0.0529945 | 0.119515 |
| Status | 0,650,1040,30 | 0.0315301 | 0.0935127 |

Remaining differences: macro icon; Places header/row layout; sidebar widths
after resize; pane outline colour, location/header/footer geometry; metadata
wrapping; unwanted status icon. Reusable navigation-list and text-only status
facilities address two of these; both await the new artifact and comparison.
No difference is accepted or referred to Fable as a proposed deviation.

Gold PNG SHA256: `d7a116de48c3fb15bc95ec9b50bcae64cd0276fd5633993a7ed6ef0162614b14`.
Diagnostic candidate: `f8975ae8055625ccc85a7969b0f4f6125394ab64552af4cacb41f0bfc65afa14`.
The private `parity-dopus/smoke.x7IJs2` archive retains figures, metrics,
queries, logs and the all-stopped receipt.

## Pending gate

Source `d1b655a` includes native navigation-list, optional status icon,
generic reply-fenced quit, Mix-owned describe replies, deduplicated explicit
Bus descriptors and the pinned gold command/action inventories. CBC
`20261107-114033-a67ba1a0` is queued for formatting preparation, focused
tests, all-target checks/Clippy, release 2,000-row measurements and release
host artifact. It must return its formatting patch, followed by a clean
exact-source gate. Superseded queued jobs `20261107-112707-102d6304`,
`20261107-112921-92b8ec74`, `20261107-113446-ec29f120` and
`20261107-113825-38f97644` were explicitly cancelled before execution.

## Round two repair checkpoint

The per-app Cargo package, host and Rust comparator have been retired. Dopus
now runs through `viu run`; the chooser presenter is a reusable Viu facility,
with chooser policy retained in Mix. Retained chooser, raw-name, filesystem,
config, form and opener checks run with the shared library. Bulk scanning and
projection remain in place, including the atomic two-pane budget refusal.

The generic host now evaluates optional bounded Mix launch policy before
activation or window creation. Dopus owns path parsing, native config-root
selection, print-config and activation arguments in `launch.mix`; Viu supplies
native activation and a headless loop. App persistence uses the existing
native one-shot timer and atomic replacement facilities.

The gold native menu probe corrected the inventory: Theme is one flat menu,
with `view.theme.*`, `view.mode.*`, `view.style.*`, decoration and caption IDs;
its separators, labels and order matter. The Bus action table remains the
original 33 entries. `view.search` and the live Ctrl+Slash shortcut are UI
commands. Same-column sort toggles direction in the gold core. Targeted
navigation preserves keyboard focus. Mail uses an owned native reply receipt
until the Email result arrives, including BUSY attachment fallback.

CBC `20261107-124448-c8da34aa` passed the 65 library tests, 11 focused tests,
checks, Clippy, release bulk profile and binary build at `0a6b42d69bf6`, with
no worker diff. This light-worker run is diagnostic; final release acceptance
and the artifact handoff must use cbc2/cbc3. Subsequent debug repair jobs
`20261107-124952-1908ef48`, `20261107-130315-f559f07f` and
`20261107-131012-f398b6e1` passed the 11 focused checks and built the generic
host. The latter two returned formatting patches, which have been applied.

Private paired diagnostics now show pixel-identical header, Places,
Properties and status regions. Remaining pixels were traced to the pane
outline being painted after its contents; gold paints it first. The repair
awaits the next paired capture. The gold witness separately completed 17
live settings variants on one unchanged process. Full candidate behaviour,
Prefs, release-byte revalidation and acceptance remain **OPEN**.

## Round three validation in progress

The baseline now matches gold pixel for pixel in all six regions. Native menu
clicks carry the original command payload, the shortcut grid uses gold order
and omits the separate search action, and the Look icon follows effective
mode. The broad font/scale geometry drift was a navigation rail inheriting
scene spacing; using native spacing removes the fractional panel growth.
The latest broad debug pair has exact File/View/Go/Help menus, shortcuts,
selection, delete, light/accent, Pro, SSD, caption placement, tunables and
alignment captures. About differs only in the truthful source stamp.
Compact mode still exposed a native-menu renderer ignoring its effective
form; that repair is undergoing a fresh capture. Debug-only unaligned
geometry markers and cursor blink must be rechecked on release bytes.

Private native witnesses retain successful copy/move bytes, committed
mkdir/rename/delete, mail success/BUSY fallback/DENIED payloads, macro
creation/text/run/output/termination/move/removal and window operations.
Executed-topic action order and envelopes match, as do macro completion
payloads apart from actual timestamps. The chooser lifecycle passed ten
cases across gold and the generic presenter: save-stem selection, replacement,
save-files ordering/refusal, independent windows/stale addressing and Escape
with idempotent cancellation. Launch witnesses passed duplicate headless
refusal and an actual mapped offline window for both applications.

CBC `dopus-r2-final-ad30f4a-a1` passed on cbc3 with no worker diff:
65 library plus 11 focused tests, check, Clippy, release profile and executable.
Its 21-sample release callback medians were 64.208 ms sort and 64.861 ms
refresh; p95 65.321/65.949 ms and p99 70.358/66.451 ms. Fresh shared-frame
adapter medians were 15.194/15.196 ms; cold draw 10.487/10.672 ms. These are
prior-source results, not final acceptance. Current exact-source release
validation, applied-model performance/refusal, full visual repeat and authority
recovery remain open. No Fable deviation is proposed.

## Round three native audit checkpoint

Implementation `13e0977d3ce7290d85630f16fa099d1b083515aa` closes native
location submission, modal Tab focus, absolute-path single-instance activation,
typed headless mkdir/rename/delete, asynchronous Email launch receipts and the
window-close reply. Diagnostic mkdir, rename, preview, drag confirmation and
hosted client/whole-surface captures are pixel-identical. The full windowed
matrix passes 17 live Prefs variants on unchanged PIDs. Final release evidence
is still required; debug captures do not grant acceptance.

The native compositor audit exposed a gold C1 defect: its published
`view.mode`, `help.macros`, `view.search`, `help.shortcuts` and theme-menu IDs
are refused by `dopus.execute`. Private paired request/reply and capture
receipts prove the refusal. The port routes these window UI commands through
its existing Mix registry while keeping the core action catalogue at 33 and
the headless core contract unchanged. The mode icon now uses gold's command
and tooltip; live UI choices also reach the native theme owner. This restores
function under the standing C1 ruling and needs no Fable question.

The exact-source release gate is detached as `dopus-r3-accept-chrome`.
Prior cbc3 gates `dopus-r3-final-scene-a1` and
`dopus-r3-accept-b1315fd-a1`, plus cbc2 `dopus-r3-accept-final-a1`, passed at
their earlier sources. Superseded modal/headless/hosted queue interests were
cancelled before execution. The latest scoped modal keyboard regression
passed; CBC formatting has been applied. The final source bundle and native
recipes are frozen privately with a file-hash manifest.


### Round four chooser checkpoint — OPEN

The chooser now uses the shared frame and Mix-authored native navigation,
tree, caller identity, save/filter/choice controls and replacement dialog.
Generic viewport settings follow live Prefs; the 17-state native witness
passes without a restart. A shared dialog geometry repair restores native
automation clicks. Enumeration and child-count work are charged to the same
65,536-entry / depth-64 admission budget, with candidate state committed only
after a successful scan. Short windows prioritise browsing as gold does.

Ten retained lifecycle cases plus filtered extension and options pass. Paired
full-window diagnostic captures have save/options SSIM distortions
0.00115605 / 0.00102301, primarily truthful caller identity and debug raster
markers; the replacement pair also needs settled-animation reinspection.
These are diagnostic figures, not visual acceptance. The frozen exact-source
release suite and its region inspection remain required. See the current
`last.md` checkpoint for source and CBC job IDs.
