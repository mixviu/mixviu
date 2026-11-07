# Monitor round 5: ACCEPTED

Exact tested code head: `5ab4207ab7a0c74aaf4b40592370a69b875e5eed` on
`lab/gold-parity`. This following commit records acceptance only. Full record:
[docs/ports/monitor.md](docs/ports/monitor.md#acceptance-2026-11-07), using
[parity-method.md](docs/ports/parity-method.md). No open items or Fable question.

Generic `viu run` now consumes gold's `--service`, `--comp` and `--noded-url`
flags while retaining canonical command/Prefs/macro identity. The native
window title is static `Monitor`. The shared frame and first-reading states
match gold; app composition and behaviour remain Mix.

The **release binary** passed the final isolated native evidence run:

- Processes and all seven Performance pages, including populated TCP:
  1040×680 whole/header/options/content/status SSIM=0 and RMSE=0.
  Resource rail/dashboard, command-line column and CPU chart/grid crops: 0.
- Eleven selected/tree/tab/menu/dialog states and compositor-hosted furniture
  (1120×788): exact. All 45 action records match gold in every state.
- Bus structured replies: **46/46**. Complete 51-command catalogue, native UI,
  macros/topics, activation, hidden restore, launch overrides and close pass.
- Nineteen live Prefs states on one unchanged PID per app: 17 exact; compact
  and fonts/scale have only node-chevron/Close antialiasing. Whole SSIM
  distortion 9.74852e-8 / 7.93036e-8; RMSE 8.50963e-5 / 8.14068e-5.
  Chrome SSIM 5.52416e-7 / 4.49387e-7; content/status 0. Authority recovery passes.
- P1 is the sole approved non-visual deviation. Stale-token refusal/refresh
  and TERM/KILL were verified using disposable children only; real monitord passes.

Private archive: `gold-parity-20261107/round5/5ab4207ab7a0c74aaf4b40592370a69b875e5eed`.
Its index contains 12 passing runs, 55 comparison sets, inspected figures,
source/input/binary hashes, executed scripts, native replies and all-stopped
cleanup receipts. Processes PNG pair SHA256:
`800c28506c24d9bc9c755c740d4cbca72486ce67aee48597e548461d12da19fc`.

CBC `20261107-053306-0497e9c3-a1`: **BUILD_RC=0**, exact source, no worker diff.
Four Mix tests; Viu/viu-engine fmt/check/tests/clippy; debug and release builds.
226 Rust tests passed; six existing environment-dependent ignores. Complete
2,614-line saved log read and archived. Earlier green scoped gate:
`20261107-051408-7e22145e-a1` at f550c0b. No install or VT work.

Desk4 handoff: release `viu` and `viu.json` saved on the successful CBC worker
under `/build/es-artifacts/5ab4207ab7a0c74aaf4b40592370a69b875e5eed/`.
The captured copy and saved executable have the same SHA256:
`3ba7412344bcd3c110164d762c0812359ba6fc2908f2943558383883a7bbb046`.

Branch lab/gold-parity; code head SHA 5ab4207ab7a0c74aaf4b40592370a69b875e5eed (receipt HEAD is the commit containing this file); Monitor ACCEPTED, whole/regions SSIM=0 RMSE=0, Prefs raster-only max whole SSIM=9.74852e-8; artifact /build/es-artifacts/5ab4207ab7a0c74aaf4b40592370a69b875e5eed/viu SHA256 3ba7412344bcd3c110164d762c0812359ba6fc2908f2943558383883a7bbb046; CBC 20261107-053306-0497e9c3-a1, 20261107-051408-7e22145e-a1.


Integration checkpoint: merging origin/lab/mixviu-poc 4c1d715. Bulk and Monitor code retained; merged-head gate and release capture revalidation pending. No replacement desk artifact is accepted yet.

Branch lab/gold-parity; integration merge head is the commit containing this checkpoint; gates pending; artifacts pending; CBC pending.

## Ced round 4 — accepted

Acceptance record: [docs/ports/ced-acceptance.md](docs/ports/ced-acceptance.md).
Exact tested code is `5ad8ad4aac6cba0363c28dc90103493842a42c2e`; this final
checkpoint adds documentation only. The private acceptance receipt records
the resulting receipt commit separately. The shared frame was merged through
`5ab4207ab7a0c74aaf4b40592370a69b875e5eed` before the final Ced gate.

The remaining dialog, recovery and conflict interactions are closed. Bus
Open/Save As/Goto and scratch Save hold replies until native acceptance or
cancellation, with disk completion before Save success. Native dirty Close,
Don't Save, Save/Save As, overwrite confirmation and cancellations match
gold; agent refusals do not create a modal. Recovered Open/Discard/Close,
native Re-insert/Dismiss and Keep Mine/Take the Service's pass against real
editd, with the four conflict/disconnected-copy capture pairs exact. Recovery
notices and the empty window now use gold's labels, controls and layout.
The shadow-plugin exit-owner repair is **gold-behaviour restoration**: pinned
gold Quit exits cleanly, and the port now releases its retained window and
exits within the same native witness. It is not a proposed deviation.

Fable C1 resolves the earlier question: show both Replace rows, preserving
gold styling and insets. The private `gold-replace-clipping-bug.md`, matched
gold/port PNGs and full/regional figures retain gold's clipped controls and
the approved repair. No Fable question remains.

Final native runs cover 61/61 successful actions across text and mail, the
53-command gold Bus surface, native drive/debug, all macro operations,
clipboard/IME, mail/attachments/uncertain Send, sessions, recovery, 16/64 MiB
and gold refusals. Ten suites plus selected-Goto and native recovery-button rechecks produce twelve
source/hash-checked receipts; all owned processes and brokers are stopped.
Live Prefs changes stay on one candidate PID through nine paired variants,
including independent editor typography, scale/density, framing, app
overrides, auto-mode and settings authority loss/recovery.

Visual acceptance: 35 matched 1000×720 pairs inspected, **31 pixel-identical**.
Problems/Output differ only in one antialiased separator row; the enlarged
compact look has small raster-edge differences in Search and the status
rule. C1's Replace repair is the sole approved design difference. Full and
regional SSIM/RMSE distortions, capture hashes, native layout/titlebar forms,
fixtures, cleanup and the source/binary provenance are indexed in the
acceptance record and private `accepted-5ad8ad4` collection. The final Goto
capture selects the same `2:2` text in both apps to match caret state.

CBC `20261107-063528-808d3be7`: fmt check, normal Clippy, 29 scoped tests and
host build passed at the exact tested source, with no worker diff; complete
saved log read. Preceding exact-source gates `20261107-062747-64d9272f` and
`20261107-061822-1babe0aa` also passed. Earlier failed gates and native runs
remain in the private evidence; assertions were preserved, including the
rendered-field/path-echo guard for synthetic Save input. No local Rust build,
deployment, VT work or main merge occurred.

branch: lab/viu-editor; head SHA: 5ad8ad4aac6cba0363c28dc90103493842a42c2e (exact tested code before this receipt-only commit); acceptance evidence: docs/ports/ced-acceptance.md, 61/61 actions, 12 native receipts, 35 matched captures (31 exact, three raster-edge-only, C1 Replace approved), nine live Prefs variants, cleanup verified; CBC job ids: 20261107-063528-808d3be7, 20261107-062747-64d9272f, 20261107-061822-1babe0aa.


## Integration close-out in progress

Frozen first source: 9643f9235819d33fc8e398ea0abfcc29d84192cf. Four release engines saved on CBC2. Monitor release revalidation passes all 12 runs and 55 figures, Bus 46/46; ordinary whole/regions distortion 0. Full Rust test stages: 10649 passed, no failures; the job was interrupted during the oracle build. Furniture/Track A continuation and worker-locked direct performance comparison are queued. First performance witness passed 480 samples.

Merged accepted Ced d4e19db5259e448190f3e877a0021f735195635a; only last.md conflicted. No source conflict. Ced release-byte acceptance, Monitor quick check and the second four-engine set remain pending.

Branch lab/gold-parity; head is the merge containing this receipt; first source 9643f9235819d33fc8e398ea0abfcc29d84192cf, CBC2 /build/es-artifacts/9643f9235819d33fc8e398ea0abfcc29d84192cf; final gates and second artifacts pending; CBC 20261107-064343-f05d5cfb-a1, 20261107-075258-c6071acd, 20261107-075542-24a1c263-a1, 20261107-075901-fe888f44.


## Dopus frame migration checkpoint — OPEN

Dopus now mounts the shared gold frame through `viu run apps/dopus`. Existing
Mix reducer, bulk rows, file jobs, previews and persistence are retained.
Native session/job/macro/keymap callbacks are wired in Mix; pane query and
open handlers have been added. Full inventory and remaining acceptance work:
[docs/ports/dopus.md](docs/ports/dopus.md). This is not parity acceptance.

CBC 20261107-112139-11c64519-a1 passed the two inherited scan-budget tests
and 2,000-row debug profile at c10e6f725a39. Complete log read. Debug callback
medians: sort 328.02 ms, refresh 327.90 ms; release budget remains unmeasured.
The new whole-bundle frame/atomic two-pane refusal test is queued at 8fd7120
in CBC 20261107-112707-102d6304. No production install or VT action.

branch lab/parity-dopus; head 8fd7120 (tested-code candidate; this receipt commit records its own HEAD); frame adopted, features/Bus/Prefs acceptance pending, SSIM/regions pending; CBC 20261107-112139-11c64519-a1 PASS, 20261107-112707-102d6304 pending.


## Dopus current handoff — NOT ACCEPTED

Current implementation source: d1b655a. Shared frame and generic runtime are
connected; the retained per-app comparator is still in the tree. New reusable
navigation-list and text-only status support replace two visible mismatches.
The 44-entry gold HELP catalogue and 33-action Bus table are captured as data;
Mix owns describe, activation and app callbacks. Native quit replies are fenced.

Only the earlier c10e6f7 scan/profile gate has passed. The latest CBC gate is
20261107-114033-a67ba1a0, still queued behind occupied workers. Preparation
must return/apply formatting, then rerun the exact-source gate. Previous own
queued sources were explicitly cancelled with their interest receipts, not
reported as failures. Do not install this checkpoint.

The inspected 1040x680 diagnostic pair has whole SSIM distortion 0.0466528;
regional differences and retained evidence are in docs/ports/dopus.md. This
is not visual acceptance: several geometry/content differences remain and
private settingsd/live Prefs, full feature/Bus/chooser/launch/shutdown suites,
release performance and exact-source artifact validation are outstanding.
No Fable deviation or question is proposed.

branch lab/parity-dopus; head d1b655a (implementation SHA; receipt commit separately); parity evidence: frame adopted, scan-budget tests PASS at c10e6f7, features/Bus/Prefs OPEN, diagnostic SSIM whole 0.0466528 and regions recorded, no visual acceptance; CBC 20261107-112139-11c64519-a1 PASS, 20261107-114033-a67ba1a0 QUEUED.

## Dopus round two — repair checkpoint

Generic `viu run` now owns the native host and chooser presenter. The Dopus
Rust package and comparator are retired; Mix retains application policy.
The shared frame, native panel sizing, Places rows, location/pane/footer flow,
macro icon, text-only status, dialogs and native search have been repaired.
Gold's flat Theme menu, UI command IDs, separators and Ctrl+Slash binding
replace incorrect inventory data. Targeted navigation preserves focus; mail
holds its native receipt until completion. Native config-root resolution,
launch forwarding, headless hosting and config-save debounce are present.

The last inspected diagnostic has exact header/Places/Properties/status
regions; pane distortion is 3.14659e-6, traced to outline paint order. The
following generic repair paints the dock outline before its content, as gold
does. This is not visual acceptance. The oracle completed 17 live Prefs
variants on one process; candidate and final release witnesses remain open.

CBC `20261107-124448-c8da34aa`: diagnostic scoped gate passed 65 library and
11 focused tests, checks/Clippy, release bulk measurement and binary build.
Later debug jobs passed the 11 focused tests and exported capture artifacts.
Final exact-source release gate and handoff will run on cbc2/cbc3. Superseded
heavy job `dopus-r2-final-3b53be1` was cancelled while queued, before execution.
No installation or desktop action. No Fable deviation is proposed.

branch lab/parity-dopus; head SHA 89db78f397c5c976bff2cea06aed67709eb41180 (implementation; receipt HEAD is the commit containing this checkpoint); OPEN: final features/44-descriptor and 33-action Bus/Prefs/release visual acceptance; CBC 20261107-124448-c8da34aa PASS diagnostic, 20261107-124952-1908ef48 PASS debug, 20261107-130315-f559f07f and 20261107-131012-f398b6e1 PASS checks with formatting returned, dopus-r2-final-3b53be1 cancelled before execution.

## Dopus round three validation checkpoint

Baseline and broad native captures now match gold for header, Places, panes,
Properties and status; menus, shortcuts, selection and most live Prefs states
are pixel-identical. The narrowed Places/font transition drift is closed.
The compact menu now follows its effective form and its focused pair is exact.
Truthful About source stamps, cursor blink and debug-only layout markers are
recorded; final release figures remain required.

Native file operations, Email success/BUSY/DENIED, macro run/output/stop/CRUD,
executed/completion topics, ten chooser lifecycle cases, offline mapped windows
and duplicate headless refusals passed. Settings loss and restart follow gold
on unchanged PIDs. A real 65,537-name release probe refuses SCAN_BUDGET without
changing either pane or model serial. Prior-source release callbacks are
64.208/64.861 ms median (sort/refresh); native Bus acknowledgement plus applied
model readback measured 126.922/128.603 ms median in that separate run.

Native input exposed a stale model echo before Enter and a submit-key leak
into file-open. Shared text retention, selection-range delivery and shortcut
fencing are under scoped regression/native validation. Do not mark accepted
until this and the final exact-source release repeat pass. No Fable question
or production action.

branch lab/parity-dopus; head SHA 680bdad (implementation before this receipt); OPEN: native input regression, final release-byte full inventory/Bus/Prefs/visual repeat and acceptance; CBC dopus-r2-final-ad30f4a-a1 PASS cbc3 prior source, 20261107-134258-f578434e PASS debug, 20261107-134533-fff897f4 and 20261107-134817-ff89f5f2 PASS checks with formatting applied, 20261107-135257-c3507e31 PASS debug, dopus-r3-final-21829a5 RUNNING detached, dopus-r3-final-643da24 QUEUED detached.

## Dopus round three — native audit converged

Implementation `13e0977d3ce7290d85630f16fa099d1b083515aa` retires the
remaining native input, modal focus, activation, headless typed-dialog,
Email launch and close-reply defects. Full windowed and 17-state live Prefs
diagnostics pass; mkdir/rename, preview, drop and hosted whole-surface pairs
are exact. The native mode button works through pointer input. The 44 gold
descriptors and 33 core actions remain separate from window UI commands.

Gold publishes compositor toolbar/menu commands that its execute dispatcher
then refuses. Both native replies and captures are retained. The port repairs
that broken function under standing Fable C1 by routing the published IDs
through its existing Mix window registry, preserving the 33-action catalogue
and headless contract. No Fable question, installation or desk action.

The release gate is detached and the source/recipes frozen privately. Continue
with its exact executable/hash, the full native evidence suite, whole/region
inspection, release 2,000-row measurements and real scan refusal, then write
the acceptance record. Earlier diagnostic and prior-source gates are not the
final acceptance receipt.

branch lab/parity-dopus; head SHA 13e0977d3ce7290d85630f16fa099d1b083515aa (implementation; receipt-only commit follows); OPEN: exact release-byte repeat, final region inspection and acceptance record; CBC dopus-r3-final-scene-a1 PASS cbc3, dopus-r3-accept-b1315fd-a1 PASS cbc3, dopus-r3-accept-final-a1 PASS cbc2 prior sources, 20261107-145411-6f4b4ed8 PASS checks with formatting applied, dopus-r3-accept-chrome QUEUED detached.


## Dopus round four — chooser validation and frozen release gate

Implementation `5570cea13a11ea912ed32c56102e64b6fa15caeb` composes the chooser in Mix on the shared
frame, navigation list, tree table and gold dialogs. Native viewport creation,
settings following, dialog target geometry, disabled controls and short-window
browsing are reusable facilities. No per-app Rust has returned. The pane scan
charges hidden/filtered names and child counts against 65,536 total entries
and depth 64 before committing a candidate.

Native save stem/Enter, raw names, replacement, filtered extension, caller
choices, independent windows and cancellation pass. Seventeen live chooser
Prefs states passed without restart; gold lacks this follower, so this is the
standing Mark-mandated improvement. Initial paired save/options/replacement
captures and difference figures are retained privately. Caller connection
identities are truthful data. Final release pixels remain unaccepted.

Source and the complete native recipe are frozen as `accept-5570cea13a11-final`.
Detached heavy CBC job `dopus-r4-final` is queued. The owned native follow-on
verifies the exact release receipt/hash before running Bus/features, 17 main
Prefs states, hosted chrome, headless operations, chooser, release 2,000-row
and real scan-refusal probes. Results still require full log and image review.
No deployment, VT action or Fable question.

branch lab/parity-dopus; head SHA 5570cea13a11ea912ed32c56102e64b6fa15caeb (implementation; this receipt-only commit follows); OPEN: compact chooser capture inspection, exact-source release suite, final whole/region inspection and acceptance record; CBC 20261107-153835-e12ca0e3 and 20261107-154200-5b579676 PASS checks with formatting returned, 20261107-154310-d23200a4 and 20261107-154537-cf8475d9 PASS checks with formatting applied, 20261107-154803-66891247 PASS checks with formatting applied, dopus-r4-final QUEUED detached.


### Chooser native validation complete; release evidence running

The shared transparent viewport recipe and gold minimum width are now restored.
Native save, replacement clicks, filtered extension, options, independent
windows and 600x300 browsing pass. The chooser follows all 17 live Prefs
variants without restart. Latest diagnostic whole-image SSIM distortions:
save 0.000473757, replacement 0.000376057, options 0.000381545, short window
0.000272413. Inspected differences are caller connection text and debug raster
marks; only exact release captures can settle the final visual verdict.

Source `d95a5a3966fd6c7d3493917c382b5feafd6b0b19` and its recipe are frozen
as `accept-d95a5a3966fd-final`. CBC `dopus-r4-release-frame` is running on
cbc3. The owned follow-on verifies the release executable/hash and runs the
complete native evidence suite; its results still require inspection.
Superseded own jobs were cancelled using their explicit native interest
tokens; no other session job was touched. No acceptance or install claimed.

branch lab/parity-dopus; head SHA d95a5a3966fd6c7d3493917c382b5feafd6b0b19 (implementation; receipt-only commit follows); OPEN: exact-source release gate/suite, final regional inspection and acceptance record; CBC 20261107-155151-c9e6be4a PASS scoped checks/debug build with no worker diff, dopus-r4-accept-chooser-a1 cancelled as superseded, dopus-r4-final cancelled before execution, dopus-r4-release-frame RUNNING detached.

### Round four final source frozen

Implementation `7c96aaed75fcd276bf196038154169ac5eff5b9c` restores the missing
mode palette entry, installs live appearance in each chooser viewport and
reports readiness after the first drawn scene. Initial decoration negotiation
matches the undecorated native builder. Gold and candidate chooser captures
now wait for mapping and complete the same native resize/repaint sequence
before capture. Matched effective legacy radii close the control geometry;
the later region audit found an outer-corner defect in this source. Images
are never rescaled. The next checkpoint records its repair.

Prior-source `955bd3f2b76b` passed its heavy gate and all twelve release-native
groups. Its audit exercises all 44 gold descriptors and 33 core actions;
the missing search row is closed. Those receipts remain prior-source evidence.
An additional native chooser probe refuses 65,537 names with SCAN_BUDGET,
preserving the prior folder, complete rows and serial and leaving the request
open. This diagnostic still needs the final release-byte repeat.

Final source and recipes are frozen as `accept-7c96aaed75fc-final`.
`dopus-r4-accept-settled` is detached in the heavy CBC queue. Owned follow-ons
verify the exact release hash, run the complete suite, repeat the matched
chooser/budget probe and generate the region audit. Full log and image
inspection remain required before ACCEPTED. No installation or desk action.

branch lab/parity-dopus; head SHA 7c96aaed75fcd276bf196038154169ac5eff5b9c (tested implementation; receipt-only commit follows); OPEN: final exact-source release gate/native suite and inspection; CBC dopus-r4-release-frame-a1 PASS, dopus-r4-native-final-a1 PASS, 20261107-163547-a13208ea PASS scoped checks/debug with formatting applied, dopus-r4-accept-settled QUEUED detached.

### Native chooser corners closed

Implementation `4bccf5913312900b504054fc3c2d277078071c4a` installs the same
native toolkit frame facility as the main generic host and honours the Mix
chooser's explicit client-side frame. This closes the square outer corners
missed by the earlier whole-image inspection. Save, replacement, options and
600x300 pairs now have zero SSIM and RMSE distortion in header, navigation,
pane and form regions at matched effective settings. Only truthful caller
connection text differs. The native 65,537-name refusal still preserves the
chooser folder, complete rows and serial and keeps the request open.

Scoped CBC `20261107-165508-fcc5458e` passed all twenty focused tests and the
debug host build with no worker diff; the complete saved log was read. Final
source/recipes are frozen as `accept-4bccf5913312-final`. Detached heavy job
`dopus-r4-accept-framing` replaces own queued job `dopus-r4-accept-settled`,
cancelled before execution. Owned follow-ons retain the exact release suite,
matched chooser/budget repeat and regional figures for final inspection.

branch lab/parity-dopus; head SHA 4bccf5913312900b504054fc3c2d277078071c4a (implementation; receipt-only commit follows); OPEN: final exact-source release gate/native suite and inspection; CBC 20261107-165508-fcc5458e PASS scoped debug, dopus-r4-accept-settled cancelled before execution, dopus-r4-accept-framing QUEUED detached.
