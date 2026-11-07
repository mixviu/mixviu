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


## Term initial checkpoint — NOT ACCEPTED

Branch `lab/parity-term`, based on `4f2c0fd`. Frame/model source
`65de543`; native descriptor extraction source `dc635f1`. Shared frame
adopted at gold initial 900×560, using native session settings and
Mix-owned close. Six model checks and Mix lint pass. The scene is empty;
there is no terminal replacement or parity claim.

Imported pinned teletypewriter child-only mapping, its MIT licence,
patch record and mutation guards, plus gold sealed descriptor ownership
into libraries/viu. Gold wire constants and session bytes remain unchanged.
The self-exec test name is adapted to the Viu module path. No session
supervisor, tabs/panes policy or per-app Rust host was imported.

CBC repair job `20261107-114140-d9b68b01` is queued for cbc2/cbc3 at
`dc635f1`; result, lock patch and exact-source repeat remain pending.
Both workers were occupied by integration work when submitted. No Rust
command ran locally.

The pinned rio-vt dependency includes C++ simdutf, which cannot be copied
under the current foreign-code rule. Its scalar parser/validation/base64
paths need a bounded pure-Rust adaptation with preserved invalid UTF-8
and base64 behaviour, dependency/licence inspection and conformance tests
before import. This is implementation work, not a proposed gold deviation.

Remaining: VT/grid/render widget, PTY/session supervisor extraction and
generation checks, full Mix tab/pane/profile/actions/Bus policy, native
clipboard/IME/input and search, live terminal Prefs, exact release gate,
matched real-session captures and region inspection. No Fable question,
no acceptance, no install, no VT work or merge.

branch: lab/parity-term; head SHA: 65de543 (code; receipt HEAD contains this checkpoint); parity evidence: frame model 6 checks, features/Bus/Prefs/SSIM/regions pending; CBC job ids: 20261107-114140-d9b68b01 (queued).


## Term round 2 — native extraction awaiting CBC

Code head `3ef390712cdbfdf551bd4ab4a3921bf6d94b5bd1`; branch `lab/parity-term`.
Fable D6 authorises simdutf as Viu debt. The previous entry claiming it
could not be imported is superseded. Added foreign policy and gate; Viu,
Rio and simdutf are forbidden to strict Mix/system/mix-shell/mixd trees.
Pinned gold semantics are retained rather than claiming scalar fallback
equivalence on malformed UTF-8/base64.

Extracted native gold VT/grid/PTY Machine, scrollback/damage, selection,
mouse reporting, bracketed paste and clipboard request mechanism, font and
Unicode glyph raster, GPU atlas/grid, and pane-owned IME cancellation.
Retained gold unit and GPU/CPU parity tests. Added chunk-boundary, VT
cursor/erase/insert/alternate-screen, scrollback/selection and real Mix
PTY input/resize/exit/reaping witnesses. Original files and hashes are
listed in provenance/terminal-native.json. No Rust tab/pane app policy
was imported. Native lifecycle uses issuer/lease and write-permit seams;
these are not a claim that verified supervisor attachment is complete.

Mix frame tests (6) and gate-script lint pass. No Rust tests have yet
passed for this source. The previous round job has no completed receipt;
superseded queued jobs 20261107-114717-a7c8ecb3 and
20261107-114739-08a14bc1 were cancelled before execution.

CBC job `term-native-r2-3ef3907` is submitted at the exact code SHA with
`--await` through buildd; await id
`1b020bda-3c88-4d1f-a49a-96806c7385d9`. Agentd registered automatic
resumption on the result and explicitly directed ending this turn without
polling. On resumption: read the complete saved log, apply the returned
lock/fmt patch, repair failures and repeat at exact source before native
widget/host integration. No user decision is requested.

Open: wire reusable widget into host; verified session supervisor and
stale generations/reconnect; Mix tabs/panes/profiles/menus/actions and
full Bus contract; native clipboard/IME/input/search integration and live
terminal Prefs; release gate and matched real-session whole/region
captures and acceptance record. NOT ACCEPTED. No install or VT changes.

branch: lab/parity-term; head SHA: 3ef390712cdbfdf551bd4ab4a3921bf6d94b5bd1 (code; receipt HEAD contains this checkpoint); open items: native CBC validation, widget/host and verified session integration, full Mix app/Bus/Prefs, conformance and visual acceptance; CBC job ids: term-native-r2-3ef3907 (await registered).


## Term round 2 continuation — native tests pass, surface integration pending

Exact extraction test source: `35c0fce38096`; code checkpoint now
`1d498ca1d92eea7ccb188ae8bc023689850e6670`. Not accepted.

Read both complete saved CBC logs. First source compiled with 129 passing
tests and one new-test failure: untouched Rio cells contain NUL, whereas
gold snapshots/raster treat them as visible spaces. Corrected the visible
row helper to gold semantics and kept an exact raw-NUL erase assertion plus
text assertions. Returned CBC lock and fmt patch applied locally. No gold
test was removed or relaxed.

`term-native-r2-repair-a1` on cbc3 ran 130 tests successfully, five original
fixture/environment/benchmark ignores, six Mix checks, no worker diff.
Includes real Mix PTY input/resize/exit/reap, sealed descriptor delivery and
CLOEXEC isolation, VT chunks/history/selection/mouse/paste, Unicode glyphs,
GPU/CPU frames at all scales and pane IME cancellation. Saved logs:
`20261107-121451-cbc3-3ef390712cdb-term-native-r2-3ef3907-a1-2483ba81.log`
and `20261107-121815-cbc3-35c0fce38096-term-native-r2-repair-a1-4d8827bb.log`
under the private CBC cache.

The repair job returned 1 because my imported foreign policy incorrectly
put mixd under the login shell static-C rule. Fable D2/D3 explicitly admits
existing vendored static C, with no new DT_NEEDED library. Recorded its
four existing static dependencies as D2 exceptions; Viu/Rio/simdutf remain
forbidden to every strict dependency tree. No ELF receipt yet. No Fable
question or new permission is needed to implement that existing ruling.

New source wires native sessions into generic runtime Services, SceneView
and Widgets, with a reusable GPU surface and `viu_terminal` in-process
binding. Mix chooses create/close; handles never reuse a closed ID, close
serializes input/teardown, PTY damage wakes the UI, and bytes remain native.
GPU resources initialise lazily on either host target. The provisional
single-pane Mix specimen now opens a real PTY and draws it. New native
checks cover stale input after close, refused geometry, retained sessions
through live font change and idle damage. Mix model checks and lint pass;
new Rust source and host build await validation.

Next CBC job `term-widget-r2-1d498ca` will prepare/check the native suite,
foreign closure and generic host build at the exact new code source.
Resume from its complete log, apply fmt changes, repair and repeat exact
source. Open: gold Mix tab/pane/profile/menu/action policy and full Bus;
verified supervisor attachment/generations/reconnect; clipboard and input
edge integration, search and all terminal Prefs; release/native acceptance
and matched real-session capture/diff/inspection. No install or VT changes.

branch: lab/parity-term; head SHA: 1d498ca1d92eea7ccb188ae8bc023689850e6670 (code; receipt HEAD contains this checkpoint); open items: new widget validation, full Mix app/Bus/Prefs and verified sessions, release and visual acceptance; CBC job ids: term-native-r2-3ef3907-a1 (129 pass, one new-test failure), term-native-r2-repair-a1 (130 pass; foreign policy failure), term-widget-r2-1d498ca (next await).


## Term widget continuation — extraction green, first app load under diagnosis

Native checkpoint source `f8b976e2cea7cc664c849982c98632759ec464b5`.
CBC `term-widget-r2-view-repair-a1` returned 0 on cbc2: 133 native tests
passed, five original ignores, Mix frame and selector/replay tests passed,
foreign dependency trees clean under D2/D6, generic debug host built, no
worker diff. Complete saved log read:
`20261107-123109-cbc2-f8b976e2cea7-term-widget-r2-view-repair-a1-20d3c64b.log`.
Debug artifact SHA256
`a75082e700d5e6a051bc4bd99537efd072ed5ffac56d8cc69037623482d3ff10`;
source/byte receipt fetched and verified from /build/term-artifacts on CBC.
This is not a release receipt or full app acceptance.

Applied the earlier complete CBC fmt patch. Fixed pinned egui API use
(run_ui and structured Preedit) and the headless harness texture-delta
ownership. Original native tests and assertions retained. Synthetic type
now refuses missing pane/tab selectors and foreign process tokens, and
Mix retains a 128-entry mutation replay FIFO. Full Bus validation remains
open. The process instance uses gold OS-random safe-integer semantics.

Latest code `78b984ae8eae` adds the pure Mix tab/pane lifecycle reducer,
selector agreement and tree composition tests, bounded generic native split
rendering, and an actual-bundle load regression probe. Model tests and lint
pass. The reducer is not yet wired to full tabs/panes/menu/Bus commands.

Private archive `term-parity-20261107` contains isolated native scripts and
failed runs with readbacks and closure receipts. Pinned installed gold
reports 8779f33284b7. Gold real-session output and 900×560 native capture
succeeded in native.t5YeUT and native.gWIe4o. The latter normal Quit returned
success but exceeded the harness 8-second exit bound; finally stopped the
recorded owned processes. No C1 closure claim yet. Earlier --comp launch
argument was a harness error: pinned Term takes the compositor via environment.

Candidate first behaviour load remains unavailable, so there is NO matched
image pair or SSIM result. Corrected the independently found icon mismatch
to gold FileTerminal (file-terminal). Artifact relocation also requires an
explicit native prefix in the isolated harness. Neither correction alone
resolved loading. Later host callbacks mask the original error. A private
diagnostic bundle catch variant is retained separately and is never used
as parity evidence. New term_bundle integration test isolates first load
and will report the original cause without those callbacks.

CBC `term-widget-r2-bundle-probe` at exact code `78b984ae8eae` is queued; both
workers are occupied by integration jobs. Registering automatic agentd
resumption on this existing job. Next: read full log, repair the first-load
cause, apply fmt diff and repeat exact source; retake native witness, then
wire the tested Mix reducer and complete all remaining inventory/Bus/Prefs,
verified supervisor and native input/clipboard/search, release captures.
No Fable question, no install, no VT changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: 78b984ae8eae (code; receipt HEAD contains this checkpoint); open items: first bundle load, app/Bus/Prefs and verified session completion, release and matched visual acceptance; CBC job ids: term-widget-r2-1d498ca-a1 (compile failure), term-widget-r2-api-repair-a1 (132 pass, harness delta failure), term-widget-r2-view-repair-a1 (133 pass, host build and foreign gate green), term-widget-r2-bundle-probe (queued).


## Term first host capture — NOT ACCEPTED

Latest code `b095d72222c3c6b018f59edf2d2be2eac483401a`. Applied complete
CBC fmt patch from term-widget-r2-bundle-probe-a1. Its failed isolated
bundle test found the actual scene/control ID mismatch (root versus
terminal); repaired at 2765100. The probe completed with rc101; its result
and complete saved log are already consumed. Attempting to attach CBC
--await to its retained ID returned a UNIQUE jobs.job error, without
resubmitting work; the provisional await had already fired with the same
completed result. Do not repeat that old job as unresolved work.

Using the verified f8b976e debug artifact plus repaired Mix files, the
candidate now starts, executes the same Mix command output as gold, and
captures at 900×560 scale 1. Private native.iceFXQ retains snapshot, calls,
capture, effective menu/titlebar/window forms and all-tracked-PIDs-stopped
closure receipt. Its normal close request exposed missing app.quit; fixed
at b095d72, pending exact gate/native-close recheck. Diagnostic bundle
variants are separate, unaccepted repair witnesses.

Compared against gold native.gWIe4o, inspected originals/side-by-side/diff.
Private first-diagnostic-diff and first-diagnostic-regions retain raw
ImageMagick 7.1.2-35 metrics, crops, figures and inspection record. Full
SSIM distortion 0.0483143 / RMSE 0.055971. Regional SSIM distortions:
header 0.0749878; tabs 0.167079; content 0.0388236; bottom 0.0798319.
FAIL: missing menus/actions/tab strip, content 32px too high, extra inset/
lower band and status bar. Visible command text/colours agree. No rescaling
or threshold acceptance; these are open work, not Fable deviations.
Full diagnostic hashes, rectangles and limitations are in docs/ports/term.md.

Next exact-source gate job term-widget-r2-node-fixed at b095d72 will test
actual bundle load, native suite, Mix policy tests, foreign closure and
host build. Register automatic resumption on this fresh job only. After
its log/repair/exact-source check, wire Mix reducer into the live tab/pane
UI and Bus, then complete inventory and verified session/clipboard/IME/
search/live Prefs/release matched acceptance. No Fable question, install,
merge or VT change.

## Exact repaired widget checkpoint

CBC3 `term-widget-r2-node-fixed-a1` passed at exact source
`b095d72222c3c6b018f59edf2d2be2eac483401a`: 134 native tests passed,
5 retained fixture/benchmark ignores, both actual-bundle integration tests,
all three Mix policy suites, foreign dependency closure and generic debug
host build. Complete saved log read; TERM_PATCH was empty. This does not
attest ELF linkage, release bytes or application acceptance.

Fetched debug artifact SHA256
`08375b79ec45e965bd258a1660080f572d3ec9dbf2b75ad27db545abae3b5990`,
verified against its source/profile receipt, and archived Mix sources from
the same commit. Private `term-parity-20261107/native.uqPes6` retains the
isolated real Mix shell output, 900×560 capture, effective forms, calls and
normal close success. All tracked app/shell/settings/compositor/broker PIDs
were stopped. This exact-source debug witness supersedes the composite
first-load witness; the earlier visual FAIL remains open.

Continuing live Mix tab/pane reducer integration and its Bus surface. Full
verified session supervisor, input/clipboard/search, Prefs and release visual
acceptance remain open. Gold unchanged. No Fable question.

## Live Mix lifecycle checkpoint — gate queued

Code `39e1208653a54e0f0bc1b0a040713abc112fa721` wires live tabs, title
pins, ordering, native pane splits, focus and close; shared document tabs
and the terminal split tree form the actual scene. Generic frame option
removes Term's extra footer. Mix owns selectors and replay; explicit input
targets preserve focus. Native metadata (dimensions, title, PID, geometry)
and change notifications stay in process. Shell exit removes its pane.

Four local Mix suites pass. Title sanitation uses gold's 256 UTF-8 byte
limit, strips controls and line separators. Replay follows gold's lookup
before argument validation once a request ID has a recorded result.
Expanded actual-bundle test exercises real PTY split/tab sessions, replay,
inactive targets, no-target refusal, stale handles and final close.

CBC `term-live-tabs-39e1208` submitted detached at exact code above:
`mix tools/term_gate.mix --prepare`. Await its native compile/runtime result,
read the complete saved log and apply worker fmt diff before an exact-source
repeat. Then fetch the matching artifact and run live tab/split/input/exit
cases plus new matched captures. The earlier exact b095d72 debug witness
and normal close are green; current live lifecycle is not yet native-tested.

Open: props.watch publication; remaining action/menu/dialog/macro/Bus surface;
raw envelope and large integer handling; verified session issuer/control
supervisor and shell integration; input/clipboard/IME/search completeness;
live full Prefs; exact release and visual acceptance. NOT ACCEPTED. No
Fable question, install, merge, gold or VT changes.

## Lifecycle scene repair

Read complete saved log for `term-live-tabs-39e1208-a1` (rc101). Rust
compiled and all four Mix suites passed. The actual-bundle test rejected
unsupported `padding` and `gap` ports on the dock root; no lifecycle test
was weakened. Removed those ports and applied the complete worker fmt
patch to the host, shared strip, native sessions and bundle test.

Repair code `849b2aec8e4d99291108a7671625354c0d7910f3` is pushed.
Next exact-source gate: `term-live-tabs-dock-849b2ae`, same CBC recipe
`mix tools/term_gate.mix --prepare`, with native automatic resumption.
On result: read complete log, repair/apply any worker patch, repeat exact
source if changed; fetch matching artifact and run isolated lifecycle,
input, shell exit and matched captures. All earlier open parity items
remain open. Gold untouched; NOT ACCEPTED; no Fable decision required.

## Live lifecycle green; pane painting repair next

Read full `term-live-tabs-dock-849b2ae-a1` log: green, no worker diff,
four Mix suites, two actual-bundle tests (expanded real PTY lifecycle),
134 native tests/five ignores, dependency gate and debug host build.
Exact tested source 849b2aec8e4d99291108a7671625354c0d7910f3.
Fetched/verified debug SHA256
56d0f178788e13a333e733f2a9e037058cda8378c20368b5b1626addb8a29e1b.

Private native.sf7min uses matching source/artifact, real Mix output,
split/replay/inactive target/tab close/shell exit preserving sibling and
normal close. All tracked PIDs stopped. New diagnostic comparison against
native.gWIe4o: tabs pixel-identical, full SSIM distortion 0.0337594,
content 0.0141672, bottom 0.274618; header still absent menus/actions.
Inspected originals, side/diff and split. FAIL, no Fable deviation.

Code 0a34c03037ca0e3bdda34f2b87208c93868b5bec repairs pane painting:
theme canvas instead of black surround, gold border/focus ring and live
settings, physical-pixel fitting, legal minimum grid and texture ceiling.
Added representative sizing cases. Next CBC term-pane-paint-0a34c03 at
that exact code, mix tools/term_gate.mix --prepare; automatic resumption.
Read complete log, apply worker fmt and repeat exact source if changed,
then retake native matched captures. Continue full outstanding Bus/topics/
actions/menu/dialogs/macros, verified native issuer/control and Mix shell
integration, input/clipboard/IME/search, full live Prefs and release visual
acceptance. Gold unchanged. NOT ACCEPTED. No Fable question.

## Pane gate green; change publication checkpoint

Read full `term-pane-paint-0a34c03-a1` log: four Mix suites, two actual-
bundle tests, 135 native tests/five ignores, dependency gate and debug
host build passed. Returned formatting-only terminal view patch applied.
No new pane capture yet; next exact-source check includes this patch.

Code `9b53f0d9f50eda3b337d05ed4be636bf2b5d4fdb` implements ordered
gold tab/pane/title change sequences and props.watch publication using
the existing Viu background job facility. Mix retains policy; a bounded
256-event queue drops new deliveries on overflow and retains sequence
gaps. No per-PTY-byte Bus traffic. Local tests pass for event order,
retitle without layout revision, unfocused/focused close, watch sequence,
no duplicate replay events and overflow. Actual broker delivery/reconnect
and last-close flushing remain to be checked; partial Bus parity only.

Next CBC `term-change-events-9b53f0d` at that exact source, same prepare
recipe. Read complete log; apply any patch and repeat exact source if
changed. Then fetch verified matching artifact/source; run isolated
native lifecycle plus actual topic observer; retake pane-paint matched
captures and inspect regions. Continue the rest of full action/menu/
dialog/macro/Bus contracts, verified session issuer/control and shell
integration, input/clipboard/IME/search, full live Prefs and release
acceptance. No Fable question; no install/VT/gold changes. NOT ACCEPTED.

## Native topic delivery repaired; pane regions identical

Read complete term-change-events-9b53f0d-a1 log: four Mix suites, two
bundle tests, 135 native tests/five ignores, dependency gate and generic
host build green, no worker diff. Exact source 9b53f0d, verified debug
SHA256 ee189b5838344d4e988d34c2c4655fdc5ca5a9ba13b55f591502e1e2460f085e.

Native.dtQmFV used matching sources/bytes and real Mix sessions. Matched
capture versus native.gWIe4o: tabs/content/bottom each SSIM/RMSE 0/0.
Full distortions 0.00535614/0.0195939, header 0.0749878/0.0752184.
Inspected originals/side/diff; missing menus/actions keep visual FAIL.
Private pane-painted-regions contains metrics/inspection; public port doc
records full hashes. This state is debug-only, not overall acceptance.

The same native run found jobs.start incorrectly nested under body and
no topic delivery. Corrected to typed fields; gold inner commands are
suffixes (pane.changed etc), now preserved. Added gold's two-second
shutdown event-drain bound, with final removal records. Local tests pass.
Repair code ca019e33aada1112b34cacdbf4351ffa14f12eb4 is pushed.

Private native.HINPJ0 is an explicitly composite repair witness using
9b53f0d binary and ca019e3 Mix sources: native lifecycle passed, 22
events revisions 6..27 delivered serially without gaps, correct inner
commands and final pane/tab removals before normal close. All tracked
PIDs stopped. An earlier relocated harness path error is retained under
repair-ca019e3/native.B9lkDi with cleanup, never used as acceptance.

Next CBC term-events-drain-ca019e3 at that exact source, same prepare
recipe with automatic resumption. Read full log/apply any patch/repeat
exact source; fetch matching artifact and rerun native topic/lifecycle
evidence. Continue gold menus/actions/dialogs/macros/full Bus, verified
native issuer/control and Mix shell integration, input/clipboard/IME/
search, full live Prefs and release visual acceptance. NOT ACCEPTED;
no Fable question, install, VT change or gold change.

## Exact topic drain green; command registry checkpoint

Read full term-events-drain-ca019e3-a1 log: green, no worker diff;
four Mix suites/two bundle tests/135 native tests/five ignores plus
dependency gate and host build. Verified matching debug bytes (same
SHA256 as 9b53f0d: ee189b58..., source changes only Mix). Private
native.BG1pld reruns real lifecycle, topic sequence and final-close
drain against exact ca019e3 bundle/artifact; passed and tracked PIDs
stopped. No composite bytes in this witness.

Captured pinned gold term.actions and native menu list in native.oBJRB9;
retained its repeated normal-Quit timeout and finally cleanup. Imported
only public registry/menu data, source/hash provenance recorded.

Code d85560415c58ce2ae023edc12dcf2461063e5e42 adds 61 gold actions,
File/Edit/View/Pane/Tabs/Help menus, standard four-icon tail, default
chords and dynamic enablement. Mix handles tab/pane commands, directional
focus with tree fallback, themes, zoom steps, clipboard effects, macro
and help/search state. Shared native command-search control validates
enabled command identity; help grid supports gold striped/monospace
rendering without changing other users' defaults. Five local Mix suites
pass; scoped CBC adds the owning command-search admission test.

Next term-actions-d855604 exact-source prepare gate with automatic
resumption. Read full log/apply worker fmt/repeat exact source if changed;
fetch matching artifact and run isolated gold/candidate registry/menu/
execute/native key/dialog/theme/clipboard witnesses and matched captures.
Action-state currently lacks full pane/cell/scale reply metrics; zoom
persistence, complete About body, keymap rebinding, action-topic overflow
and full Bus matching remain open. Continue verified native session/
control/shell integration, input/clipboard/IME/search, full live Prefs
and release acceptance. NOT ACCEPTED. Gold untouched; no Fable question.

## Native actions: shared menu fix; shortcut renderer open

Read complete term-actions-d855604-a1 log: five Mix suites/two bundle
tests/135 terminal tests/five ignores plus command-search admission,
dependency gate and host build green. Full worker formatting patch
applied. Verified debug SHA256 94dfa290cf005964ef8dc0ce4eeb5b8ad3be28e4e5c9d0599dc7b996c93e96f2.

Native.zPpnfQ rendered menus but menu New Tab failed. Diagnostic trace
native.EjTjXv found NativeMenu::draw double-wrapping an already structured
command (value.command was an object, so app lookup returned UNKNOWN_ACTION).
Generic fix 388b38a passes it through unchanged, matching rich menu intent
shape. Must verify native menu callbacks on the next compiled artifact.

New matched actions-first-regions figure: tabs/content/bottom each 0/0,
header SSIM/RMSE distortion 0.0353355/0.0448267, full 0.00266545/0.0116771.
Inspected originals/side/diff. Labels/menus align; icon/caption differences
remain. FAIL, debug only. No acceptance implied by low whole-frame score.

Native.LSmstT passed direct execute, real Ctrl+Shift+T, targeted close,
help dialog open/answer, command-search text+selection, local light/session
theme, zoom/reset and normal close. Corrected an invalid pointer-button
name in an earlier harness variant; failed runs retained. Recorded PIDs
stopped; action-created tabs explicitly closed via native shutdown.

Screenshot inspection found Keyboard Shortcuts body blank although native
model has all 23 chord rows. Native.eaUKMo retains model and repeat
captures; new pure projection assertion prevents omission at Mix level.
Diagnose native dialog drawing (and capture settling) next; do not call
the help dialog visually accepted merely because open/answer passes.

Code 2b580ce99bd8ea85049bebc444bcb38abeef22da includes fmt/menu fix and
projection assertion. Next CBC term-menu-intents-2b580ce, same prepare
recipe with automatic resumption. Read full log/apply any patch/repeat
exact source if changed, fetch artifact and repeat actual menu/native
callbacks, match gold keyboard dialog and fix renderer. Then action
metrics/keymaps/About/zoom persistence; full Bus/raw envelope/verified
session/control/shell/input/clipboard/IME/search/live Prefs/release visual
acceptance. NOT ACCEPTED; no Fable question, gold/install/VT changes.

branch: lab/parity-term; head SHA: 2b580ce99bd8ea85049bebc444bcb38abeef22da (code; receipt HEAD contains this checkpoint); open items: menu callback gate/native repeat, shortcut dialog renderer, full Mix app/Bus/Prefs, verified session and input/clipboard/search, release acceptance; CBC job ids: term-actions-d855604-a1 (green with fmt patch), term-menu-intents-2b580ce (next await).

## Term menu witness and dialog/metrics checkpoint — NOT ACCEPTED

Read the complete saved log for `term-menu-intents-2b580ce-a1`: rc 0,
no TERM_PATCH diff. Five Mix suites, two bundle tests, 135 terminal tests
(five existing ignores), command-search admission, foreign gate and debug
host build pass. Verified fetched artifact SHA256
`b311f41117755b0ec08ab59c0faa4dcf48ae2431b17407ebb351136312fa3f56`.
Native `native.h5uA6t` passes the actual File → New Tab callback plus
targeted close, keyboard/search actions, mode, zoom/reset and normal close.
All recorded PIDs stopped. Earlier header differences still unaccepted.

Blank shortcuts body reproduced after two seconds. The grid code was in
an unreachable branch below gold_dialog's early return. Moved it into
the active reusable renderer, removed the dead branch, bounded/validated
grid rows and added a native widget geometry regression. Gold capture
`native.Dae992` shows the expected rows; recorded PIDs stopped.

Native session placement now supplies measured logical cell, device scale
and unscaled font size. Mix action state now includes gold's visible pane
geometry/grid/focus data. Text-scale Prefs rebuild the existing raster;
owning test checks observed values and retained session handles. Live
keymaps use the existing native gold resolver and update terminal chords,
menus and help, including multiple bindings and explicit unbinding.
Five local Mix suites pass. Native tests/captures of these changes pending.

Next CBC: `term-dialog-metrics-9ea52c7`, exact code
`9ea52c727cd2d7f4f602fc485e152664eb84edf1`, term_gate --prepare. On
result read full saved log, apply any fmt/lock patch and repeat exact
source if changed. Fetch matching artifact/source; repeat native menu
suite and compare settled keys capture with gold, then exercise live
text scale/keymap/metrics on one unchanged PID. Continue full standing
brief: verified session/control/sealed shell integration, complete Bus,
clipboard/input/IME/search, zoom persistence/About, all live Prefs and
release visual acceptance. No Fable question, install or VT changes.

branch: lab/parity-term; head SHA: 9ea52c727cd2d7f4f602fc485e152664eb84edf1 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — native dialog/metrics/keymap witnesses, full Bus/Prefs/session/input/clipboard/search and release visual acceptance open; CBC job ids: term-menu-intents-2b580ce-a1 (green exact source), term-dialog-metrics-9ea52c7 (next await).

## Term dialog, live metrics/keymaps and exact-frame repair

CBC `term-dialog-metrics-9ea52c7-a1` passed, including the live widget
dialog regression and font/text-scale metric checks. Read complete log;
formatting-only patch applied in f55e499 and pushed. Debug artifact
SHA256 `44770e2417b1347dc0bd4ecf1b521ab87203f92457daed3b887f85e098712dfe`.

Native.uIceLC passes: dialog crop exact against gold, measured cell
9×20→14×30/grid 98×23→63×15 after text scale 1→1.5 on the same
PID 3815378, settingsd rebind to F2, actual F2 New Tab and normal close.
All recorded PIDs stopped. Captures/readbacks/harness retained privately.

Gold view.rs calls titlebar::seamless(true). Candidate omitted it; the
one-pixel rule also shifted caption/action glyph centres. e945bd9 now
supplies that existing frame setting. Composite native.BvAeFe (9ea52c7
debug executable, e945bd9 Mix bundle; NOT exact-source acceptance) matches
both gold real-session and keys captures byte-for-byte. Whole and every
region SSIM/RMSE distortion 0/0; originals/montages/diffs inspected.
Live metric/rebinding witness repeats and recorded PIDs all stop.

Next CBC exact-source repeat `term-seamless-e945bd9` at
e945bd9815fadde1296715fa288db8b4129cdd5a. Read full log, apply any patch
and repeat if needed; fetch exact matching executable/bundle and repeat
keys-metrics plus native menu/lifecycle/topics suites. Native golden keys
run native.Dae992, real output identical to native.gWIe4o. Private harness
keys-metrics-e945-diagnostic.mix is diagnostic; derive an exact-artifact
version for the next witness, retaining binary and source receipts.
Then continue full session/control/sealed Mix shell handoff, full Bus,
clipboard/input/IME/search, zoom persistence/About/all live Prefs and exact
release visual acceptance. No Fable question, install or VT changes.

branch: lab/parity-term; head SHA: e945bd9815fadde1296715fa288db8b4129cdd5a (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — exact-source frame repeat, full Bus/Prefs/session/input/clipboard/search and release acceptance open; CBC job ids: term-dialog-metrics-9ea52c7-a1 (green, fmt applied), term-seamless-e945bd9 (next await).

## Term exact frame verified; clipboard classification checkpoint

`term-seamless-e945bd9-a1` green, no worker diff, complete log read.
Fetched verified exact-source debug artifact and matching bundle. SHA256
44770e2417b1347dc0bd4ecf1b521ab87203f92457daed3b887f85e098712dfe.
Native.GDMFZD passes real Mix output, shortcuts dialog, unchanged-PID
text-scale metrics and live F2 rebinding, normal close, recorded PIDs all
stopped. Both session and keys PNGs byte-identical to gold; all whole and
region SSIM/RMSE distortion 0/0. Inspected figures retained privately in
session-e945-exact and keys-e945-exact. Debug frame checkpoint only.

Code 327ab52 restores gold platform clipboard classification, held-chord
release/modifier suppression, Meta control codes and image behavior.
Native widget emits captured-owner clipboard intents; Mix performs
actions and already-read paste without another clipboard read. Closed
owners ignored and stale Cut cannot close a new focus. Menu paste still
uses native asynchronous clipboard read. Five Mix suites pass. New native
classifier/widget tests await CBC. Primary/OSC52/mouse clipboard remains
open; no full clipboard acceptance claim.

Next CBC term-clipboard-327ab52 at
327ab521c783643a6c7430c09e9c4641ef1b5382, term_gate --prepare. Read full
log and apply any fmt patch; exact-source repeat if changed. Fetch matching
artifact and drive actual platform clipboard events (Ctrl-C interrupts
Mix, Ctrl-Shift-C copies, Ctrl-V control vs Ctrl-Shift-V supplied paste,
Cut, repeats, Unicode/brackets and closed-owner refusal). Repeat native
menu/lifecycle/topics where affected. Continue full standing brief:
verified session/control/sealed Mix shell, full Bus, mouse/clipboard/IME,
search, zoom persistence/About/all Prefs, release gate and acceptance.
No Fable question, installs, merges or VT changes.

branch: lab/parity-term; head SHA: 327ab521c783643a6c7430c09e9c4641ef1b5382 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — clipboard native gate, full Bus/Prefs/session/input/search and release acceptance open; CBC job ids: term-seamless-e945bd9-a1 (green exact source), term-clipboard-327ab52 (next await).

## Clipboard gate repair

Read complete log for term-clipboard-327ab52-a1, rc 101. Five Mix suites
and both bundle tests pass. Terminal test compile failed because the new
test used removed RawInput.modifiers. No terminal tests ran. Applied
returned fmt patch; cf75bd9 uses ModifiersChanged events and fixes runtime
modifier ordering: clipboard/text events see modifiers at their position
in the native event stream, not the frame's final state. Added same-frame
clipboard-before-Shift-release coverage. No weakening of expected results.

Next CBC term-clipboard-order-cf75bd9 at
cf75bd955fbc23c65c6b8b70e6c9e33d7441b166, term_gate --prepare. Read full
saved log, apply any fmt patch and repeat exact source if changed. Then
fetch matching artifact and perform the platform clipboard event witness
specified above. The e945bd9 exact-source pixel/Prefs checkpoint remains
valid historical evidence, not acceptance of these new clipboard bytes.
Continue full brief; no Fable decision, install or VT change.

branch: lab/parity-term; head SHA: cf75bd955fbc23c65c6b8b70e6c9e33d7441b166 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — clipboard native validation, full Bus/Prefs/session/input/search and release acceptance open; CBC job ids: term-seamless-e945bd9-a1 (green exact source), term-clipboard-327ab52-a1 (test compile failure), term-clipboard-order-cf75bd9 (next await).

## Clipboard native tests green; platform event driver checkpoint

Read complete saved log for term-clipboard-order-cf75bd9-a1: green,
137 terminal tests/five existing ignores, two bundle tests, five Mix
suites, command-search/dialog tests, foreign gate and debug build pass.
Returned fmt-only patch applied/pushed as ff3161f8. Fetched cf75bd9
debug executable verified SHA256
fbcdfd9e4949f4a6d8b86de4d4adc8c15cc65291dde7a491fb00c5ddec6130a8.

Native driver gap: key/text automation cannot generate Copy/Cut/Paste
events. Added bounded clipboard operation to the existing generic
viu.ui event queue (06f6f251), including modifiers and key-release/reset
follow-up. Driver tests added to term gate. Prepared private
platform-clipboard-06f6f251.mix (lint passes) for shell Ctrl-C, supplied
Unicode Shift-paste, plain Ctrl-V rejecting supplied paste text and
Shift-Cut closing the active split. No live clipboard witness yet.

Next CBC term-platform-clipboard-06f6f251 at
06f6f251e4d31d1184529a09f1ebe490b7278583, term_gate --prepare. Read full
log/apply fmt/repeat exact source as needed. Fetch matching executable
and bundle, update artifact path in prepared native harness and execute;
verify endpoint/event settlement and actual shell behavior, save all
replies/snapshots/cleanup. Then remaining clipboard primary/OSC52/mouse,
verified session/control/sealed shell integration, full Bus/IME/search,
zoom persistence/About/live Prefs and release acceptance. No Fable
question, install, VT change or merge.

branch: lab/parity-term; head SHA: 06f6f251e4d31d1184529a09f1ebe490b7278583 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — native platform clipboard witness, full Bus/Prefs/session/input/search and release acceptance open; CBC job ids: term-clipboard-order-cf75bd9-a1 (green, fmt applied), term-platform-clipboard-06f6f251 (next await).

## Platform driver host catalogue repair

Read complete term-platform-clipboard-06f6f251-a1 log: green. 137 terminal
tests/five existing ignores, seven input tests, two bundle tests, five Mix
suites, command-search/dialog/foreign/build pass. Applied returned fmt-only
patch in 7dd0022d. Matching fetched debug executable SHA256
3511432b2aa848cf0cffe656f663e5b37c79bd26c7533ae5998a0f643f4a7db7.

Ran prepared native platform-clipboard-06f6f251.mix; native.JxY16p fails
at first viu.ui.clipboard call with UNKNOWN_COMMAND. Generic host's
allowlisted command catalogue omitted the new operation. No clipboard
event was delivered, no PTY classification conclusion. All tracked PIDs
stopped. b2a7b8aa registers it in the existing host catalogue.

Next CBC term-clipboard-catalogue-b2a7b8aa at
b2a7b8aaf8ce3c97467af5febc8393d3febedc2a. Read full log/apply any patch,
repeat exact source if needed. Fetch artifact and matching bundle; derive
platform-clipboard-b2a7b8aa.mix from prepared script replacing artifact
path. Run Ctrl-C/Shift-paste/plain-paste/Shift-Cut witness; inspect native
replies, snapshots and cleanup. Continue all remaining parity work as
above. No Fable question, installs or VT changes.

branch: lab/parity-term; head SHA: b2a7b8aaf8ce3c97467af5febc8393d3febedc2a (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — platform clipboard runtime witness, full Bus/Prefs/session/input/search and release acceptance open; CBC job ids: term-platform-clipboard-06f6f251-a1 (green, fmt applied; runtime catalogue defect), term-clipboard-catalogue-b2a7b8aa (next await).

## Clipboard runtime witness passes; live cursor blink checkpoint

term-clipboard-catalogue-b2a7b8aa-a1 green, no worker diff; full log read.
Fetched exact matching debug executable/bundle, verified SHA256
b2ceef799e80f1069f85f7d8226750946530af9a2a950627efdc53ae0d7353d3.
Native.fjLLL5 passes Copy→Ctrl-C at real login Mix, Shift platform Paste
Unicode supplied text, plain platform Paste→Ctrl-V without supplied text,
Shift-Cut active split close/sibling retention, normal close and tracked
PIDs all stopped. Private platform-witness.json retains exact scope.

Native.R56uYX and native.TkKsDs failed harness assumptions: bracketed
paste waits for explicit Enter; Ctrl-V quotes the next Ctrl-C, requiring
a second Ctrl-C to clear. Snapshots show correct shell behaviour; only
the private harness changed. No application change or acceptance waiver.
Selection-copy readback/primary/OSC52/races remain open.

27c71218 adds reusable gold 530 ms cursor phase and live blink Prefs
wiring. Focus/typing restarts shown phase; no scheduled blink while
unfocused, disabled or program-hidden. Deterministic phase/no-wake tests
added. Next CBC term-cursor-blink-27c71218 at
27c71218f895f016917498fceb47cd391997ce78. Read full log/apply fmt and
repeat exact source if changed. Fetch matching artifact/bundle; native
live terminal.cursor.blink toggle on unchanged PID and capture cursor
phase/steady unfocused/typing reset; preserve matched captures with blink
off. Continue remaining primary/OSC52/mouse, verified session/control/
sealed Mix shell, full Bus/IME/search/zoom/About/all Prefs/release gate.
No Fable question, installs or VT changes.

branch: lab/parity-term; head SHA: 27c71218f895f016917498fceb47cd391997ce78 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — blink native validation, remaining clipboard/session/Bus/Prefs/input/search and release acceptance open; CBC job ids: term-clipboard-catalogue-b2a7b8aa-a1 (green exact source and scoped native witness), term-cursor-blink-27c71218 (next await).

## Blink mechanics pass; terminal-only Prefs follower repaired

term-cursor-blink-27c71218-a1 passed: 138 terminal tests/five ignores,
seven input tests, two bundle tests, five Mix suites, command-search/
dialog/foreign and host build. Complete saved log read (including the
separately re-read truncated middle); fmt-only patch applied in152a7dfe.
Matching artifact SHA256
2e4cd45271fb78dba0da5700723c6e02793503a00453e8ceeaea338bd9b048a6.

Native.yijAlb found terminal-only settings updates absent from model.
Shared follower subscribes to look/keymaps only. 21e53cc7 adds gold's
terminal/email/ced settings namespaces to subscriptions and native hints.
About provenance still pending; this live-Prefs defect took precedence.

Native.g5bXp4 passes actual two cursor pixel phases plus steady blink-off
on one PID, normal close, using explicit appearance co-changes to make
the old follower read. This is diagnostic, NOT evidence of subscription
repair. Native.SXo6yj failed PNG file-hash check because ImageMagick added
date metadata; raw RGBA hashes show exactly two pixel states. Corrected
private harness strips metadata. Failed runs/closures retained.

Next CBC term-terminal-prefs-21e53cc7 at
21e53cc7f8c6f4767d59a2f49901acd5144898b0. Read full log/apply any patch
and repeat exact source if changed. Fetch matching artifact and bundle;
derive new blink script from blink-27c71218.mix (original terminal-only
changes, now with -strip crops), update artifact path, and run. Do NOT
use appearance-diagnostic variant to attest terminal-only updates. Verify
blink on/off/model revisions/unchanged PID/cleanup, then other terminal
Prefs: font size/family/weight, cursor shape, padding, scrollback, TERM,
focus ring, copy-on-select and OSC52. Continue session/control/sealed
shell, full Bus/mouse/IME/search/zoom/About and release acceptance.
No Fable question, install, merge or VT work.

branch: lab/parity-term; head SHA: 21e53cc7f8c6f4767d59a2f49901acd5144898b0 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — terminal-only live Prefs validation, remaining clipboard/session/Bus/input/search and release acceptance open; CBC job ids: term-cursor-blink-27c71218-a1 (green, fmt applied), term-terminal-prefs-21e53cc7 (next await).

## Terminal-only blink Prefs pass; About provenance checkpoint

term-terminal-prefs-21e53cc7-a1 green, no worker diff. Full log read in
two chunks; matching artifact/bundle verified SHA256
d6c3a997fe7a811d8f063beca525ace271df15913add3dced2529abda95725d2.
Native.lrkN8M uses terminal-only blink changes: two cursor pixel phases,
then four steady identical whole PNGs, unchanged PID 3933235, model and
settings readback false, normal close and tracked PIDs stopped. Steady
PNG hash equals gold e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865.
This closes the bounded terminal-only settings follower defect.

43dcbf58 adds actual compiled host provenance to reusable runtime context
via own buildinfo helper/build.rs, replacing unrelated environment names
that yielded unknown. Term About now has gold's description/licence/live
settings-source lines and own Viu version/SHA/build time. Pure tests cover
line order/source labels; actual bundle test checks About host build
line and absent authority. Five Mix suites pass; Rust/native About pending.

Next CBC term-about-provenance-43dcbf58 at
43dcbf5820c98359827c1362bc48351812e8f54c, term_gate --prepare. Read full
log, apply Cargo.lock/fmt patch and exact-source repeat if changed. Fetch
matching artifact/bundle. Private gold-about.mix launched/prepared using
gold; get its saved run/closure then derive matched candidate About test.
Read own provenance and settings source, capture data-region differences
truthfully; no fabricated gold build values. Repeat authority loss/recovery
as needed. Continue all remaining clipboard/primary/OSC52/mouse, verified
session/control/sealed shell, full Bus/IME/search/zoom/all Prefs and release
acceptance. No Fable question, installs, merges or VT changes.

branch: lab/parity-term; head SHA: 43dcbf5820c98359827c1362bc48351812e8f54c (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — About native gate, remaining clipboard/session/Bus/Prefs/input/search and release acceptance open; CBC job ids: term-terminal-prefs-21e53cc7-a1 (green exact source and native terminal-only witness), term-about-provenance-43dcbf58 (next await).

## About native witness; exact-source repeat

Read complete term-about-provenance-43dcbf58-a1 log in two chunks: green.
Actual bundle About provenance tests pass. Applied returned Cargo.lock
dependency and test fmt patch as 8c5f6333, committed/pushed. Fetched
matching debug executable SHA256
f9952a67a1143ee5dbe1305e274336585d09f5326cdc619f6a2dfbb52c13944d.

Native.U7W85I About versus gold native.m4sBwh: layout/title/fixed body/
licence/settingsd connected/button exact. Only own Viu version/SHA/dirty
marker/build time differ; dirty correctly reflects worker prepare diff.
Originals, montage and diff inspected. about-43dcbf58-regions: header,
above provenance and below provenance distortion 0/0; full SSIM .0050312,
RMSE .0396607, entirely build data row [230,255,440,18]. Truthful runtime
data ruling applies. All tracked candidate PIDs stopped; gold cleanup
receipt retained. No final release acceptance claim.

Next exact-source CBC term-about-exact-8c5f6333 at
8c5f6333c002bb1db8ad4684457394c06e60ef25. Read full log, expect no patch;
if changed apply/repeat. Fetch artifact and bundle, repeat About provenance
readback/capture and source status after authority loss/recovery. Then
implement gold zoom persistence from apps/term/src/font.rs in Mix: 400ms
debounce, one fenced write/read, conflict fresh read, refusal restore,
failed writes event-driven retry, Ctrl+0 cancellation/in-flight correction.
The full gold source was read; existing pure FontSize extraction is only
size arithmetic, not that persistence machine. Keep app policy in Mix and
use native settingsd, existing Jobs/timer facilities. Continue primary/
OSC52/mouse, verified session/control/sealed shell, full Bus/IME/search/
all Prefs and release acceptance. No Fable question, installs or VT work.

branch: lab/parity-term; head SHA: 8c5f6333c002bb1db8ad4684457394c06e60ef25 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — exact About repeat, zoom persistence and remaining clipboard/session/Bus/Prefs/input/search/release acceptance open; CBC job ids: term-about-provenance-43dcbf58-a1 (green, lock/fmt applied), term-about-exact-8c5f6333 (next await).

## Exact About passed; zoom persistence wired in Mix

term-about-exact-8c5f6333-a1 green, no worker diff; complete log read.
Exact executable SHA256
f361ee4f3e477144fb03b814a9d5d8fe6344b1533b015435eb058d36b04891c1.
Native.3l2dVA About/close/cleanup passes. Inspected about-8c5f6333-exact:
all measured non-data regions 0/0; full .0046487 SSIM/.038541 RMSE,
only own version/SHA/time data differs. About authority-loss/recovery
and release still pending.

Implemented gold font.rs persistence machine as pure Mix, tested debounce,
reset/in-flight correction, conflicts, refused/failed writes, queued reads,
retarget and no timer retries after failure. Wired existing Jobs/timers
to serial native settings.get/settings.apply with revision/incarnation
fences. Configured size stays distinct from an unsaved local zoom; fresh
profile becomes the reset target. Added source hash/provenance.

Composite native.mnoDiz passes actual settings save and reset cancellation
(8c executable +78d bundle), tracked PIDs stopped. Earlier native.o6EiK2/
native.pLLrd4 failed when write reached missing embedded uuid(); trace
retained. Native process identity plus counters now name operations and
clipboard-read tokens. Final native font-size bridge uses gold f32
FontSize; composite did not cover those final bytes. Six Mix suites pass.

Next CBC term-zoom-persist-f0d36085 at
f0d3608585f7791e4b3f536a6a6c78237466f26d, term_gate --prepare. Read full
log, apply patch and exact repeat as needed. Fetch matching executable/
bundle; derive exact zoom harness from zoom-78d62995-diagnostic.mix,
updating artifact path. Verify settings size equals measured native f32,
400ms latest zoom, reset cancellation/correction, conflict fresh fence,
authority absence/return and failure no-poll retries. Continue remaining
primary/OSC52/mouse, verified session/control/sealed Mix shell handoff,
full Bus/IME/search/all Prefs and release acceptance. No Fable question,
installs, merges or VT changes.

branch: lab/parity-term; head SHA: f0d3608585f7791e4b3f536a6a6c78237466f26d (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — zoom exact/native gate and remaining clipboard/session/Bus/Prefs/input/search/release acceptance open; CBC job ids: term-about-exact-8c5f6333-a1 (green exact source/native About), term-zoom-persist-f0d36085 (next await).

### Zoom recovery and new-session Prefs checkpoint

`term-zoom-persist-f0d36085-a1` passed: 139 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog
checks, foreign gate and debug host build. Complete saved log read; the
fmt-only patch was applied in 9ba8cd5b. Fetched debug executable SHA256
`63e08b125d12ac9abfca9014848d14a6816739ed22f752650455ecbab4c58632`.
Native `native.8KmDfR` verifies persisted size exactly equals observed
native f32 `15.333666801452637`, reset cancels the next unwritten zoom,
and all tracked processes stop. Its initial same-command session PNG is
byte-identical to gold (e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865).
These are debug witnesses, not final release acceptance.

Authority-return comparison exposed an app defect: a missing settingsd
read returns rc=10/body=null, and the unguarded field access left the
persistence machine reading forever. Gold `native.4KfmwO` recovers;
candidate `native.78tJtT` and traced `native.YaCgRO` retain the failure
and all-stopped receipts. e1278485 normalises failed/empty/malformed job
replies before applying the persistence event; regression cases cover
null bodies and job failures. The pure machine's no-timer-retry rule is
unchanged. An earlier outer harness timeout (`native.9qAtHu`) required
explicit owned-process cleanup, recorded separately; no live service was
touched. The harness now runs directly without the 30-second wrapper.

Gold config changes affect TERM/history capacity only on new shells.
The port now forwards `terminal.env.term` and the correct
`terminal.scrollback.lines` key for tab and split creation. Dispatch
tests verify profile arguments and preservation of existing sessions.
Composite diagnostic `native.dWebDK` (f0d36085 executable plus scoped
e1278485 Mix runtime changes) passes settings authority loss/return,
retained local zoom saved as `16.000333786010742` on unchanged app PID,
new shell `TERM=vt100`, history readback exactly 17 lines, and cleanup.
`native.nSfFg8` retains the earlier wrong harness history-field assertion;
its snapshot already returned 21 screen +17 history lines. No app change
was made to satisfy that assertion. Six Mix suites pass after repairs.

Next exact-source gate: `term-zoom-recovery-e1278485` at
`e1278485b79544767f9b9416630c4a80968f0eca`, including applied worker fmt.
Fetch the matching executable/bundle and repeat the native witness before
claiming this checkpoint exact. Native conflict/refusal/in-flight reset,
verified session issuer/control and sealed Mix shell integration, full
Bus/clipboard/primary/OSC52/mouse/IME/Prefs, and final release captures
remain open. Overall **NOT ACCEPTED**. No deviation or Fable question.
On resume: read the full saved CBC log, apply any TERM_PATCH and repeat
exact source if it changes. Fetch matching debug artifact from the actual
worker and verify SHA256/source. Derive the next native witness from
zoom-recovery-fixed-diagnostic.mix using the new exact artifact path;
its profile/history assertions and owned cleanup are corrected. Replace
the diagnostic receipt with actual source/profile data. Then continue
the full inventory; do not rerun old failed builds or install gold.

branch: lab/parity-term; head SHA: e1278485b79544767f9b9416630c4a80968f0eca (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — exact zoom-recovery/profile gate, remaining session/Bus/input/Prefs and release acceptance open; CBC job ids: term-zoom-persist-f0d36085-a1 (green, fmt applied), term-zoom-recovery-e1278485 (next await).
### Exact zoom recovery passes; keyboard ownership checkpoint

`term-zoom-recovery-e1278485-a1` passed on CBC2 with no worker diff;
complete saved log read in two chunks. Six Mix suites, 139 terminal tests
(five existing ignores), seven input tests, two bundle tests, search,
dialog, foreign gate and debug host build pass. Matching executable and
bundle fetched and receipt verified: source
`e1278485b79544767f9b9416630c4a80968f0eca`, SHA256
`5ef0e856a13afc37e3c793959cfc47573cf7f2e3473d17d7f5fab90e184affc5`.
Exact debug witness `native.MZX2kE` passes f32 save/reset, settingsd
loss/return preserving local zoom on the same app PID, new shell TERM
vt100, 17 history lines, and all tracked PIDs stopped. Initial real-Mix
session PNG remains gold's identical e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865.
This closes the bounded recovery/new-session diagnostic at exact source;
it is not final release acceptance or full font-race coverage.

65fafcd5 ports gold keyboard ownership rules. Mix supplies repeat and
normal-screen-only metadata for its bindings. Native widget consumes held
non-repeating commands without forwarding control bytes; zoom and scroll
commands may repeat. Alternate-screen programs receive scrollback chord
keys through the existing VT encoder. Plain Tab/Shift+Tab cycles split
panes without repeat, while single-pane Tab remains shell input. Existing
live binding overrides stay in Mix. Added Mix policy checks and native
widget/encoder regressions; all six local Mix suites pass. Rust and live
keyboard coverage are pending the next gate.

Next CBC `term-keyboard-65fafcd5` at
`65fafcd5c50134fd3cddef13ad4452c64c1e5493`, term_gate --prepare.
Read full log, apply returned fmt patch and repeat exact source if changed.
Fetch matching executable/bundle and test native split Tab/Shift+Tab,
held command consumption, zoom repeats and alternate-screen scroll keys.
Native driver key currently always emits repeat=false; extend the generic
driver if a native held-event witness needs it, rather than claiming a
repeated direct execute proves keyboard handling. Gold input.rs Action
repeats includes font reset, and app.rs intercepts plain split Tab before
shell encoding. Continue the full remaining verified session issuer/
control/sealed login-shell handoff, Bus/clipboard/primary/OSC52/mouse/IME,
all Prefs, conformance and exact release visual acceptance. Overall
**NOT ACCEPTED**. No Fable deviation, installs, merges or VT changes.
branch: lab/parity-term; head SHA: 65fafcd5c50134fd3cddef13ad4452c64c1e5493 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — keyboard native gate and remaining session/Bus/input/Prefs/release acceptance open; CBC job ids: term-zoom-recovery-e1278485-a1 (green exact source/native recovery and new-session Prefs), term-keyboard-65fafcd5 (next await).
### Native Tab exposes missing terminal focus lock

`term-keyboard-65fafcd5-a1` passed: six Mix suites, 141 terminal tests,
five existing ignores, seven input tests, two bundle tests, search/dialog,
foreign gate and debug build. Complete saved log read in three chunks.
Worker formatting applied to view.rs. Matching debug artifact verified
SHA256 `3c188251a1d40248ce847e88f3cae1cfbbafa8ea9083bfbb0a9d340a90e995a1`.
The new native tests cover held command consumption and alternate-screen
scroll key routing; they do not prove toolkit Tab delivery.

Native `native.3A9c9v` fails plain Tab after splitting: toolkit navigation
takes focus before the terminal widget sees the event. All tracked PIDs
stopped; failed script/replies retained. Gold shell.rs lines 768 onward
installs EventFilter with Tab, horizontal/vertical arrows and Escape all
locked. The reusable widget now does the same after requesting focus,
matching gold's order. Added a two-frame regression with another focusable
control; it must still deliver Tab to the terminal command binding.

Code `bee0187865841e8c35d8afd7d9f586e8c43b427b` includes the applied fmt
and focus repair. Next gate `term-focus-bee01878`, term_gate --prepare.
Read full log, apply any fmt and repeat exact source if changed. Fetch
matching artifact/bundle and derive the repaired native witness using
`keyboard-65fafcd5.mix` (replace artifact path and receipt source).
Expected active pane 2 after split, 1 after Tab, 2 after Shift+Tab. Do not
claim held native repeat coverage from this driver: term.ui.key currently
emits repeat=false. Native repeat and full alternate-screen session tests,
verified session/control/sealed login-shell integration, full Bus, mouse,
primary/OSC52/clipboard/IME/all Prefs, conformance and release acceptance
remain. Overall **NOT ACCEPTED**. No Fable question or desk changes.
branch: lab/parity-term; head SHA: bee0187865841e8c35d8afd7d9f586e8c43b427b (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — native terminal focus repair gate and remaining full parity/release acceptance open; CBC job ids: term-keyboard-65fafcd5-a1 (green with fmt; native Tab failed), term-focus-bee01878 (next await).
### Focus regression setup corrected

`term-focus-bee01878-a1` failed the new Tab regression; 141 terminal
tests passed, five existing ignores, six Mix suites and two bundle tests
passed before it. Complete saved log read; fmt-only test patch applied.
No artifact was produced after that failure. Production focus-filter code
compiled, but the prior native failure remains open.

Source inspection of egui 0.37 memory::set_focus_lock_filter confirms it
requires both had_focus_last_frame(id) and has_focus(id). The two-frame
test requested initial focus and immediately sent Tab, before egui could
install the filter. bd978a23 adds a focused setup frame, then sends Tab
and retains the same command-delivery assertion with another focusable
control. The old widget without the filter still cannot satisfy it.
Production code is unchanged. This is fixture setup, not a parity waiver.

Next gate `term-focus-ready-bd978a23` at
`bd978a239b87164b81d34860c0c5d533d8a1da4c`, term_gate --prepare. Read the
complete log; apply any fmt and repeat exact source if changed. Fetch
matching artifact and repeat keyboard-65fafcd5.mix with its artifact path
updated. Native Tab/Shift+Tab must pass before closing the focus defect.
All remaining full-parity work listed above remains open; NOT ACCEPTED.
branch: lab/parity-term; head SHA: bd978a239b87164b81d34860c0c5d533d8a1da4c (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — focus regression/native validation and remaining full parity/release acceptance open; CBC job ids: term-focus-bee01878-a1 (regression fixture failed), term-focus-ready-bd978a23 (next await).
### Exact native Tab passes; Ctrl+wheel checkpoint

`term-focus-ready-bd978a23-a1` passed with no worker diff: 142 terminal
tests, five existing ignores, seven input tests, two bundle tests, six
Mix suites, search/dialog, foreign gate and debug build. Complete saved
log read in two chunks. Matching executable/source receipt verified:
`bd978a239b87164b81d34860c0c5d533d8a1da4c`, SHA256
`e91f8546a4e3c7f201e2e3f9be67c0802dadc34cf8d2f802422ab8b62c05a54d`.
Native `native.c6qNxS` passes real split Tab/Shift+Tab: focused pane
2 → 1 → 2, with unchanged grid metrics; all tracked PIDs stopped.
Initial command-output PNG remains byte-identical to gold e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865.
This closes the bounded native focus defect, not final release acceptance.

Gold app.rs wheel/modifiers_changed routes Ctrl+wheel to zoom actions,
retains fractional travel across panes, clears it when Ctrl is released,
and resets scroll travel when modifiers or target pane change. The new
reusable Wheels state follows those rules; ordinary wheel still uses VT
mouse reporting or local scroll. Mix supplies increase/decrease command
IDs through terminal properties, keeping zoom/persistence policy in Mix.
Added gesture-state and widget-to-command regressions. Six Mix suites
pass. New native wheel tests and live wheel evidence remain pending.

Next gate `term-wheel-d6b0c7bc` at
`d6b0c7bc34db17d4614ddded8d8f1c66bde2f404`, term_gate --prepare. Read
the complete log; apply fmt and repeat exact source if changed. Fetch and
verify matching artifact/bundle. Derive live wheel witness from the zoom
harness: native viu.ui.scroll at x=100,y=150,delta=40 with ctrl modifier
is one point-unit zoom step (driver uses Point and Unknown source).
Verify saved f32 and raster size, partial/reversed travel, Ctrl release,
plain wheel and hovered inactive pane, then matched gold zoom captures.
Held-key native repeat, full alternate-screen sessions, session issuer/
control/sealed Mix handoff, full Bus/mouse/primary/OSC52/IME/Prefs and
release acceptance remain. Overall NOT ACCEPTED. No Fable question.
branch: lab/parity-term; head SHA: d6b0c7bc34db17d4614ddded8d8f1c66bde2f404 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — wheel native gate and remaining full parity/release acceptance open; CBC job ids: term-focus-ready-bd978a23-a1 (green exact source/native Tab), term-wheel-d6b0c7bc (next await).
### Wheel owner compile repair

`term-wheel-d6b0c7bc-a1` failed compilation before native tests/artifact
build; all six Mix suites passed. Complete saved log read. The wheel
state field was accidentally inserted into shortcut() parameters instead
of Views, producing the missing-field/missing-argument errors. 60212c48
moves it to Views and applies the worker's formatting patch. Gesture and
widget regression assertions are unchanged. No native wheel evidence yet.

Next gate `term-wheel-owner-60212c48` at
`60212c48ad022d4610838c44b697d4117d63aeb9`, term_gate --prepare. Read full
log, apply any fmt and repeat exact source if changed. Fetch and verify
artifact/source, then replace WHEEL_ARTIFACT_PENDING in the private
wheel-prepared.mix (lint passed). It derives from the exact-font-size
zoom witness, replacing only the first zoom with native viu.ui.scroll:
x100/y150/delta40/modifiers ctrl. Verify one real wheel step persists the
native measured f32, reset cancellation and owned cleanup. Prepared is
not executed evidence. Continue broader wheel/native input, all remaining
session/Bus/clipboard/IME/Prefs and exact release acceptance. NOT ACCEPTED.
branch: lab/parity-term; head SHA: 60212c48ad022d4610838c44b697d4117d63aeb9 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — wheel owner native gate and remaining full parity/release acceptance open; CBC job ids: term-wheel-d6b0c7bc-a1 (compile failure repaired), term-wheel-owner-60212c48 (next await).
### Exact Ctrl+wheel witness; lease cleanup ordering checkpoint

`term-wheel-owner-60212c48-a1` passed with no worker diff. Complete log
read in two chunks: 144 terminal tests, five existing ignores, seven input
tests, two bundle tests, six Mix suites, search/dialog, foreign gate and
debug build. Matching artifact/source verified:
`60212c48ad022d4610838c44b697d4117d63aeb9`, SHA256
`1a2e024fa6131bce50991180a13913e7250065e025e3f910048a84e09fd9c4c4`.
Native `native.1qSr40` passes actual Ctrl+wheel, exact persisted/rendered
f32 15.333666801452637, reset cancellation, normal close and all tracked
PIDs stopped. The driver contract is viu.ui.scroll with ctrl:true, not
the modifier list accepted by term.ui.key. Earlier native.JkQJgC and
native.80Cjsw passed the wrong argument shape and therefore delivered
plain scroll; they are retained harness failures with clean shutdown.
No widget change was made for those failures. Broader gesture and matched
zoom capture coverage remain pending; this is debug evidence only.

785b342e fixes the native session callback seam to invalidate local pane
authority before the cleanup callback can wait or release the PTY. Gold
term-core/native_session.rs invalidates live before requesting remote
Close, and Drop repeats idempotent revocation. The previous seam called
cleanup before Drop's revocation. Added tests asserting cleanup sees
revoked authority and failed launch reports only after revocation.
This repairs ordering in the reusable seam; it does NOT implement the
missing verified issuer/supervisor or grant/control attachment.

Next gate `term-lease-order-785b342e` at
`785b342eff93919bfae7aff3513791bc8aeaa8e1`, term_gate --prepare. Read full
log, apply any fmt and repeat exact source if changed. Continue the full
native session work: pinned term-core/native_session.rs contains the
verified identity actor, grant renewal/reconnect and prepared sealed-fd
handoff, but its Control is coupled to Rust TabSet. Port the actor as a
reusable native facility while adapting protected policy/layout requests
to existing in-process Mix ownership; do not import Rust app tab policy.
Current Host still supplies no issuer. Full session/Bus/clipboard/mouse/
IME/Prefs and release captures remain open. Overall NOT ACCEPTED.
branch: lab/parity-term; head SHA: 785b342eff93919bfae7aff3513791bc8aeaa8e1 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — lease ordering gate, verified issuer/control and remaining full parity/release acceptance open; CBC job ids: term-wheel-owner-60212c48-a1 (green exact source/native Ctrl+wheel), term-lease-order-785b342e (next await).
### Verified session actor extraction checkpoint

`term-lease-order-785b342e-a1` passed: 146 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog,
foreign gate and debug build. Complete saved log read in two chunks;
worker test-format patch applied in the actor checkpoint. The ordering
tests confirm local invalidation before cleanup and failed-launch report.

652c704e extracts pinned term-core/native_session.rs production actor
into reusable terminal/session_actor.rs. Its native Bus discovery, verified
attachment, grant pool, conservative deadlines, renewal, lifecycle gap/
reconnect reconciliation, bounded dispatch queue and revocation logic are
retained. Rust TabSet-dependent Control is replaced by typed ControlLane
futures; attachment/VerifiedCommand stay native values, never JSON-minted
authority. The protected command manifest is copied from pinned control.rs.
The actor implements existing SessionIssuer, retaining descriptor, issuing
endpoint and ordered callbacks. Added existing test-broker as a dev-only
workspace dependency, not a second transport or production service.
Provenance includes source hash and adaptations.

Five initial tests cover diagnostic transitions, real-broker issuing
endpoint, expiry launch margin, nonblocking prepared consumption and
typed lease-bridge invalidation. Native compile/tests are pending. This
does not port the full original enforcement/bootstrap/reconnect test
suite yet. Host still supplies no issuer; ControlLane has no app adapter.
There is no claim of working protected commands or sealed Mix login
integration in the candidate at this checkpoint.

Next CBC `term-session-actor-652c704e` at
`652c704ef4a57b73a7c714cdc4a1edb392fbe2b6`, term_gate --prepare. Read
complete log; apply fmt/Cargo.lock and repeat exact source if changed.
Then implement host supervisor lifetime/startup wait and protected policy
adapter through existing in-process Mix mailbox. Preserve exact native
pane IDs/generations across failed creations, map native handles to Mix
layout explicitly, and keep Rust free of app tab/pane policy. Adapt the
remaining pinned actor/enforcement/Mix bootstrap tests and drive real
sessions through native Bus. All other inventory and exact release visual
acceptance remain open. NOT ACCEPTED; no Fable deviation requested.
branch: lab/parity-term; head SHA: 652c704ef4a57b73a7c714cdc4a1edb392fbe2b6 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — session actor gate/host/protected adapter, remaining full parity and release acceptance open; CBC job ids: term-lease-order-785b342e-a1 (green, fmt applied), term-session-actor-652c704e (next await).
### Lazy host session ownership checkpoint

`term-session-actor-652c704e-a1` passed: 151 terminal tests including all
five new actor/broker cases, five existing ignores, seven input tests,
two bundle tests, six Mix suites, search/dialog, foreign gate and debug
build. Complete saved log read in three chunks. Applied worker formatting
and Cargo.lock's own test-broker dependency in the next code checkpoint.

4a76a9b6 connects the generic host's terminal collection to one lazily
started Supervisor. First terminal creation runs its one bounded startup
wait on the behaviour thread; shared ownership retains the actor through
PTY cleanup. Non-terminal applications do not start an actor. Queryable
term.session diagnostics are registered in both the Mix handler and host
contract; new-tab replies use actual granted/graphics-only/unavailable
binding status. Physical key/pointer/wheel paths refresh presence; direct
diagnostic type does not. Application layout remains entirely Mix-owned.

Mix supplies pane_id explicitly, independent of native vt-N handle
allocation. Native state reserves/rejects duplicate IDs through launch,
releases reservations on failure/close, and exposes native_id/binding for
the future protected adapter. Added real installed-Mix sealed-handoff/
enrolment/close and independent pane-ID tests; these are pending CBC.
They do not replace the original exact-current-shell enforcement suite.
All six local Mix suites pass, including session diagnostic dispatch.

Next gate `term-session-owner-4a76a9b6` at
`4a76a9b6ca4e03adb527f4aa6007c25cb4e7f5a1`, term_gate --prepare. Read full
log, apply fmt/lock and repeat exact source if changed. Fetch exact bundle
and executable for isolated real host term.session/enrolment/tab/split/
exit/cleanup checks and unchanged matched output. The actor now has a
host lifetime, but ControlLane is not installed: protected requests still
refuse uniformly. Next implement typed protected admission plus in-process
Mix policy dispatch, adapt remaining gold control/bootstrap/reconnect
tests, then complete all Bus/input/clipboard/IME/Prefs/release acceptance.
Overall NOT ACCEPTED. No Fable deviation or desktop changes.
branch: lab/parity-term; head SHA: 4a76a9b6ca4e03adb527f4aa6007c25cb4e7f5a1 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — lazy owner/native shell gate and protected adapter, remaining full parity/release acceptance open; CBC job ids: term-session-actor-652c704e-a1 (green, fmt/lock applied), term-session-owner-4a76a9b6 (next await).
### Session-owner fixture isolation repair

`term-session-owner-4a76a9b6-a1` failed its new real-Mix enrolment test:
152 terminal tests passed, one failed and five were ignored. Six Mix
suites and both bundle tests passed before the failure. The complete saved
CBC log was read; its worker formatting patch is applied in 5e4e7344.

The failed fixture configured the parent actor with the embedded broker's
worker-owned account, but launched Mix with inherited installed broker
configuration and home. Native client resolution defaults to the noded
account; a delivered descriptor does not override trusted account lookup.
Pinned gold's production e2e fixture explicitly isolates that configuration.

5e4e7344 runs the owner test in a fresh subprocess with an isolated home,
node configuration, explicit kernel-derived fixture broker account and
private data/runtime/config/cache roots. It now exercises normal
Supervisor::start discovery as well as normal Sessions::call creation;
there is no injected supervisor or substitute PTY launcher. The child must
both enrol through the sealed handoff and show its real Mix login prompt.
Close still must invalidate local authority immediately and reap the child.
No assertion is weakened. A failed enrolment now includes the PTY snapshot.
This configuration diagnosis awaits the repeat; no native pass is claimed.

Next CBC `term-owner-isolated-5e4e7344`, exact code
`5e4e7344c0472b2594aaa18eb5ac8c5c2bae07d7`, term_gate --prepare.
Read the full saved log, apply any worker diff and repeat exact source.
Then fetch and verify the artifact for isolated host/session evidence.
Protected ControlLane, remaining gold enforcement tests and the complete
Bus/input/clipboard/IME/Prefs/release acceptance remain open. NOT ACCEPTED.
No Fable deviation proposed; no gold or desktop changes.
branch: lab/parity-term; head SHA: 5e4e7344c0472b2594aaa18eb5ac8c5c2bae07d7 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — isolated session-owner gate, protected adapter and remaining full parity/release acceptance open; CBC job ids: term-session-owner-4a76a9b6-a1 (failed; fixture isolation repaired), term-owner-isolated-5e4e7344 (next await).

### Verified host session witness and real-exit regression

Exact code 5e4e7344c0472b2594aaa18eb5ac8c5c2bae07d7 passed CBC
`term-owner-isolated-5e4e7344-a1`: 153 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog
checks, foreign gate and debug build. Complete saved log read in three
chunks; TERM_PATCH empty. Debug executable SHA256:
`b38ca9c84ad947e14429a77a00dd9163f06e3e52f8567130c152568f5634cbb5`.
Source/profile/byte hash verified against the fetched receipt before use.

Private run native.bu3oPA uses the exact bundle/executable and native
headless compd/settingsd/broker with the real installed Mix login shell.
First pane, split pane and new-tab pane each reached child binding
Attached. New tab replied binding=granted. Plain Tab/Shift+Tab kept split
focus ownership. Real shell exit removed pane 3 and its tab, returning to
the original two-pane tab. Normal window close stopped all seven tracked
processes; closure.json records all dead. Parent shutdown logged an
unconfirmed revoke, retaining gold's bounded expiry fallback; complete
remote revocation/expiry acceptance remains open.

Same-command initial capture is byte-identical to the retained gold
native.4KfmwO capture: both PNG SHA256
`e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865`.
Matched 900x560 client, scale 1, Studio Dark, Inter 400/14.667 and
terminal 400/14.667. parity_compare generated compare-session with full,
header [0,0,900,38], tabs [0,38,900,32] and terminal [0,70,900,490].
ImageMagick 7.1.2-35 SSIM distortion and RMSE are zero in every region.
Candidate, side-by-side and difference inspected: no region differences.
This debug witness does not replace fresh release acceptance captures.

Failed runs retained: native.sdIVrK stopped on a harness type builtin
typo; native.khklo4 exposed the harness's 0700 capture-directory ancestor
disabling native Unix ingress. Both cleanup receipts show all tracked
processes dead. Corrected harness uses a separate owned traversable
ingress sibling; capture/home directories retain their permissions.
Earlier TCP-only visual/input evidence remains valid for those cases,
but cannot attest native sealed-session enrolment.

b8cd6d1f extends the owner regression to isolated close and natural-exit
subprocesses. Both must enrol/show the real prompt. A fresh verified
observer must read the child's Revoked state from the actual broker.
Natural exit must invalidate the still-owned Terminal before any close
or drop, preventing explicit cleanup from making the proof vacuous.
Next CBC `term-owner-exit-b8cd6d1f` at
`b8cd6d1fe3728a1f86f57515212e1490ea2667e2`, term_gate --prepare.
Read complete log; apply formatting/lock and repeat exact source if
changed. Protected ControlLane adapter, original enforcement/bootstrap/
reconnect suite, complete Bus/input/clipboard/IME/search/Prefs and exact
release acceptance remain open. NOT ACCEPTED. No Fable deviation or
desktop/gold changes.
branch: lab/parity-term; head SHA: b8cd6d1fe3728a1f86f57515212e1490ea2667e2 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — broker close/exit regression pending, protected adapter and remaining full parity/release acceptance open; CBC job ids: term-owner-isolated-5e4e7344-a1 (green exact, native witness passes), term-owner-exit-b8cd6d1f (next await).

### Broker revoke regression passes; typed protected protocol checkpoint

`term-owner-exit-b8cd6d1f-a1` passed: 153 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog,
foreign gate and debug build. Complete saved log read in three chunks.
Worker formatting applied in 68631fa7. The owner regression now proves
Revoked through a fresh verified broker observer for both close and real
Mix exit. For natural exit that observation happens before Terminal close
or drop, while its retained handle reports exited and local authority is
already invalid. This closes that regression case; it does not attest all
parent-shutdown/broker-bounce/expiry cases.

68631fa77fd858888be07bac85369ea4413de3d0 extracts the shared typed control
protocol from pinned gold control.rs (source SHA256
565a82602ad1a21262bf69ab1d5e4f09f2f6a50ebc704c7b613cda458a642d4b).
Target/request types preserve canonical decimal u64 values in Rust rather
than converting authority-bearing identifiers through Mix numbers. The
actor now uses the shared command manifest and reply schema. Closed error
vocabulary, child-refusal recovery details, uniform denials and the read/
mutation unknown-response distinction retain gold's behaviour. Strict
8192-byte decoding, property alias normalization and exact raw-byte retry
digest are reusable helpers for the adapter. No TabSet policy was imported.

Retained the two original vocabulary tests; added four regression tests
for u64 boundaries/noncanonical encodings, JSON authority-field refusal,
UTF-8 byte budgets and byte-distinct retries, property aliases and refusal
detail confidentiality/recovery. These await native compilation. This is
the protocol foundation, not an installed protected ControlLane: commands
still refuse uniformly on that lane. Admission, liveness checks, replay,
operation retention, child execute/task forwarding and in-process Mix
layout policy dispatch remain the next implementation pieces.

Next CBC `term-control-protocol-68631fa7`, exact code 68631fa7 above,
term_gate --prepare. Read full saved log, apply returned fmt/lock and
repeat exact source as needed. Keep the native.bu3oPA debug session witness
and zero whole/header/tabs/terminal distortions as prior evidence; final
release and complete Bus/input/clipboard/IME/search/Prefs inventory remain
open. NOT ACCEPTED. No Fable deviation or gold/desktop changes.
branch: lab/parity-term; head SHA: 68631fa77fd858888be07bac85369ea4413de3d0 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — typed protocol gate and protected adapter, remaining full parity/release acceptance open; CBC job ids: term-owner-exit-b8cd6d1f-a1 (green; fmt applied), term-control-protocol-68631fa7 (next await).

### Typed recipient admission checkpoint

`term-control-protocol-68631fa7-a1` passed: 159 terminal tests (all six
protocol cases), five existing ignores, seven input tests, two bundle
tests, six Mix suites, search/dialog, foreign gate and debug build. Full
52403-byte saved log read in four chunks. Worker formatting applied in
18dd4d9c; this changed source still needs its own exact gate.

18dd4d9cb3d9c84516037fda700225832a8298b5 adds typed recipient admission
extracted from gold dispatch. It accepts native VerifiedCommand and
VerifiedConnection plus the actor's current parent/Hello/Deadline.
Connection loss, broker-origin stamps, strict decoding/property aliases,
cross-incarnation outcome handling, target matching, supported capability
and recipient lease binding/liveness retain gold's check order. No request
body can supply these native values. No admission path performs a fresh
lease RPC. Replay namespace, caller epoch, label and digest derive from
the broker-stamped event. Mutation-epoch binding remains a separate check
to preserve gold's later ordering after replay and affected-pane checks.

Added a real embedded-broker/actor ControlLane probe test for an unbound
verified caller: valid state/preflight and one-shot mutation epoch, stale
epoch, wrong-instance denial versus unknown outcome, forged body metadata,
and a recipient binding generation inconsistent with the actual lease.
The probe performs no PTY writes or layout mutation. The test is pending
CBC, not a protected-command acceptance claim. Production ControlLane is
still uninstalled; replay/operation retention, pane/write commitment,
child execute/task forwarding and in-process Mix application dispatch
remain open. Complete input/clipboard/IME/search/Prefs/Bus and release
acceptance are also open. NOT ACCEPTED. No Fable deviation or gold changes.

Next CBC `term-control-admission-18dd4d9c`, exact code above,
term_gate --prepare. Read complete log, apply fmt/lock and repeat exact
source if changed; continue through replay/commitment and host adapter.
branch: lab/parity-term; head SHA: 18dd4d9cb3d9c84516037fda700225832a8298b5 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — typed admission gate, protected handler/replay/Mix adapter and remaining full parity/release acceptance open; CBC job ids: term-control-protocol-68631fa7-a1 (green; fmt applied), term-control-admission-18dd4d9c (next await).

### Admission probe passes; retained-operation store checkpoint

`term-control-admission-18dd4d9c-a1` passed: 160 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog,
foreign gate and debug build. The actual broker/actor admission probe passed,
including one-shot grantless epoch binding and the stale-recipient lease
target refusal. Full 49246-byte saved log read in four chunks; worker
formatting applied in a778efc5. No native artifact from that source was used
as new visual or release evidence.

a778efc597a39a5093bc3bf00dd97dc05ecd4cff ports gold's retained-operation
state into a reusable exclusive-mutation store. Entry/History, reserve,
forwarded ID allocation and outcome recording preserve gold's algorithms.
Lookup/operation query are extracted from gold dispatch. Retention stays
900 seconds, 1024 entries per caller, 4096 instance entries, 256 caller
namespaces. Expiry retires outcomes but keeps request high-water marks;
lost answers never make reserved IDs executable again. Capacity evicts
cold caller namespaces as gold permits, and forward capacity evicts one
mapping. Child operation IDs are remapped to instance operation IDs while
retaining the child link. Settled notice outcomes remain separate from the
original replay acknowledgement. The handler must retain gold's serialized
admission/replay/commit ordering; mutable store access is not itself a
recipient/pane/write permit.

Six pending regressions cover exact retry/conflict and a lost committed
outcome at u64::MAX; target/generation-scoped operation retention and expiry;
child operation remapping/notice separation; per-caller and instance caps;
cold namespace eviction after churn; distinct forwarded IDs for callers
sharing request ID 1, stable retries, conflicts and one-at-a-time mapping
eviction. No production handler is installed yet. Next connect admission,
retained outcomes, pane guards/write permits and affected-layout validation,
then implement child execute/task forwarding and in-process Mix dispatch.

Next CBC `term-control-replay-a778efc5`, exact code above,
term_gate --prepare. Read full saved log; apply fmt/lock and repeat exact
source if changed. Remaining complete Bus/input/clipboard/IME/search/Prefs,
full gold enforcement suite and exact release acceptance remain open.
NOT ACCEPTED. No Fable deviation or gold/desktop changes.
branch: lab/parity-term; head SHA: a778efc597a39a5093bc3bf00dd97dc05ecd4cff (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — replay gate, production protected handler/Mix adapter and remaining full parity/release acceptance open; CBC job ids: term-control-admission-18dd4d9c-a1 (green; fmt applied), term-control-replay-a778efc5 (next await).

### Replay gate passes; first protected native recipient installed

`term-control-replay-a778efc5-a1` passed: 166 terminal tests including all
six replay cases, five existing ignores, seven input tests, two bundle
tests, six Mix suites, search/dialog, foreign gate and debug build. Full
62938-byte saved log read in five chunks. Worker formatting applied in
7474c477. No new capture/release claim is made from that artifact.

7474c4777cbf8cd8eb5303d7b9b33fa290ae7cae installs the first native
protected recipient with the lazy host supervisor. It implements typed
snapshot/contents, input and retained-operation queries. Admission and
retained replies precede pane liveness; new input rechecks the recipient
lease, native pane generation and foreground generation, reserves its ID
before execution, and uses the existing final PTY write permit. The handler
retains no tabs/layout policy. Pane lookup holds weak session state, avoiding
a handler/native-owner ownership cycle. It releases retained-state locks
before waiting on a pane so close's bounded actor cleanup cannot wait on
the reverse lock order. Dispatch remains single-flight.

Input revocation retains gold's bounded notice queue (256), 16 notices per
turn, 200ms total and 100ms send budgets; private delivery is stamped with
the original broker epoch/caller connection. PTY byte counts settle the
retained outcome separately from the original acknowledgement. Direct
mutations now allocate instance operation IDs through the retained store.

Expanded the existing isolated real-Mix close/exit regression to call the
actual protected service before teardown: metadata snapshot, generation-
bound input executing an assembled marker in the real shell, exact retry,
conflicting retry, stale pane and foreground refusals, then retained PTY
byte-count completion and unchanged replay acknowledgement. These new
cases await CBC compilation/runtime and are not claimed as passed yet.

This is a partial handler: protected session/list metadata, layout operations
through the in-process Mix adapter and child execute/task forwarding remain
open, as do full original enforcement/reconnect tests, all remaining app
inventory and exact release acceptance. No Fable deviation is proposed.
NOT ACCEPTED; no gold/desktop changes.

Next CBC `term-protected-input-7474c477`, exact code above,
term_gate --prepare. Read complete log; fix failures, apply formatting/lock
and repeat exact source. Fetch verified artifact for isolated protected
input/snapshot/notice and unchanged matched-session captures, then continue
the Mix policy adapter and child forwarding.
branch: lab/parity-term; head SHA: 7474c4777cbf8cd8eb5303d7b9b33fa290ae7cae (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — protected input gate, Mix layout/session adapter and child forwarding, remaining full parity/release acceptance open; CBC job ids: term-control-replay-a778efc5-a1 (green; fmt applied), term-protected-input-7474c477 (next await).

### Protected input verified; child forwarding checkpoint

`term-protected-input-7474c477-a1` passed: 166 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog,
foreign gate and debug build. Full 62238-byte saved log read in three
chunks. Formatting-only worker patch applied in dd1a5ac5. The installed
recipient's real-Mix regression passed in both isolated close/exit modes:
protected snapshot, real PTY input execution, identical retry, conflict,
stale native pane/foreground refusal, retained PTY completion with exact
written byte count and replay of the original acknowledgement.

Fetched the worker debug artifact/source bundle; verified receipt source,
profile and executable SHA256 before execution:
de5576255c97a14d46766c49e258829444d55209d8221127b5bb4b4dc14358a0.
Private host run native.H9rrqG passed enrolled panes/tab/split/focus/real
shell exit and normal close; all seven tracked PIDs stopped. Its initial
PNG remains byte-identical to retained gold, SHA256
e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865.
This is a diagnostic debug witness after worker formatting, not a clean
exact-source or release acceptance record; the formatting is now committed
and needs the next source gate. Protected API cases above were exercised
by the real-broker CBC regression, not asserted by the UI witness script.

dd1a5ac59f9081038302de3d900d7eb9d01d346c adds pinned gold forward_execute
as a recipient child module, including execute/result/cancel and task
submit/result/cancel. The exact native child binding determines its shell
Source. Prompt/task policy stays in the child; no pane or retained-state
lock spans the five-second native RPC. Caller epoch, on-behalf-of label,
shared forwarded-id mapping, request retirement, child operation mapping,
refusal vocabulary/details and byte limits retain gold behaviour. Child
answers are retained; timeout/transient resource/oversized placeholders
are not, so exact retries can reach child dedupe. New absent task options
stay omitted rather than relayed as null. The handler stays single-flight.

Extended the real-Mix regression with current prompt generation, verified
caller epoch, forwarded evaluation completion, isolated task result 42,
identical execute/task retries and result/cancel family separation. These
new cases await CBC. Layout/session metadata through the in-process Mix
adapter, complete gold enforcement/reconnect/descriptor coverage, remaining
app inventory and exact release acceptance remain open. NOT ACCEPTED.
No Fable deviation or gold/desktop changes.

Next CBC `term-child-forward-dd1a5ac5`, exact code above,
term_gate --prepare. Read full log; fix failures, apply fmt/lock and repeat
exact source if changed. Continue the Mix-owned metadata/layout adapter and
native acceptance; do not treat these debug witnesses as final release.
branch: lab/parity-term; head SHA: dd1a5ac59f9081038302de3d900d7eb9d01d346c (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — child forwarding gate, Mix metadata/layout adapter and remaining full parity/release acceptance open; CBC job ids: term-protected-input-7474c477-a1 (green; fmt applied), term-child-forward-dd1a5ac5 (next await).

### Child task fixture correction

`term-child-forward-dd1a5ac5-a1` compiled and passed six Mix suites, two
bundle tests and 165 terminal tests; the real-owner regression failed after
its protected input and forwarded evaluation checks had passed. Its first
task submission correctly returned INVALID_ARGUMENT because the fixture
omitted cwd. Gold's session_execute::task_submit explicitly requires BOTH
cwd and timeout_ms; the relay must not invent a default. Full 32530-byte
saved log read in two chunks; worker formatting applied in 50b1e98f.

50b1e98f99d76bec4b5cdda9e8451b685b9f1aca keeps the missing-cwd request
as an asserted INVALID_ARGUMENT case and adds a new valid submission ID
with cwd set to the isolated HOME. Typed task result, retry and cross-family
assertions remain unchanged. This is a fixture correction, with no relaxed
assertion or production-policy change. The full close/exit variants and
remaining gate checks still need to complete; no task success is claimed.

Next CBC `term-task-cwd-50b1e98f`, exact code above, term_gate --prepare.
Read complete log; fix failures, apply fmt/lock and repeat exact source.
Then continue Mix-owned session metadata/layout adapter, remaining native
enforcement coverage, full app inventory and exact release acceptance.
NOT ACCEPTED. No Fable deviation or gold/desktop changes.
branch: lab/parity-term; head SHA: 50b1e98f99d76bec4b5cdda9e8451b685b9f1aca (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — child task fixture repeat, Mix metadata/layout adapter and remaining full parity/release acceptance open; CBC job ids: term-child-forward-dd1a5ac5-a1 (failed at missing-cwd fixture; fmt applied), term-task-cwd-50b1e98f (next await).

### Exact child-forwarding gate passes; Mix metadata integration checkpoint

`term-task-cwd-50b1e98f-a1` passed all scoped checks at exact clean code
50b1e98f99d76bec4b5cdda9e8451b685b9f1aca: 166 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog,
foreign gate and debug build. Full 37440-byte saved log read in two chunks;
TERM_PATCH empty. Both isolated close/exit modes passed real protected input,
evaluation completion/retry, missing-cwd task refusal, valid typed task
result 42/retry and execution/task result/cancel separation. This does not
replace the full original exact-current-shell/bootstrap/enforcement suite.

3d5dcd16db31351d73f1f6857a07cc13fab9774b adds a read-only in-process
metadata document. Mix projects tab order, title, active tab, selected pane
and split membership from its existing model on publish_term. The generic
native terminal binding validates 32 total slots, unique/positive ids,
sanitised titles, one active tab, selected membership and references to
owned native panes, then replaces the accepted document atomically. It
does not derive tab policy or carry native session authority.

Protected session/list/tabs/panes and state-property readbacks now combine
this Mix-authored document with native pane generations, dimensions/PIDs,
recipient lease, caller epoch/high-water and retained limits. Unbound panes
and partially unbound tabs are filtered as gold does. Target authority stays
native. Live tab metadata changes do not replace the child or its identity.

Added Mix projection assertions for reordering, title/active/selected/split
membership and an empty closed model; all six Mix suites pass. Native tests
add malformed/duplicate/inconsistent metadata validation, rejected missing
pane updates retaining prior state, protected metadata/alias fields, live
title publication with unchanged PID, and forwarding using the epoch read
from term.session. These native additions and bundle integration await CBC.

Protected layout mutations still need the Mix mailbox/commit adapter and
affected-pane/generation validation. Remaining full Bus/input/clipboard/IME/
search/Prefs, original enforcement/reconnect coverage and exact release
acceptance remain open. NOT ACCEPTED. No Fable deviation or gold changes.

Next CBC `term-mix-metadata-3d5dcd16`, exact code above, term_gate --prepare.
Read full saved log; fix failures, apply fmt/lock and repeat exact source.
Then continue protected Mix layout commitment and matched native evidence.
branch: lab/parity-term; head SHA: 3d5dcd16db31351d73f1f6857a07cc13fab9774b (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — Mix metadata gate, protected layout commit adapter and remaining full parity/release acceptance open; CBC job ids: term-task-cwd-50b1e98f-a1 (green exact; no diff), term-mix-metadata-3d5dcd16 (next await).

### Metadata gate passes; native layout callback context checkpoint

`term-mix-metadata-3d5dcd16-a1` passed: 167 terminal tests, five existing
ignores, seven input tests, two bundle tests, six Mix suites, search/dialog,
foreign gate and debug build. Real-Mix protected state/list/alias fields,
live metadata publication with unchanged child PID and metadata epoch used
for forwarding passed. Complete 52130-byte saved log read in three chunks;
worker formatting applied in f47e845d. No new visual/release claim.

f47e845d64f525ae2a585c49f6b71c3e2ba33398 adds a native-only context to
the existing ordered runtime mailbox. An owner installs a non-serialisable
scope on the evaluator thread for its one callback. Ordinary Bus/UI events
cannot supply that context. A reusable terminal layout bridge queues at
most 32 requests and binds scope calls to the owning bridge. It exposes
inspect/begin/finish to the trusted Mix callback; begin invokes native guard
validation once, and finish reports whether commitment began. Abandonment
before begin refuses; abandonment after commitment reports unknown outcome.
Cancellation and begin serialise under one state lock. Cancelled entries
release capacity, and no authority enters the event's JSON arguments.

Generic host pump dispatches these native requests through the existing
runtime. No new transport/service/thread is introduced. Four pending tests
cover native-owner scope and single use, cancellation/refusal/abandonment,
bounded queue recovery, and actual Mix runtime rejection of forged ordinary
events before/after a valid native callback. The scoped gate now also runs
existing runtime/calls/timers integration tests because mailbox code changed.

This is the secure handoff foundation: the recipient does not yet enqueue
protected layout requests and Term has no policy reducer for them. Next
connect the typed admission/permit/replay guards to this bridge, port gold's
affected source/destination-pane calculation into Mix, and implement the
Mix-owned tab/pane mutations with native commitment/retained replies.
Remaining full inventory, enforcement/Prefs and exact release acceptance
stay open. NOT ACCEPTED. No Fable deviation or gold/desktop changes.

Next CBC `term-layout-context-f47e845d`, exact code above,
term_gate --prepare. Read full saved log; repair failures, apply fmt/lock
and repeat exact source if changed before continuing the policy adapter.
branch: lab/parity-term; head SHA: f47e845d64f525ae2a585c49f6b71c3e2ba33398 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — native callback context gate, protected Mix layout producer/reducer and remaining full parity/release acceptance open; CBC job ids: term-mix-metadata-3d5dcd16-a1 (green; fmt applied), term-layout-context-f47e845d (next await).

### Native layout context compile repair

CBC term-layout-context-f47e845d-a1 failed with Rust E0282 before tests: the local completion reply shadowed the optional supplied reply. Complete 26542-byte saved log read; returned formatting applied. Code a374b0974590c2257595f0a276d6022498c78e01 names the supplied argument separately, preserving the existing cancellation/commit rules. No test weakened and no acceptance claim.

CBC term-layout-reply-a374b097 is pending at that exact code, term_gate --prepare, await a8f168f1-32bc-4c66-aa26-7b98585d64d4. Read its complete saved log on resume, repair any failures, apply returned fmt/lock and repeat exact source as required. Protected layout producer and Mix affected-pane policy/reducer remain next; the full feature/Bus/Prefs, conformance and release visual gates remain open. No desktop or gold changes.

branch: lab/parity-term; head SHA: a374b0974590c2257595f0a276d6022498c78e01 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — callback bridge gate pending, protected layout producer/reducer and full parity/release acceptance open; CBC job ids: term-layout-context-f47e845d-a1 (compile failure), term-layout-reply-a374b097 (pending).

### Native layout bridge passes; protected Mix policy connected

CBC term-layout-reply-a374b097-a1 passed exact a374b0974590c2257595f0a276d6022498c78e01 with no worker diff: 171 terminal tests, five existing ignores, 13 runtime/calls/timers tests, seven input tests, two bundle tests, six Mix suites, dialog/search, foreign gate and debug build. Complete 42410-byte saved log read in two chunks. Four bridge tests prove owner-bound native scope, single commit, cancellation/abandonment, queue recovery and ordinary-event forgery rejection. No release or new visual acceptance claim.

Code be8edc5959b0666579eaaa55b29db641f2949c09 connects protected ManageLayout/Terminate delivery to the existing native-only runtime callback. Native guards validate target/sibling generations and current parent/connection permits, then check mutation epoch and reserve the native replay sequence. Native completion retains begun outcomes. The request authority stays in Rust closures, not callback JSON. Mix computes affected tabs/panes from pinned gold TabSet::control_affected and owns tab new/select/close and pane split/select/close, including replacement-tab focus and profile defaults. The actual Term handler publishes metadata/scene and settles native completion. No new transport, service or Rust application policy.

New pure Mix tests and lint pass locally: source/destination/replacement affected sets, explicit inactive targets, denied mutations without side effects, split directions, close cleanup, new tabs and gold focus retained on spawn failure. The CBC gate now includes this seventh Mix suite. Expanded real-Mix owner fixture drives verified protected selection, exact retry, invalid split, actual split/PTY creation, omitted/stale sibling authority and high-water retention through a fixture runtime pump loading the real authored reducer. These native cases and actual bundle loading are pending CBC, not yet proven.

Next CBC term-protected-layout-be8edc59 at the code SHA above, term_gate --prepare. Read the complete saved log, repair failures, apply fmt/lock and rerun exact source. Then exercise actual generic host protected layout and close/replacement/replay cases, broader enforcement/reconnect, remaining full Bus/input/IME/search/Prefs inventory and release matched visual acceptance. NOT ACCEPTED. No Fable deviation, install or desktop/gold changes.

branch: lab/parity-term; head SHA: be8edc5959b0666579eaaa55b29db641f2949c09 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — protected Mix layout native gate and remaining full parity/release acceptance open; CBC job ids: term-layout-reply-a374b097-a1 (green exact), term-protected-layout-be8edc59 (next await).

### Protected Mix layout pipeline passes; close/replacement proof expanded

CBC term-protected-layout-be8edc59-a1 passed: seven Mix suites including affected-pane policy, 171 terminal tests with five existing ignores, 13 runtime/calls/timers checks, seven input checks, two actual bundle tests, search/dialog, foreign gate and debug artifact. Complete 51070-byte log read in three chunks. The close and natural-exit subprocess fixtures both proved verified native layout selection, exact retries, invalid-direction refusal, real split child creation, omitted/stale sibling-authority refusal and unchanged high-water on denial through the actual authored reducer and native runtime scope. Worker formatting applied in d16d82fd. No release or new visual claim.

Code d16d82fd586c030f47f66a0f29edb174655af9d9 extends those real-broker/real-Mix fixtures: protected new tabs and retries; closing an inactive singleton selects its following replacement without closing the former active child; closing the active last-position tab requires every leaf of the previous replacement tab; incomplete authority refuses without consuming the sequence; close retries and term.operation survive target removal; the original pane remains, metadata/high-water agree, removed handles fail and removed child PIDs disappear from /proc. These new cases are pending CBC. No implementation policy moved into Rust.

Next CBC term-layout-close-d16d82fd at that exact code, term_gate --prepare. Read complete saved log, repair and apply any fmt/lock diff, then repeat exact source if changed. Generic-host native layout captures, broader enforcement/reconnect, remaining full Bus/input/IME/search/Prefs inventory and final exact release visual acceptance stay open. NOT ACCEPTED; no Fable deviation or desktop/gold changes.

branch: lab/parity-term; head SHA: d16d82fd586c030f47f66a0f29edb174655af9d9 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — close/replacement native gate, actual-host proof and remaining full parity/release acceptance open; CBC job ids: term-protected-layout-be8edc59-a1 (green; fmt applied), term-layout-close-d16d82fd (next await).

### Exact native close gate and matched protected-host captures

CBC term-layout-close-d16d82fd-a1 passed exact d16d82fd586c030f47f66a0f29edb174655af9d9 with no worker diff: 171 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input tests, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug artifact. Complete 42007-byte saved log read in two chunks. Both real-Mix close/exit fixture modes now pass protected new tabs/retries, inactive singleton close and following replacement, last-position close and previous replacement, incomplete authority refusal before reservation, retries/retained outcomes after removal, metadata/high-water and child-process cleanup.

Fetched and verified debug executable SHA256 fb737f382caf00458f40be9c6c84ccecb82de4ab86b79a04c5af2e9e0947e9d9, source as above. Gold stays pinned 8779f33284b7d3f1178c3d57a699efb9b0867b22, executable SHA256 153c369007f2da15bdb28fcd1dcbc93f511ff07834f11c2b046650d4c1c51bf6. Private archive native.8ss83B contains executed harness, exact command/function fixture, binary/capture/native readbacks, protected request/reply log, comparisons, inspection and closure receipt. Owned native Bus/settingsd/headless compositor; sequential canonical term identity, matched 900x560 clients, scale 1, Studio Dark, Inter 400/14.667, terminal 400/14.667, CSD right captions, bar menu and row actions.

Real host endpoints on BOTH apps passed protected split and exact retry, native Tab/Shift+Tab focus, a new enrolled shell and natural exit, protected pane close and retry after removal, retained term.operation and frame readback with only the intended survivor. All 11 tracked PIDs, including broker and six shell children, were stopped. Discovery uses native noded.session.list through Mix send with the strict bus/native-session envelope; no additional transport or automation language.

The same short pair() command invokes an identical private .mixrc function which clears the display and prints red/bold, a constant marker, wide and combining text. No fabricated terminal output. Original gold/candidate session PNGs share SHA256 e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Protected split PNGs share SHA256 165ac3aed587310d3180dd5bb870986f7c66bb7d5dd770954dcf209912b1f75d. Fresh parity_compare output directories compare-session and compare-protected-split contain full/cropped figures. ImageMagick 7.1.2-35 SSIM is distortion (zero identical): full [0,0,900,560], header [0,0,900,38], tabs [0,38,900,32] and terminal [0,70,900,490] all have SSIM 0 and RMSE 0 in both states. Inspected both originals, both full montages and both full difference figures; no differences remain in these states.

Earlier native.5IDCZa retains unequal long synthetic-command echo: full session SSIM distortion 0.0135522/RMSE 0.055697, split 0.0084703/0.0445039; header/tabs zero. Images inspected: extra echo line on gold shifts output; no acceptance claimed for that mismatched-data run. An alternative protected-evaluation recipe native.PhXX9p stopped when local gold shell.status advertised evaluation UNSUPPORTED and term.execute relayed UNSUPPORTED; its five tracked PIDs stopped. That local capability differs from the newer worker shell used by positive forwarding tests. It is recorded separately, without claiming forwarding support in the local gold shell or weakening positive worker checks.

This is debug functional/visual evidence for the tested states, not release acceptance. Remaining mouse/selection/primary/OSC52, IME/search/Prefs, full Bus/topics/errors/enforcement/reconnect, full conformance inventory and exact release matched acceptance remain open. NOT ACCEPTED. No install, gold change or Fable deviation.

branch: lab/parity-term; head SHA: d16d82fd586c030f47f66a0f29edb174655af9d9 (tested code; receipt HEAD contains this checkpoint); NOT ACCEPTED — debug protected-host states match all regions at zero distortion, remaining full feature/Bus/Prefs/conformance/release acceptance open; CBC job ids: term-layout-close-d16d82fd-a1 (green exact).

### Shared native clipboard and primary-selection path checkpoint

Code 13b95e5ced6018a1502ce8ea2ec7af702a5a5f22 adapts pinned gold apps/term/src/selection.rs and clipboard.rs into reusable Viu terminal-native facilities. Source hashes and adaptations are in provenance/terminal-native.json; original licences retained. Existing smithay-client-toolkit/Wayland/raw-window-handle dependencies are reused, with no new foreign dependency. The display-owning eframe host constructs the worker; a layer-only host without a display refuses reads as gold does. The worker uses a separate event queue on the same native window connection, matching gold; it is not a service or parallel Bus.

viu_platform retains its bounded in-process queue, with a validated optional primary boolean. Host writes/reads go through the gold clipboard/primary worker; original Mix request IDs are mapped to the worker completion owner. Four original tests remain: read-busy/nonblocking write and refusal isolation in the wrapper; byte/UTF-8/newline bounds and actual pipe busy/timeout/oversize/ordinary-read recovery in the worker. The existing terminal scoped gate includes them. Removed the former viewport RequestPaste wait path so native clipboard reads have one path. No application tab or paste-target policy moved into Rust.

This checkpoint adds native platform capability only; terminal OSC52 draining/replies and mouse selection/copy-on-select/middle-paste ownership still need their Mix/native adapter and real Wayland evidence. Ordinary platform Paste input events still follow the existing native input route. Build/runtime/clipboard readback not yet claimed. Next CBC term-native-selection-13b95e5c, term_gate --prepare at the exact code above. Read its full saved log, repair failures, apply fmt/lock and repeat exact source; then verify actual clipboard/primary ownership, empty/refused/late/busy reads, original paste-owner survival, and connect OSC52/live policy.

The previous exact native lifecycle and matched protected-host evidence at d16d82fd remains recorded above; it does not attest these new bytes. Overall NOT ACCEPTED: remaining input/clipboard/IME/search/Prefs, complete Bus/enforcement/conformance and release visual acceptance open. No desktop/gold change or Fable deviation.

branch: lab/parity-term; head SHA: 13b95e5ced6018a1502ce8ea2ec7af702a5a5f22 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — native clipboard/primary gate, remaining full features/Bus/Prefs/conformance/release acceptance open; CBC job ids: term-layout-close-d16d82fd-a1 (green exact; matched debug host evidence), term-native-selection-13b95e5c (next await).

### Native selection wrapper import repair

CBC term-native-selection-13b95e5c-a1 failed before tests with E0433: the imported clipboard wrapper still used the gold sctk alias without importing the existing smithay_client_toolkit crate under that name. Complete 9771-byte saved log read. Worker formatting applied and the missing alias added in e59f321259ef1606c166b8d2b0cb7d015c1bec61. No dependency, test or runtime behaviour weakened.

Next CBC term-selection-alias-e59f3212 at that exact code, term_gate --prepare. Read the complete log, repair further failures and apply fmt/lock changes before exact-source repeat. Native clipboard/primary readback and terminal OSC52/mouse adapters remain unproven/open; prior matched protected-host debug evidence applies only to d16d82fd. NOT ACCEPTED. No install, gold change or Fable deviation.

branch: lab/parity-term; head SHA: e59f321259ef1606c166b8d2b0cb7d015c1bec61 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — native selection gate and remaining full parity/release acceptance open; CBC job ids: term-native-selection-13b95e5c-a1 (compile failure), term-selection-alias-e59f3212 (next await).

### Native selection gate passes; OSC52/live read policy wired

CBC term-selection-alias-e59f3212-a1 passed exact e59f321259ef1606c166b8d2b0cb7d015c1bec61 with no worker diff: 175 terminal tests, five existing ignores, four imported clipboard/pipe tests, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug artifact. Complete 42772-byte saved log read in two chunks. Actual compositor-backed clipboard/primary readback remains to be driven; unit success does not claim it.

Code 0c68440672ce66d600f337d713b8048b093e75d9 connects program OSC52 effects to the native window selection worker. Mix forwards the live terminal.clipboard.osc52_read flag to the in-process terminal binding. Existing and newly created panes obey it; reads default off and writes remain allowed. Native host drains hidden as well as visible owned panes in bounded fair turns, retaining any remaining queued requests for another frame. Native reply formatter closures stay outside JSON.

Selection completion has explicit typed owners for a Mix platform request versus an OSC52 query. Each OSC52 read retains its original immutable terminal identity/formatter with weak ownership; close or handle replacement refuses the delayed reply rather than retargeting another pane. Concurrent refused reads retain distinct IDs, so an OSC52 completion cannot fall through into ordinary paste. The ownership and terminal locks cover the bounded queue write without retaining a pane across the asynchronous read.

Four pending native regressions cover hidden-pane request order and queue remainder, fairness under a busy low handle, successful original-owner reply versus close/replacement, and live permission changes including new-pane inheritance while writes remain enabled. Existing parser/model OSC52 tests and the full scoped gate remain unchanged. Next CBC term-osc52-owner-0c684406, term_gate --prepare at exact code above. Read complete saved log, repair any failures, apply fmt/lock and repeat exact source. Then fetch verified bytes and test real clipboard/primary/OSC52 queries, busy/refused/late reads and matched native captures.

Prior protected-host matched debug evidence applies to d16d82fd only. Mouse selection/middle-paste/copy-on-select, complete IME/search/Prefs, full Bus/enforcement/conformance and release acceptance remain open. NOT ACCEPTED. No gold/desktop changes or Fable deviation.

branch: lab/parity-term; head SHA: 0c68440672ce66d600f337d713b8048b093e75d9 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — OSC52/live policy gate, native clipboard readback and remaining full parity/release acceptance open; CBC job ids: term-selection-alias-e59f3212-a1 (green exact), term-osc52-owner-0c684406 (next await).

### OSC52 native gate and real compositor clipboard/primary witness

CBC term-osc52-owner-0c684406-a1 passed: 179 terminal tests, five existing ignores, including the four new ownership/policy/draining regressions; 13 runtime/calls/timers checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug artifact. Complete 64314-byte saved log read in three chunks. Returned formatting applied and committed as c1afd7011c3315b635cf06e2c485355a9884cef1; an exact-source repeat at that revision is still required.

Fetched/verified debug artifact source 0c68440672ce66d600f337d713b8048b093e75d9, executable SHA256 8c74fe7214626ac3f3e232c0da74876518af0697563543c73c45f6fc17ae7f47 (worker formatting present; diagnostic, not release acceptance). Private native.Hr6vCc contains the executed fixture/harness, all requests/replies, snapshots, fenced settings receipts, binary receipt, witness.json and closure.json. On sequential canonical gold/candidate hosts over an owned Bus/settingsd/headless compositor, both real Mix shells emitted OSC52 clipboard writes and the normal Paste action read back TERM_OSC52_CLIPBOARD through native Wayland selection transfer into the original shell. Only encoded fixture text was emitted by the copy command; the visible marker came from actual paste readback.

Both also emitted an OSC52 primary-selection write containing TERM_OSC52_PRIMARY. Live read permission false -> true -> false, without restarting either app or shell, produced exact PTY reply-written deltas 0 -> 32 -> 0. The enabled response placed the exact expected primary base64 token in the originating shell; the 32-byte OSC52 envelope matches the parser formatter. Disabled queries produced no reply bytes. This covers native primary read transfer and query permission, independently of the still-open mouse middle-paste gesture. All seven tracked processes stopped; no live clipboard, desktop or Prefs were touched.

Earlier native.2NLAyw passed clipboard roundtrip plus the protected split/retry/close/exit sequence, with all 11 tracked PIDs stopped. Longer native.yQYxUE passed the clipboard and primary subcases on both apps, but a later candidate split had no usable ready grant and entered the existing gold actor 60-second mint-floor graphics-only fallback; the combined run failed and all nine tracked PIDs stopped. That failure is retained and is not a full-run pass. Matched long-idle grant-pool/enforcement/reconnect behaviour remains open; no mint security floor was changed.

Next CBC term-osc52-exact-c1afd701 at exact c1afd7011c3315b635cf06e2c485355a9884cef1, term_gate --prepare. Read complete saved log and require no worker diff; fetch its verified bytes for the native repeat before advancing acceptance. Then complete mouse reporting/selection/middle-paste/copy-on-select, native IME/search/Prefs, full Bus/topics/errors/enforcement/conformance and exact release matched visual acceptance. NOT ACCEPTED. No Fable deviation or install/gold change.

branch: lab/parity-term; head SHA: c1afd7011c3315b635cf06e2c485355a9884cef1 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — OSC52 exact repeat, remaining mouse/Bus/Prefs/enforcement/conformance/release acceptance open; CBC job ids: term-osc52-owner-0c684406-a1 (green; fmt applied; native clipboard/primary/policy witness), term-osc52-exact-c1afd701 (next await).

### Exact OSC52 witness; ordered pointer/selection checkpoint

CBC term-osc52-exact-c1afd701-a1 passed exact c1afd7011c3315b635cf06e2c485355a9884cef1 with no worker diff: 179 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug build. Complete 43670-byte saved log read in two chunks. Fetched and verified executable SHA256 19bd7ef38e2cc9b2daa09447c67ed74bb14c62218473c85da4e3d4c259fbb7fe. Private native.31X4Jv repeated actual gold/candidate OSC52 clipboard write/menu paste, primary transfer and live false-true-false permission at unchanged processes: exact reply-written deltas 0/32/0 and expected primary base64 text in the original shell on both. All seven tracked PIDs stopped. Harness, all receipts, snapshots and witness.json retained. Debug functional evidence only, not release acceptance.

Code 0bffd1a7b7e486e94bf77cca8c90a4a20fbe0689 ports gold mouse.rs gesture decisions into the reusable compound widget. Source hash/adaptations recorded in provenance. Events retain their own coordinates; a grab latches its original native terminal, button and local/reporting ownership. Matching releases use the press modifiers even after Shift changes; unrelated releases cannot end a grab. All three buttons and hover motion report to the program. Half-cell anchors and gold same-cell 300ms simple/word/line click cycles replace the prior delayed response-based selection. Local selection clears other owned panes, writes PRIMARY on finish and optionally clipboard via live terminal.mouse.copy_on_select. Local middle press requests native primary paste through Mix with its immutable handle. These raw selection/paste effects do not invent registry action-executed events.

The generic widget records geometry before dispatching pointer and keyboard input together in original event order, so a same-frame click changes the native input owner before the next key rather than typing into the former pane. Focus/clipboard application effects remain authored Mix. Wheel ownership follows ordered pointer positions, including exclusive pane edges; existing wheel zoom/scroll mechanics retained. Disappeared widgets, hidden/closed grab owners and focus loss cancel the old grab once. Popup layers block underlying pointer handling. Shared Widgets begin/finish hooks only manage native widget gesture lifetime; no app policy was added there.

Seven new primitive regressions cover matching/latched release and clamping, three buttons/hover, hidden/focus cancellation, half-cell selection/primary/copy-on-select, Shift middle-read ownership, click timing plus actual double-click selection, and event-position wheel ownership. A real-widget regression sends text before and after a click in one frame and requires each byte sequence on its correct native input channel. Existing input/wheel/clipboard/IME tests remain. Mix core/model/actions/Bus/protected suites pass locally, including primary read owner/effect validation. Rust compilation, new native tests and actual mouse/selection captures are pending.

Next CBC term-pointer-order-0bffd1a7 at exact code above, term_gate --prepare. Read full saved log, fix failures, apply fmt/lock and repeat exact source. Then fetch verified artifact and drive native primary middle-paste, drag/word/line/copy-on-select/reporting/cancellation and matched captures against gold. Full input/IME/search/Prefs, Bus/topics/errors/enforcement/conformance and release acceptance remain open, including matched long-idle grant-pool behaviour noted above. NOT ACCEPTED. No Fable deviation, install or gold/desktop changes.

branch: lab/parity-term; head SHA: 0bffd1a7b7e486e94bf77cca8c90a4a20fbe0689 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — ordered pointer gate/native mouse captures and remaining full parity/release acceptance open; CBC job ids: term-osc52-exact-c1afd701-a1 (green exact; native repeat passed), term-pointer-order-0bffd1a7 (next await).

### Pointer gate compile repair

CBC term-pointer-order-0bffd1a7-a1 failed before tests with E0027: the new MouseWheel match omitted egui phase/source fields. Read the complete 71956-byte saved log in four chunks and applied its formatting patch. Code dd61895fb832f36b39769134f371da7072e00673 ignores those fields in the match and explicitly supplies Move/Unknown in the wheel regression fixture. No behaviour assertion was weakened. Seven Mix suites passed before the compile failure; the new pointer and ordered-focus tests have not run yet.

Next CBC term-pointer-fields-dd61895f runs term_gate --prepare on that exact code. Read its full saved log, repair any failures and apply returned diffs before an exact-source repeat. Native pointer/selection captures, remaining full functional/Bus/Prefs/conformance checks and the release visual gate remain open. NOT ACCEPTED; gold and desks remain untouched.

branch: lab/parity-term; head SHA: dd61895fb832f36b39769134f371da7072e00673 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — pointer compile repair gate and native mouse proof pending, remaining full parity/release acceptance open; CBC job ids: term-pointer-order-0bffd1a7-a1 (E0027; full log read; fmt applied), term-pointer-fields-dd61895f (next await).

### Ordered focus surrender repair

CBC term-pointer-fields-dd61895f-a1 compiled and passed both bundle tests, 13 runtime/call/timer tests and seven Mix suites. Terminal results: 186 passed, one failed, five existing ignores. All seven new pointer controller tests passed, as did existing wheel, Tab, clipboard and live font view tests. Read the complete 30426-byte saved log in two chunks; no worker diff. The real-widget same-frame test failed because bytes preceding a click were lost: egui surrendered the old pane focus while registering its response from the frame-wide pointer state, before the ordered input loop ran.

Code f6c849a24dfffbd1caa7f22234804d1243b41e3a suppresses that automatic surrender only during terminal response registration, immediately restores the shared option, and handles outside/blocked pointer presses in event order. Existing before/after-click assertions remain; a new outside-click case requires preceding text to reach the old pane and subsequent text to reach neither pane. Warm-up explicitly asserts the original pane owns focus. No test expectation was relaxed.

Next CBC term-focus-order-f6c849a2 runs term_gate --prepare at that code. Its tests, any formatting return and exact repeat are pending. Then fetch verified bytes for native mouse/selection/primary/copy-on-select/reporting/cancellation captures. Full functional, Bus, Prefs, conformance and release visual acceptance remain open. NOT ACCEPTED; no Fable deviation or desk/gold change.

branch: lab/parity-term; head SHA: f6c849a24dfffbd1caa7f22234804d1243b41e3a (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — ordered focus gate/native pointer captures and remaining full parity/release checks open; CBC job ids: term-pointer-fields-dd61895f-a1 (186 passed, one focus regression failed, five ignores; no diff), term-focus-order-f6c849a2 (next await).

### Pointer gate and native matched selection witness

CBC term-focus-order-f6c849a2-a1 passed: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search checks, foreign gate and debug build. Both ordered-focus tests passed without weakened assertions. Complete 47979-byte saved log read in three chunks. Applied its formatting patch in 155d3c1ceb06b8f7613e257b6d99a34062492152; exact repeat pending.

Fetched and verified f6c849a2 debug executable SHA256 eaf0eae488d0a9dc9577bc1eacc39ed06dd54c6d4f0bda80648eb43445f605af. Private native.cVYXIK exercised gold then candidate on owned Bus/compositor/settings authority: actual OSC52 PRIMARY write followed by middle-button paste; native drag of TERM_PAIR_NATIVE and PRIMARY pasteback into its original real Mix shell; live terminal.mouse.copy_on_select false then true; true additionally copied to clipboard and menu-paste read back the selected text. Each app retained its process during live changes. All seven tracked processes stopped. Script, calls, snapshots, captures, cleanup and witness.json retained privately.

Three 900x560 capture pairs at scale 1, Studio Dark/Inter 400 14.667, terminal regular 14.667, CSD right captions/bar menus/row actions: baseline session, drag with copy-on-select false, drag with true. All full/header/tabs/terminal SSIM and RMSE distortions are zero (ImageMagick 7.1.2-35, zero means identical). Rectangles: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Both baseline PNG hashes e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865; all four selection PNG hashes 78897e18d3b10893a22c29e98408adbe20c513df3b09ea5b2c089c24f2d60fb9. Inspected originals, montages and differences: no region differences. These are worker-formatted debug diagnostic witnesses, not release visual acceptance.

Earlier native.tlv09y failed on gold because the harness compared clipboard paste against a snapshot taken before an intervening PRIMARY paste and Ctrl-C output. The real clipboard text was present; corrected the assertion to compare immediately before/after that paste. Retained failed run, all five tracked processes stopped. No application change or expectation relaxation.

Next CBC term-pointer-exact-155d3c1c at exact 155d3c1ceb06b8f7613e257b6d99a34062492152. Require complete log/no worker diff, fetch matching verified bytes and repeat native proof. Remaining: native program mouse reporting/cancellation, word/line selection, IME/search/full Prefs, clipboard failure notices, complete Bus/topics/errors/enforcement/conformance, long-idle grant-pool parity and scoped release visual matrix. NOT ACCEPTED; gold and desks unchanged.

branch: lab/parity-term; head SHA: 155d3c1ceb06b8f7613e257b6d99a34062492152 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — exact pointer repeat, remaining input/Bus/Prefs/enforcement/conformance and release acceptance open; CBC job ids: term-focus-order-f6c849a2-a1 (green; fmt applied; native matched selection proof), term-pointer-exact-155d3c1c (next await).

### Exact pointer witness; clipboard refusal and notice checkpoint

CBC term-pointer-exact-155d3c1c-a1 passed exact 155d3c1ceb06b8f7613e257b6d99a34062492152 with no worker diff: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug build. Complete 49451-byte saved log read in three chunks. Fetched and verified executable SHA256 a8a4d9a56a1fec7a2a9f3cee3e3f9e3c9efe46c09633f85c460e24044e66a345.

Private native.m9EJYm repeated actual gold/candidate middle PRIMARY paste, drag selection/PRIMARY pasteback and live copy-on-select false/true with clipboard readback on true. All seven tracked processes stopped. Three matched 900x560 capture pairs, same settings/regions as the preceding checkpoint: all full/header/tabs/terminal SSIM and RMSE distortions zero; baseline hash e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865, selection hash 78897e18d3b10893a22c29e98408adbe20c513df3b09ea5b2c089c24f2d60fb9 on both apps. Originals, montages and differences inspected; no differences. Private witness.json includes exact bytes/source/profile, region figures and cleanup. This attests debug bytes and these states only, not final release acceptance.

Code 050e935002a810c4b3804cdabcd9e8d7c7b1d86e restores gold clipboard completion policy in Mix: Empty is silent, TooLarge/TimedOut/Busy/NoFocus/Failed produce the pinned labels, read tokens retire once, successful paste clears the notice, stale or closed owners never retarget another pane, and refused copies do not end pending reads. Generic host forwards native write refusals over existing in-process platform completions; OSC52 query failures remain native and never ordinary paste. No broker or application policy added to Rust. Supplied platform text uses the same success/refusal policy. Mix publishes changed notices immediately.

Gold also renders clipboard refusal and unsaved zoom rows above the terminal. The port now composes those rows from existing scene columns and warning labels, card background and 8x3 margins, independently visible in gold order. Zoom label is exactly Zoom is not saved. No new native presentation widget was needed. Eight Mix suites pass locally, including new completion ownership/refusal regressions and frame notice visibility/labels. App lint passes with its five embedded host functions declared; plain standalone lint otherwise correctly reports those embedding-only names. Rust/scene integration and native notice captures pending. Source hashes and adaptations recorded in provenance.

Next CBC term-clipboard-notices-050e9350 runs term_gate --prepare at code above. Read complete saved log, apply any returned formatting and require an exact repeat; fetch verified bytes for native clipboard refusal/zoom notice and unchanged baseline captures. Then continue program mouse reporting/cancellation, word/line selection, native IME/search/full Prefs, complete Bus/topics/errors/enforcement/conformance, long-idle grant-pool parity and scoped release visual acceptance. NOT ACCEPTED; no Fable deviation, install or gold/desk changes.

branch: lab/parity-term; head SHA: 050e935002a810c4b3804cdabcd9e8d7c7b1d86e (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — clipboard notice gate/native captures and remaining full input/Bus/Prefs/enforcement/conformance/release acceptance open; CBC job ids: term-pointer-exact-155d3c1c-a1 (green exact; native repeat and zero-difference selection captures), term-clipboard-notices-050e9350 (next await).

### Native clipboard refusal notice witness

CBC term-clipboard-notices-050e9350-a1 passed: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, eight Mix suites, dialog/search, foreign gate and debug build. Complete 50419-byte saved log read in three chunks. Applied the returned host formatting in 7b97f07344d0cc1fffd44a093b470887069f7311. Exact repeat pending. Verified worker-formatted 050e9350 debug binary SHA256 1af77fecdee94bd015e8088846160089576086308ad672eee073b485f6b897d1.

Private native.W8jl8A passed actual gold/candidate NoFocus clipboard refusal and success clearing. An owned ced window takes keyboard focus, the real Mix shell emits an ordinary OSC52 clipboard write, and the terminal reports the exact gold notice. After closing ced, both apps are restored/focused through PID/generation-matched native compositor commands. A real focused clipboard write/menu paste clears the notice. Captured baseline, refusal and cleared states on unchanged term processes. All nine tracked processes stopped. Private script, request/reply logs, snapshots, captures, comparison.json files and witness.json retained.

All three 900x560 pairs are byte-identical. Full/header/tabs/terminal and refusal notice region SSIM/RMSE distortions are zero (ImageMagick 7.1.2-35; zero identical). Refusal rectangles: header 0,0,900,38; tabs 0,38,900,32; notice 0,70,900,24; terminal 0,94,900,466; full 0,0,900,560. Baseline/cleared use preceding checkpoint regions. Refusal PNG SHA256 on both apps 87f8f03b1d6be4b6580ec1bf95c63fc4d9aa7bb015ce53214a7db6fba789d794; baseline/cleared e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, montages and differences inspected: notice text, warning colour, card background, margins and grid displacement match; no region differences. Debug witness only.

Retained probe failures: native.7iLuOu and native.tYTO51 wrongly expected a notice for unfocused PRIMARY writes, which gold intentionally keeps silent; native.BmdeZd used window action focus, correctly refused UNSUPPORTED by gold. Corrected the probe to ordinary clipboard write and native compositor focus; no app expectation was weakened. Native.dwBKOc passed gold notice/clearing but candidate returned UNKNOWN_COMMAND for term.show. All six/six/six/nine tracked PIDs respectively stopped.

Concrete open Bus gap: gold term.show restores/focuses its own compositor window and returns shown plus target; candidate has no term.show command. This is required parity work, not a proposed deviation. Gold implementation is libs/citizen/src/lib.rs show: native wait, app-id and PID lookup, generation-bound restore, then raise/focus; no viewport-focus substitution. Port through existing Mix jobs/native Bus ownership and test matched refusals/results. Final native.W8jl8A uses the documented native compositor alternative for both apps and does not claim term.show passed.

Next CBC term-notices-exact-7b97f073 at exact 7b97f07344d0cc1fffd44a093b470887069f7311. Read complete saved log/require no worker diff, fetch verified bytes and repeat notice proof. Continue term.show/full Bus, native zoom-unsaved row, transfer-limit/busy/timeout cases, program mouse/word/line/cancellation, IME/search/full Prefs, enforcement/conformance/long-idle grant-pool cases and scoped release visual acceptance. NOT ACCEPTED. Gold and desks unchanged; no Fable question.

branch: lab/parity-term; head SHA: 7b97f07344d0cc1fffd44a093b470887069f7311 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — exact notice repeat, term.show and remaining full Bus/input/Prefs/enforcement/conformance/release acceptance open; CBC job ids: term-clipboard-notices-050e9350-a1 (green; fmt applied; native refusal/clearing captures zero differences), term-notices-exact-7b97f073 (next await).

### Exact notice witness; Mix window activation checkpoint

CBC term-notices-exact-7b97f073-a1 passed exact 7b97f07344d0cc1fffd44a093b470887069f7311 with no worker diff: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, eight Mix suites, dialog/search, foreign gate and debug build. Complete 49758-byte saved log read in three chunks. Fetched and verified exact debug SHA256 f1a5d9c12def5989ed5022b5f015df20769bd1f67366449634b0c1d3162177f6. Native.d1YCMr repeated actual gold/candidate clipboard refusal and successful paste clearing; all nine tracked processes stopped. Baseline/refusal/cleared 900x560 captures retain the preceding checkpoint hashes; all full/header/tabs/notice/terminal SSIM and RMSE distortions zero, with the same rectangles and ImageMagick distortion convention. Originals, montages and differences inspected: no differences. Private witness.json records exact source/bytes/profile and closure. Debug proof only.

Code bdf9de1a9bd17277b941375e9aa24749bedd4af7 implements term.show in Mix through the existing bounded Jobs/Calls facilities. Shared reusable window-show.mix follows pinned citizen::show_with: wait mapped (10s compositor budget), list windows, select the first matching real app-id and host PID, restore its exact id/generation, require minimized false, then raise/focus and require focused true. Returns the real shown/target response or gold ACTIVATION/show_failed refusal. Each native compositor call has the gold 15s bound. User body cannot supply compositor, app-id, pid or target. A held native reply is completed once from its matching job owner/id; cancelled receipts retire the waiting map, with bounded native work retaining ownership until completion/drop. No application window policy moved into Rust and no transport was added.

Shared launch code now resolves one actual window app-id for both the viewport and injected app_launch context, including service overrides and explicit app-id. Term uses its resolved compositor and actual host PID. The manifest declares term.show, its descriptor and existing-instance activation; launch callers allow the pinned gold twenty-second show budget. A second launch should activate rather than create another session, but native proof is pending. App-id override/service tests added to the CBC gate. Nine Mix suites pass locally. New show policy tests verify exact stage order/arguments, app-id plus PID ownership, generation preservation, missing windows, stale restore, unsuccessful focus, transport failure and native completion/refusal shape. CBC/runtime/native show and launch-repeat tests remain pending.

Next CBC term-window-show-bdf9de1a runs term_gate --prepare at code above, now including launch regressions and nine Mix suites. Read complete saved log, apply formatting/lock and exact-repeat if needed. Fetch verified bytes and run gold/candidate term.show from another focused app, minimized restore, stale/refused compositor replies, caller cancellation, --comp/--service targeting and second-launch PID/session invariance. Update the missing-show case only after actual native proof passes. Continue native zoom-unsaved row, transfer limits/busy/timeout, mouse reporting/word/line/cancellation, IME/terminal search/full live Prefs, full Bus/topics/errors/enforcement/conformance, long-idle grant-pool and scoped release visual acceptance. NOT ACCEPTED; no install, gold or desk changes and no Fable deviation.

branch: lab/parity-term; head SHA: bdf9de1a9bd17277b941375e9aa24749bedd4af7 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — window-show/activation gate and native proof, remaining full Bus/input/Prefs/enforcement/conformance/release checks open; CBC job ids: term-notices-exact-7b97f073-a1 (green exact; native repeat zero differences), term-window-show-bdf9de1a (next await).

### Worker-compatible activation test repair

CBC term-window-show-bdf9de1a-a1 failed before Rust checks: eight existing Mix suites passed, then the new show test hit a parse error at global declarations. Read the complete 2722-byte saved log; no worker diff. The worker-installed Mix predates the local global syntax used only by this test.

Code afe888d652bb97f95a24054330a3165ebad2d480 replaces that test-only global mutation with an owned mktemp fixture recording ordered compositor requests and remaining replies. The same success, ownership, exact stage ordering, stale-generation, refusal and completion assertions remain. Cleanup runs in finally. Local lint and the full show test pass. No application, host, transport or expected outcome changed; no worker software upgrade was attempted.

Next CBC term-show-fixture-afe888d6 runs term_gate --prepare at that exact code. Read the complete log; repair failures and apply returned changes, then require an exact repeat before native activation acceptance. Existing exact notice evidence remains valid at 7b97f073; new window-show/activation, runtime, native targeting/second-launch proof and all previously recorded full parity/release cases remain open. NOT ACCEPTED; gold/desks unchanged.

branch: lab/parity-term; head SHA: afe888d652bb97f95a24054330a3165ebad2d480 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — worker-compatible show gate and native activation proof, remaining full parity/release checks open; CBC job ids: term-window-show-bdf9de1a-a1 (test-only global syntax parse failure; full log read; no diff), term-show-fixture-afe888d6 (next await).

### Activation descriptor repair

CBC term-show-fixture-afe888d6-a1 passed all nine Mix suites, including the worker-compatible ordered show fixture, and compiled Viu. The real bundle-load test then refused the new show descriptor because CommandDescriptor requires args. Read the complete 9972-byte saved log; applied its launch/native-bus formatting return.

Code 27f1a82644f433424985bd30ab77e8a2d54fc72e adds the required empty args array for the no-argument term.show descriptor. Confirmed the exact descriptor schema in libraries/bus/src/protocol.rs. Local show tests and diff check pass; no assertion changed. Remaining gate stages and native activation checks have not passed yet.

Next CBC term-show-descriptor-27f1a826 runs term_gate --prepare at that exact code. Read complete log, repair failures/apply returned changes and require an exact repeat. Then fetch verified bytes and exercise term.show, minimized restore, service/compositor overrides, refusals/cancellation and second-launch PID/session invariance against gold. All earlier full parity/release open items remain. NOT ACCEPTED; no gold, install or desk changes.

branch: lab/parity-term; head SHA: 27f1a82644f433424985bd30ab77e8a2d54fc72e (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — activation descriptor gate and native proof, remaining full parity/release checks open; CBC job ids: term-show-fixture-afe888d6-a1 (nine Mix suites passed; bundle descriptor args missing; full log read; fmt applied), term-show-descriptor-27f1a826 (next await).

### Activation launcher regression repair

CBC term-show-descriptor-27f1a826-a1 passed all nine Mix suites, both real bundle-load checks, thirteen runtime/call/timer checks and 188 terminal tests (five existing ignores). The launch suite then failed an incorrect new assertion: Monitor explicitly declares dev.mixos.monitor, so its service override must preserve that window app-id. Read the complete 35080-byte saved log in two chunks; no worker diff. No debug artifact was emitted by this failed gate.

Code 06ff3077e731c4bcafe3b67e7107cc1c3720e757 corrects the declared-id assertion and checks the service-derived default separately against the Term bundle before applying an explicit override. The production resolution policy is unchanged: explicit window identity wins, otherwise use mixviu plus the resolved service. This retains checks on the injected Mix launch context as well as native app-id resolution. Diff check passes; Rust tests remain CBC-only.

Next CBC term-show-launch-06ff3077 runs term_gate --prepare at that exact code. Read the complete saved log, repair any failures and apply returned changes, requiring an exact repeat when necessary. Then fetch verified bytes for native term.show/minimized restore, compositor/service targeting, refusal/cancellation and second-launch host/session invariance against gold. Earlier full input/Bus/Prefs/enforcement/conformance and final release visual acceptance items remain open. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: 06ff3077e731c4bcafe3b67e7107cc1c3720e757 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — activation launcher gate/native proof and remaining full parity/release items open; CBC job ids: term-show-descriptor-27f1a826-a1 (nine Mix suites, two bundle, thirteen runtime and 188 terminal checks passed; incorrect launcher expectation; full log read; no diff), term-show-launch-06ff3077 (next await).

### Native show witness and repeated-launch inventory correction

CBC term-show-launch-06ff3077-a1 passed exact 06ff3077e731c4bcafe3b67e7107cc1c3720e757 without worker diff: nine Mix suites, two bundle checks, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher tests, seven input tests, dialog/search checks, foreign gate and debug build. Complete 52533-byte saved log read in three chunks. Verified debug binary SHA256 2eea193b53a169204c44302c0efcd390dfec220bd87d814ea974e241d0cdad99. Strict Mix/mixd dependency tiers remain clean.

Private native.uHavh6 passed real gold/candidate term.show with ignored body targeting and generation-bound restore/focus, then native minimize followed by term.show restored/focused the same window. Clipboard refusal and success clearing still pass. All nine tracked processes stopped. Three fresh matched 900x560 captures of baseline/refusal/cleared sessions are byte-identical. All full/header/tabs/terminal and refusal-notice SSIM/RMSE distortions are zero (ImageMagick 7.1.2-35, zero identical). Same rectangles and PNG hashes as the exact notice checkpoint: baseline/cleared e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865; refusal 87f8f03b1d6be4b6580ec1bf95c63fc4d9aa7bb015ce53214a7db6fba789d794. Originals, montages and differences inspected; no region differences. Private witness includes executed script, exact artifact receipt, calls, comparisons and closure. The script's inherited printed result mentioned second-launch invariance, but that block was removed before this run; witness explicitly corrects the label. No repeat-launch claim derives from this run. Debug witness only, not release acceptance.

Gold inventory correction: native.riTOpe disproved the preceding single-instance assumption. A repeated gold invocation stays alive as a separate window and reports serving as term-<pid> after the base registration is refused. Pinned apps/term/src/main.rs and apps/term/crates/term-core/src/bus.rs confirm that startup does not call show: it retries a refused base registration once using the PID suffix. A second refusal or a transport/authentication failure is not another fallback rung. This supersedes earlier notes proposing second-launch activation and unchanged parent/shell identity. Gold wins; no Fable deviation proposed.

Code 1891af2f3f11959937d19ee4be7fdf6d0a39a163 removes the incorrect term activation manifest entry and corrects provenance. term.show remains the independent window command. The generic host's registration-refusal collision-name path still needs implementing and native multi-window proof; removing activation alone does not complete that behaviour. Existing generic activation support remains available to apps whose gold contract uses it.

Next CBC term-show-gold-launch-1891af2f checks the corrected manifest at exact code above. Read complete log/no worker diff, fetch verified artifact for native repeat if needed, then implement gold's registration-refusal-only PID suffix ladder with command namespace/application identity preserved and real two-window/session/cleanup proof. Native show refusal/cancellation, zoom-unsaved notice, clipboard transfer limits/busy/timeout, reporting/word/line/cancellation, IME/grid search/full live Prefs, full Bus/topics/errors/enforcement/conformance, long-idle grant pool and final release visual acceptance remain open. NOT ACCEPTED; no install, gold or desk changes.

branch: lab/parity-term; head SHA: 1891af2f3f11959937d19ee4be7fdf6d0a39a163 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — gold collision-name fallback, full Bus/input/Prefs/enforcement/conformance and release acceptance open; CBC job ids: term-show-launch-06ff3077-a1 (green exact; native show and three zero-difference pairs), term-show-gold-launch-1891af2f (next await).

### Refused-registration PID fallback implementation checkpoint

CBC term-show-gold-launch-1891af2f-a1 passed exact 1891af2f3f11959937d19ee4be7fdf6d0a39a163 without worker diff. Complete 52534-byte saved log read in three chunks: nine Mix suites, two bundle checks, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher tests, seven input checks, dialog/search, foreign gate and debug build. Artifact exists at the worker's term-artifacts source directory; it has not been fetched or claimed as a new native witness.

Code 7b1599af6aabb552668afa85253cfa1f6133cad8 adds app-authored registration_pid_fallback, enabled only in Term's contract. The reusable native adapter attempts the base name and, only following an initial registration rejection, retries exactly once as base-PID. Both attempts have the pinned gold two-second deadline. Second refusal, transport, authentication and timeout errors preserve their own diagnostic and do not earn further names. The selected supervised client owns reconnects; no reconnect reruns this ladder. With the opt-in policy, failed initial Bus setup leaves the window alive as gold's graphics-only path; other apps retain their existing close-on-failure behaviour.

Wire command descriptors are renamed with the actual selected service, and incoming commands map that exact dot-delimited prefix back to the original Mix handler namespace. Initial bus.connected reports the selected service. This is native transport adaptation; Term tab/pane/actions remain Mix. Added CBC regressions for base success, one refusal, second refusal, disabled policy, non-registration failures and namespace dot boundaries. No native broker, external transport or daemon added. Diff check and Mix gate lint pass locally; Rust compilation and tests remain CBC-only.

Next CBC term-registration-7b1599af runs term_gate --prepare at this exact code, now including native_registration tests. Read complete saved log, repair failures/apply worker fmt, then exact repeat if changed. Fetch verified matching artifact and drive actual gold/candidate two-window registration collision: fallback HELP descriptors and snapshot/type/show/UI commands under term-PID; independent native parents/shells; close one without affecting the other; second refusal and unavailable broker graphics-only cases. Audit suffixed service identity in menus, macro/outbox/topics/full Bus replies before calling fallback parity complete. No new native acceptance is claimed at this checkpoint.

Earlier native.uHavh6 show/minimized restore and zero-difference captures attest only 06ff3077 debug bytes. Native show cancellation/refusals, complete global Bus surface/topics/errors/enforcement/conformance, clipboard transfer limits/busy/timeout, zoom-unsaved notice, mouse reporting/word/line/cancellation, IME/grid search/full live Prefs, long-idle grant pool and final release visual acceptance remain open. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: 7b1599af6aabb552668afa85253cfa1f6133cad8 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — fallback CBC/native/full identity audit and remaining full parity/release checks open; CBC job ids: term-show-gold-launch-1891af2f-a1 (green exact; complete log read; no diff), term-registration-7b1599af (next await).

### Two-window native witness and selected-service routing repair

CBC term-registration-7b1599af-a1 passed nine Mix suites, two bundle checks, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher tests, three registration ladder/namespace tests, seven input tests, dialog/search, foreign gate and debug build. Read the complete 63293-byte saved log in four chunks. Applied its formatting-only return in c90ea9d72cbeb10480fd992ef2a6a63da8f889fb; exact repeat pending. Verified worker-formatted 7b1599af debug executable SHA256 1318939350a4f89be54886006f21bc1e941d8dad68151109b0b19c8fbd3134bb.

Private native.eHaAXB passed actual gold/candidate simultaneous-window checks: the first owns term, the second term-PID; suffixed HELP describes its snapshot command; distinct native parents and Mix shells are attached; each window executes the same pair() output; show matches each host PID and compositor generation; suffixed native UI capture works. Closing the second stops its host/shell and leaves the first host/shell unchanged and accepting new command output. All eleven tracked processes stopped. Both gold and candidate logged unconfirmed parent revoke and existing lease/window expiry on close; this witness proves process cleanup, not immediate broker revocation.

The first and second window pairs are both byte-identical 900x560 captures, Studio Dark/Inter400/14.667/scale1, existing deterministic real-Mix command. All full/header/tabs/terminal SSIM and RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, full montages and differences inspected: no region differences. Private witness.json records artifact, executed script, native calls/discovery, comparisons and cleanup. Diagnostic debug evidence with returned formatting; not exact release acceptance.

The service-identity audit found static canonical names in native UI advertisement and Mix change-topic replies/publication. Code 08b17569a63561da033a34f50f2860d1bee07ff3 makes native UI use the actual supervised service, and Mix retain bus.connected's selected service for props.watch and change publication. Pure Mix regression checks suffix watch topics and exact namespace boundaries; bus-test and event-job lint pass locally. Code dce0310e08bef48e676149ea0a40a11581b070b8 also includes the actual service in every reconnect notice, preventing Mix from reverting its topic namespace after reconnect. These routing changes are not yet native-attested.

Next CBC term-registration-routing-dce0310e checks exact code above, including applied formatting. Read complete saved log, apply any returned changes and exact-repeat if needed. Private prepared fetch script and registration-dce0310e harness repeat two-window native cases plus each window's props.watch and titlebar service readbacks. Continue actual topic subscriptions/delivery, hosted menu execution, macro/outbox/service identity and whole HELP/app.describe/global Bus parity; real second-refusal/unavailable broker graphics-only/reconnect cases remain open. Do not duplicate earlier diagnostic artifact builds.

Remaining full parity/release items from prior checkpoints still bind: native show cancellation/refusals, clipboard limits/busy/timeout, zoom-unsaved row, mouse reporting/word/line/cancellation, IME/grid search/full live Prefs, full topics/errors/enforcement/conformance, long-idle grant pool and final scoped release visual matrix. NOT ACCEPTED; no install, gold or desk changes, no Fable deviation proposed.

branch: lab/parity-term; head SHA: dce0310e08bef48e676149ea0a40a11581b070b8 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — routing exact gate/native repeat, full Bus/input/Prefs/enforcement/conformance and release acceptance open; CBC job ids: term-registration-7b1599af-a1 (green; fmt applied; diagnostic two-window native proof and zero differences), term-registration-routing-dce0310e (next await).

### Clipboard fixture descriptor isolation repair

CBC term-registration-routing-dce0310e-a1 failed with rc101. Read the complete 32375-byte saved log in two chunks; no worker diff and no artifact emitted. Nine Mix suites (including selected-service topic regression), two bundle tests and thirteen runtime/call/timer checks passed. Terminal suite: 187 passed, five existing ignores, one failure in reads_are_one_at_a_time_bounded_and_time_out. Read id4 returned TimedOut instead of pasted-newline. Remaining gate stages did not run. This is a red test result, not a queue/admission failure.

Inspection found the test fixture creates libc::pipe without CLOEXEC, unlike the production smithay-client-toolkit 0.19.2 receive path, which uses rustix pipe_with(PipeFlags::CLOEXEC). The scoped suite concurrently launches real child processes; inheritance of a fixture writer can postpone EOF after its local owner closes it. That is an inferred cause of this observed intermittent failure; no production regression is established by this log.

Code c727969315cab9fb0819df7e1c38e0a5ea438c82 makes the fixture create both ends atomically with pipe2(O_CLOEXEC), matching the production pipe contract. Ordinary read, single-reader Busy, actual oversized pipe transfer and timeout assertions are unchanged, including their five-second budgets. No ignore, serialisation, retry loop or production timeout change. Diff check passes; Rust checks remain CBC-only.

Next CBC term-clipboard-fixture-c7279693 runs term_gate --prepare at exact code above. Read the complete log, repair actual failures/apply returned formatting, require an exact repeat if source changes, then fetch the artifact from the actual reported worker. Use the prepared registration-dce0310e harness with its artifact path/source updated to the new verified bytes: repeat both-window parent/shell/show/capture/type/close checks plus selected-service props.watch and titlebar readbacks. The older native.eHaAXB witness remains valid only for its worker-formatted 7b1599af diagnostic artifact. All full Bus/topics/menu/macros/reconnect/refusal/graphics-only/input/Prefs/enforcement/conformance and release acceptance open items remain. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: c727969315cab9fb0819df7e1c38e0a5ea438c82 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — descriptor-isolated clipboard gate, native selected-service repeat and remaining full parity/release checks open; CBC job ids: term-registration-routing-dce0310e-a1 (nine Mix suites, two bundle and thirteen runtime checks passed; clipboard pipe fixture timeout; full log read; no diff), term-clipboard-fixture-c7279693 (next await).

### Exact selected-service witness; authored discovery checkpoint

CBC term-clipboard-fixture-c7279693-a1 passed exact c727969315cab9fb0819df7e1c38e0a5ea438c82 with no worker diff. Complete 52912-byte saved log read in three chunks: nine Mix suites, two bundle tests, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher and three registration tests, seven input tests, dialog/search, foreign gate and debug build. The clipboard pipe regression passes with unchanged deadlines. Verified exact debug SHA256 e03318875820f61252ef35c6bb299a4d13bdd385c5e627e96c1c44b0287647c9.

Private native.1tuEgl repeated two-window native parent/shell independence, actual suffixed snapshot/type/show/capture and independent close/survivor input on those exact bytes. Both windows report props.watch topics in their actual service namespace. All eleven tracked processes stopped. First/second matched 900x560 pairs are byte-identical, same settings/command/regions as native.eHaAXB: all full/header/tabs/terminal SSIM/RMSE distortions zero (ImageMagick 7.1.2-35; zero identical). All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, montages and differences inspected; no region differences. Private witness.json records exact bytes, calls, readbacks and closure. Debug witness only.

Retained failed probes native.eLacC0 and native.NtWP1z incorrectly treated ui.titlebar.form.service as a readback of drive's MenuService registration. The second window reports null/hosted=false in this CSD hosting form on BOTH gold and candidate; this is not a discovered gold defect or approved deviation. Corrected the probe to preserve the CSD readback and check routing through actual suffixed commands/watch topics. Seven/eleven tracked processes stopped respectively. Actual hosted menu execution and topic subscription delivery remain open.

The same oracle probe found gold app.describe rc0 with schema/app_id/commands, versus candidate UNKNOWN_COMMAND. Code dc9fbd01319e78f62f5c75c1cfd2934b2902b105 adds immutable pinned discovery data and Mix-owned handlers: full HELP (56 descriptors), lowercase help (core-only 36 descriptors), app.describe (53 command entries), INFO/info/term.info usage and the port's own actual instance/PID. Names are rewritten only at exact command namespace boundaries; descriptors preserve gold order, argument shapes, read-only flags and descriptions. INFO's canonical command occurrences follow the actual service. This is a discovery implementation checkpoint, not proof of every advertised command's behaviour.

The generic host gains an opt-in authored_discovery switch so the SDK does not pre-empt Mix's HELP reply. Term enables the existing shared native macro session for its advertised macro commands. Explicit window_app_id chooses dev.mixos.term for both viewport and injected launch context, including service overrides; the shared resolver retains legacy/default behaviour for other apps. Launcher regression now checks declared identity, fallback and explicit override separately. Ten local Mix suites pass, including discovery schema/namespace/identity checks. Actual bundle tests now invoke HELP/app.describe/INFO. No runtime PID/version was copied from gold; immutable contract data came from the pinned native oracle and INFO source literal. Provenance records sources and catalogue hash.

Next CBC term-discovery-dc9fbd01 runs term_gate --prepare at that exact code. Read complete log, repair failures/apply returned fmt/lock, then exact-repeat if needed. Fetch from the actual reported worker (prepared fetch script's worker must match), verify bytes/source/profile and run discovery-dc9fbd01 native harness. Compare complete HELP/help/app.describe replies on first and fallback services after canonicalising only service names; compare INFO/info/term.info layout/text while checking each process's own identity. Repeat two-window/watch/show/captures/cleanup. Full hosted menus/macros/outbox/topics, raw payload/refusal/enforcement/conformance, broker failure/reconnect and remaining input/Prefs/search/release acceptance items still bind. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: dc9fbd01319e78f62f5c75c1cfd2934b2902b105 (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — discovery gate/native matching and remaining full Bus/input/Prefs/enforcement/conformance/release acceptance open; CBC job ids: term-clipboard-fixture-c7279693-a1 (green exact; native selected-service repeat and zero differences), term-discovery-dc9fbd01 (next await).

### Discovery reply test representation repair

CBC term-discovery-dc9fbd01-a1 failed rc101 after all ten Mix suites passed and Viu compiled. Read the complete 10697-byte saved log. The actual bundle loaded and HELP returned rc0; the new Rust assertion then called as_str on its already-decoded array. Confirmed Runtime's reply adapter parses valid JSON into typed Json before storing Completion.reply. This was a test representation mistake, not an application refusal. Later scoped checks and artifact build did not run.

Code 5c69b0becfa305b71e72f8bde890df757681808c checks HELP's typed array and app.describe's typed object directly, retaining rc, term.show descriptor/flag, schema/app-id and actual INFO PID assertions. Applied the returned app-host/test formatting in the same code checkpoint. No application policy, expected Bus payload, descriptor or timeout changed. Diff check passes.

Next CBC term-discovery-typed-5c69b0be runs term_gate --prepare at that exact source. Read complete log, repair failures/apply formatting and exact-repeat if needed. Fetch the reported worker's verified artifact; adapt the prepared discovery-dc9fbd01 harness/fetch to the tested SHA before native comparison. Match complete HELP/help/app.describe and INFO identity formatting on canonical and fallback services; repeat unchanged capture/watch/show/two-window cleanup. The exact c7279693 native witness remains the latest completed baseline; discovery and all previously recorded full parity/release items remain open. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: 5c69b0becfa305b71e72f8bde890df757681808c (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — typed discovery gate/native comparison and remaining full parity/release checks open; CBC job ids: term-discovery-dc9fbd01-a1 (ten Mix suites passed; bundle HELP test decoded-array assertion mistake; full log read; fmt applied), term-discovery-typed-5c69b0be (next await).

### Discovery module import repair

CBC term-discovery-typed-5c69b0be-a1 passed all ten Mix suites, the real bundle HELP/app.describe/INFO assertions, thirteen runtime/call/timer checks and 187 terminal tests. Read complete 32605-byte log in two chunks; no worker diff. One native owner/control integration test failed: its embedded Mix reducer imports the real Bus policy from a different program entrypoint, and discovery's read_file of bus-contract.json resolved from that entrypoint rather than the module. The reducer load fault caused the control request to refuse FORBIDDEN. No authority assertion was removed; the catalogue import must work in this reusable policy path. No artifact or later gate checks ran.

Code abfb41c50000d60fcad593f6de2f4f9a29ced51b converts immutable bus-contract.json into bus-contract.mix and imports its catalogue using relative require. data_encode preserved every value exactly (deep equality checked before removing the JSON form); the module now requires no entrypoint-relative file I/O. Provenance destination/hash updated. Local discovery and Bus suites plus catalogue lint pass. A private probe whose entrypoint and working directory are outside the app imports the full nested Bus policy/catalogue and passes. This fixes module use in both app and embedded native recipient paths without changing any authority, descriptor or diagnostic expectation.

Next CBC term-discovery-import-abfb41c5 checks term_gate --prepare at this exact source. Read complete saved log, repair failures/apply fmt and require exact repeat if changed. Then fetch verified matching bytes from the reported worker and update the prepared native discovery harness's artifact source. Compare canonical/fallback HELP/help/app.describe and INFO own-identity formatting, repeat two-window/watch/show/captures/cleanup. The latest exact native baseline is still c7279693; discovery native and all earlier full parity/release open items remain. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: abfb41c50000d60fcad593f6de2f4f9a29ced51b (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — module-safe discovery gate/native matching and remaining full parity/release checks open; CBC job ids: term-discovery-typed-5c69b0be-a1 (ten Mix suites, bundle discovery and 187 terminal tests passed; embedded catalogue path failure; full log read; no diff), term-discovery-import-abfb41c5 (next await).

### Exact native discovery witness; namespace and text-wire checkpoint

CBC term-discovery-import-abfb41c5-a1 passed exact abfb41c50000d60fcad593f6de2f4f9a29ced51b with no worker diff. Complete 53599-byte saved log read in three chunks: ten Mix suites, two real bundle tests including discovery, thirteen runtime/call/timer checks, 188 terminal tests (five retained ignores), three launcher and three registration tests, seven input tests, dialog/search, foreign gate and debug build. Verified debug binary SHA256 436c81936a0862d13ca5092887f27752cb2d572d6988843d4e96b8c623fbfad6.

Private native.lQTiN0 matched all twelve discovery comparisons: HELP/help/app.describe/INFO/info/term.info on both canonical and fallback services. Whole replies retain descriptor order, argument shapes, descriptions, flags, schema and app-id; only service prefixes and final live instance/PID values were normalised. Each reported PID independently matched its actual spawned host. All twelve per-command comparison files and raw native replies retained. Two-window independent parents/shells, suffixed snapshot/type/show/capture, watch topics and close/survivor input also passed; all eleven tracked processes stopped.

First/second real-session 900x560 pairs are byte-identical with matched settings and command output. Full/header/tabs/terminal SSIM/RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Rectangles unchanged: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, montages and differences inspected; no region differences. Private witness.json records exact source/bytes, twelve comparisons, readbacks and cleanup. Debug acceptance of these cases only, not final release/full Bus acceptance.

Code e10885b2fa8cfea189f20b40a1c6c24e4e3a0eda adds app-authored command_namespace, text_replies and unknown_reply contract data. The reusable native adapter maps only the actual registered prefix into the authored namespace; another instance's prefix is refused before invoking its reducer. The same mapping covers explicit generic service overrides. Reserved generic host controls retain their route. Unknown reply shape comes from app data. Text replies emit original string bytes rather than extra JSON quotes, while structured maps/arrays retain JSON. Other apps retain defaults. This follows pinned core canonical_command/dispatch/plain peer.reply behaviour; no authority or replay gate relaxed.

Added native Bus regressions cover fallback/override namespace mapping, dot-boundary refusals, framework controls, exact multiline/quoted/Unicode text bytes, and preserved structured JSON/default encoding. The scoped gate now includes native_bus tests. Diff check and Mix gate lint pass locally. CBC/native namespace refusals and raw-wire acceptance are pending. Exact raw request-envelope size/parsing/duplicate-key behaviour remains open; no full wire parity claimed by this patch.

Next CBC term-wire-policy-e10885b2 runs term_gate --prepare at exact code above. Read full saved log, repair failures/apply fmt and exact-repeat if changed. Fetch verified bytes from the reported worker; prepared wire-e10885b2 harness repeats discovery and two-window captures and records unknown/foreign-prefix read and mutation refusals. Compare refusal bodies with gold, verify no foreign request mutates another instance, and attest raw encoding through native transport. Continue service overrides, macro/hosted-menu/outbox/topic delivery, broker refusal/graphics-only/reconnect, full payload/errors/enforcement/conformance, remaining input/Prefs/search and release visual acceptance. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: e10885b2fa8cfea189f20b40a1c6c24e4e3a0eda (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — namespace/text gate/native wire verification and remaining full Bus/input/Prefs/enforcement/conformance/release acceptance open; CBC job ids: term-discovery-import-abfb41c5-a1 (green exact; twelve matched native discovery replies and zero-difference captures), term-wire-policy-e10885b2 (next await).

### Native namespace witness and original-envelope repair

CBC term-wire-policy-e10885b2-a1 passed e10885b2fa8cfea189f20b40a1c6c24e4e3a0eda. Read the complete 58768-byte saved log in three chunks: ten Mix suites, two real bundle tests, thirteen runtime/call/timer checks, 188 terminal tests (five retained ignores), three launcher and three registration tests, four native Bus adapter tests, seven input tests, dialog/search, foreign gate and debug build. Applied the returned formatting-only diff in f6998503; exact source repeat is included in the next gate. Verified diagnostic debug binary SHA256 4ef0550f7d992f2dee1f3a254b81a70144641ab02ed71a051229d46b15aae62b.

Private native.ORH31y matched all eight unknown/foreign-prefix read and mutation refusals across canonical and fallback services: rc10 and the exact gold text unknown command; use HELP. The subsequent two-window native state/output checks show no extra tab or changed shell. All twelve discovery replies still match after the documented service/live identity normalisation. Distinct attached native parents and Mix shells, selected-service watch topics, PID/generation-bound show/capture, independent close and survivor input passed. All eleven tracked processes stopped. Both gold and candidate retain the existing unconfirmed parent-revoke diagnostic; broker expiry remains unverified.

The two real-session pairs are byte-identical 900x560 captures at matched Studio Dark/Inter400/14.667/scale1 and deterministic Mix output. All four PNG hashes e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM and RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Rectangles full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. All four originals and both full montages/differences inspected; no region differences. witness.json retains source/artifact, eight full refusal comparisons, twelve discovery comparisons, figures and cleanup. This is diagnostic debug evidence with formatting applied afterwards, not exact release acceptance.

Private native.ncw3tv records the next raw-body probe on those same tested bytes. It confirms mismatches for empty no-argument requests, malformed JSON diagnostics and bare INFO with a non-JSON body. Both sides already refuse non-object JSON and permit a corrected request after target validation. All eleven tracked processes stopped. The existing ABP decoder trims trailing whitespace before delivery: whitespace-only and oversized-whitespace-only bodies reach gold as empty. This is a frozen shared transport behaviour, not a proposed change. The next harness includes oversized leading whitespace before an object and oversized Unicode inside JSON, whose bytes survive framing.

Code a6a647a42938479e1c2fb123e588f8279fe8b66d adds the opt-in preserve_request_body contract flag. The reusable native adapter carries the delivered body; the existing bounded runtime mailbox can pass it unchanged to Mix, preserving malformed input, leading whitespace and duplicate keys. Native menu argument remaps update their derived body. Existing app defaults retain typed dispatch. Term Mix owns byte-budget admission before parsing/replay, empty-body handling by required field, serde parse error text, object checks, and parsed-key retry semantics. Bare INFO/info ignore body contents like gold while namespaced term.info validates. No app policy moved to Rust and no authority/replay assertion was relaxed.

Local wire and Bus suites pass, including exact 8192/8193-byte and multibyte envelopes, malformed/non-object/required-empty bodies, duplicate-key last value, parse/budget refusals preserving state/cache, corrected request and semantic retry. Runtime integration now checks delivered raw bodies, parsed args, caller and malformed-body survival. A real embedded-noded native adapter regression checks request envelope preservation, unquoted multiline/Unicode response bytes, structured refusal JSON and foreign-prefix rejection. These Rust checks are pending CBC; no local Rust execution.

Next CBC term-raw-envelope-a6a647a4 runs term_gate --prepare at that exact code, including f6998503 formatting. Read the complete saved log, repair actual failures/apply formatting and exact-repeat if needed. Prepared term-envelope-fetch.mix must use the reported worker; verify artifact source/profile/hash and run raw-a6a647a4.mix. Compare raw response cases with gold, preserving parser details and checking original-byte oversize cases plus corrected-request replay on both service identities. Repeat discovery/refusals/two-window captures and cleanup. Native transport unit checks and all end-to-end repaired cases remain unverified until that gate/run.

Full parity still open: exact u64/i64 JSON payloads, all Bus errors and ordering including oversized unknown commands, raw macro/debug/UI argument handling, actual topic delivery/hosted menus/macros, broker refusal/graphics-only/reconnect, show cancellation/refusals, input/IME/grid search/full Prefs, clipboard provider limits, enforcement/expiry/conformance and final scoped release visual matrix. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

branch: lab/parity-term; head SHA: a6a647a42938479e1c2fb123e588f8279fe8b66d (code; receipt HEAD contains this checkpoint); NOT ACCEPTED — delivered-envelope CBC/native proof and remaining full parity/release checks open; CBC job ids: term-wire-policy-e10885b2-a1 (green; fmt applied; native namespace/discovery/capture witness), term-raw-envelope-a6a647a4 (next await).

### Native envelope proof; formatting exact repeat

CBC term-raw-envelope-a6a647a4-a1 passed code a6a647a42938479e1c2fb123e588f8279fe8b66d. Complete 61853-byte saved log read in three chunks: ten Mix suites, two actual bundle tests, fourteen runtime/call/timer tests, 188 terminal tests (five existing ignores), three launcher, three registration and five native Bus tests, seven input tests, dialog/search, foreign gate and debug build. The new real embedded-noded test verifies delivered original envelopes, unquoted multiline/Unicode reply bytes, structured refusal JSON and namespace rejection through actual transport. Native runtime preservation test passes. Strict mix/mix-shell/system/mixd remain clean. Only formatting in the two new test bodies was returned; applied and pushed as 6d517798858b5ba71997e3e8fd8d1eef7e01e15b. Exact source gate still required. Verified debug executable SHA256 1f02b2155c66e428fc1a7124c2a2c557ca01593efd5e15fb6c078e06332f114a.

Private native.TY889K matches 26/28 complete raw-case responses across canonical and fallback windows. Matched cases include empty read, whitespace-only (ABP-trimmed), malformed JSON exact diagnostic/location, arrays/null, original exact-size leading whitespace, oversized leading whitespace and multibyte JSON, bare INFO own live identity, empty required body, target-validation refusal and same-ID corrected request. Each full case comparison and raw call is retained. Eight namespace refusals and all twelve discovery comparisons also match. Both windows attach distinct native parents/Mix shells; selected-service watch/show/capture/input/close/survivor checks pass. All eleven tracked processes stopped.

The two complete duplicate-key snapshot comparisons remain FAILED. Both return the last contents=true value and identical header/screen after live PID normalisation, but the diagnostic blocks differ: gold exposes VT_notification_to_GPU_snapshot/gpu_prepare_cpu_side, candidate VT_notification_to_RGBA/raster_paint_into_texture, with actual sample/timing differences. The first comparison removed only the single DIAGNOSTIC heading line and still detects these differences. Full failed comparisons are preserved; a separately labelled projection attests the screen/header only. Next inspect real renderer selection and compare the same backend; do not invent GPU labels for CPU work or claim 28/28. No proposed deviation or Fable approval.

Both real-session capture pairs remain byte-identical 900x560 at matched Studio Dark/Inter400/14.667/scale1 and command output. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM and RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Four originals, both full montages and differences inspected; no region differences. witness.json retains source/profile/hash, all comparisons, limitations and closure. Debug proof with a test-format diff, not final release acceptance.

Next CBC term-envelope-exact-6d517798 checks term_gate --prepare at 6d517798858b5ba71997e3e8fd8d1eef7e01e15b. Read full saved log and require no diff. Prepared term-envelope-exact-fetch.mix currently targets cbc3; change it to the actual reported worker before fetching. Verify source/profile/hash, use raw-6d517798.mix for exact native repeat and retain the two diagnostic failures until resolved. Continue renderer selection/diagnostics, full numeric/payload/refusal matrix, actual topics/hosted menus/macros, broker refusal/graphics-only/reconnect, native input/IME/grid search/full Prefs, clipboard providers, enforcement/expiry/conformance and final release matrix. No install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: 6d517798858b5ba71997e3e8fd8d1eef7e01e15b (code; receipt HEAD records this checkpoint); NOT ACCEPTED — exact formatting repeat, two native snapshot diagnostic mismatches and remaining full parity/release checks open; CBC job ids: term-raw-envelope-a6a647a4-a1 (green; full log read; fmt applied; native 26/28 raw, 12 discovery, 8 refusal matches and zero-difference captures), term-envelope-exact-6d517798 (next await).

### Exact envelope witness and GPU diagnostics repair

CBC term-envelope-exact-6d517798-a1 passed exact 6d517798858b5ba71997e3e8fd8d1eef7e01e15b with no worker diff. Read complete 54102-byte saved log in three chunks: ten Mix suites, two bundle tests, fourteen runtime/call/timer tests, 188 terminal tests (five retained ignores), three launcher, three registration, five native Bus tests, seven input tests, dialog/search, foreign gate and debug build. Verified exact debug SHA256 23ebd05d07fbc1bb855db656c27d4a885d3c0c0262dbc643c91b45a228c99271.

Private native.dmPu9O repeats the exact-source native witness: 26/28 full raw responses, twelve discovery responses and eight namespace refusals match; duplicate-key screen/header projections match while both full snapshot diagnostic comparisons still fail. Actual parent/shell independence, selected-service watch/show/input/capture, close and survivor input pass. All eleven tracked processes stopped. Both real-session 900x560 capture pairs remain byte-identical, same Studio Dark/Inter400/14.667/scale1 and deterministic command output. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM and RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions unchanged: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. All four originals and both full montage/difference pairs inspected; no region differences. Private witness.json retains full comparisons and failures. Debug evidence only; no final release acceptance.

Source inspection corrects the prior renderer inference: the candidate ALREADY draws through Pane.update/callback and the real GPU renderer. Its terminal widget never invokes the imported Metrics.gpu_frame helper, so the native diagnostic retains gpu=false, CPU labels and zero handover samples. Gold app.rs records gpu_frame at its actual snapshot handover and collects the preceding renderer prepare duration. The prior CPU-label observation was accurate; concluding a CPU rendering path from it was incorrect. No Fable deviation is proposed.

Code 8916946c4c1e9e97bfd88a8439d6813392c8551c records each actual widget GPU snapshot handover, consumes a completed render-thread prepare duration once, and leaves unrendered UI frames without a fabricated prepare sample. The offscreen GPU pixel/dirty-row tests now check completed prepare publication and single consumption through actual callbacks. Existing real-widget/live-font checks verify GPU stage labels, handover count and absence of prepare samples when the fixture has not rendered. This is reusable widget instrumentation; Mix app policy is unchanged. Rust checks pending CBC, no local Rust execution.

Next CBC term-gpu-metrics-8916946c runs term_gate --prepare at that code. Read full saved log, apply returned formatting and exact-repeat if needed. Prepared term-gpu-fetch.mix currently targets cbc3: use the actual reported worker, verify bytes/source/profile, then run gpu-8916946c.mix. The harness additionally requires candidate snapshots after capture to expose GPU labels with positive handover and completed-prepare counts. Compare diagnostic field names/shape with gold while preserving actual runtime timings/counters; do not replace measurements with gold values. Retain historical 26/28 results. Continue full payload/numeric/refusal matrix, actual topic/menu/macro delivery, broker/refusal/reconnect, input/IME/search/full Prefs, clipboard providers, enforcement/expiry/conformance and final scoped release matrix. NOT ACCEPTED; no gold, desk, install or merge changes.

branch: lab/parity-term; head SHA: 8916946c4c1e9e97bfd88a8439d6813392c8551c (code; receipt HEAD records this checkpoint); NOT ACCEPTED — GPU instrumentation CBC/native proof, complete Bus/input/Prefs/enforcement/conformance and release checks open; CBC job ids: term-envelope-exact-6d517798-a1 (green exact, native 26/28 raw and byte-identical captures), term-gpu-metrics-8916946c (next await).

### GPU metrics native proof and replay-conflict wording

CBC term-gpu-metrics-8916946c-a1 passed code 8916946c4c1e9e97bfd88a8439d6813392c8551c. Read complete 57242-byte saved log in three chunks: ten Mix suites, two bundle tests, fourteen runtime/call/timer tests, 188 terminal tests (five existing ignores), three launcher, three registration and five native Bus tests, seven input tests, dialog/search, foreign gate and debug build. Real offscreen GPU tests verify completed prepare publication and single consumption; widget test verifies real GPU handover labels/counts without inventing samples before rendering. Returned formatting applied in badfcb7a35768da4518379c63aed649b7830b78c. Verified debug SHA256 3ad7e08eef6d90f188dc25083249eb2e1b213c2b9bb8104155bf8ad4c67b60d1.

Private native.wOC7SX now passes positive GPU handover/completed-prepare counts and GPU diagnostic labels in both candidate windows. All 28 raw-case replies match after documented live-data normalisation, plus twelve discovery and eight namespace refusals. Snapshot comparison validates exact diagnostic counter-field names/order, five gold GPU stage names and each sample format (compact n=0 or count/window/quantiles with valid bounds and p50<=p95<=max), then normalises only live counters/timings. It retains the entire grid and stable header. The two previous missing-GPU-label failures remain preserved; they were repaired by actual instrumentation, not renamed CPU timings. Measurements remain process-side, not presentation evidence.

Two independent native parents/shells, selected-service watch/show/capture/input and close/survivor cases passed; all eleven processes stopped. Both 900x560 real-session capture pairs remain byte-identical at matched Studio Dark/Inter400/14.667/scale1 and command output. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM/RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Four originals and both full montages/differences inspected; no differences. witness.json retains bytes/source/profile, full comparisons, raw diagnostics, normalisation, figures and closure. Diagnostic debug evidence with returned formatting; exact repeat and release acceptance pending.

The next refusal audit found the port emits a conflict-prefixed paraphrase where pinned ReplyCache::lookup returns the exact REQUEST_ID_CONFLICT string. Code c74949cdf2cc2ffc9ed1f851b9507f45f27e1e5a copies gold wording: request_id already used by a different request; retries must resend the identical command and body. Pure Mix tests now require exact wording for changed payload/command and preserve state/validation ordering; both changed suites pass. No replay/authority rule relaxed. This is a gold-wins inventory/implementation correction, not a proposed deviation.

Next CBC term-conflict-exact-c74949cd checks term_gate --prepare at c74949cdf2cc2ffc9ed1f851b9507f45f27e1e5a, including applied GPU formatting. Read complete log and require no worker diff. Fetch through the reusable private term-fetch-artifact.mix with full SHA and actual cbc2/cbc3 worker arguments. This avoids a prepared-script SHA replacement error that initially targeted a nonexistent artifact; no incorrect bytes ran. Verify source/profile/hash, then conflict-c74949cd.mix adds actual changed-payload and changed-command retry conflicts after a recorded success on both service identities. Compare exact rc/body and verify no synthetic text/new tab; repeat GPU/raw/discovery/refusal/capture/cleanup. Adapt raw comparison case count from fourteen to sixteen, keeping validated runtime normalisation. All full numeric/payload/refusal/topic/menu/macro/broker/input/IME/search/Prefs/clipboard/enforcement/expiry/conformance and final scoped release work remains. No Fable deviation, gold, desk, install or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: c74949cdf2cc2ffc9ed1f851b9507f45f27e1e5a (code; receipt HEAD records this checkpoint); NOT ACCEPTED — conflict exact gate/native repeat and full parity/release checks open; CBC job ids: term-gpu-metrics-8916946c-a1 (green; fmt applied; native 28 raw, 12 discovery and 8 refusal matches; zero-difference captures), term-conflict-exact-c74949cd (next await).

### Exact retry witness and numeric validation checkpoint

CBC term-conflict-exact-c74949cd-a1 passed exact c74949cdf2cc2ffc9ed1f851b9507f45f27e1e5a with no worker diff. Read complete 54102-byte saved log in three chunks: ten Mix suites, two actual bundle tests, fourteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher, three registration and five native Bus tests, seven input tests, dialog/search, foreign gate and debug build. Verified exact debug SHA256 cdbd552b1f5c15264baca9320e945222dce07d0927aec324b7935346e343cfeb.

Private native.gnIYuI matches all 32 raw/retry cases (including exact changed-command and changed-payload conflict replies before validation), twelve discovery replies and eight namespace refusals. Live diagnostic normalisation and validation are unchanged from native.wOC7SX. GPU labels and positive actual handover/prepare counters, distinct attached native parents/Mix shells, selected-service watch/show/capture/input and independent close/survivor checks pass. All eleven tracked processes stopped. Both 900x560 real-session capture pairs remain byte-identical at Studio Dark/Inter400/14.667/scale1 and matched command output. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM/RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Originals and both full montage/difference pairs inspected: no region differences. Exact debug witness only, not release/full acceptance.

The same run extends the validation probe to 58 cases per window. Complete replies match 45/58 on each window, with thirteen mismatches retained in full comparison records: null targets treated as absent; required text validation ordered after optional target validation; integral-valued JSON floats/exponents accepted as integers; u64 maximum selectors and tab-move index rejected. Cases and fixture hash are archived in validation-probe.json. No failed case discarded and no difference approved. This is the measured baseline for the next repair.

Code 080539c03acd7eddcc46421edf982b0bc13cca46 adds reusable viu_json_numbers metadata for top-level numeric envelope fields, preserving serde integer/float kind, exact decimal text, signed range and safe f64 range beside normal Mix values. The bridge uses existing Rust/serde facilities and adds no strict-tier dependency. Mix owns field presence, gold validation order, signed/unsigned domains, large selector materialisation and state operations. Large selectors become exact decimal text only after validation; reorder indices clamp against actual tab count. The terminal scroll primitive accepts canonical in-process decimal i64 text for exact large scroll values; external JSON string arguments still fail Mix validation. No authority moved to data or application policy moved to Rust.

Retry digests preserve integer/float distinctions and exact numeric text with a separate outer tag for ordinary JSON values. A caller-provided array/object cannot imitate numeric metadata to replay a different request. Pure Mix wire/Bus suites pass null/precedence/kind/range/materialisation/retry cases, including the tag-collision refusal. App lint passes with the new embedded function declared. Added actual-bundle raw numeric cases, runtime serde-boundary/duplicate-key metadata tests, and exact i64 scroll bridge checks; these Rust checks await CBC. No local Rust execution.

Next CBC term-numeric-envelopes-080539c0 checks term_gate --prepare at that exact code. Read complete log, repair real failures/apply fmt and exact-repeat if needed. Fetch via private term-fetch-artifact.mix with full SHA and actual worker. Run numeric-080539c0.mix: it retains prior raw/retry/capture cases, adds integer-vs-float and forged-tag retry conflicts, and repeats the validation matrix extended to 66 cases including valid i64 min/max scroll/page and out-of-range/negative-zero numbers. Compare all raw rc/bodies and verify no mutation on refusal; preserve own diagnostic readings. The latest completed native witness is c74949cd; the numeric repair is not yet native verified. Remaining full Bus/actions/topics/menu/macros/broker, input/IME/grid search/Prefs, clipboard providers, enforcement/expiry/conformance and final scoped release evidence still bind. NOT ACCEPTED; no Fable deviation, gold, desk, install or merge changes.

branch: lab/parity-term; head SHA: 080539c03acd7eddcc46421edf982b0bc13cca46 (code; receipt HEAD records this checkpoint); NOT ACCEPTED — numeric-kind/full-width integer gate/native matrix and remaining full parity/release work open; CBC job ids: term-conflict-exact-c74949cd-a1 (green exact; 32 raw/retry matches; expanded 45/58 validation baseline per window), term-numeric-envelopes-080539c0 (next await).

### Numeric native proof and unknown-envelope ordering

CBC term-numeric-envelopes-080539c0-a1 passed code 080539c03acd7eddcc46421edf982b0bc13cca46. Read complete 65658-byte saved log in four chunks: ten Mix suites, two actual bundle tests including raw u64/float/precedence cases, fifteen runtime/call/timer tests including numeric metadata, 189 terminal tests (five existing ignores), three launcher, three registration and five native Bus tests, seven input tests, dialog/search, foreign gate and debug build. Exact signed-scroll bridge cases pass. Returned formatting applied in b9e3d4127a187a84ebab73a76410e12b27b4fa9c. Verified debug SHA256 76986360cbd3a1a6fa0263a514092c0f7674e469124b79b183e5c6a3787f9783.

Private native.MeBhUJ matches every expanded validation reply: 66/66 on each window, 132 total, compared directly without normalisation. All thirteen prior mismatches per window are closed. Actual u64 maximum selectors produce the exact not-found text, large tab index clamps correctly, integral-valued JSON floats/exponents and signed out-of-range values refuse correctly, i64 min/max lines/pages execute safely, and null-presence/error precedence match gold. The 36 raw/retry cases also match, including integer-vs-float and caller-shaped-tag conflicts before validation; twelve discovery and eight namespace replies match. Raw snapshot live-data validation/normalisation remains documented and separate from the direct numeric comparison.

Two-window native parents/Mix shells, GPU handover/prepare counters, selected-service watch/show/capture/input, close and survivor checks passed. All eleven tracked processes stopped. Both matched 900x560 real-session pairs remain byte-identical at Studio Dark/Inter400/14.667/scale1 and identical command output. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM/RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Four originals and both full montage/difference pairs inspected; no differences. witness.json archives full direct numeric comparisons, fixture hash, raw cases, normalisation, figures and closure. Diagnostic debug with formatting applied afterwards, not exact release acceptance.

Pinned core dispatch reveals the remaining unknown-name ordering: after namespace canonicalisation it checks the original 8192-byte budget before command lookup; unknown names are refused before JSON parsing. The previous generic host answered every own unknown name before Mix could check its budget. Code c735c5c31a80dec0e7161cb3bd7763be562a3477 adds opt-in unknown_handler contract data so Mix can own the refusal. Host writes the original canonical command header, overwriting a caller-supplied value. Term checks own/bare unknown body size then returns the exact unknown text. Foreign namespace refusal remains at the native boundary before that budget. Other apps retain default refusal handling. Pure Mix limit/bare/foreign/malformed ordering checks pass; no authority/replay rule changed and no app policy moved to Rust.

Next CBC term-unknown-envelope-c735c5c3 runs term_gate --prepare at that code, including applied numeric formatting. Read full saved log, apply any changes and exact-repeat if needed. Fetch via term-fetch-artifact.mix with full SHA and actual worker. Prepared unknown-c735c5c3.mix repeats all numeric/raw/two-window/capture checks and adds seven unknown-name cases per window: malformed own, oversized own/bare/foreign, exact-limit own, empty own and forged original-command header. Match exact rc/body; retain any failures. The numeric witness is complete only for 080539c0 diagnostic bytes; the new unknown handler is not yet native verified. Full actions/topics/menus/macros/broker/refusal/reconnect, input/IME/grid search/live Prefs, clipboard providers, enforcement/expiry/conformance and final scoped release acceptance remain open. No Fable deviation, gold, desk, install or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: c735c5c31a80dec0e7161cb3bd7763be562a3477 (code; receipt HEAD records this checkpoint); NOT ACCEPTED — unknown-envelope CBC/native ordering proof and remaining full parity/release work open; CBC job ids: term-numeric-envelopes-080539c0-a1 (green; fmt applied; native 132/132 validation and 36 raw/retry matches), term-unknown-envelope-c735c5c3 (next await).

### Unknown-envelope compilation repair

CBC term-unknown-envelope-c735c5c3-a1 returned rc101. Read the complete saved log: all ten Mix suites passed, then Viu compilation refused mutation of the immutable request returned by facilities.request (E0596/E0594). No Rust tests or artifact build ran; TERM_PATCH is empty. This is a real failed gate, not an infrastructure refusal.

Code 69ee6044f1d0b4206bf4780549df9a9a48bc8824 makes that returned request binding mutable so the opt-in unknown handler can set its host-owned original-command header and dispatch name. No policy or authority changes. git diff --check passed. No local Rust execution. Native unknown-69ee6044.mix is prepared against the matching artifact path; no candidate bytes have run for this repair.

Next CBC term-unknown-mutable-69ee6044 repeats term_gate --prepare at that code. Read the complete saved log, apply any returned formatting and repeat exact-source if necessary. Fetch full SHA and actual worker through term-fetch-artifact.mix, verify source/profile/hash, then run unknown-69ee6044.mix directly. Preserve all seven unknown cases per window, the 132 numeric and 36 raw/retry comparisons, discovery/namespaces, real-session captures and cleanup. Existing full Bus/actions/topics/menu/macros/broker, input/IME/grid search/live Prefs, clipboard providers, enforcement/expiry/conformance and final scoped release work remains open. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: 69ee6044f1d0b4206bf4780549df9a9a48bc8824 (code; receipt HEAD records this checkpoint); NOT ACCEPTED — unknown-envelope gate/native proof and full parity/release work open; CBC job ids: term-unknown-envelope-c735c5c3-a1 (rc101 compile failure; full log read; no diff/artifact), term-unknown-mutable-69ee6044 (next await).

### Exact unknown-envelope native proof

CBC term-unknown-mutable-69ee6044-a1 passed exact code 69ee6044f1d0b4206bf4780549df9a9a48bc8824 with no worker diff. Read the complete 54282-byte saved log in three chunks: ten Mix suites, two actual bundle tests, fifteen runtime/call/timer checks, 189 terminal tests (five retained ignores), launcher/registration/native Bus/input/dialog/search checks, foreign gate and debug build. Strict Mix/mixd tier remains clean. Verified exact debug executable SHA256 c7c28e34938d931bcf55c63226d5e1b2a8d5eb591275e8c6c6575c1cff5c09b9.

Private native.fDBvCC passes all fourteen unknown-name cases (seven per service identity): malformed own, oversized own/bare/foreign, exact-limit own, empty own and forged original-command header. Full rc/result matches gold directly without normalisation. Numeric validation remains 132/132 exact, raw/retry 36/36 and discovery 12/12 with previously documented live-data normalisation. Eight namespace refusals, native parent/shell independence, GPU handover/prepare counts, selected-service watch/show/capture/input and close/survivor cases pass. All eleven tracked processes stopped. Native parent revoke remains unconfirmed in some logs and relies on lease/window expiry; full enforcement/expiry proof stays open.

Both real-session 900x560 pairs at matched Studio Dark/Inter400/14.667/scale1 and deterministic Mix output are byte-identical. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. ImageMagick 7.1.2-35 SSIM/RMSE distortions zero for full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Four originals and both full montages/differences inspected; no region differences. witness.json retains source/binary, complete comparisons, figures, normalisation and cleanup. Exact debug witness; final release and full parity remain unaccepted.

Continue actual change-topic delivery and the remaining full actions/menus/macros/broker/input/IME/grid search/live Prefs/provider/enforcement/expiry/conformance/release matrix. No proposed Fable deviation, gold, desk, install or merge changes.

branch: lab/parity-term; head SHA: 69ee6044f1d0b4206bf4780549df9a9a48bc8824 (tested code; receipt HEAD records this checkpoint); NOT ACCEPTED — full parity and scoped release acceptance open; CBC job ids: term-unknown-envelope-c735c5c3-a1 (failed compile), term-unknown-mutable-69ee6044-a1 (green exact; native 14 unknown,132 numeric,36 raw/retry,12 discovery matches and zero-difference capture pairs).

### Native topic delivery and VT conformance expansion

Exact debug 69ee6044 native topic witness adds an independent Mix --serve subscriber, confirmed subscriptions before props.watch, and eight tab operations per window: new, exact retry, title, select, no-op select, invalid target, move and close. Private native.loOTR2 delivered identical fourteen events per window (topic, inner command, kind/tab/pane/order/revisions), but full replies matched 14/16: second candidate new/retry returned graphics-only where gold returned granted. Failed records retained; all nineteen tracked processes stopped.

Pinned gold native_session.rs has the same 30-second launch window and 60-second mint floor as the imported actor. The initial run did not match process age at mutation, so it cannot establish a port-specific grant defect. Private native.da1ixk repeats mutations after a recorded minimum 68 seconds from each app pair launch. Both windows of both apps return binding=granted, including exact retries. All sixteen full operation reply/event comparisons pass after normalising only selected service and event revisions relative to each native props.watch starting revision. Gold second-window watch starts at 7 and candidate at 5 due pre-subscription startup activity; absolute records and the failed absolute comparison remain. Event revisions stay strictly ordered; mutation counts, inner commands and payloads match. Retries/no-op/refusal produce no new events. No authority state is normalised. witness.json records clocks, comparisons and cleanup; all nineteen processes stopped. This proves these topic cases only, not loss/reconnect/outbox/macro/menu or full enforcement. Earlier failed native.tIAQy8 is a harness call error (kill requires numeric signal15, not TERM); finally stopped all nine tracked processes, corrected without an interpreter fallback.

The earlier native.fDBvCC remains the latest fully diffed and inspected visual witness (two byte-identical 900x560 pairs, whole/header/tabs/terminal distortions zero); topic-run captures are diagnostic and are not promoted to visual acceptance. Expanded pane split/select/close subscriber harness is prepared privately but not yet run.

Code d54313b12d73add62dce6a82cb7288f8534a664c adds five real parser/grid conformance cases: DEC scroll region plus origin addressing, character delete/insert/erase, line insert/delete and reverse index within margins, delayed wrap and disabled-wrap overwrite, and DEC cursor save/restore plus explicit tab stops. Assertions include untouched outer rows and exact columns. No production policy changes and no local Rust execution; checks pending CBC.

Next CBC term-conformance-release-d54313b1 runs tools/gold_parity_gate.mix --prepare: shared Viu/engine check/tests/clippy and debug/release artifacts, including the new conformance cases. This prepares exact release bytes for remaining witnesses; it is not app acceptance. Read complete log and GOLD_PARITY_PATCH through end; apply any diff and exact-repeat. Fetch /build/es-artifacts/full-SHA/viu and viu.json from actual worker, verify source/profile/release hash, archive exact app/library bundle. Update the private expanded topic harness to the verified release artifact and receipt profile before running it; do not label earlier debug bytes release. Remaining full Bus/actions/topics/menu/macros/broker, input/IME/grid search/live Prefs/provider/enforcement/expiry/conformance and release visual matrix still bind. No Fable deviation, gold, desk, install or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: d54313b12d73add62dce6a82cb7288f8534a664c (conformance code; latest native-tested production code 69ee6044; receipt HEAD records this checkpoint); NOT ACCEPTED — conformance/release gate and full parity/release matrix open; CBC job ids: term-unknown-mutable-69ee6044-a1 (green exact; native unknown/numeric/raw and topic witnesses), term-conformance-release-d54313b1 (next await).

### Raw tab-marker conformance correction

CBC term-conformance-release-d54313b1-a1 returned rc101. Read the complete 53977-byte saved log in three chunks, including GOLD_PARITY_PATCH_END. All-target Viu/engine check passed. The library suite reported 266 passed, one failed and five retained ignores. Four of the five new VT cases passed, including scroll regions/origin, line editing/reverse index, character editing and wrap. The DEC save/restore and tab case passed its cursor assertions, then failed because its expected raw row replaced a stored HT marker with a space. Clippy, integration test binaries and debug/release artifact builds were not reached. This is a failed test gate; no release artifact is claimed.

Inspected pinned Rio 932c1a7 crosswords::put_tab: it writes a literal HT into the current cell only when that cell was untouched NUL, then advances to the tab stop. This preserves tab text-copy semantics. Code 3f91bd5a886c46f4babc4dd86402f5ff22bbfa92 corrects both exact raw-row expectations to retain the marker and adds independent assertions for B at column8 and X at the explicit column3 stop. Existing content at the second tab origin remains A, per the same parser branch. No production behaviour changed, no geometry assertion removed, no test ignored. Applied the returned formatting hunk. git diff --check passed; no local Rust execution.

Next CBC term-tabmarker-release-3f91bd5a repeats tools/gold_parity_gate.mix --prepare at that exact code. Read complete saved log; diagnose failures, apply worker diffs and exact-repeat until clean. Fetch and verify release executable/source/profile/hash only after the gate builds it; then run the prepared expanded native topic/pane witness against those exact bytes, updating artifact and receipt profile explicitly. Latest native-tested production code remains 69ee6044, with fDBvCC complete raw/numeric/unknown and inspected visual evidence and da1ixk matched-idle topic proof. Prior failed topic/comparison records remain untouched. Full remaining Bus/menus/macros/broker/input/IME/grid search/live Prefs/providers/enforcement/expiry/conformance/release matrix still binds. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: 3f91bd5a886c46f4babc4dd86402f5ff22bbfa92 (test code; native-tested production code 69ee6044; receipt HEAD records this checkpoint); NOT ACCEPTED — corrected conformance/release gate and full parity/release matrix open; CBC job ids: term-conformance-release-d54313b1-a1 (rc101 raw HT expectation; complete log read and fmt applied), term-tabmarker-release-3f91bd5a (next await).

### Exact release witness and reusable grid-search mechanism

CBC term-tabmarker-release-3f91bd5a-a1 passed exact 3f91bd5a886c46f4babc4dd86402f5ff22bbfa92 with no worker diff. Read complete 165247-byte saved log in ten contiguous chunks through GOLD_PARITY_PATCH_END. Viu/engine fmt/check/all-target clippy passed (existing warnings retained); all test binaries passed: 462 tests, eleven existing ignores, including all five new VT conformance cases. Debug and release builds completed. Verified release executable SHA256 568f5d6cbff430d191a0b50bf7131ab5ca128016072325d1072dd2902d17c2fe from /build/es-artifacts/full-SHA/viu plus matching source/profile receipt. Private exact-source bundle, full log, readelf and ldd are archived. Release ldd loads libstdc++, libxkbcommon, libgcc_s, libm and libc; Viu remains outside the strict tier, consistent with D6 simdutf debt. No strict Mix/mixd ELF claim is inferred from this Viu report.

Private native.Rzya1x runs those exact release bytes against gold. Fourteen operations per window (28 full comparisons) match after only selected-service and initial-watch-relative event revision normalisation: tab new/retry/title/select/no-op/refusal/move/close, pane split/retry/select/no-op/refusal/close. Each window receives exactly 22 ordered change events with matching topic/inner-command/tab/pane/kind. Replies retain true granted binding, no authority normalisation. Actual native parents/Mix shells, selected-service watch/show/capture/input and survivor checks pass. Recorded minimum68s idle exercises grant replenishment after the shared gold mint floor. All 23 tracked processes stopped. Parent revoke remains sometimes unconfirmed and full remote expiry/enforcement acceptance stays open.

Both matched 900x560 real-session pairs remain byte-identical at Studio Dark/Inter400/14.667/scale1 and identical Mix command output. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. ImageMagick7.1.2-35 SSIM/RMSE distortions zero for full0,0,900,560/header0,0,900,38/tabs0,38,900,32/terminal0,70,900,490. Four originals and both full montages/differences inspected: no region differences. witness.json preserves bytes/source/profile, complete topic comparisons, normalisation, figures and closure. This is a bounded exact release witness, not overall acceptance.

Code 3f7faa22dd929eff8dbda2c773b93b8d1a1b40a8 starts the explicit reusable grid-search requirement, using the existing pinned Rio RegexSearch rather than a string output mirror. SearchPattern enforces 1..4096 UTF-8 bytes, literal versus regex and explicit case policy; Terminal::search reads native history/wrapped cells under the grid lock with a 1..10000-line budget, validates signed-row origins, reports inclusive native cell ranges, wrap and limited-window status, and never changes selection/viewport/PTY input. viu_terminal search bridges the same owned session handles, refuses stale handles and malformed options, and returns structured observations. No extra dependency, app-owned Rust policy or Bus authority path is introduced. Mix query/navigation and visible highlights still need wiring and acceptance; command search is not counted as terminal search.

Added real-grid tests cover literal metacharacters, regex/case/direction/wrap, retained history, wide wrapped text, selection/scroll preservation, query/origin/budget refusals, missing matches and the in-process bridge. Rust checks pending; no local Rust execution. Next CBC term-grid-search-3f7faa22 runs tools/term_gate.mix --prepare. Read full saved log, fix real failures/apply worker changes and exact-repeat. Existing exact release3f91bd5a is preserved; do not call a later debug search artifact release. Continue search highlight/navigation UI in Mix and the full remaining Bus/actions/topics/menu/macros/broker/input/IME/Prefs/provider/enforcement/expiry/conformance/release matrix. No Fable deviation, gold, desk, install or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: 3f7faa22dd929eff8dbda2c773b93b8d1a1b40a8 (search code; latest native-tested release3f91bd5a; receipt HEAD records this checkpoint); NOT ACCEPTED — search mechanism gate/UI/highlights and remaining full parity/release matrix open; CBC job ids: term-tabmarker-release-3f91bd5a-a1 (green exact release; native28 topic comparisons and zero-difference pairs), term-grid-search-3f7faa22 (next await).

### Grid-search public type import repair

CBC term-grid-search-3f7faa22-a1 failed with rc101 after all ten Mix suites passed. Read complete 18068-byte saved log in two contiguous chunks through TERM_PATCH_END. Viu compilation reported E0603: Direction and Side at crosswords are private imports. Inspected pinned Rio pos.rs: Direction is public there and Side is its public alias, matching the existing selection module import convention. No search tests or artifact build ran.

Code e0b1c6561a28a7773fec4cae573a62968eb7117b imports both types from crosswords::pos and applies the complete returned rustfmt patch to search/model/sessions. No search logic, test expectation or authority policy changed; git diff --check passed. No local Rust execution.

Next CBC term-search-types-e0b1c656 repeats tools/term_gate.mix --prepare at this exact code. Read full saved log; fix actual failures, apply any worker changes and exact-repeat. Search mechanism tests still await their first run. Mix query/navigation and visible search highlights remain open; do not count command search as grid search. Preserve exact release3f91bd5a and its native.Rzya1x witness independently of later search debug builds. Full Bus/actions/topics/menus/macros/broker/input/IME/live Prefs/providers/enforcement/expiry/conformance/release matrix remains open. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: e0b1c6561a28a7773fec4cae573a62968eb7117b (search code; latest native-tested release3f91bd5a; receipt HEAD records this checkpoint); NOT ACCEPTED — search gate/UI/highlights and full parity/release matrix open; CBC job ids: term-grid-search-3f7faa22-a1 (rc101 imports; complete log read; fmt applied), term-search-types-e0b1c656 (next await).

### Wrapped wide grid-search anchor repair

CBC term-search-types-e0b1c656-a1 compiled the search code and returned rc101 in terminal tests. Read the complete34043-byte saved log in two contiguous chunks through TERM_PATCH_END (empty diff). Ten Mix suites, actual bundle load and fifteen runtime/call/timer checks passed. Terminal suite:197 passed, one failed, five retained ignores. Literal/regex/case/reverse/wrap, refusal/non-mutation and owned-session bridge tests passed. The history/wide test passed history, selection preservation and budget assertions before failing the wrapped-wide range: Rio returned start(row0,col4) at LeadingSpacer while the expected actual glyph lead is(row1,col0); end(row1,col2) matched. No artifact build ran.

Code51e7580d maps a search start anchored at LeadingSpacer to the following row first cell only when it is the actual Wide lead. It computes wrap status from canonical returned coordinates. The original expected visible glyph range remains unchanged; added raw-grid assertions prove the leading spacer and actual lead in the fixture. Code b680a1530cb01f3f7bdd527ba55b129acfdd4ddd also requires that canonical lead lie inside the original range, so an empty regex match cannot become inverted. Added empty-match tests near the wide spacer for forward/backward origins and range ordering. No selection, viewport, PTY input or authority change; no test ignored or geometry assertion removed. git diff --check passed; no local Rust execution.

Next CBC term-search-wide-b680a153 repeats tools/term_gate.mix --prepare at that exact source. Read full saved log, fix failures, apply returned formatting and exact-repeat. After native mechanism proof, continue Mix query/navigation, highlight ownership and actual input/clipboard/captures, not just the command-search palette. Preserve exact release3f91bd5a and its native.Rzya1x proof separately; later search source is unaccepted. Full remaining Bus/actions/topics/menu/macros/broker/IME/live Prefs/providers/enforcement/expiry/conformance/release matrix still binds. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA: b680a1530cb01f3f7bdd527ba55b129acfdd4ddd (search code; latest native-tested release3f91bd5a; receipt HEAD records this checkpoint); NOT ACCEPTED — wide-search repair gate/UI/highlights and full parity/release matrix open; CBC job ids: term-search-types-e0b1c656-a1 (rc101 wide anchor; complete log read; no diff/artifact), term-search-wide-b680a153 (next await).

### Grid-search mechanism proof and atomic highlight capture

CBC term-search-wide-b680a153-a1 passed code b680a1530cb01f3f7bdd527ba55b129acfdd4ddd. Read complete56502-byte saved log in four contiguous chunks through TERM_PATCH_END. Ten Mix suites, actual bundle load, fifteen runtime/call/timer checks,199 terminal tests (five retained ignores), launcher/registration/native Bus/input/dialog/search checks, foreign gate and debug build passed. All five search mechanism/bridge tests pass, including canonical wrapped-wide anchors and empty-match range ordering. Returned formatting-only change applied with the following highlight work. This is mechanism test proof, not native search UI acceptance or final release bytes.

Code5d74ed97bacad8a0eef972126dde54959888d948 adds grid_snapshot_with_search: native matching and captured colour cells share one grid lock, so a PTY update cannot slip between the matched range and rendered snapshot. Search remains separate from the clipboard selection. The caller supplies current theme foreground/background and request origin/direction/budget. Existing captures remain unchanged without a search request. The last consumed highlight range/colours participate in incremental damage: move invalidates old and new rows, clear invalidates old rows, live colour changes invalidate affected rows, and identical idle search does not dirty them. Wide glyph membership paints both lead/spacer cells while excluding the preceding blank LeadingSpacer. No app policy or Bus authority is added.

Added tests check exact highlight colours, unpainted neighbours, selection/scroll preservation, old/new/idle/clear/retheme damage and wrapped-wide halves. A real offscreen GPU callback test moves and clears the highlight and compares the final pixels with the CPU renderer. These new highlight tests await CBC; no local Rust execution. Mix query/navigation, visible controls, widget ownership and live input/capture acceptance still need wiring, and are not claimed complete by this primitive.

Next CBC term-search-highlight-5d74ed97 runs tools/term_gate.mix --prepare at that code, including returned search formatting. Read full saved log, fix real failures/apply worker changes and exact-repeat until no diff. Then wire Mix-owned query/navigation and the native view to this atomic capture path; keep search owner bound across pane changes/close and test actual input, copy, output, resize, history eviction and live theme/font updates. Preserve exact release3f91bd5a/native.Rzya1x independently. All remaining full Bus/actions/topics/menu/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance/release matrix work still binds. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:5d74ed97bacad8a0eef972126dde54959888d948 (highlight code; latest native-tested release3f91bd5a; receipt HEAD records this checkpoint); NOT ACCEPTED — highlight gate, Mix/native view search wiring and full parity/release matrix open; CBC job ids:term-search-wide-b680a153-a1 (green; all search mechanism tests passed; formatting applied), term-search-highlight-5d74ed97 (next await).

### Atomic highlight mouse-fixture compilation repair

CBC term-search-highlight-5d74ed97-a1 returned rc101. Read complete23278-byte saved log in two contiguous chunks through TERM_PATCH_END. All ten Mix suites, two actual bundle checks and fifteen runtime/call/timer checks passed. Production library compiled; terminal lib-test compilation failed E0063 because mouse.rs constructs a fixture Terminal without the new captured_search field. Highlight tests did not run and no artifact was built. Searched all terminal constructors/selection-cache initialisers to check the field is present across real and fixture owners.

Code055f31ee1e243a6e643f76b269fce7d8e6e35da5 initialises captured_search to None in that mouse fixture and applies the complete worker formatting diff to model/search/GPU checks. No search, highlight, damage or authority logic changed; no tests ignored. git diff --check passed; no local Rust execution.

Next CBC term-highlight-fixture-055f31ee repeats tools/term_gate.mix --prepare at that source. Read the complete result, repair failures/apply worker formatting and exact-repeat as needed. Next dependent work remains Mix query/navigation controls plus native-view highlight wiring/ownership and real input/output/resize/history/theme/font captures. Existing exact release3f91bd5a/native.Rzya1x proof stays separate from unaccepted search/highlight debug code. Full remaining Bus/actions/topics/menus/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance/release matrix still binds. No proposed Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:055f31ee1e243a6e643f76b269fce7d8e6e35da5 (highlight code; latest native-tested release3f91bd5a; receipt HEAD records this checkpoint); NOT ACCEPTED — highlight gate, Mix/native search UI wiring and full parity/release matrix open; CBC job ids:term-search-highlight-5d74ed97-a1 (rc101 mouse fixture; complete log read; formatting applied; no artifact), term-highlight-fixture-055f31ee (next await).

### Live widget search request and result bridge

CBC term-highlight-fixture-055f31ee-a1 passed exact 055f31ee1e243a6e643f76b269fce7d8e6e35da5 with no worker diff. Read the complete 55964-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it privately. Ten Mix suites, two actual bundle tests, fifteen runtime/call/timer checks, 202 terminal tests (five retained ignores), launcher/registration/native Bus/input/dialog checks, foreign gate and debug build passed. Highlight damage, wide-cell colours and real GPU move/clear tests all passed. This proves the atomic mechanism, not a native search UI or final release.

Code 1a1e010fd96946517e3f9f1808b4c7ed3061ebc9 connects atomic search capture to the reusable terminal view. Optional widget properties.search names a mandatory native handle, query and unsigned request serial, with regex/case/direction/origin/line-budget options. It compiles a request once, recomputes the match under the live grid capture lock, and paints using current chrome text/selected-row colours. The widget emits an in-process search intent with handle/request plus matched range, wrapped/limited flags or error, only when the observed result changes. Absent or differently owned requests clear the highlight; invalid or expired origins paint the actual unhighlighted grid and report refusal for Mix to resolve. Search does not take clipboard selection or redirect through focus. No new Bus method or app policy in Rust.

Added tests exercise exact owner/request routing, idle event suppression, old/new highlights, expired origins, invalid regex, sibling isolation, unchanged selection, output removal/reappearance, live colours and clear. A real Views draw test checks the rendered owner result, sibling silence and closed-owner cache removal. These new view tests await CBC; no local Rust execution. git diff --check passed.

Next CBC term-search-view-1a1e010f runs tools/term_gate.mix --prepare at this exact source. Read the complete saved result, repair real failures/apply worker formatting and exact-repeat as needed. Next dependent work is Mix query/navigation controls, viewport reveal and native input/capture acceptance across pane changes/close, output, resize/history eviction and live theme/fonts. Gold command search remains separate. Preserve exact release3f91bd5a/native.Rzya1x evidence independently of the later search debug source. The full remaining Bus/actions/topics/menus/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance/release matrix still binds. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:1a1e010fd96946517e3f9f1808b4c7ed3061ebc9 (view-search code; latest native-tested release3f91bd5a; receipt HEAD records this checkpoint); NOT ACCEPTED — view-search gate, Mix search UI/navigation/reveal and full parity/release matrix open; CBC job ids:term-highlight-fixture-055f31ee-a1 (green exact; atomic/GPU highlights passed), term-search-view-1a1e010f (next await).

### Live search view history fixture correction

CBC term-search-view-1a1e010f-a1 returned rc101. Read complete 44649-byte saved log in three contiguous chunks through TERM_PATCH_END. Ten Mix suites, two bundle tests and fifteen runtime/call/timer checks passed. Terminal suite:204 passed, one failed, five retained ignores. Actual Views draw search/owner/close test and search ownership/refusal/selection test passed. The live-grid test expected no match after deleting the first row; pinned Rio delete_lines calls scroll_up_relative at row0 and retains that row in history, so search correctly still found it. No artifact build ran.

Code5bcbc1185b135f801cf83aa8ffe600a2bcb1838b applies the complete worker formatting diff and strengthens that fixture: assert the retained match at row-1, explicitly clear scrollback and screen, then require no match and correct event count before text reappears. The prior no-match requirement remains after actual history removal. No production logic, search scope, selection or authority policy changed; no test ignored. git diff --check passed; no local Rust execution.

Next CBC term-search-history-5bcbc118 repeats tools/term_gate.mix --prepare at this exact code. Read full result, repair failures/apply any formatting and exact-repeat. Next work remains Mix query/navigation controls and viewport reveal, native search input/capture acceptance across output/resize/history eviction/pane lifecycle/live fonts/theme, and the full outstanding parity matrix. Preserve latest native-tested release3f91bd5a/native.Rzya1x independently. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:5bcbc1185b135f801cf83aa8ffe600a2bcb1838b (search view history fixture; latest native-tested release3f91bd5a; receipt HEAD records checkpoint); NOT ACCEPTED — view gate, Mix search controls/navigation/reveal and full parity/release matrix open; CBC job ids:term-search-view-1a1e010f-a1 (rc101 history expectation; complete log read and formatting applied), term-search-history-5bcbc118 (next await).

### Search view proof and Mix find controls

CBC term-search-history-5bcbc118-a1 passed exact5bcbc1185b135f801cf83aa8ffe600a2bcb1838b with no worker diff. Read complete56290-byte log in four contiguous chunks through TERM_PATCH_END and archived privately. Ten Mix suites, two actual bundle tests, fifteen runtime/call/timer checks,205 terminal tests (five retained ignores), launcher/registration/native Bus/input/dialog/search checks, foreign gate and debug build passed. All three new search view tests pass, including actual widget draw, owner/close isolation, clipboard selection preservation, history retention/clear and live output/colour recomputation. This is not native input/capture or final release acceptance.

Codea1d3b7b1d9f8dc8f30f660335371e9b91ea07677 adds Mix-owned terminal grid find, separate from gold Search menus. Ctrl+Shift+F opens the existing reusable Viu search-bar; Previous/Next, Enter and terminal-focused F3/Shift+F3 navigate. Regex/case options and a result/refusal/limited-window label drive explicit pane-bound native search properties. Replacement is disabled for terminal output. Query and navigation state, request serials, reveal, error recovery and lifetime stay in apps/term/find.mix. No native matching policy or new Bus methods added. This implements the user explicit grid-search requirement as an additive capability; normal closed-search layout, gold catalogue and menu-search behaviour stay intact.

Navigation advances beyond inclusive ranges, wraps at retained-history/live-grid boundaries and reports wrap. A valid match outside the viewport triggers one native scroll per request; later output observations do not repeatedly steal manual scroll. Invalid/evicted origins rebase once to the current viewport with a fresh request, then stop retrying on repeated refusal. Stale query/owner observations are ignored. Pane/tab owner changes and close discard search presentation rather than redirect it. Closing the bar returns terminal focus. publish_term now returns its synchronised state and callers retain it, respecting Mix copied-map semantics.

Added find-test.mix to the scoped CBC gate. All eleven Mix suites passed locally, and find/core/actions/gate/app lint passed (app lint lists the existing embedder functions explicitly). Tests cover hidden initial layout, reused search-bar without replacement, exact native request projection, stale owner/serial, one-time reveal, next/previous and both history boundaries, native regex errors, bounded no-match wording, origin recovery/retry bound, close/focus and owner retirement. git diff --check passed; no local Rust work.

Next CBC term-find-ui-a1d3b7b1 runs tools/term_gate.mix --prepare at this code. Read full saved result; repair failures/apply worker formatting and exact-repeat. Then fetch matching debug artifact and exact source bundle, run isolated real Mix sessions with native keyboard/pointer query input, options/navigation/history, selection/copy, pane close/retarget, output/resize and live theme/font checks, retaining captures and cleanup. Baseline closed-search real sessions still need matched gold comparisons at the new source; additive search UI needs inspected mechanism captures. Latest native-tested release3f91bd5a/native.Rzya1x remains separate from this unaccepted debug search work. The rest of the full Bus/actions/topics/menus/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance/release matrix stays open. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:a1d3b7b1d9f8dc8f30f660335371e9b91ea07677 (Mix find UI; latest native-tested release3f91bd5a; receipt HEAD records checkpoint); NOT ACCEPTED — Mix find bundle/native input/capture acceptance and full parity/release matrix open; CBC job ids:term-search-history-5bcbc118-a1 (green exact;205 terminal tests), term-find-ui-a1d3b7b1 (next await).

### Native find input exposed scene-number decoding gap

CBC term-find-ui-a1d3b7b1-a1 passed exacta1d3b7b1d9f8dc8f30f660335371e9b91ea07677 with no worker diff. Read complete56215-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks,205 terminal tests (five retained ignores), other scoped host/input/dialog checks, foreign gate and debug build passed. Fetched executable/source-profile receipt, verified debug SHA25652e68f472f34318c1d12fa2e00a27f393918771baa64a7875f723e034a41c09e, archived exact-source app/shared-Mix bundle.

Private native.poYUfZ used those bytes and bundle with owned Bus/headless comp/settings/home. Real Mix output appeared; native Ctrl+Shift+F opened the reusable bar and native key/text input reached Mix query TERM_PAIR_NATIVE. The live scene converts numeric properties into f64 JSON values; search refused max_lines10000.0 as expected-usize. Raw model/result and failed run are preserved, not accepted. No completed search capture/SSIM claim. This older keyboard harness reported graphics-only, so it proves no issuer/control authority. Finally stopped all five tracked app/shell/services/broker PIDs; closure has no survivors.

Coded70026f8514feea814d41e286b4b047ca80d7517 normalises only validated numeric search-option fields before typed decoding: whole nonnegative request/line budget/column within Mix exact integer range, signed whole row within i32. Strings, fractional/negative unsigned values and out-of-range values remain errors; query/booleans and matcher budgets retain their existing checks. Native integer callers keep their behaviour. Added regression passes float-token request/max_lines/origin through actual snapshot highlight and checks the exact match/request, plus malformed/fractional/string/range refusals. No test ignored or weakened; no local Rust execution. git diff --check passed.

Next CBC term-find-numbers-d70026f8 runs tools/term_gate.mix --prepare. Read full result, apply any formatting/lock changes and exact-repeat. Then fetch matching debug bytes/bundle and repeat native find input using the later owned-session harness with private ingress/native provisioning, not this older graphics-only scaffold. Complete search options/navigation/history/clipboard/pane/output/resize/live font/theme captures, matched closed-search gold pairs and cleanup; full remaining Bus/actions/topics/menus/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance/release matrix still binds. Latest accepted-bounded native release3f91bd5a evidence stays separate; overall NOT ACCEPTED. No Fable deviation, install, gold, desk or merge changes.

branch: lab/parity-term; head SHA:d70026f8514feea814d41e286b4b047ca80d7517 (scene numeric search repair; latest native-tested release3f91bd5a; receipt HEAD records checkpoint); NOT ACCEPTED — numeric repair gate and native find/full parity/release matrix open; CBC job ids:term-find-ui-a1d3b7b1-a1 (green exact; native.poYUfZ exposed float option refusal, all5 stopped), term-find-numbers-d70026f8 (next await).

### Scene-number regression proof and exact-source formatting repeat

CBC term-find-numbers-d70026f8-a1 passed with a formatting-only worker diff. Read complete57932-byte saved log in four contiguous chunks through TERM_PATCH_END (re-read the tool-truncated third chunk separately) and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks,206 terminal tests (five retained ignores), other scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. The new scene-float snapshot regression and all refusal cases pass. No later native input/capture verdict is inferred.

Code8e31c4d41fba7b9a59924139f1ae8790a72ce18a applies the complete returned formatting-only diff; no logic changes. Next CBC term-find-exact-8e31c4d4 repeats tools/term_gate.mix --prepare. Read complete result, apply any changes and repeat until exact no diff. A lint-clean private find-owned-8e31c4d4.mix harness is ready: it uses the later sibling ingress directory and requires child binding Attached before search input, retains literal/no-match/Enter/Escape captures, exact-source bundle checks and finally cleanup. Fetch its exact debug artifact/bundle after the gate passes and run it directly through an exec session (not the default30s run_argv wrapper). Expand native options/navigation/history/selection/pane/output/resize/Prefs evidence and matched closed-search gold pairs after the first repair proof. No native search acceptance yet; earlier native.poYUfZ remains a preserved failed graphics-only numeric witness with all5 stopped.

The full remaining Bus/actions/topics/menus/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance/release matrix stays open. Preserve bounded latest native-tested release3f91bd5a independently. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:8e31c4d41fba7b9a59924139f1ae8790a72ce18a (numeric repair plus worker fmt; latest native-tested release3f91bd5a; receipt HEAD records checkpoint); NOT ACCEPTED — exact search gate/native input/captures and full parity/release matrix open; CBC job ids:term-find-numbers-d70026f8-a1 (green;206 terminal tests; worker fmt applied), term-find-exact-8e31c4d4 (next await).

### Exact debug native find witness

CBC term-find-exact-8e31c4d4-a1 passed exact8e31c4d41fba7b9a59924139f1ae8790a72ce18a, no worker diff. Read complete56322-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks,206 terminal tests (five retained ignores), other scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. Fetched matching debug/source receipt and exact app/shared-Mix bundle; executable SHA2560d462136ef0b95dd1ac9655ea523ebe57b32ab7896f68dde89358f172e66bc44 verified.

Private native.DCp5Sk passes native Ctrl+Shift+F/query input, literal match, Enter wrap, missing-query feedback and Escape close using an Attached real Mix child. All four search captures inspected; all five tracked app/shell/services/broker processes stopped. Earlier native.m4smEN passed literal highlight and wrap but harness typed after Enter released field focus; explicit pointer focus before editing fixes that harness call, with no production changes. Failed run and all5-stopped closure retained.

Expanded private native.e3d9fN passes the same path plus native case-checkbox folding, literal bracket no-match, invalid regex refusal, regex matching, and Previous/Next pointer navigation over eighty real Mix output lines. Retained history matches at rows-61 and-23 are revealed and Previous returns to the original range. Native child remains Attached; same process/shell throughout. All eleven original baseline/search PNGs inspected, including literal/wrap/miss, case/regex error/recovery, both history positions and cleared presentation. Full witness.json retains source/binary/capture hashes, checks, raw models/calls, comparison and closure. All five tracked processes stopped. Parent revoke remains unconfirmed and remote15s lease/30s window expiry/enforcement is still an open acceptance item.

The new closed-search baseline is byte-identical to the retained pinned-gold native.Rzya1x capture of the same pair() output and matched settings:900x560, Studio Dark/Inter400/14.667/scale1. Both PNG SHA256e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Ran parity_compare without resizing into a new directory; ImageMagick7.1.2-35 SSIM/RMSE distortions0 for full0,0,900,560/header0,0,900,38/tabs0,38,900,32/terminal0,70,900,490. Original gold/candidate, full montage and difference inspected: no region differences. This explicitly reuses the retained gold oracle; it is a bounded debug baseline proof, not final release acceptance or a claim that gold has the additive grid-search UI.

Next CBC term-find-release-8e31c4d4 runs tools/gold_parity_gate.mix --prepare for the full shared Viu/engine test/check/clippy/debug/release gate. Read complete log and apply any GOLD_PARITY_PATCH changes/exact-repeat; then fetch/verify matching release bytes and repeat native search/baseline evidence. Extend search pane lifetime, output/resize/history eviction, selection/clipboard and live Prefs on the same PID, and continue the full remaining Bus/actions/topics/menus/macros/broker/IME/Prefs/providers/enforcement/expiry/conformance matrix. No Fable deviation, install, gold, desk or merge changes. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:8e31c4d41fba7b9a59924139f1ae8790a72ce18a (exact debug search witness; latest native-tested release3f91bd5a; receipt HEAD records checkpoint); NOT ACCEPTED — release search witness and full parity matrix open; CBC job ids:term-find-exact-8e31c4d4-a1 (green exact;206 terminal tests; native.e3d9fN search and zero-difference baseline), term-find-release-8e31c4d4 (next await).

### Release find witness, live Prefs/pane lifetime and light contrast repair

CBC term-find-release-8e31c4d4-a1 passed exact8e31c4d41fba7b9a59924139f1ae8790a72ce18a with no worker diff. Read complete175151-byte saved log in ten contiguous chunks through GOLD_PARITY_PATCH_END;474 tests passed, eleven existing ignores. Full Viu/engine fmt/check/tests/all-target clippy/debug/release completed. Warnings remain, including the new test-only drop(draw) warning (removed in the following repair); do not describe this as warning-free. Release receipt source/profile/hash verified; executable SHA25615db6fb0d2cd04ab4e5b44da041d642079359b55d74c1b3caef533dbf98f8f33. Exact source bundle/full log/ELF diagnostics archived privately. Preceding exact scoped gate includes all eleven Term Mix suites and foreign gate.

Private native.zGjQCK repeats the expanded native find witness using these exact release bytes and an Attached real Mix child. Literal/case/regex/error/wrap/no-match/history/Previous/Next/Escape checks pass on unchanged app and shell. All five tracked processes stopped. Release originals with changed text-caret phase inspected directly; remaining capture hashes match the inspected debug originals. witness.json records these inspection paths. New regional comparison against retained pinned-gold native.Rzya1x baseline:900x560, Studio Dark/Inter400/14.667/scale1, same real pair() output; both PNG SHA256e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Full/header/tabs/terminal SSIM and RMSE distortions0, ImageMagick7.1.2-35 lower-better convention. Full montage/difference inspected, no region differences. This reuses the existing gold oracle and proves only this baseline, not the complete visual matrix.

Private native.XRSUgQ extends the same release witness: acknowledged live light mode/font18, weight700/textscale1.25 and reset on the same PID; search query/owner/match retained across raster/grid changes. Split/focus change closes the old search; second pane is child binding Attached with its own vt-2 search; closing that owner removes its request and reaps its shell. All six tracked processes stopped. Five new Prefs/pane captures inspected. Earlier native.bcQbmZ failed only because the harness read settings.apply revision outside receipt and used an uninitialised flag; actual native state had acknowledged revision3. Corrected to receipt.revision plus an initial false flag; all five failed-run processes stopped, evidence preserved.

The light capture exposed a real presentation defect: palette.text is dark in light mode while row_selected is dark, making highlighted terminal output unreadable. Code9e23168ecc5d5add5488a63cf2499d38bdd8d71c uses the theme paired menu_highlight/menu_highlight_text roles for native search colours. Added dark/light Studio contrast regression through actual installed appearance; the native grid matching/selection mechanism and Mix policy are unchanged. Removed unnecessary drop(draw) in the widget test. git diff --check passed; Rust validation pending CBC, no local Rust execution. This is a port bug repair, not a proposed gold deviation.

Next CBC term-find-contrast-9e23168e runs tools/term_gate.mix --prepare. Read full saved log, fix failures/apply worker diff and exact-repeat. Then repeat the live light/scale/pane capture harness with matching source bytes; inspect contrast and continue full Prefs matrix rather than counting only acknowledged values. Preserve prior release8e31c4d4 as bounded evidence; its light highlight remains failed. Parent revoke/remote expiry enforcement, full Bus/actions/topics/menu/macros/broker/IME/clipboard/Prefs/conformance/visual matrix and final exact release acceptance remain open. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:9e23168ecc5d5add5488a63cf2499d38bdd8d71c (light search contrast repair; latest native-tested release8e31c4d4 has light contrast defect; receipt HEAD records checkpoint); NOT ACCEPTED — contrast gate/native recapture and full parity matrix open; CBC job ids:term-find-release-8e31c4d4-a1 (green exact release;474 tests; native.zGjQCK zero-difference baseline and native.XRSUgQ live Prefs/panes), term-find-contrast-9e23168e (next await).

### Search contrast regression passed; exact formatting repeat

CBC term-find-contrast-9e23168e-a1 passed at9e23168ecc5d5add5488a63cf2499d38bdd8d71c. Read the complete58262-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks,207 terminal tests (five retained ignores), other scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. The actual installed Studio dark/light search colour pair passes the new contrast regression. Existing compiler warnings remain; no warning-free claim. Worker returned only formatting in the regression. Code0fa7fe18b69c0874160566a2d0937f926e3e578a applies the complete patch with no logic change; git diff --check passed.

Next CBC term-find-contrast-exact-0fa7fe18 repeats tools/term_gate.mix --prepare. Read complete result, apply any changes and repeat until exact no diff. Then fetch matching executable/source bundle and rerun the private find-prefs-panes-receipt harness against new bytes, inspecting light highlights and retaining all cleanup evidence. Build final release through gold_parity_gate before release acceptance captures. Prior release8e31c4d4/native.XRSUgQ retains its failed light highlight; the passing unit regression alone does not close that visual failure. Full Bus/actions/topics/menu/macros/broker/IME/clipboard/Prefs/enforcement/expiry/conformance/visual matrix stays open. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:0fa7fe18b69c0874160566a2d0937f926e3e578a (contrast repair plus worker formatting; receipt HEAD records checkpoint); NOT ACCEPTED — exact contrast gate/native recapture and full parity matrix open; CBC job ids:term-find-contrast-9e23168e-a1 (green;207 terminal tests; fmt applied), term-find-contrast-exact-0fa7fe18 (next await).

### Exact search contrast repair: native debug proof

CBC `term-find-contrast-exact-0fa7fe18-a1` passed exact source `0fa7fe18b69c0874160566a2d0937f926e3e578a` with no worker diff. Read the complete 56429-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks, 207 terminal tests (five retained ignores), other scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. Fetched source/profile receipt and executable, verified debug SHA256 `75f3fce4eca1da4d60a0dd99f80e2685c8d8a8a3c23986718d7e77ea13f73e32`, and archived the exact-source app/shared Mix bundle.

Private native.TOt9Cy passes native keyboard/pointer literal, wrap, miss, case folding, regex error/recovery, retained-history Next/Previous and Escape with real Mix output and Attached children. Live light mode/font18, text scale1.25/weight700 and reset acknowledge effective revisions on the same PID; the matched text remains readable in both light captures. Split clears the old query, second-pane search binds vt-2, and closing that pane clears its query and stops its child. All six tracked app/shell/services/broker processes are dead. Sixteen capture hashes and checks retained in witness.json. Light, second-pane and owner-close originals viewed directly; the remaining originals are byte-identical to inspected native.q8DXYh captures. Before/after light repair figure inspected. This closes the bounded debug light-highlight defect, not full Prefs acceptance.

The closed-search baseline is byte-identical to retained pinned-gold native.Rzya1x of the same pair() command output/settings: 900x560, Studio Dark/Inter400/14.667/scale1, both PNG SHA256 `e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865`. Ran parity_compare without resizing: full, header[0,0,900,38], tabs[0,38,900,32], terminal[0,70,900,490] all SSIM/RMSE distortion0 (ImageMagick7.1.2-35, lower is better). Gold/candidate originals and full montage/diff inspected; the retained gold oracle reuse is explicit. Search UI itself is additive mandated functionality, with inspected mechanism captures rather than a claimed gold search screenshot.

Failed native.q8DXYh retained: search/Prefs/contrast passed, split attempted before native60s grant floor and failed binding; all five tracked PIDs stopped. Failed native.YZ6TKY retained: waited68s and both children Attached, then pointer helper read model-open before bar geometry was drawn; all six stopped. Final harness waits at least68s before split and waits for bar geometry before query input. Neither correction changes production code or weakens native grant policy.

Next CBC `term-contrast-release-0fa7fe18` runs `tools/gold_parity_gate.mix --prepare`. Read the full saved log, apply any worker changes and exact-repeat if needed. Fetch matching release executable/receipt and bundle, verify hash/profile/source, then run the prepared private find-contrast-drawn release harness; debug evidence does not attest release bytes. Parent revoke remains unconfirmed; remote expiry/enforcement, full Bus/actions/topics/menu/macros/broker/IME/clipboard/Prefs/conformance/visual matrix remain open. No Fable deviation, install, gold, desk or merge changes. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:0fa7fe18b69c0874160566a2d0937f926e3e578a (exact tested contrast code; receipt HEAD records checkpoint); NOT ACCEPTED — release recapture and full parity matrix open; CBC job ids:term-find-contrast-exact-0fa7fe18-a1 (green exact;207 terminal tests; native.TOt9Cy bounded search/Prefs/panes and zero-difference baseline), term-contrast-release-0fa7fe18 (next await).

### Release search contrast proof and next VT conformance cases

CBC `term-contrast-release-0fa7fe18-a1` passed exact `0fa7fe18b69c0874160566a2d0937f926e3e578a`, no worker diff. Read the complete 163792-byte saved log in ten contiguous chunks through GOLD_PARITY_PATCH_END and archived it. Shared Viu/engine fmt, all-target check/clippy, debug/release builds and all test binaries passed: 475 tests, eleven retained ignores. Existing compiler/clippy warnings remain. Verified release source/profile receipt and SHA256 `57085f73a296e497fd697be07204987289cc9fab8aea1a26918f6cd83da51e38`; exact-source bundle, full log, readelf and ldd retained privately. Viu loads libstdc++, libxkbcommon, libgcc_s, libm and libc; D6 Viu debt stays outside strict Mix/mixd.

Private native.Uoecgc repeats the full corrected search harness against those release bytes. Native real Mix children Attached, literal/wrap/miss/case/regex/refusal/recovery/history Next/Previous/Escape pass. Live light/font18, scale1.25/weight700 and reset preserve owner/query/PID with readable highlights. Second pane binds vt-2, closing it clears search and stops its child. Native grant floor remains60s; harness waits68s before split and waits for drawn search geometry. All six tracked PIDs stopped. Sixteen capture hashes and complete calls/models/settings/closure retained. Six changed/key release originals inspected directly; remaining captures byte-identical to inspected debug originals, with only query-caret phase differences in the changed four.

Closed-search release baseline and retained pinned-gold native.Rzya1x pair() capture remain byte-identical: 900x560, Studio Dark/Inter400/14.667/scale1, PNG SHA256 `e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865`. New full/header/tabs/terminal comparison has SSIM/RMSE distortion0 throughout, ImageMagick7.1.2-35 lower-is-better convention. Release original, full side-by-side and diff inspected; pinned gold original previously inspected and oracle reuse explicit. This is bounded release proof, not the complete visual gate. No release install or desk change.

Next test-only code `8563d1b3e9538dc6629155a1a27a8874df57d33f` adds three real-parser/grid conformance cases: EL0/1/2 include cursor cell and preserve position; combined truecolour foreground/background, inverse, bold brightening and SGR reset; insert-mode on/off with CR and LF independence. The pinned gold captured-cell path explicitly brightens bold RGB channels by40, confirmed in term-core terminal.rs, and that exact behaviour is asserted. Existing tests are unchanged; git diff --check passed. No local Rust execution.

Next CBC `term-vt-modes-8563d1b3` runs `tools/term_gate.mix --prepare`. Read full result, diagnose failures against actual pinned parser/gold, apply worker formatting and exact-repeat if required. Latest native-tested production/release remains0fa7fe18; these additional tests have no pass claim yet. Continue broader conformance plus full Prefs and matched visual states, Bus/actions/topics/menu/macros/broker/IME/clipboard/search resize/eviction/copy, native show failures, parent revoke/remote expiry and enforcement matrix. All remain open until evidenced; no Fable deviation proposed. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:8563d1b3e9538dc6629155a1a27a8874df57d33f (new conformance tests; native-tested release0fa7fe18; receipt HEAD records checkpoint); NOT ACCEPTED — added VT gate and full parity matrix open; CBC job ids:term-contrast-release-0fa7fe18-a1 (green exact;475 tests; native.Uoecgc bounded release search/Prefs/panes and zero-difference baseline), term-vt-modes-8563d1b3 (next await).

### VT erase/SGR/insert witnesses passed; exact formatting repeat

CBC `term-vt-modes-8563d1b3-a1` passed at `8563d1b3e9538dc6629155a1a27a8874df57d33f`. Read the complete 86326-byte log in five contiguous chunks through TERM_PATCH_END and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks, 210 terminal tests (five retained ignores), scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. All three new EL/SGR/insert/CR-LF conformance cases passed without changed expectations. Worker diff is formatting-only in the new colour assertions. Code `b42c75f1fd021d758fbedc84050d21d27f343ad5` applies it completely; git diff --check passed.

Next CBC `term-vt-exact-b42c75f1` repeats `tools/term_gate.mix --prepare`; read full result and require no worker diff. Production code and latest native-tested release remain0fa7fe18 (native.Uoecgc, release SHA25657085f73a296e497fd697be07204987289cc9fab8aea1a26918f6cd83da51e38); this later source differs only in conformance tests/formatting and receipts. After exact proof, continue the live Prefs matrix on that verified release, extending the private owned harness beyond mode/font/scale to scheme/style/accent/density/roundness/families/icons/app overrides/auto/tune and settings authority loss/recovery, with matched gold captures for supported states. Continue full Bus/actions/topics/menu/macros/broker, native IME/reporting/clipboard/search edge cases, show failures, issuer expiry/revoke and remaining conformance/visual acceptance. Do not duplicate the completed release contrast run or treat bounded evidence as acceptance. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:b42c75f1fd021d758fbedc84050d21d27f343ad5 (conformance fmt; native-tested release0fa7fe18; receipt HEAD records checkpoint); NOT ACCEPTED — exact VT gate and full parity matrix open; CBC job ids:term-vt-modes-8563d1b3-a1 (green;210 terminal tests; fmt applied), term-vt-exact-b42c75f1 (next await), term-contrast-release-0fa7fe18-a1 (green exact release; native.Uoecgc).

### Exact VT proof; live app-scale mismatch found and repaired

CBC `term-vt-exact-b42c75f1-a1` passed exact `b42c75f1fd021d758fbedc84050d21d27f343ad5`, no worker diff. Read complete 56731-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it. Eleven Mix suites, two bundle tests, fifteen runtime/call/timer checks, 210 terminal tests (five retained ignores), scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. This closes the exact gate for the added erase/SGR/insert conformance witnesses; full vttest-style coverage remains broader.

A new private release probe native.dVSSou used verified0fa7fe18 bytes and exact bundle. After existing search/font/scale checks passed, settingsd accepted `apps.term.text_scale=1.5`; frame and session appearance reported1.5 while terminal.properties.text_scale remained1. The original capture was inspected: enlarged frame/query UI with unchanged grid glyphs. Full calls/state, failed witness and capture hash retained; all five tracked processes stopped. This is a port Prefs defect, not a proposed gold deviation. Prior bounded release search/contrast proof stays valid for its tested settings but does not close app-override scale.

Code `c7efc31c1b0db76375ac8aef7b78076c2057c249` fixes Term Mix projection to pass the resolved session appearance text_scale to the compound terminal, falling back to raw global scale only before resolved appearance arrives. Native font/raster/session ownership mechanisms are unchanged. Four frame assertions cover global fallback, app override shared by frame/grid, retained native owner and resolved-value precedence; all17 frame checks passed through installed Mix. git diff --check passed. No local Rust execution.

Next CBC `term-app-scale-c7efc31c` runs `tools/term_gate.mix --prepare`. Read complete saved result, fix failures/apply worker diff and exact-repeat. Fetch matching debug executable/receipt and exact-source bundle, then run the lint-clean private prefs-app-scale probe prepared for this source; inspect app-scale geometry and glyph change on unchanged app/shell, reset and pane checks. Continue the rest of live Prefs/matched gold states, Bus/actions/topics/menu/macros/broker/IME/reporting/clipboard/search edge cases, show failures and issuer expiry/revoke/conformance/visual matrix. Final release must include this repair. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:c7efc31c1b0db76375ac8aef7b78076c2057c249 (app-scale repair; latest native-tested release0fa7fe18 has app-scale defect; receipt HEAD records checkpoint); NOT ACCEPTED — repair gate/native recapture and full parity matrix open; CBC job ids:term-vt-exact-b42c75f1-a1 (green exact;210 terminal tests), term-app-scale-c7efc31c (next await), term-contrast-release-0fa7fe18-a1 (bounded release proof; native.dVSSou exposed app override mismatch).

### Exact app-scale repair gate and native debug evidence

CBC `term-app-scale-c7efc31c-a1` passed exact `c7efc31c1b0db76375ac8aef7b78076c2057c249`, no worker diff. Read complete56731-byte saved log in four contiguous chunks through TERM_PATCH_END and archived it. All eleven Mix suites (including17 frame assertions), two bundle tests, fifteen runtime/call/timer checks, 210 terminal tests (five retained ignores), scoped host/native Bus/input/dialog checks, foreign gate and debug build passed. Fetched source/profile receipt, verified debug SHA256 `93fde37e8056814643db5b933f5c6e313516b28b6ba93c048f8dd207fa584f1f`, and archived exact-source app/shared Mix bundle.

Private native.0ovg7l passes the focused live app-scale probe with Attached real Mix child: effective frame and grid both1.5, same PID and vt-1 owner, visible glyph enlargement and reset captures inspected. Existing search options/history and second-pane ownership/close sequence also passes; all six tracked processes stopped. Complete state/calls/settings, focused witness and capture hashes retained. This closes the bounded debug repair; final release must include it.

Expanded private native.D8cydN captured eleven additional accepted-revision states on the same process/query: Ocean scheme, Breeze style, orange accent, compact/square, spacious/round, UI family/size/weight, DejaVu Sans Mono, icon stroke, per-Term scheme/mode/contrast, style/chrome tunables and reset. All eleven original PNGs inspected and hashes retained. This is candidate-only debug evidence, not matched gold or complete Prefs acceptance. Ocean and subsequent Breeze-style images are byte-identical despite the effective style change; native gold comparison is still required before judging that behaviour. The orange overprint first misread as Unconfigured is egui Unaligned, emitted by register_rect under cfg(debug_assertions) when debug.show_unaligned sees fractional geometry; release excludes it. No production fix/deviation is inferred from that diagnostic.

The expanded run later failed second-pane binding with no usable ready grant while the60s mint floor was pending. All five tracked PIDs stopped. The failed long-run admission case is retained independently; previous successful shorter pane proof does not close it. A lint-clean Prefs-only release variant is prepared, with this separate pane block excluded from its scope and long-idle admission explicitly still failed/open. Do not weaken the floor or label this harness change a fix.

Next CBC `term-prefs-release-c7efc31c` runs `tools/gold_parity_gate.mix --prepare`. Read complete result/worker diff, exact-repeat as required, fetch matching release bytes/receipt and exact bundle and verify source/profile/hash. Run the prepared Prefs-only matrix on release, inspect captures including app-scale and absence of debug overlay, retain cleanup. Then compare equivalent gold states; complete auto/tune/app inputs and settings authority loss/recovery, long-idle pool/issuer expiry/revoke/enforcement, Bus/actions/topics/menu/macros/broker/IME/reporting/clipboard/search edge cases/show failures/conformance/visual acceptance. No Fable deviation, install, gold, desk or merge changes. NOT ACCEPTED.

branch: lab/parity-term; head SHA:c7efc31c1b0db76375ac8aef7b78076c2057c249 (exact app-scale repair; receipt HEAD records checkpoint); NOT ACCEPTED — release Prefs proof/matched gold and full parity matrix open, long-run pane grant failure retained; CBC job ids:term-app-scale-c7efc31c-a1 (green exact;210 terminal tests; native.0ovg7l focused pass/native.D8cydN partial matrix), term-prefs-release-c7efc31c (next await).


### Release Prefs matrix and matched Gold capture checkpoint

CBC `term-prefs-release-c7efc31c-a1` passed exact source `c7efc31c1b0db76375ac8aef7b78076c2057c249`, no worker diff. Complete188137-byte log read in eleven contiguous chunks through GOLD_PARITY_PATCH_END and archived. Shared Viu/engine fmt, all-target check/clippy, tests and debug/release builds passed:478 tests, eleven retained ignores; existing warnings remain. Verified release source/profile receipt and SHA256 `70413e272cf96ca3aef4493179cefe7bc1d45446c6049acd509ddd695abdf84a`, exact-source bundle and ELF dependencies retained privately. This includes the app-specific text-scale repair and new VT conformance cases.

Private native.48EkzI passes the release Prefs/search matrix on one unchanged candidate PID and vt-1 owner, with a real Attached Mix child. App-scale1.5 and reset, eleven broader Prefs states, query preservation and native search options/history pass; all thirteen relevant original captures inspected. Debug Unaligned overprint is absent from release. All five tracked PIDs stopped. Pane admission was explicitly outside this run; the earlier long-run grant failure remains open.

Private native.YDKyxj then runs pinned Gold and these exact candidate release bytes sequentially on the same owned Bus, settingsd, canonical term identity and isolated home. Same real Mix pair() output,900x560 client pixels, same settings for each capture; all sixteen live changes preserve each app's PID. Settings apply receipts, titlebar/menu/window readbacks, snapshots,34 PNG hashes, complete calls,17 comparison sets and an inspected witness index are retained. All seven tracked processes stopped. Both originals plus all full side-by-side/difference figures inspected. Every tabs and terminal region has SSIM/RMSE distortion0.

ImageMagick7.1.2-35 reports normalised distortion, lower is better:

| Matched states | Whole SSIM / RMSE | Header SSIM / RMSE | Inspected difference |
| --- | --- | --- | --- |
| Baseline, baseline reset, app-scale1.5/reset, Ocean, Breeze style, orange accent, all reset (8 pairs) | 0 / 0 | 0 / 0 | Byte-identical |
| Light/font18 and light/scale1.25/weight700 | 0.000394758 / 0.00393857 | 0.0058908 / 0.0151196 | Incorrect selected mode toolbar button |
| Compact/square | 0.000141901 / 0.00474248 | 0.00248339 / 0.0198392 | Macro/search and Close icon raster edges |
| Spacious/round, UI font, mono font, icon stroke | 8.11716e-9 / 0.0000306314 | 9.88176e-8 / 0.000106876 | Close glyph raster edge |
| Per-app light/Crimson/high contrast and tunables | 0.000852412 / 0.0119387 | 0.010503 / 0.0416554 | Incorrect selected mode button plus Close raster edge |

Regions follow Gold geometry: header height38/tabs32 normally, compact32/28, spacious46/38; terminal fills from their sum to560. Separate toolbar crop x620,width150 uses matching header height. Light toolbar SSIM/RMSE0.0471264/0.0427647; app-look/tune0.0840232/0.117819; compact0.019863/0.05611; all other toolbar regions0. Ocean to Breeze produces identical pixels in Gold too, closing the earlier question about this particular state. This does not establish every style/tunable or authority-recovery case.

Code `d6aa18cbbeb3363ae82e4d58a94347cd52faea48` repairs the four light-mode pairs: pinned Gold toolkit titlebar::icon_bar explicitly excludes view.mode from toggled styling. Term now suppresses checked only on that toolbar projection, retaining the true checked action registry and moon/sun icon selection. Three regression assertions, Mix lint and diff check pass. This is Gold restoration, no Fable deviation. The exact-source release gate `term-toolbar-release-d6aa18cb` is submitted; gate result and new release-byte paired captures remain pending. Previous c7 captures remain failed for those four toolbar states, not retroactively accepted.

Continue exact gate/log/patch processing, fetch matching release/receipt, rerun prepared17-state paired harness and region inspection. Then complete automatic mode/settings authority loss/recovery, remaining Prefs inputs/tunables, long-idle pane grant failure/issuer expiry/revoke, full Bus/actions/topics/menu/macros/broker, native IME/mouse/paste/clipboard failures/search edges/show failures, broader conformance and visual acceptance. No install, Gold/desk/VT or merge changes. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:d6aa18cbbeb3363ae82e4d58a94347cd52faea48 (mode-toolbar repair; latest tested release c7efc31c; receipt HEAD records checkpoint); NOT ACCEPTED — toolbar exact release/recapture and full parity matrix open, long-run pane grant failure retained; CBC job ids:term-prefs-release-c7efc31c-a1 (green exact;478 tests; native.48EkzI and native.YDKyxj17 matched states), term-toolbar-release-d6aa18cb (submitted).


### Toolbar release repair closed; settings authority-loss header repair

CBC `term-toolbar-release-d6aa18cb-a1` passed exact `d6aa18cbbeb3363ae82e4d58a94347cd52faea48`, no worker diff. Complete159493-byte saved log read in nine contiguous chunks through GOLD_PARITY_PATCH_END and archived.478 tests passed, eleven retained ignores; shared fmt/check/all-target clippy/debug/release pass, existing warnings remain. Verified release source/profile/SHA256 `cec4d8ab4867d03465f061745b370f7f66c814a774e2acb8e7dc4697a4dc18d4` and exact-source bundle. All eleven Term Mix suites additionally pass with installed Mix.

Private native.V2Gfxc repeats17 paired captures, all seven tracked processes stopped. Four light-mode toolbar differences are closed: light/font and light/scale pairs now byte-identical; per-app light/Crimson and tune pairs have only Close glyph raster edges (whole SSIM/RMSE distortion9.56394e-9/0.0000316119; header1.16431e-7/0.000110297; toolbar0). Across sixteen live-change states, nine whole pairs identical and all tabs/grid regions0. Other remaining compact/spacious raster edges equal the preceding receipt. Changed originals and four changed full figures inspected;29 unchanged originals hash-match previously inspected evidence. Initial Gold session image retained an extra command-echo row and is explicitly UNMATCHED (whole0.00899218/0.0453169), not a pass. Later baseline-reset pair is exact. All comparison sets and a source/hash/inspection witness retained.

Private native.hlVr7d probes auto without daylight, app light override/reset, invalid mode, settingsd stop/restart and recovery, all on unchanged app PIDs with Attached real Mix children. Five of six matched900x560 pairs are byte-identical in every region: three auto/app states, explicit recovery light and restored authority. Invalid mode returns identical rc10 validation_failed diagnostics and unchanged revision in both runs. Candidate explicitly observes lost=true then false with durable settings restored; no claim of a different authority incarnation. All nine tracked processes stopped. All twelve originals and six side-by-side/difference figures inspected; calls, refused replies, state, hashes and comparison witness retained.

Authority-loss header FAIL remains explicit: Gold switches to default window policy (server decorations, no client captions/title/icon); port retained prior CSD policy. Tabs/grid0, whole SSIM/RMSE distortion0.006207/0.0261336, header0.0914716/0.100323, toolbar0.115106/0.0938356. Gold's loss screenshot and existing native default-decoration resolver identify the mismatch. This supersedes any implication that loss retained the preceding light frame in Gold.

Code `1c88506fc5cfcaee7bf63c1069b3191723430b7e` carries the explicit session lost flag into Term's Mix projection and uses default window policy only during actual authority absence. Transient read errors keep the accepted policy; recovery restores live policy. Existing PTY/model and seamless terminal band remain owned. Four new assertions pass (21 frame checks); lint passes including the host-provided function declarations; diff check passes. This is a targeted repair pending native proof, not an acceptance or approved deviation. Prepared private prefs-recovery-1c88506f-release.mix will repeat the failed loss/recovery case on matching release bytes.

Next CBC `term-authority-release-1c88506f` runs the full gold_parity_gate at that exact source. Read complete log, apply any returned diff/repeat, fetch source/profile/hash-verified release, repeat authority harness, inspect all relevant originals/figures and require header closure. Then continue daylight/mode_override publications, remaining Prefs/input/session/Bus/macros/menu/conformance/full visual cases. Long-idle pane grant failure and issuer revoke/expiry remain open; the pinned Gold native actor also contains the same no-ready-grant fallback, requiring a matched runtime diagnosis before proposing repair. No mint-floor weakening, install, desk/VT or merge changes. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:1c88506fc5cfcaee7bf63c1069b3191723430b7e (authority-loss frame repair; exact tested release d6aa18cb; receipt HEAD contains checkpoint); NOT ACCEPTED — authority-loss exact gate/native closure and full parity matrix open; CBC job ids:term-toolbar-release-d6aa18cb-a1 (green exact;478 tests; native.V2Gfxc and native.hlVr7d), term-authority-release-1c88506f (next await).


### Exact authority-loss release proof; native IME/paste witness

CBC `term-authority-release-1c88506f-a1` passed exact `1c88506fc5cfcaee7bf63c1069b3191723430b7e`, no worker diff. Complete164323-byte saved log read in six contiguous chunks through GOLD_PARITY_PATCH_END and archived.478 tests pass, eleven retained ignores; shared fmt/check/all-target clippy/debug/release pass with existing warnings. Verified release source/profile receipt, SHA256 `cd22f53743a671a8cbe39f41c386712ccb8a04cdc9154001a8088346a6c094c5`, exact-source bundle and ELF dependencies retained.

Private native.NNLq2N repeats auto fallback, per-app light override/reset, explicit light, settingsd loss and restart on those exact release bytes. All six900x560 pairs are byte-identical, with whole/header/toolbar/tabs/grid SSIM/RMSE distortion0. The loss-state header defect is closed: default server-decoration policy matches Gold. Changed candidate original and full side-by-side/diff inspected; eleven other originals hash-match the inspected preceding run. Identical validation_failed rc10 invalid-mode refusals, unchanged settings revision, lost=true/recovered=false-to-true readbacks and unchanged app PIDs retained. All nine owned PIDs stopped. Durable settings incarnation survives restart; this does not attest replacement with another authority incarnation or daylight/mode_override publications. Private witness contains source/bytes, capture hashes, comparison regions, exact refusal and closure records.

Private native.8LbcdN adds bounded candidate-only native IME and clipboard input against the same release. Preedit appears over the terminal while neither Japanese character enters the PTY grid; committed Japanese text reaches real Mix and is printed. Native Ctrl+Shift+V Paste then prints Unicode through Mix. Three original captures inspected, raw snapshots/calls retained; all five tracked PIDs stopped. CJK fallback glyphs on this node are tofu, as in preceding matched Gold output; this is not full font/IME visual acceptance. Pane switch/close/delayed commit and actual platform cancellation, bracketed-paste protocol byte capture and clipboard-provider failures remain open.

Two failed harnesses retained: native.S0Lexe expected compact Japanese text despite the raw snapshot's wide spacer cells; corrected test checks each preedit character is absent before commit and the exact visible wide-cell output after commit. Native.f99hsq omitted Shift on Paste; gold input policy correctly sends control byte22, not clipboard text. Corrected native probe explicitly uses Ctrl+Shift+V; no production input changes or weakened encoding assertions. Each failed run stopped all five tracked PIDs.

Next test-only code `b58c6a129a42bc9570d00730da466025547a7474` adds real-parser/grid conformance for ED0/1/2 (all affected rows, cursor cell inclusion and preserved position) and DEC special graphics line drawing plus return to ASCII. Existing expectations remain unchanged; diff check passes. No local Rust execution or pass claim for these new cases. Next CBC `term-vt-display-b58c6a12` runs the scoped term_gate, including all Term Mix suites/foreign checks. Process full log/fmt diff and exact repeat. Latest native-tested production/release remains1c88506f.

Continue remaining full Bus/actions/topics/menu/macros/broker, session long-idle admission/revoke/expiry enforcement, native IME/mouse/paste/clipboard/search edge cases, remaining Prefs/daylight/authority cases, conformance and full visual acceptance. Initial unmatched command-echo capture from native.V2Gfxc remains failed; subsequent baseline-reset states match. No Fable deviation, install, Gold/desk/VT or merge changes. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:b58c6a129a42bc9570d00730da466025547a7474 (new VT tests; native-tested release1c88506f; receipt HEAD contains checkpoint); NOT ACCEPTED — added VT gate and full parity matrix open; CBC job ids:term-authority-release-1c88506f-a1 (green exact;478 tests; native.NNLq2N six exact matched pairs and native.8LbcdN bounded input), term-vt-display-b58c6a12 (next await).


### VT ED1 conformance found a functional Gold defect (Fable C1)

CBC `term-vt-display-b58c6a12-a1` failed with211 terminal tests passed, one new ED1 failure and five retained ignores. Complete35406-byte log read in two contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites (21 frame checks), two bundle tests and fifteen runtime/call/timer tests passed; DEC special graphics passed. ED1 from zero-based row1 left row0 as AAAAAAAA instead of erasing it. No expected row was weakened. Foreign gate/host-build stages were not reached. Returned formatting-only fixture diff applied.

Inspected pinned Rio932c1a7 clear_screen(ClearMode::Above): cursor.row>1 skips clearing row0 from cursor row1. Private native.OswUGj confirms the defect in pinned Gold8779f33 and verified unpatched release1c88506f, using the same real Mix ANSI output and settled900x560 captures. Both originals/full side-by-side/diff inspected; PNGs are byte-identical SHA256 `9201f837040dcec02a92a793394391e65a392e2b191139756e670cf1209b86e3`, whole SSIM/RMSE distortion0. Both incorrectly retain the first row. All seven tracked PIDs stopped. Gold bug note, raw snapshots/calls/source/binary receipt and witness retained. Earlier native.vZ4Stc used a marker overwritten by the shell prompt and failed its harness; native.7GgUa5 retained the defect with an unmatched initial echo row. Both failed/diagnostic runs stopped all seven owned PIDs; neither is acceptance evidence.

Under Fable C1 this breaks erase function and is repaired rather than copied. Code `adb505e9fb2151ad77f6aed5a8e1be99a3aeebfe` changes only the pinned parser's row guard to>0. To carry the reviewed one-line change, imported the pinned rio-vt crate (59 upstream files, MIT licence), materialised its upstream workspace manifest inheritance and retained sibling Rio Git pins/dependency flags. Cargo patch routes only rio-vt locally; no new dependency or strict-tier exception. D6 simdutf remains Viu-only debt. provenance/rio-vt.json records every original SHA256/Git blob and adaptation; vendor/rio-vt/PATCHES.md records the defect and exit path (fixed reviewed upstream revision). No other imported Rust source differs. ED0/1/2 assertions and DEC test remain intact.

Next CBC `term-ed1-c1-adb505e9` runs term_gate --prepare. Read complete log, inspect the lock change (local rio-vt source only expected), apply fmt/lock patch, repair any manifest/gate issue and repeat exact source without worker diff. Then fetch matching native artifact and run the prepared ED1 C1 harness, requiring A retained in Gold and absent in repaired candidate, with both captures, same unaffected regions and cleanup. Final release must include the parser repair; prior exact release1c88506f remains valid only for its bounded tested cases and retains this bug. Complete remaining full parity matrix; no install/desk/VT/merge changes. Overall NOT ACCEPTED, no Fable question required for this C1 functional correction.

branch: lab/parity-term; head SHA:adb505e9fb2151ad77f6aed5a8e1be99a3aeebfe (C1 ED1 parser repair; latest native-tested release1c88506f retains Gold defect; receipt HEAD records checkpoint); NOT ACCEPTED — parser repair gate/native proof and full parity matrix open; CBC job ids:term-vt-display-b58c6a12-a1 (failed new ED1 witness;211 terminal pass), term-ed1-c1-adb505e9 (next await).


### ED1 repair passes; preserve the PTY patch guard

CBC `term-ed1-c1-adb505e9-a1` ran and failed:210 terminal tests passed, two PTY patch-guard failures, five retained ignores. The new ED0/1/2 and DEC special-graphics conformance tests both pass with the patched parser. All eleven Term Mix suites, two bundle tests and fifteen runtime/call/timer tests also pass. Complete35815-byte saved log read in two contiguous chunks through TERM_PATCH_END and archived privately. Foreign/host-build stages were not reached.

The unchanged PTY guard requires its override immediately after the Rio patch header. Inserting rio-vt first broke that exact manifest-shape assertion; no PTY source/hash drift occurred. Code `5ff72340445a85e8b442cd789c9ab9f99c11d6b4` restores that entry ordering and applies the entire returned lock diff: only rio-vt loses its Git source because it now resolves to the reviewed local patch. No dependency versions or flags change; guard tests and all conformance expectations remain intact. Diff check passes.

Next exact-source CBC `term-ed1-lock-5ff72340` runs term_gate --prepare. Require the full scoped/foreign/build gate and no worker diff, then fetch matching debug bytes and run the prepared C1 Gold/candidate harness with inspected original/region figures and verified cleanup. Final release and full parity matrix remain open; latest native-tested release1c88506f retains the Gold ED1 defect. No installs, desk/VT work or merges. NOT ACCEPTED.

branch: lab/parity-term; head SHA:5ff72340445a85e8b442cd789c9ab9f99c11d6b4 (manifest/lock repair; receipt HEAD contains checkpoint); NOT ACCEPTED — exact parser gate/native proof and full parity matrix open; CBC job ids:term-ed1-c1-adb505e9-a1 (ED1 pass; two manifest guards failed), term-ed1-lock-5ff72340 (next await).


### Exact ED1 scoped gate and native C1 proof

CBC `term-ed1-lock-5ff72340-a1` passed exact source `5ff72340445a85e8b442cd789c9ab9f99c11d6b4` with no worker diff. Complete57298-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites pass (21 frame checks), two bundle tests, fifteen runtime/call/timer tests,212 terminal tests with five retained ignores, and twenty scoped host/input/editor tests pass. Foreign gate passes:17 listed foreign crates; strict mix/mix-shell/system/mixd clean. Debug host build passes. Verified source/profile/hash receipt: SHA256 `57b884c69ddd1d930844e9769137440ef8b451d8e2c8ecfbd70e7ed90fd41bf5`.

Private native.XP25Gn runs identical settled real Mix ANSI output sequentially in pinned Gold and those verified debug bytes. Gold retains AAAAAAAA on row0 after ED1 from row1; candidate correctly erases it. Both originals and full side-by-side/diff inspected.900x560 comparison: header/toolbar/tabs/remaining-grid SSIM and RMSE distortion0. The intentional erased-row correction has SSIM0.0285673/RMSE0.0962853; whole0.00106247/0.0172624 (ImageMagick distortion convention; lower is closer). Gold PNG SHA256 `9201f837040dcec02a92a793394391e65a392e2b191139756e670cf1209b86e3`; repaired candidate `29759c05a71f52e8896b652d2d027b50d5525591716c07caffc662e298d4d900`. Current-row erasure, cursor/prompt and all other regions match. Fable C1 functional bug note, raw snapshots/calls, source/profile/hash, region figures and witness retained privately. All seven tracked PIDs stopped.

Next `term-ed1-release-5ff72340` runs gold_parity_gate --prepare at that exact source. Read complete saved log, apply/repeat any returned diff, fetch matching verified release, repeat ED1 C1 native capture proof on release bytes. Then continue full Bus/actions/topics/menu/macros/broker, long-idle admission/revoke/expiry, native input/provider/search edges, daylight/settings-authority/remaining Prefs and full visual matrix. This debug repair proof does not attest final release bytes or overall parity. No installs, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:5ff72340445a85e8b442cd789c9ab9f99c11d6b4 (exact scoped/debug C1 proof; receipt HEAD contains checkpoint); NOT ACCEPTED — release C1 proof and full parity matrix open; CBC job ids:term-ed1-lock-5ff72340-a1 (green exact; native.XP25Gn), term-ed1-release-5ff72340 (next await).


### ED1 C1 repair verified on exact release bytes

CBC `term-ed1-release-5ff72340-a1` passed exact source `5ff72340445a85e8b442cd789c9ab9f99c11d6b4`, no worker diff. Complete163799-byte saved log read in five contiguous chunks through GOLD_PARITY_PATCH_END and archived.480 tests passed, eleven retained ignores; shared fmt/check/all-target clippy/debug/release pass with existing warnings. The preceding exact scoped gate separately passed all eleven Term Mix suites,212 terminal tests and foreign/strict-tier checks. Verified release source/profile/hash: SHA256 `51f3f4b1b467cbe2097e382a47be7bc94b459b3df83fb1886f6a3a58da7202e4`; exact-source bundle and ELF dependency readback retained.

Private native.VWJ50K repeats the C1 ED1 proof on that release, with identical Gold/repair PNG hashes and all regional metrics to debug native.XP25Gn. Both originals and full side-by-side/diff inspected again. Only the AAAAAAAA row is erased; header/toolbar/tabs/remaining grid SSIM/RMSE0, whole0.00106247/0.0172624 for the intentional C1 functional correction. Real attached Mix children, raw command/output and seven-PID all-stopped receipt retained. The ED1 correction is closed for release bytes; full app acceptance is still open.

Investigating the earlier native.D8cydN idle-pane assertion against the explicit Gold graphics-only fallback contract. Pinned tabs/bus/session source documents opening a functional shell without an enrolled control binding when no ready grant exists, including look-ahead exhaustion. Do not classify this designed refusal/fallback as a parser-style C1 bug or weaken the60s mint floor. A new paired release diagnostic opens a split35s after launch to measure both apps in the expired-grant/floor interval; input/recovery and long-idle expiry/revoke remain to verify. Continue remaining full Bus/actions/topics/menu/macros, native input/provider/search, daylight/authority/Prefs and visual cases. No installs, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:5ff72340445a85e8b442cd789c9ab9f99c11d6b4 (exact release ED1 proof; receipt HEAD contains checkpoint); NOT ACCEPTED — full parity matrix open; CBC job ids:term-ed1-release-5ff72340-a1 (green exact;480 tests; native.VWJ50K), term-ed1-lock-5ff72340-a1 (green scoped/foreign; native.XP25Gn).


### Idle split fallback matches the Gold contract; resize conformance checkpoint

Private native.JjpWpv and native.Yk186O drive the pinned Gold and exact5ff72340 release sequentially in owned services, opening a split35s after launch. Both reply `split id=2 dir=v tab=1 pane=2 revision=2` and report the identical pane2 graphics-only diagnostic while waiting for the60s mint floor. Pane1 stays Attached. The pinned tabs/native-session/Bus source explicitly documents this fallback, including no usable ready grant/look-ahead exhaustion. This resolves the earlier blanket Attached expectation for this measured interval; native.D8cydN remains a failed harness, not silently relabelled as a passing test. No production admission change or mint-floor weakening.

Native.Yk186O then types `print("IDLE_".."SHELL_OK")` to pane2 through term.type; each real Mix child prints IDLE_SHELL_OK. The shell remains functional without enrolled protected control. Both900x560 original captures and full figures inspected: header/tabs/left/right/full SSIM/RMSE distortion0; PNG pair SHA256 `9c775d16541cad02651a706079ad5533515f3f397123fef1522c3c3241550680`. Earlier blank-split pair is also identical SHA256 `cb4005a5c2f16abb6acdda7269b4d824ee9a63e9109c03f5413c86b92133469f`. All nine tracked PIDs stopped in each run. Private witnesses retain native replies, real-output snapshots, source/binary hashes, comparisons and cleanup. The >5min no-presence path, refreshed grant/revoke/expiry recovery, protected refusals and CPU/frame-pacing acceptance remain open; captured diagnostic timing/frame counts are not presented-frame or CPU evidence.

Next test-only code `6ecbe3e6e2dc5ce15ef8e3b5f10f6eeadf8786e1` adds real-parser/grid resize conformance: narrow/widen logical-line reflow with cursor and wrapped selection, plus alternate-screen resize followed by restoration of main text/saved cursor. Existing conformance expectations remain unchanged. Diff check passes; no local Rust execution or pass claim for new cases. Next CBC `term-resize-vt-6ecbe3e6` runs term_gate --prepare. Process complete log and any worker diff, repeat exact source if needed. Production/release remains verified5ff72340; receipt-only docs do not change it. Continue the full open Bus/actions/topics/menu/macros/broker, native IME/mouse/paste/provider/search, Prefs/daylight/authority and visual matrix. No install/desk/VT/merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:6ecbe3e6e2dc5ce15ef8e3b5f10f6eeadf8786e1 (resize conformance; exact native-tested release5ff72340; receipt HEAD contains checkpoint); NOT ACCEPTED — new VT gate and full parity matrix open; CBC job ids:term-ed1-release-5ff72340-a1 (green exact;480 tests; native.VWJ50K/JjpWpv/Yk186O), term-resize-vt-6ecbe3e6 (next await).


### Resize fixture correction against pinned Gold semantics

CBC `term-resize-vt-6ecbe3e6-a1` failed with212 terminal tests passed, two new resize-fixture failures and five retained ignores. All eleven Term Mix suites, two bundle tests and fifteen runtime/call/timer tests passed. Complete35077-byte saved log read in two contiguous chunks through TERM_PATCH_END and archived; no worker diff. Foreign/host-build stages were not reached.

The new fixtures assumed viewport pinning and automatic cursor homing that pinned Gold does not promise. Rio swap_alt explicitly clones the primary cursor into the alternate screen; the fixture now explicitly homes before writing alt. Rio shrink_columns keeps the tail in the live viewport and moves the leading segment into history; the test now checks both exact live tail/cursor and exact history rows/offset, then wrapped selection, full widen restoration and final cursor. No text-preservation, selection, main-screen restoration or final-cursor assertion is removed. This corrects test inputs/viewport assumptions to the Gold contract; no production/parser change. Pinned and imported grid/resize.rs are byte-identical SHA256 `35a92a6e146ecd976e96ec86681362376d5057cc26d29c3f0e39651b0fe4ae8e`; pinned swap_alt source inspected. Native narrow/widen reflow capture acceptance remains open; the fixture correction is not a native pass claim.

Code `f4a3ff4d762e4b967e8c142e2c954f6999d3090d` carries those corrected fixtures. Diff check passes. Next CBC `term-resize-history-f4a3ff4d` runs term_gate --prepare at exact source. Read complete log, repair actual failures without dropping coverage, process fmt/lock diff and exact-repeat if needed. Latest native-tested production release remains5ff72340 with C1 ED1 repair verified; exact release gate480 tests/eleven ignores and matching idle-fallback evidence remain valid for those bytes. Continue the standing full parity matrix; no installs/desks/VT/merges. NOT ACCEPTED.

branch: lab/parity-term; head SHA:f4a3ff4d762e4b967e8c142e2c954f6999d3090d (resize fixture correction; exact native-tested release5ff72340; receipt HEAD contains checkpoint); NOT ACCEPTED — resize gate/native reflow and full parity matrix open; CBC job ids:term-ed1-release-5ff72340-a1 (green exact;480 tests), term-resize-vt-6ecbe3e6-a1 (two new fixture failures), term-resize-history-f4a3ff4d (next await).


### Resize widening exposed a second functional Gold defect (C1)

CBC `term-resize-history-f4a3ff4d-a1` failed with213 terminal tests passed, one widening restoration failure and five retained ignores. Alternate-screen resize now passes. All eleven Term Mix suites, two bundle tests and fifteen runtime/call/timer tests pass. Complete35493-byte log read in two contiguous chunks through TERM_PATCH_END and archived; no worker diff. Foreign/host-build not reached. Narrowing/live tail/history/selection assertions pass; the widened live row is blank instead of restored abcdefgh, so the final restoration assertion is retained.

Private native.xswpOd reproduces the defect in pinned Gold and verified unmodified release5ff72340. Real Mix write_stdout emits80 characters with cursor80,0 and sleeps. Native comp.window.place resizes900→450→900 at constant560 height:98→48→98 columns,23 rows. Both apps hide the rejoined text in history and reset cursor0,0 over a blank live viewport after widening. Six originals and three full side-by-side/diffs inspected; baseline/wide pairs exact, wide PNG SHA256 `41a00b2769013b2c8ebe2a036a3dde0bb83d2dfb78d3b5103bc13386e02e2ff8`. All seven owned PIDs stopped. Native calls, history snapshots, binary/source hashes, figures, witness and Gold bug note retained privately. This visibly hides function and qualifies for Fable C1; no question/stop needed.

Code `957dc25b0b83cfb42c306ca61522bdedd7f1862e` repairs the pinned grow_columns calculation: use unbounded temporary cursor positions for two subtractions before deriving movement/relocating content. Early viewport clamping lost the column and row delta for a merge target in history. No other parser production change; exact text/selection/final-cursor assertions remain. Provenance and local patch/exit notes updated. Diff check passes, no local Rust execution/pass claim. Next CBC `term-reflow-c1-957dc25b` runs term_gate --prepare; process full log/diffs and exact-repeat, then fetch matching debug artifact and run the prepared C1 harness requiring the live80-character line/cursor80,0 only in repaired candidate. Follow with exact release proof.

Separate newly measured narrow-window visual item stays OPEN:450px Gold collapses menus to hamburger, candidate shows File. Header SSIM/RMSE distortion0.0105186/0.0338239; tabs/grid0; whole0.000713761/0.00881094. This is not raster noise and not accepted. Repair the generic menu-overflow policy after reflow closure. Process frame counts also differ; no CPU/presented-frame acceptance claim. Full Bus/Pprefs/input/authority/conformance/visual matrix remains open. No install, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:957dc25b0b83cfb42c306ca61522bdedd7f1862e (C1 reflow repair; native-tested release5ff72340 retains this Gold defect; receipt HEAD contains checkpoint); NOT ACCEPTED — reflow gate/native/release, narrow menu and full parity matrix open; CBC job ids:term-resize-history-f4a3ff4d-a1 (one widening failure;213 terminal pass), term-ed1-release-5ff72340-a1 (green exact; native.xswpOd before repair), term-reflow-c1-957dc25b (next await).


### Reflow repair: avoid position helpers while raw storage is taken

CBC `term-reflow-c1-957dc25b-a1` failed:213 terminal tests passed, one reflow panic and five retained ignores. All eleven Term Mix suites, two bundle tests and fifteen runtime/call/timer tests pass. Complete34309-byte saved log read in two contiguous chunks through TERM_PATCH_END and archived; no worker diff. Foreign/host-build stages not reached. Existing exact text/selection/cursor checks remain unchanged.

The initial C1 repair used Pos::sub with Boundary::None to preserve temporary history coordinates. That helper still consults grid bounds; grow_columns temporarily takes the raw row storage, so total_lines becomes zero and the helper panics in remainder arithmetic. Code `4c5a1dfa90874a2bfe000045efe9676bf3a1ee28` computes the temporary row/column directly from the known new width instead, including the delayed-wrap decrement, before the existing relocation logic. This does not change the fixture expectation or remove any assertion. Added160 width/cursor combinations (eight narrow widths, twenty input lengths), requiring exact full live text, final cursor and zero viewport offset after widening. Patch/provenance notes updated; no unrelated parser changes. Diff check passes. No local Rust execution or passing claim for this repair.

Next CBC `term-reflow-arithmetic-4c5a1dfa` runs term_gate --prepare. Read complete log, process any returned diff and exact-repeat, then fetch verified matching debug artifact and run prepared resize-reflow-c1-4c5a1dfa-debug harness. It requires the80-character line/cursor80,0 to remain live after widening only in repaired candidate, with pinned Gold retaining the recorded defect. Capture/inspect C1 difference, run exact release gate and repeat native on release. Native.xswpOd remains pre-repair evidence; narrow450px menu/header mismatch and process-frame pacing remain OPEN. Continue full standing parity matrix; no install/desk/VT/merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:4c5a1dfa90874a2bfe000045efe9676bf3a1ee28 (reflow arithmetic repair; native-tested release5ff72340 retains widening defect; receipt HEAD contains checkpoint); NOT ACCEPTED — reflow gate/native/release, narrow menu and full parity matrix open; CBC job ids:term-reflow-c1-957dc25b-a1 (failed position-helper panic;213 terminal pass), term-ed1-release-5ff72340-a1 (green exact; native.xswpOd before repair), term-reflow-arithmetic-4c5a1dfa (next await).


### Exact scoped reflow gate and native C1 repair proof

CBC `term-reflow-arithmetic-4c5a1dfa-a1` passed exact `4c5a1dfa90874a2bfe000045efe9676bf3a1ee28` with no worker diff. Complete58625-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites (21 frame checks), two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and twenty scoped host/input/editor tests pass. The resize reflow test and160-combination width/cursor matrix pass with all exact restoration assertions retained. Foreign gate passes:17 listed foreign crates, strict mix/mix-shell/system/mixd clean. Debug build passes. Matching source/profile/hash-verified executable SHA256 `91f3e27dae43cfcf55714f536f8cb10e735e007175bd76d7fb8839e9b351e23a` and exact-source bundle fetched.

Private native.nZ29M5 repeats same real Mix write_stdout80-character line and native compositor900→450→900 resize at fixed560 height. Candidate now restores the full live line and cursor80,0; Gold leaves a blank live row/cursor0,0 with the text in history. Wide candidate PNG SHA256 `314897e801ce0f6446cfda3d5bb841e96139a6415566f79782b3c5dc88102406` exactly equals its original baseline; Gold wide remains `41a00b2769013b2c8ebe2a036a3dde0bb83d2dfb78d3b5103bc13386e02e2ff8`. Both wide originals/full side-by-side/diff inspected. Wide header/tabs/remaining-grid SSIM/RMSE distortion0; restored-row0.252078/0.245939, whole0.0101951/0.0492975 reflect the intentional C1 repair. Baseline exact in all regions. Four baseline/narrow originals hash-match the preceding inspected native.xswpOd originals; explicit inspection-reuse receipts retained. Narrow tabs/grid0, header File-vs-hamburger mismatch persists and remains unaccepted. All seven owned PIDs stopped. Private witness includes source/binary receipt, command/calls/snapshots, region metrics, Gold bug note and closure.

Next `term-reflow-release-4c5a1dfa` runs gold_parity_gate --prepare at that exact source. Read complete log, apply/repeat any returned diff, fetch verified release and repeat the prepared C1 native harness. Reflow is only debug-proven until those release bytes are tested. Then repair/test the generic narrow-menu overflow policy, investigate measured idle frame pacing, and continue full Bus/actions/topics/menu/macros/broker, native input/provider/search, Prefs/daylight/authority and final visual matrix. No installs, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:4c5a1dfa90874a2bfe000045efe9676bf3a1ee28 (exact scoped/debug reflow proof; receipt HEAD contains checkpoint); NOT ACCEPTED — reflow release proof, narrow menu and full parity matrix open; CBC job ids:term-reflow-arithmetic-4c5a1dfa-a1 (green exact;215 terminal tests; native.nZ29M5), term-reflow-release-4c5a1dfa (next await).


### Exact release reflow proof; generic narrow-menu repair checkpoint

CBC `term-reflow-release-4c5a1dfa-a1` passed exact source `4c5a1dfa90874a2bfe000045efe9676bf3a1ee28`, no worker diff. Complete167679-byte saved log read in five contiguous chunks through GOLD_PARITY_PATCH_END and archived.483 tests passed, eleven retained ignores; shared fmt/check/all-target clippy/debug/release pass with existing warnings. Matching source/profile/hash-verified release SHA256 `983800633ac604763ff65723e6bf18cb87bae4515a6ebfd4089406655a49e5a0` and exact-source bundle fetched. The preceding exact scoped gate separately attests all Term Mix suites,215 terminal tests and foreign/strict-tier checks.

Private native.hiTUr1 repeats the C1 reflow harness on these release bytes: real Mix80-character output remains live after900→450→900 and cursor returns80,0; pinned Gold retains blank live row/cursor0,0. All six originals hash-match the inspected debug native.nZ29M5 originals. Fresh regional comparisons retained: baseline all0; wide header/tabs/remaining-grid0, restored-row SSIM/RMSE0.252078/0.245939 and whole0.0101951/0.0492975 are solely the C1 correction. Narrow tabs/grid0, known header mismatch unchanged. Inspection reuse explicitly keyed by image hashes; source/binary/calls/snapshots/figures/Gold bug note/witness retained. All seven tracked PIDs stopped. Reflow C1 is closed on these release bytes; this is not full app acceptance.

Code `93b387f91725a45cc25acc65e2071e6a6846dd50` addresses the separate450px mismatch in the generic native scene-menu adapter. NativeMenu::draw had hardcoded compact=false, even while toolkit::menu::effective selected Button below640px. It now consumes that same native toolkit result, preserving hosting and all command mappings. No app-specific Rust policy. Added a render-level regression through NativeMenu::draw at900→450→639→640→900: File is painted only in bar form, and the application command mapping stays unchanged. Scoped term_gate now runs native_menu tests too. Existing tests/assertions retained, Mix gate lint/diff check pass; no local Rust build or pass claim for the new code.

Next CBC `term-menu-form-93b387f9` runs term_gate --prepare. Read full saved log, process fmt/lock diff and repeat exact source, then fetch matching debug artifact and rerun native narrow/wide capture plus actual menu open/command selection before full release proof. Narrow-menu visual status remains OPEN until measured. Continue frame-pacing diagnosis and full Bus/actions/topics/menu/macros/broker, native IME/mouse/paste/provider/search, Prefs/daylight/authority and final visual acceptance matrix. No install/desk/VT/merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:93b387f91725a45cc25acc65e2071e6a6846dd50 (generic native menu form repair; exact native-tested release4c5a1dfa; receipt HEAD contains checkpoint); NOT ACCEPTED — menu gate/native/release, frame pacing and full parity matrix open; CBC job ids:term-reflow-release-4c5a1dfa-a1 (green exact;483 tests; native.hiTUr1), term-menu-form-93b387f9 (next await).


### Generic narrow-menu gate passed; exact formatting repeat pending

CBC `term-menu-form-93b387f9-a1` returned rc0. Complete60838-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and22 scoped host/input/editor/menu tests pass. The actual NativeMenu render regression passes at900/450/639/640/900 widths, preserving command identity. Foreign gate passes:17 listed crates; strict mix/mix-shell/system/mixd clean. Debug build passes with existing warnings.

The worker returned two rustfmt-only changes in native_menu.rs. Applied verbatim in code `e12e9f9ffeee16494cbab02f70002dc57f0d1e8a`; no behavioural or test-expectation change. The fetched93b387f9 debug artifact has source/profile/hash receipt but was not used for native evidence because the gate built the formatted working tree. Exact repeat `term-menu-exact-e12e9f9f` is next. Read its complete saved log, process any diff, fetch matching verified debug artifact and run the prepared paired real-Mix resize harness. Then test actual hamburger/menu command interaction and exact release bytes. Narrow-menu visual status stays OPEN until captured and inspected.

Latest native-tested release remains4c5a1dfa, with C1 reflow and earlier ED1 repair evidence retained. Frame pacing and the full Bus/actions/topics/menu/macros/broker, native input/provider/search, Prefs/daylight/authority and visual matrix remain open. No install, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:e12e9f9ffeee16494cbab02f70002dc57f0d1e8a (CBC menu formatting applied; exact native-tested release4c5a1dfa; receipt HEAD contains checkpoint); NOT ACCEPTED — exact menu gate/native/release, frame pacing and full parity matrix open; CBC job ids:term-menu-form-93b387f9-a1 (rc0 with formatting diff;215 terminal tests and2 menu tests), term-menu-exact-e12e9f9f (next await), term-reflow-release-4c5a1dfa-a1 (green exact;483 tests).


### Exact menu gate and native narrow-menu proof; accessible-label follow-up

CBC `term-menu-exact-e12e9f9f-a1` passed exact source `e12e9f9ffeee16494cbab02f70002dc57f0d1e8a`, no worker diff. Complete59637-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and22 scoped host/input/editor/menu tests pass. Both native_menu tests pass, including900/450/639/640/900 rendering and stable application command identity. Foreign gate passes:17 listed crates; strict mix/mix-shell/system/mixd clean. Verified matching debug executable SHA256 `19dd20fc3002bfca21327e29cf825c3b200ea94ed1e567197a16ba4c6c281a68` and exact-source bundle.

Private native.qP45IY repeats real Mix80-character output and900→450→900 resize. Baseline900 and narrow450 whole/header/tabs/grid are byte-identical to Gold, SSIM/RMSE0. Narrow PNG pair SHA256 `b4e6757e2af89b01c7de0c11392a377d3b61b57eb6c51f2c5542637e8f2a5b5f`. The prior File-vs-hamburger visual defect is closed on these debug bytes. All eight originals and four full comparison figures inspected. Wide retains only the recorded C1 reflow correction: whole0.0101951/0.0492975, restored-row0.252078/0.245939, other regions0. Native pointer press/release opens the hamburger; header/popup regions0. Exposed text behind popup differs from the preceding C1 reflow correction (whole0.00225503/0.0218059), not a menu defect. All seven owned PIDs stopped; receipts, comparisons and witness retained privately.

Private native.ZvAKTA then drives native pointer hover into Help and selects the submenu Keyboard Shortcuts control by its native accessibility identity. Both apps return the same structured keys-dialog reply. The resulting450x560 dialog pair is byte-identical, whole/header/overlay SSIM/RMSE0; all four new originals and full figures inspected. Closing the dialog then the window stops all seven owned PIDs. Earlier native.akdWEM failed a baseline cursor check because initial shell echo raced the probe; native.5chYDc selected the Gold command but failed window-close while its dialog remained open. Both failed runs and all-stopped cleanup receipts retained; corrected harness waits for initial prompt settlement, asserts cursor80,0 and explicitly dismisses the dialog before window close. No production behaviour or expectation weakened.

Native tree inspection found the generic compact button accessible label was menu while Gold uses Menus. Follow-up code `d9bc739e381b58050dd5310667d4312ccfc2576b` uses the same toolkit window-menu string/tooltip as Gold. No per-app Rust policy or menu command mapping change. Diff check passes; no local Rust build. Prepared native harness now asserts the Menus accessible name in both apps and repeats submenu command/readback/capture. Next exact CBC `term-menu-label-d9bc739e` runs term_gate --prepare; process full log/diffs, fetch verified artifact and rerun. Then exact release gate/native proof, frame-pacing diagnosis and full remaining parity matrix. Native menu closure is debug-only until release verified. Overall NOT ACCEPTED; no install, desk/VT or merge work.

branch: lab/parity-term; head SHA:d9bc739e381b58050dd5310667d4312ccfc2576b (compact-menu accessible-label follow-up; native-tested debuge12e9f9f; receipt HEAD contains checkpoint); NOT ACCEPTED — label gate/native/release, frame pacing and full parity matrix open; CBC job ids:term-menu-exact-e12e9f9f-a1 (green exact; native.qP45IY/ZvAKTA), term-menu-label-d9bc739e (next await), term-reflow-release-4c5a1dfa-a1 (latest native-tested release).


### Compact label follow-up: public toolkit API repair

CBC `term-menu-label-d9bc739e-a1` failed compilation with E0603: toolkit::strings::own is crate-private. All eleven Term Mix suites passed before the bundle-test compile stopped; no Rust tests, foreign gate or debug artifact completed. Complete5979-byte saved log read and archived, including returned rustfmt hunk. The prior exact e12e9f9f native narrow-menu/command/dialog proofs remain valid for those bytes; this failed follow-up does not replace them.

Code `5ad761149976fc65d8003fd510045d439b44a66c` resolves the same window-menu string through the existing public toolkit::action::label_text API using toolkit::Strings, matching the adapter window-menu-mark convention. Applied returned formatting around IconButton construction; no toolkit visibility expansion, behavioural policy or expectation change. Diff check and prepared native harness lint pass. The native harness retains the explicit Menus accessible-name assertion, actual submenu selection, exact structured dialog reply and finally cleanup. No local Rust execution/pass claim.

Next exact CBC `term-menu-public-label-5ad76114` runs term_gate --prepare. Read complete saved log, apply/repeat any worker diff, fetch matching verified debug artifact and run menu-label-5ad76114-debug. Then exact release gate and native proof before considering the menu checkpoint closed. Overall NOT ACCEPTED; frame pacing and full remaining parity matrix remain open. No install, desk/VT or merge work.

branch: lab/parity-term; head SHA:5ad761149976fc65d8003fd510045d439b44a66c (public compact-menu label API repair; native-tested debuge12e9f9f; receipt HEAD contains checkpoint); NOT ACCEPTED — label gate/native/release, frame pacing and full parity matrix open; CBC job ids:term-menu-label-d9bc739e-a1 (E0603 compile failure), term-menu-public-label-5ad76114 (next await), term-menu-exact-e12e9f9f-a1 (green exact; native.qP45IY/ZvAKTA).


### Exact compact-menu label gate and native debug proof

CBC `term-menu-public-label-5ad76114-a1` passed exact source `5ad761149976fc65d8003fd510045d439b44a66c`, no worker diff. Complete59637-byte log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and22 scoped host/input/editor/menu tests pass; foreign gate passes17 listed crates with strict mix/mix-shell/system/mixd clean. Verified debug executable SHA256 `28b22abe1aac7ff493a4a83d762ecde1961bba954c11504a6759bfbf60eddfd0` and exact-source bundle fetched.

Private native.RZqNJx passes explicit Menus accessible-name assertions for both apps (button rectangle44,5–72,33), real Mix output baseline, native900→450→900 resize, pointer opening/Help submenu selection and matched structured Keyboard Shortcuts dialog reply. All twelve originals hash-match preceding inspected native.ZvAKTA originals; explicit per-image inspection-reuse receipt and fresh comparisons retained. Baseline/narrow/shortcuts-dialog whole and regions SSIM/RMSE0. Narrow PNG pair remains `b4e6757e2af89b01c7de0c11392a377d3b61b57eb6c51f2c5542637e8f2a5b5f`. Open popup/header geometry is exact; exposed background text differs solely from C1 reflow restoration, whole0.00225503/0.0218059. Wide C1 metrics unchanged (restored-row0.252078/0.245939; other regions0). All seven owned PIDs stopped. Source/profile/hash, native calls/trees/replies/snapshots, originals/figures/witness and closure retained privately.

The compact-menu form and accessible-label repairs are debug-proven at this exact source. Next `term-menu-release-5ad76114` runs gold_parity_gate --prepare. Read complete saved log, process worker diff and exact-repeat if needed, fetch verified matching release and run prepared menu-label-5ad76114-release directly. Native release proof remains required before closing this checkpoint. The release harness retains C1 cursor/text checks, Menus name and native submenu command/dialog checks. Continue frame-pacing diagnosis and full remaining Bus/actions/topics/menu/macros/broker, input/provider/search, Prefs/daylight/authority, conformance and visual acceptance matrix. No install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:5ad761149976fc65d8003fd510045d439b44a66c (exact menu label gate/native debug proof; receipt HEAD contains checkpoint); NOT ACCEPTED — menu release proof, frame pacing and full parity matrix open; CBC job ids:term-menu-public-label-5ad76114-a1 (green exact;215 terminal tests; native.RZqNJx), term-menu-release-5ad76114 (next await), term-reflow-release-4c5a1dfa-a1 (latest native-tested release).


### Exact release menu closure; measured idle repaint regression

CBC `term-menu-release-5ad76114-a1` passed exact source `5ad761149976fc65d8003fd510045d439b44a66c`, no worker diff. Complete159970-byte log read in five contiguous chunks through GOLD_PARITY_PATCH_END and archived.484 tests passed, eleven retained ignores; shared fmt/check/all-target clippy/debug/release pass with existing warnings. Exact scoped gate separately attests the eleven Term Mix suites,215 terminal tests and foreign/strict-tier checks. Verified release SHA256 `84a02db821f997fa7738bf7be034db49bcfa23160fb4ee2546a49afe13bd752c` and exact-source bundle fetched.

Private native.kdohCl repeats the complete menu-label/submenu diagnostic on these release bytes. Menus accessible-name checks, same real Mix output and900→450→900 cursor/text checks, pointer-open/Help submenu command and structured shortcuts-dialog reply pass. All twelve originals hash-match the inspected debug native.RZqNJx originals; explicit hash-keyed inspection reuse and fresh comparisons retained. Baseline/narrow/dialog whole and regions SSIM/RMSE0; open popup/header geometry0, exposed background C1 difference whole0.00225503/0.0218059; wide C1 restored-row0.252078/0.245939 with all other regions0. All seven owned PIDs stopped. Compact-menu form and accessible-label corrections are closed on these release bytes. Overall app acceptance remains open.

Private native.hzM9Xg measures process CPU during an eight-second no-input interval using monotonic timestamps, proc stat and100Hz clock readback. Gold:8.044214s, zero measurable CPU ticks and zero extra process-side frames. Exact candidate release:8.046192s,400 CPU ticks (4.0 CPU seconds, approximately49.7% of one core),485 extra process-side frames. Neither PTY bytes nor terminal wake count changes during the interval. All seven owned PIDs stopped; raw readings, harness, binary receipt and summary retained. This is a confirmed idle process CPU/repaint regression; not a presented-frame/GPU measurement. No arbitrary polling cap or input/Prefs delay was applied.

Diagnostic code `33abb840df837e974bd73e0bccb00a36487e9bf6` adds the last native timing stats to generic viu.state and at most16 egui repaint callsite/reason records alongside existing stats. No terminal application policy or repaint scheduling changes. Prepared idle-causes-33abb840-debug harness records three bounded cause snapshots and repeats the CPU/native-menu probes. Next CBC `term-idle-causes-33abb840` runs term_gate --prepare; process complete log and any worker diff/exact-repeat, fetch verified artifact and run directly. Use native callsites to diagnose and repair the owning shared component, then repeat idle CPU and relevant release/Prefs/input/capture evidence. Full remaining parity matrix still open; no install, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:33abb840df837e974bd73e0bccb00a36487e9bf6 (bounded repaint diagnostic; exact native-tested release5ad76114; receipt HEAD contains checkpoint); NOT ACCEPTED — idle CPU/repaint regression and full parity matrix open; CBC job ids:term-menu-release-5ad76114-a1 (green exact;484 tests; native.kdohCl/hzM9Xg), term-menu-public-label-5ad76114-a1 (green scoped/foreign), term-idle-causes-33abb840 (next await).


### Repaint diagnostic scoped gate; exact formatting repeat

CBC `term-idle-causes-33abb840-a1` returned rc0. Complete63420-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and22 scoped host/input/editor/menu tests pass. Foreign gate passes17 listed crates, strict mix/mix-shell/system/mixd clean; debug build passes with existing warnings. Returned rustfmt-only changes affect the new bounded repaint-stats readback and its call layout. Applied verbatim in code `344b788479fefcb9b3c8a0eb1232c85f079973ba`; no scheduling, application behaviour or assertion change.

The fetched33abb840 debug artifact has a matching source/profile/hash receipt but was not used for native evidence because the worker formatted its source. Next CBC `term-idle-exact-344b7884` repeats term_gate --prepare at the formatted source. Read complete log, apply/repeat any diff, fetch verified matching debug artifact and run prepared idle-causes-344b7884-debug directly. Record native repaint callsites and process CPU, then repair the owning component and verify native/release behaviour. The prior exact5ad76114 release menu proof remains closed; its measured idle regression remains open (4.0 CPU seconds/8.046s,485 extra process frames versus Gold zero). Full parity matrix remains open. No install, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:344b788479fefcb9b3c8a0eb1232c85f079973ba (CBC diagnostic formatting applied; native-tested release5ad76114; receipt HEAD contains checkpoint); NOT ACCEPTED — exact diagnostic gate, idle CPU/repaint regression and full parity matrix open; CBC job ids:term-idle-causes-33abb840-a1 (rc0 with formatting diff;215 terminal tests), term-idle-exact-344b7884 (next await), term-menu-release-5ad76114-a1 (green exact;484 tests; native.kdohCl/hzM9Xg).


### Native idle repaint cause identified; shared title-update repair checkpoint

CBC `term-idle-exact-344b7884-a1` passed exact source `344b788479fefcb9b3c8a0eb1232c85f079973ba`, no worker diff. Complete59637-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and22 scoped host/input/editor/menu tests pass; foreign gate17 listed/strict tiers clean and debug build pass. Verified debug SHA256 `0c1b25a9d868b366f11355fc35fe1d0cb3a84b73e215f9b35cb19dc8b930b232` and exact-source bundle fetched.

Private native.5iLaDy records three bounded native repaint-cause snapshots. Each identifies egui0.37.0 context.rs:4570, the unconditional request_repaint_of inside send_viewport_cmd_to. Generic Host::frame sends the identical model window_title on every frame; other ordinary titlebar viewport commands are already change/interaction gated. Pinned dependency source and native stats inspected. During the separate eight-second debug idle interval, Gold uses zero measurable CPU ticks/zero extra process frames; candidate uses795 ticks (7.95 CPU seconds/8.040934s) and95 extra process frames with unchanged PTY counters. Earlier exact release measurement remains4.0 CPU seconds/8.046192s and485 process frames. All seven owned PIDs stopped. Native cause samples, raw stats, binary receipt, commands and process cleanup retained. Debug CPU is not substituted for release CPU or GPU/presentation evidence.

Code `1a0576a7d32f81af92802f9a7bc663df6eaf95cf` adds a per-viewport last-sent native-title cache in the shared window frame and uses it from the generic host. The initial title and actual later changes still send native Title commands; equal values stop rearming the next frame. No terminal-specific Rust policy, throttle, delayed Prefs/input or poll cap. A regression drives real egui passes through initial/equal/renamed/equal/restored titles, requiring exact native command counts and payloads; scoped term_gate now includes it. Existing tests unchanged. Diff check and gate/native harness lint pass; no local Rust execution or new-code pass claim.

Next CBC `term-idle-title-1a0576a7` runs term_gate --prepare. Read full log, repair failures/process any formatting and exact-repeat, fetch verified matching debug artifact and run prepared idle-title-1a0576a7-debug directly. Inspect new causes and repeat paired process CPU, real output/reflow/menu/command/closure evidence. Then exact release and live Prefs/input checks before declaring the idle regression fixed. Latest native-tested release remains5ad76114 with menu corrections closed but idle CPU regression present. Full remaining parity matrix open. No install, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:1a0576a7d32f81af92802f9a7bc663df6eaf95cf (shared title-update repair; native-tested release5ad76114; receipt HEAD contains checkpoint); NOT ACCEPTED — title gate/native/release idle CPU proof and full parity matrix open; CBC job ids:term-idle-exact-344b7884-a1 (green exact; native.5iLaDy cause proof), term-idle-title-1a0576a7 (next await), term-menu-release-5ad76114-a1 (green exact;484 tests; native.kdohCl/hzM9Xg).


### Shared title cache: exact gate and native debug improvement

CBC `term-idle-title-1a0576a7-a1` passed exact source `1a0576a7d32f81af92802f9a7bc663df6eaf95cf`, no worker diff. Complete61963-byte saved log read in three contiguous chunks through TERM_PATCH_END and archived. All eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests with five retained ignores and23 scoped host/input/editor/menu/title tests pass. The real egui title-command regression passes for initial/equal/changed/restored titles. Foreign gate passes17 listed crates with strict tiers clean; debug build passes. Verified matching debug SHA256 `1e049bebda1687180282c18b1a456bc6c6a9160f9cc867d9e941c7cdb1264c11` and exact-source bundle fetched.

Private native.f5VJpi repeats the paired idle/real-Mix/reflow/menu/command probe. Candidate repaint-cause samples are now empty, removing the previously repeated egui viewport-command cause. During8.045518s the candidate uses20 CPU ticks (0.20 CPU seconds) and four extra process frames, versus debug pre-repair795 ticks/95 frames. Gold uses zero measurable ticks/frames during8.044841s. These endpoint-driven debug measurements demonstrate the continuous loop has stopped; they are not final release idle acceptance. Release probe now allows0.5s after the first snapshot for endpoint-triggered frames to settle before starting proc CPU measurement, equally for both apps. No production polling, input or Prefs delay added.

Real Mix output,900→450→900 C1 cursor/text checks, Menus accessibility name, native pointer/menu/submenu command and identical structured shortcuts-dialog reply all pass. Baseline/narrow/shortcuts-dialog whole/regions SSIM/RMSE0; C1 wide metrics unchanged. Eleven originals hash-match prior inspected captures. The changed Gold Help original and both Help originals/full figures were freshly inspected: an additional blinking insertion caret is present in Gold menu-search only. Help whole0.0024199/0.0231223 includes that temporal caret plus exposed C1 background; it is not labelled an accepted visual difference, and matched caret-phase capture remains open. All seven owned PIDs stopped. Private fresh comparisons, explicit inspection reuse, CPU stats/cause samples, replies/binary receipt and witness retained.

Next `term-title-release-1a0576a7` runs gold_parity_gate --prepare at this exact source. Read complete log, apply/repeat any diff and fetch matching verified release. Run prepared idle-title-1a0576a7-release directly, then prefs-pair-1a0576a7-release to verify live settings still wake/draw on unchanged app PIDs after removing the idle loop. Inspect matched captures and native input/readback/closure before closing release CPU behaviour. Full remaining parity matrix open; no install, desk/VT or merge work. NOT ACCEPTED.

branch: lab/parity-term; head SHA:1a0576a7d32f81af92802f9a7bc663df6eaf95cf (exact title cache gate/native debug improvement; receipt HEAD contains checkpoint); NOT ACCEPTED — release idle CPU/Prefs/input proof, Help caret-phase match and full parity matrix open; CBC job ids:term-idle-title-1a0576a7-a1 (green exact;215 terminal tests; native.f5VJpi), term-title-release-1a0576a7 (next await), term-menu-release-5ad76114-a1 (latest exact native-tested release).


### Release title-cache closure and live Prefs repeat

CBC `term-title-release-1a0576a7-a1` passed exact code `1a0576a7d32f81af92802f9a7bc663df6eaf95cf`. Complete169349-byte saved log read through GOLD_PARITY_PATCH_END and archived; no worker diff.485 tests passed, eleven existing ignores; fmt/check/all-target clippy/debug/release pass with existing warnings. The preceding exact scoped `term-idle-title-1a0576a7-a1` separately attests eleven Term Mix suites,215 terminal tests/five existing ignores,23 host/input/editor/menu/title checks and foreign gate17 listed/strict tiers clean. Verified release SHA256 `52cd390129e301326007abd0bd54c05ecff1eded407619d7c6694b7e37cbc886` and matching source/profile bundle.

Private native.4LA13p measures candidate zero CPU ticks during8.045937s, versus Gold one tick (0.01 CPU seconds) during8.050339s at100Hz. Candidate has four endpoint-driven process frames, Gold zero; all three repaint-cause samples empty and PTY counters unchanged. This closes the previously measured continuous idle repaint regression (4.0 CPU seconds/8.046s and485 frames before repair) on release bytes. It is a bounded process CPU observation, not GPU/presentation timing. Equal0.5s endpoint-settlement allowance applies to both apps; no production delay/throttle introduced. Native real Mix output,900→450→900 C1 cursor/text, Menus accessibility name, pointer/submenu dispatch and identical shortcuts-dialog reply pass. All seven owned PIDs stopped. Baseline/narrow/dialog whole and regions SSIM/RMSE0; wide and exposed menu background retain only recorded C1 reflow correction. All six full figures inspected, eleven original hashes reuse prior inspection and changed Help original freshly inspected.

Private native.WhGvRR retains eight native Help captures per app at150ms intervals. First pair with identical decoded-pixel caret crops has both carets absent; both originals/full side-by-side/difference inspected. Header, parent menu, Help popup and remaining grid distortion0; exposed C1 text row0.108003/0.153507, whole0.00225503/0.0218059. This closes the temporal Help insertion-caret mismatch without editing/resizing captures or hiding a control. All sixteen sampled originals retained and all seven owned PIDs stopped.

Private native.LFowf8 repeats17 paired live Prefs states at900x560 against the same real Mix command output. All sixteen settings transitions acknowledged on unchanged candidate PID; both apps remain on their own unchanged PIDs. All tabs/terminal grids distortion0, ten whole captures byte-identical. The seven remaining header differences equal previously inspected icon raster edges: compact macro/search/Close whole0.000141901/0.00474248; spacious Close edge whole8.11716e-9/3.06314e-5; app-look/tune Close edge9.56394e-9/3.16119e-5.33 originals reuse exact inspected bytes; corrected initial Gold session and new session full figures freshly inspected, remaining32 full figures have identical decoded pixels to prior inspected figures. Compact header re-inspected. All seven owned PIDs stopped. Binary receipts, calls, revisions, images, fresh comparisons, explicit inspection-reuse and closure witnesses retained privately.

Next native.ZvLgX9 is running the greater-than-five-minute absence and native-input launch-pool recovery probe at these exact release bytes. Presence/mint floors remain unchanged. Full remaining Bus/actions/topics/macros/broker, provider/input/IME/search/show failures, daylight/authority/tune, conformance and visual acceptance matrix remains open. Overall NOT ACCEPTED. No install, desk/VT or merge work.

branch: lab/parity-term; head SHA:1a0576a7d32f81af92802f9a7bc663df6eaf95cf (exact native-tested release; receipt HEAD contains checkpoint); NOT ACCEPTED — bounded idle/Prefs/Help phase closed, presence recovery and full parity matrix open; CBC job ids:term-title-release-1a0576a7-a1 (green exact;485 tests; native.4LA13p/WhGvRR/LFowf8), term-idle-title-1a0576a7-a1 (green scoped/foreign).


### Native auto-mode, clipboard-provider and presence recovery witnesses

All following runs use exact release code `1a0576a7d32f81af92802f9a7bc663df6eaf95cf`, SHA256 `52cd390129e301326007abd0bd54c05ecff1eded407619d7c6694b7e37cbc886`, against pinned8779f33 Gold. CBC remains exact `term-title-release-1a0576a7-a1` (485 tests/eleven existing ignores) plus scoped/foreign `term-idle-title-1a0576a7-a1`; private harnesses add native evidence without changing executable/bundle bytes.

Private native.K1OcA8 covers thirteen paired states: initial session; auto/daylight true; temporary dark override; daylight false under override; temporary light override; explicit dark taking precedence; return to auto; app dark override; app reset; temporary override reset to night; daylight true again; daylight reset; explicit dark reset. Every whole/header/toolbar/tab/grid distortion is0, all26 original captures pairwise byte-identical. Two unique original images and four unique decoded-pixel full figures inspected; explicit hash/pixel inspection index retained. All twelve changes acknowledged on unchanged candidate PID; all seven owned PIDs stopped. This closes the bounded daylight/mode-override precedence cases, not different-authority-incarnation or every tune input.

Private native.CdHo0M drives candidate native egui IME ingress through non-empty preedit, tab switch, owner close, switch away/back before delayed commit, platform reset acknowledgement and fresh commits. Stale text never reaches the new/surviving pane; separate real Mix output lines FRESH_B_OK, SURVIVOR_OK and RESET_OK verify fresh commits still execute. Four new originals inspected; baseline reuses inspected exact bytes. All seven owned PIDs stopped. Gold composition implementation/tests inspected, but this candidate-only IME witness does not claim paired Gold platform IME visual acceptance.

Private native.suW0TC uses external wl-clipboard2.3.0 only as an owned edge provider on the private Wayland display. A17MiB native source is refused with Paste exceeds16MiB limit and no payload insertion. Pausing the owned foreground provider produces native Busy followed by the real five-second TimedOut read, with matching notices; resuming it permits a fresh paste, clears the notice and executes separate RECOVER_PASTE_OK Mix output. Five paired900x560 captures are byte-identical; notice rectangle0,70,900,30 plus whole/header/toolbar/tabs/grid SSIM/RMSE0. New originals and full figures inspected; initial baseline reuses prior inspected bytes. All fifteen owned PIDs, including provider workers/supervisors, stopped. Private source hashes, executable/version receipt, native calls/trees/snapshots/figures and witness retained. Earlier native.8Bki0D failed provider PID discovery because Linux child ownership was inspected on the evaluator's main thread rather than across threads; helper now queries process parentage. All five tracked run PIDs stopped, provider returned Broken pipe after private broker closure and process inspection found no remaining provider; failed run retained.

Private native.5L11G8 pauses a real provider while requesting paste, switches tabs, then resumes it. Completion stays with the original pane and executes separate OWNER_PASTE_OK output; new active pane receives no owner text. A second read's owner closes before resume; surviving pane total/input/reply write counters remain47/8/39, and the grid stays unchanged. Three paired captures are byte-identical, whole/header/tab/grid/notice/toolbar distortions0; new Gold originals and full figures inspected, candidate hashes identical. All thirteen owned PIDs stopped. Retained raw snapshots support the stronger total/reply-counter audit (paste writes are counted as replies, so input counter alone was insufficient); prepared harness now checks all three counters.

Private native.7Cp5k9 waits310s without native input for each app. Both retain pane1 Attached, hold the launch pool for user presence, open pane2 graphics-only and execute IDLE_SHELL_OK through its real Mix shell. Native pointer/key activity, then the normal five-second actor cadence, permits a new pane3 sealed-fd handoff, Attached enrolment and RECOVERED_MIX_OK output. Both split replies match exactly. Two paired captures are byte-identical; whole/header/tabs/left/right-top/right-bottom/grid distortions0; new Gold originals and full figures inspected, candidate hashes identical. All eleven owned PIDs stopped. No presence, grant lifetime or60s mint floor change. Earlier native.ZvLgX9 confirmed Gold's presence hold but the harness incorrectly required a transient next-grant-ready diagnostic that child-binding Pending immediately replaces. Corrected check proves actual pane enrolment; all six tracked failed-run PIDs stopped and evidence retained. Remote revoke/expiry/reconnect faults remain open.

The next native terminal-Prefs matrix is running at the same release bytes: padding/border/cursor/focus settings and new-pane TERM/scrollback behaviour against real Mix output. Full remaining Bus/actions/topics/macros/broker, platform IME/mouse/search/show faults, settings incarnation/tune, wider conformance/visual matrix and final acceptance record remain open. Overall NOT ACCEPTED; no install, desk/VT or merge work.

branch: lab/parity-term; head SHA:1a0576a7d32f81af92802f9a7bc663df6eaf95cf (exact native-tested release; receipt HEAD contains checkpoint); NOT ACCEPTED — bounded daylight/clipboard failures/paste ownership/presence recovery closed; terminal Prefs and full parity matrix open; CBC job ids:term-title-release-1a0576a7-a1 (green exact;485 tests), term-idle-title-1a0576a7-a1 (green scoped/foreign); native witnesses:K1OcA8,CdHo0M,suW0TC,5L11G8,7Cp5k9.


### Terminal Prefs closure and View/menu checkpoint — NOT ACCEPTED

Exact release `1a0576a7d32f81af92802f9a7bc663df6eaf95cf` (SHA256 `52cd390129e301326007abd0bd54c05ecff1eded407619d7c6694b7e37cbc886`) completed fourteen paired terminal Prefs states in private native.mtdTQM: initial; padding1/16; border4; block/bar/underline cursor; neutral/accent focus; scroll speed200; future history0 and TERM vt100; split accent/neutral. Every whole/header/toolbar/tab/grid SSIM and RMSE distortion is0; all28 originals pairwise byte-identical. Seven new unique Gold originals and fourteen full figures inspected, baseline inspection reused by exact hash; candidate originals identical. All nine owned PIDs stopped. New pane uses vt100 and retains no history after80 real Mix output lines; original pane retains xterm-256color and history. Gold constructor-only TERM/history scope preserved. Every setting change acknowledged without candidate restart. This proves the bounded settings/configuration behaviour; actual wheel speed and blink timing remain open. Earlier native.IPGrJo retained a harness reset error in its first three pairs: Gold padding/border persisted before candidate launch. The corrected run resets every terminal axis before each launch; no failed pair is accepted.

Private native.JmRXci exercises44 native View choices at the same exact release: thirteen schemes in dark/light, four mode choices, eight style choices, four caption alignments and auto/csd decorations. Execute replies, menus and titlebars match in all44 states.43 whole images are byte-identical; Pro exposes a tab background mismatch. Fifteen light-mode action readbacks differ solely in view.mode.checked: pinned Gold reports false, port reported true. Gold theme_menu registers this toggle with an always-false checked closure; earlier inventory/test expectations of true were wrong and are superseded. Actual mode radio selection remains checked. Pro has32px header/26px tabs; corrected regions show header/toolbar/captions/grid0, tab SSIM .00818432/RMSE .0113387, whole .000347842/.00211586. The earlier generic38/32 crops mixed the first tab pixels into the header and are superseded by this measured geometry. Pro originals and full/regional figures inspected; other43 states still require figure inspection. All seven owned PIDs stopped.

Private native.ApSCkE measures24 open menus: all six menus in dark/light at900/450px, with native bar or hamburger input, real Mix output and multiple tabs/panes. All24 menu-open assertions pass for both apps. Four150ms captures per app/state retained; first matching decoded-pixel phase selected, without editing or resizing originals. All24 chosen pairs are byte-identical, whole/header/tabs/content/menu-band distortions0. All eleven unique owned PIDs stopped. Only the Gold dark900 File original has been inspected so far; remaining originals and full figures await inspection. Preliminary witnesses explicitly withhold visual acceptance.

Code `fa201ea13d63b231c5fbfec2640e9d5aaf2f8004` corrects view.mode.checked to Gold's always-false contract and paints the tab control over a zero-padding canvas container, matching Gold's CentralPanel canvas behind the translucent Pro strip. Changes stay in Term's Mix scene/actions/tests. Actions/core/model tests and local Mix lint pass. No Gold deviation or Fable question. Private native.l5aVGG prototypes both repairs using the verified1a release executable plus replaced private Mix scene/actions: Pro and light-mode whole/all-region distortions0; execute/actions/menu/titlebar readbacks all match. Both candidate originals and full side/diff figures inspected, Gold originals byte-identical; all seven owned PIDs stopped. This mixed-source diagnostic is explicitly not exact-source acceptance; its receipt includes base executable and replacement hashes.

Next scoped CBC is `term-mode-pro-fa201ea1` at the full repair SHA. Read the complete saved log, apply worker fmt/lock diff and repeat if needed, fetch/verify its debug source bundle and run prepared exact-source Pro/light,44 View,24 menu and terminal Prefs repeats. Release gate, release captures and remaining full parity work follow. Full Bus/action/topic/macro/broker faults, remote issuer revoke/expiry, paired platform IME/mouse/search/show failures, settings incarnation/tune, wider conformance/performance and final visual acceptance remain open. No install, desk/VT or merge work.

branch: lab/parity-term; head SHA:fa201ea13d63b231c5fbfec2640e9d5aaf2f8004 (repair code; receipt HEAD contains checkpoint); NOT ACCEPTED — exact repair gates/captures and full parity matrix open; CBC job ids:term-mode-pro-fa201ea1 (next scoped), term-title-release-1a0576a7-a1 (green exact release485 tests), term-idle-title-1a0576a7-a1 (green scoped/foreign).


### Exact mode/Pro debug gate and native repeat

CBC `term-mode-pro-fa201ea1-a1` passed at `fa201ea13d63b231c5fbfec2640e9d5aaf2f8004` on cbc2. Complete64,526-byte saved log read and archived; TERM_PATCH is empty. Eleven standalone Term Mix suites, two bundle tests, fifteen runtime/call/timer tests,215 terminal tests (five existing ignores),23 scoped host tests and foreign gate pass. All17 foreign crates listed; strict Mix/system/mix-shell/mixd remain clean. Debug artifact and archived source bundle verified against receipt; executable SHA256 `ec59194db9d42b67b4f3ad68c924ca084952c7e46623f7c4701f8ce52c56c6cd`.

Private native.yrZdKV repeats Pro and light mode at those exact debug bytes. Both whole/all-region distortions0 and execute/actions/menu/titlebar readbacks match. Pro's32px header/26px tabs separately measured and exact. Four original images reuse inspected native.l5aVGG images by exact SHA256; four fresh full side/diff figures inspected. Seven owned PIDs stopped. This closes the bounded debug repair check, not release acceptance.

Private native.Vh5cIJ completes all44 View states. Raw audit finds44 pairwise byte-identical original images and all176 structured execute/actions/menu/titlebar readbacks equal. All seven owned PIDs stopped. The regional comparator is generating full/crop figures; complete comparisons and inspection remain to be reviewed. Private native.vp7SoN repeats fourteen terminal Prefs states with whole/header/toolbar/tab/grid distortions0 and28 originals identical to the previously inspected native.mtdTQM captures by hash. All nine owned PIDs stopped. Fresh full figures await inspection or explicit decoded-pixel inspection reuse.

Private native.RUlIKg completes24 open-menu states: all48 Gold/candidate menu-open assertions pass,18 phase-matched pairs whole/regions0. Six dark450 states FAIL matching: the Gold terminal grid contains an undefined-function error from the real Mix input/resize sequence, absent in candidate. First failed File side/diff inspected; header/tabs exact, content differs. No menu acceptance claim, no threshold relaxation and no classification as a Gold defect yet. All eleven unique owned PIDs stopped. Retain the calls, four capture phases per state and failed originals; investigate command completion/input/resize ordering and add a separate-output-line guard before sequential release repeats. Concurrent private runs provide functional evidence, not performance evidence.

Next CBC `term-mode-pro-release-fa201ea1` gates the same repair source through gold_parity_gate. Read full saved log and worker patch; verify exact release artifact/source, then run prepared release Pro/View/menu/terminal-Prefs harnesses sequentially. View comparator results and figure inspection remain pending, alongside the menu output failure and the previously listed full Bus/authority/input/settings/conformance/visual acceptance work. No install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:fa201ea13d63b231c5fbfec2640e9d5aaf2f8004 (exact tested debug code; receipt HEAD contains checkpoint); NOT ACCEPTED — release, figure inspection, narrow menu output failure and full parity matrix open; CBC job ids:term-mode-pro-fa201ea1-a1 (green exact scoped/foreign), term-mode-pro-release-fa201ea1 (next release), term-title-release-1a0576a7-a1 (previous release green485 tests).


### Exact mode/Pro release and expanded visual checkpoint

CBC `term-mode-pro-release-fa201ea1-a1` passed on cbc3 at `fa201ea13d63b231c5fbfec2640e9d5aaf2f8004`:485 tests, eleven existing ignores, fmt/check/all-target Clippy and debug/release builds. Complete169,349-byte saved log read through empty GOLD_PARITY_PATCH_END and archived privately. Existing warnings remain. Release source/profile/executable hash and archived Mix bundle verified; SHA256 `ef812d6d79e24f6a0fdbd5332d355cd1f86bb83c6253b16e6adf564b0ee83444`. The preceding exact scoped/foreign gate remains `term-mode-pro-fa201ea1-a1`; no source changed between profiles.

Private native.hu6ryY repeats the Pro/light repair at exact release bytes: whole/all-region SSIM and RMSE distortion0, execute/actions/menu/titlebar replies match. Four originals reuse inspected debug images by exact SHA256; four fresh full figures inspected. Pro32px header/26px tab geometry retained with matching original hashes. All seven owned PIDs stopped.

The interrupted debug View figure generator was resumed without overwriting earlier evidence: completed comparison sets reused and missing sets written under fresh compare-resumed names. Private native.Vh5cIJ now has44 complete exact comparisons. All29 unique originals and58 full figures inspected; remaining originals/full figures have explicit file-hash/decoded-pixel inspection reuse. Private native.uhvCuz repeats all44 states at exact release bytes:44 PNG pairs identical; all176 execute/actions/menu/titlebar readbacks equal; whole and fixed coverage-band distortions0. Its88 originals match inspected debug originals by SHA256,88 fresh full figures match inspected figures by decoded pixels; complete inspection-reuse index retained. All seven owned PIDs stopped. Ordinary fixed coverage bands are not asserted to be each style's intrinsic boundary; measured Pro geometry is recorded separately. Compositor-hosted whole-surface states remain separate work.

Private native.fDI6j8 repeats fourteen terminal Prefs states at exact release bytes, including future-pane TERM vt100/history0 versus the original pane's retained TERM/history. Whole/header/tab/toolbar/grid distortions0. All28 originals match inspected native.mtdTQM originals by SHA256 and28 fresh full figures match inspected figures by decoded pixels. All nine owned PIDs stopped. This closes the bounded release terminal settings matrix; wheel speed and blink timing remain separate cases.

The stronger menu harness requires a new separate Mix marker line, exact TERM_PAIR_NATIVE output and no undefined-function error after every size/mode transition. Private native.zbsQHj failed in Gold before candidate launch: repeated Escape dismissal followed by term.type delivered an air() command rather than pair(), leaving an undefined-function error. All seven unique owned PIDs stopped. This reproduces the prior native.RUlIKg Gold-only output fault; the Escape/input path is still unclassified and open, not silently accepted. Private native.nWNJ14's first click-dismiss probe accidentally double-clicked the native title area and maximised the window, so resize refused; all seven unique owned PIDs stopped. Removed only the duplicate before-menu click; no app source or functional assertion changed.

Private native.chhfWc then completes24 release menu pairs using one real title-area click to dismiss each menu, with fresh real Mix output guarded before every size/mode. All48 menu-open assertions pass; all24 selected native phase pairs are byte-identical, whole/header/tab/content/menu-band distortions0. All24 Gold originals and48 full figures inspected, candidate originals identical. Narrow hamburger View/Pane horizontal overflow and Help vertical scrollbar match Gold. Four150ms phases per app/state retained without editing originals; all eleven unique owned PIDs stopped. This closes those menu states and click dismissal only; callback execution, deeper theme submenus, scrolling and repeated Escape retain separate coverage requirements.

Source/profile/hash receipts, commands, native replies/trees/output snapshots, originals, figures, inspection indexes and process closures retained privately. Macro Bus coverage is next, using private native macro roots and real Mix child runs at the same release bytes. Full Bus/actions/topics/replay/broker faults, issuer expiry/revoke, paired platform IME/program mouse/search/show failures, remaining settings/tune/performance/conformance and final acceptance record remain open. Overall NOT ACCEPTED; no install, desk/VT or merge work.

branch: lab/parity-term; head SHA:fa201ea13d63b231c5fbfec2640e9d5aaf2f8004 (exact tested release code; receipt HEAD contains checkpoint); NOT ACCEPTED — remaining Bus/authority/input/settings/conformance and final acceptance open; CBC job ids:term-mode-pro-release-fa201ea1-a1 (green exact release485 tests), term-mode-pro-fa201ea1-a1 (green exact scoped/foreign).


### Release macro Bus and native program mouse checkpoint

Exact code remains `fa201ea13d63b231c5fbfec2640e9d5aaf2f8004`, release executable SHA256 `ef812d6d79e24f6a0fdbd5332d355cd1f86bb83c6253b16e6adf564b0ee83444`. No product source changed. Existing exact scoped/foreign and release gates remain green.

Private native.kPBzlc completes31 paired macro Bus replies using private native macro roots and actual Mix children. List/reload/create/put/get/run/output/status/stop/copy/remove/move, builtin shadow/reveal, missing/invalid/busy/ownership refusals match structurally. Short output is exactly MACRO_PAIR_OK plus newline, dropped=false; stopping the running long macro yields exit143 in both apps. Only actual numeric started timestamps are excluded from comparison copies under the truthful-runtime-data ruling; raw replies are unchanged. All nine unique owned PIDs stopped. The earlier native.Bj9RxT harness failure expected integer0 instead of Gold's boolean false for dropped; corrected the harness to the actual contract, preserving the failed run. No visual or macro UI claim.

Private native.5TByzI proves actual program mouse bytes through each real Mix login PTY using Mix run_stream with stty/od edge programs. At900x560, grid origin8,78, cell9x20, native pointer30,100 reports one-based cell3,2. SGR primary press/release bytes are ESC[<0;3;2M and ESC[<0;3;2m; legacy X10 bytes are1b5b4d2023221b5b4d232322. Each program restores terminal modes. Both capture pairs are byte-identical; whole/header/tabs/grid/report SSIM and RMSE distortions0. All four originals and four full side/diff figures inspected. All fifteen unique owned PIDs stopped, including reader programs. Source/profile/hash, script hashes, raw native calls/output, comparisons and closure retained in private witnesses.

Failed native.HeHg4K retained: JSON-encoded Unicode escapes were literal in generated Mix source, so the program emitted text instead of enabling mouse mode. Corrected private script generation to native Mix escape syntax. Failed native.7M4NUR retained: Gold's native drive button is primary, not left; corrected the harness to Gold's documented payload. All recorded processes stopped in both failures; no product change or weakened assertion.

Next: expand actual program motion/buttons/wheel/cancellation, then remaining full Bus/action/topic/broker faults, remote issuer expiry/revoke, paired platform IME/search/show failures, settings incarnation/tune/performance/conformance and final acceptance. Repeated Escape/input fault remains open. Overall NOT ACCEPTED; no install, desk/VT or merge work.

branch: lab/parity-term; head SHA:fa201ea13d63b231c5fbfec2640e9d5aaf2f8004 (exact tested release code; receipt HEAD contains checkpoint); NOT ACCEPTED — remaining full input/Bus/authority/settings/conformance and final acceptance open; CBC job ids:term-mode-pro-release-fa201ea1-a1 (green exact release485 tests), term-mode-pro-fa201ea1-a1 (green exact scoped/foreign).


### Release motion, cancellation, wheel Prefs and selection

Code and release bytes remain fa201ea1 as above. Native.6tFCZu proves middle/right press/release, button-motion reporting, hover, fractional wheel up/down and hidden-tab cancellation through actual Mix PTYs. Six capture pairs are byte-identical; whole/header/tabs/grid/report SSIM and RMSE distortions are zero. Six Gold originals and twelve full figures inspected; candidate originals reuse the identical Gold bytes with explicit hashes. All31 tracked PIDs stopped. Native.EtNBRs retained a harness expectation failure: the proposed same-half-cell move crossed the selection half-cell boundary and correctly generated another report in Gold. Adjusted the point within the same half-cell; no product change. Native.Up6z3j's initial six-case pass is retained separately.

Native.VznoWH strengthens cancellation: a new tab's child is Attached and explicitly tracked. The original hidden PTY receives its matching release and completes its reader before returning to the original tab or sending button-up. The sibling's total/input/reply byte counters stay0 across the later release. Both original PNGs match inspected native.6tFCZu originals by SHA256; fresh full figures match inspected figures by decoded pixels. Whole/regions zero. All13 unique owned PIDs stopped, including both second-tab shells.

Native.A7uQsI changes terminal.scroll.speed live through20,40,80 on one unchanged PID per app. Actual program reports prove fractional accumulation at20 and doubled reports at80; candidate settings revision and projected widget speed acknowledge each change. Three capture pairs are byte-identical, whole/header/tabs/grid/report distortions zero. All three Gold originals and six full figures inspected, candidate originals hash-identical. All19 owned PIDs stopped. This covers program wheel reporting; local history scrolling remains separate.

Native.x8cu0r drives real double/triple primary clicks and reads native Wayland PRIMARY with wl-paste. Word bytes are exactly TERM_PAIR_NATIVE; whole-line bytes add one newline. Both selected-state pairs are byte-identical, whole/header/tabs/grid/selection distortions zero. All four originals and four full figures inspected. All seven owned PIDs stopped. Native scripts, fixtures, binary receipts, calls, exact bytes, comparisons, inspection reuse and closures remain private.

Remaining: full Bus/action/topic/broker faults, remote issuer expiry/revoke, paired platform IME and remaining input/search/show faults, settings incarnation/tune, wider conformance/performance and final acceptance. Repeated Escape/input behaviour remains under diagnosis. No Fable question or product deviation is proposed; no install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:fa201ea13d63b231c5fbfec2640e9d5aaf2f8004 (exact tested release code; receipt HEAD contains checkpoint); NOT ACCEPTED — remaining full Bus/authority/input/settings/conformance and final acceptance open; CBC job ids:term-mode-pro-release-fa201ea1-a1 (green exact release485 tests), term-mode-pro-fa201ea1-a1 (green exact scoped/foreign).


### Query conformance checkpoint and Escape diagnosis

Release evidence still uses exact fa201ea1 bytes recorded above. Private native.kJoSyA sends actual cursor/status/device queries from Mix programs and reads the PTY replies with od. Ordered CPR5;7, DSR0 and Gold DA1?62;4;6;22;52 match byte-for-byte. Alternate-screen CPR12;30 followed by restored main-screen CPR2;4 also matches. Both capture pairs are byte-identical; whole/header/tabs/grid/report distortions are zero. All four originals and four full figures inspected. Seven persistent owned PIDs stopped; each reader checks exit status and restores stty. Native.pHxkip retained an earlier private generator error that omitted the query bytes; it is not a terminal defect.

Escape diagnostics native.aBNNZh (one capture phase), native.GAewO8 (original four-phase timing) and native.UPXtK2 (both dark/light, original timing) pass all4/4,4/4 and8/8 fresh real Mix output assertions after repeated menu dismissal and900→450 resizing. Each run stopped eleven unique owned PIDs. Original assertions remain; diagnostic handling retains both apps before raising any failure. This is functional evidence only, with no new menu visual acceptance claim. The earlier intermittent Gold dropped-character failure did not reproduce; it remains recorded and unclassified, not declared fixed or a C1 deviation.

New code `fe66f620485a141b361f46ab4a6922798cf4944c` adds two reusable-widget conformance tests: ordered CPR/DSR/DA1 replies across every escape split, unchanged visible grid, and alternate/main saved cursor reports. These tests do not change product behaviour. No Rust command ran locally; compilation/test/format validation is pending CBC `term-vt-query-fe66f620`, term_gate --prepare at that exact SHA. Read its complete saved log, apply any returned fmt/lock diff and repeat at exact source before advancing. The last green scoped/foreign and full release gates remain at fa201ea1.

Remaining full parity work includes Bus/action/topic/broker faults, issuer expiry/revoke, paired platform IME and remaining input/search/show faults, settings incarnation/tune, wider conformance/performance and final release acceptance. No Fable question, install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:fe66f620485a141b361f46ab4a6922798cf4944c (new conformance tests; receipt HEAD contains checkpoint); NOT ACCEPTED — new CBC validation and remaining Bus/authority/input/settings/conformance/final acceptance open; CBC job ids:term-vt-query-fe66f620 (next scoped), term-mode-pro-release-fa201ea1-a1 (green exact release485 tests), term-mode-pro-fa201ea1-a1 (green exact scoped/foreign).


### VT query scoped gate: tests pass, formatting applied

CBC `term-vt-query-fe66f620-a1` passed on cbc3: 217 terminal tests, five existing ignores, eleven standalone Term Mix suites, two bundle tests, fifteen runtime/call/timer tests, 23 scoped host tests, foreign gate and debug host build. Both new query conformance tests passed. All 17 foreign crates are listed; strict Mix/system/mix-shell/mixd closures remain clean. Existing warnings remain. The complete 64,075-byte saved log was read and archived privately as term-vt-query-fe66f620-a1.log.

The worker returned one rustfmt-only hunk collapsing the query helper signature. Applied it exactly, committed and pushed `7a886e4fccbf4fa6bc772570bf5d201030f4d9dd`. No assertion or product behaviour changed. The fe66f620 artifact is not used for exact-source acceptance because its worker source was formatted. Next CBC `term-query-exact-7a886e4f` repeats term_gate --prepare at the formatted SHA; require an empty worker diff. Then gate the exact release, fetch verified source/profile/bytes and repeat the native query witness before advancing release evidence.

The last exact release evidence remains fa201ea1 (485 tests; inspected native macro/input/selection/wheel/query receipts above). Full remaining Bus/authority/input/settings/conformance/performance and final visual acceptance work stays open. No Fable question, install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:7a886e4fccbf4fa6bc772570bf5d201030f4d9dd (formatted query tests; receipt HEAD contains checkpoint); NOT ACCEPTED — exact scoped/release validation and remaining full parity acceptance open; CBC job ids:term-query-exact-7a886e4f (next exact scoped), term-vt-query-fe66f620-a1 (green, formatting applied), term-mode-pro-release-fa201ea1-a1 (previous exact release green485 tests).


### Exact query source passes scoped and native debug checks

CBC `term-query-exact-7a886e4f-a1` passed on cbc2 at `7a886e4fccbf4fa6bc772570bf5d201030f4d9dd`, with an empty TERM_PATCH. The complete 64,746-byte saved log was read and archived privately. Eleven Term Mix suites, two bundle tests, fifteen runtime/call/timer tests, 217 terminal tests (five existing ignores), 23 scoped host tests, foreign gate and debug build pass. Both query conformance tests pass at the formatted source. All 17 foreign crates remain listed and the strict closures remain clean; existing warnings remain.

Fetched debug executable SHA256 `c64384e9f9c280d4ca3fcbbd397cd741d47bfba2017431cd9b5316973491c2ae`; source/profile/hash and archived Mix bundle verified. Private native.L71aBB repeats both actual Mix PTY query cases: ordered cursor/status/DA1 and alternate/main saved cursor replies match Gold exactly. Whole/header/tabs/grid/report distortions are zero. Four originals match inspected native.kJoSyA originals by SHA256; four fresh full figures match inspected figures by decoded pixels, with explicit inspection reuse retained. All seven persistent owned PIDs stopped. This validates debug bytes only.

Next CBC `term-query-release-7a886e4f` runs gold_parity_gate --prepare at that exact source. Read its complete saved log, process any worker diff and require an exact repeat if needed; fetch verified release bytes with term-fetch-release.mix and run the prepared query-protocol-7a886e4f-release.mix directly. Release proof, full Bus/authority/input/settings/conformance/performance and final acceptance remain open. No Fable question, install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:7a886e4fccbf4fa6bc772570bf5d201030f4d9dd (exact tested debug source; receipt HEAD contains checkpoint); NOT ACCEPTED — release gate/native proof and remaining full parity acceptance open; CBC job ids:term-query-release-7a886e4f (next release), term-query-exact-7a886e4f-a1 (green exact scoped), term-vt-query-fe66f620-a1 (green before formatting).


### Exact release query conformance passes

CBC `term-query-release-7a886e4f-a1` passed on cbc2 at `7a886e4fccbf4fa6bc772570bf5d201030f4d9dd`: 487 tests, eleven existing ignores, fmt/check/all-target Clippy and debug/release builds. The complete 168,110-byte saved log was read through an empty GOLD_PARITY_PATCH and archived privately. Existing warnings remain. The exact scoped/foreign gate is term-query-exact-7a886e4f-a1.

Verified release source/profile/executable hash and archived Mix bundle. Executable SHA256 `3bb3c630ea8c3e0a557426d7285cc8ac5193d980fcdf8d9083c0bda1a4951a28`. Private native.98nD37 repeats both real Mix PTY query cases at these release bytes: ordered CPR/DSR/DA1 and alternate/main saved cursor replies match Gold exactly. Both capture pairs are byte-identical; whole/header/tabs/grid/report SSIM and RMSE distortions zero. Four originals match inspected native.kJoSyA originals by SHA256; four fresh full figures match inspected figures by decoded pixels. Explicit inspection reuse and closure retained; all seven persistent owned PIDs stopped. This closes the bounded release query checkpoint, not overall acceptance.

Next work tests a genuinely new settings authority incarnation with a lower revision and a changed theme, preserving each app PID. The first private attempt native.AEvZPk was refused before candidate launch because the harness guessed an unsupported scheme; Gold correctly returned the allowed scheme list. All six owned PIDs stopped. Corrected the private harness to the supported Ocean scheme; native.FvWYHJ is running. No product change or relaxed assertion.

Full remaining Bus/authority/input/settings/conformance/performance and final acceptance work remains open. No Fable question, install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:7a886e4fccbf4fa6bc772570bf5d201030f4d9dd (exact tested release source; receipt HEAD contains checkpoint); NOT ACCEPTED — remaining full Bus/authority/input/settings/conformance/performance and final acceptance open; CBC job ids:term-query-release-7a886e4f-a1 (green exact release487 tests), term-query-exact-7a886e4f-a1 (green exact scoped/foreign).


### New settings incarnation recovers at exact release bytes

Private native.Y4inp9 uses the verified7a886e4f release executable and changes settings authority incarnation while each app remains running. Gold moves from revision6 to a new incarnation at2; candidate moves from7 to a new incarnation at2. The replacement authority applies supported Ocean/Dark instead of the preceding Studio/Light. Candidate acknowledges the new incarnation, revision and scheme; both apps retain their own unchanged PIDs. Invalid mode changes still refuse without advancing revision.

Seven paired states cover initial appearance, auto mode without daylight, app light override/reset, light settings, authority loss and new-incarnation recovery. All seven PNG pairs are byte-identical; whole/header/tabs/grid/report SSIM and RMSE distortions zero. Four unique Gold originals and eight full figures inspected; other originals/full figures have explicit file-hash/decoded-pixel inspection reuse. All nine owned PIDs stopped. Binary, authority/PID, native output, comparison, inspection and closure receipts remain private. This closes the different-incarnation/lower-revision recovery case at release bytes.

Private native.FvWYHJ retained a mismatched initial baseline: Gold contains one echoed command line above the same output, while its six subsequent transition pairs are exact. The first baseline originals and full figures were inspected; no acceptance is claimed for that baseline. The repeat adds an explicit interactive Mix prompt readiness guard before typing; all seven states then match. No product change or assertion relaxation, and the earlier startup mismatch is not classified as a C1 defect.

Next CBC `term-raster-bench-7a886e4f` runs the existing ignored cached ASCII/emoji/box-glyph raster benchmark serially in release mode at the same source. It checks warm painting performs no new shaping/resampling and records mean/p50/p99 CPU paint time. This is bounded raster timing, not presented GPU latency or a Gold performance comparison. Read the full saved log and retain its limits. Full Bus/authority/input/settings/conformance/performance and final acceptance work remains open. No install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:7a886e4fccbf4fa6bc772570bf5d201030f4d9dd (exact tested release source; receipt HEAD contains checkpoint); NOT ACCEPTED — remaining full Bus/authority/input/settings/conformance/performance and final acceptance open; CBC job ids:term-raster-bench-7a886e4f (next serial release benchmark), term-query-release-7a886e4f-a1 (green exact release487 tests), term-query-exact-7a886e4f-a1 (green exact scoped/foreign).


### Release raster timing and local scrollback checkpoint

Exact source remains `7a886e4fccbf4fa6bc772570bf5d201030f4d9dd`; verified release executable SHA256 `3bb3c630ea8c3e0a557426d7285cc8ac5193d980fcdf8d9083c0bda1a4951a28`. CBC `term-raster-bench-7a886e4f-a1` passed serially in release mode. The complete 5,841-byte saved log was read and archived privately. At 90 columns by 25 rows, scale 2.5, 1800×950 pixels, 200 measured frames after 20 warm frames, full dirty-row CPU painting measured:

| Fixture | Mean ms | p50 ms | p99 ms |
| --- | ---: | ---: | ---: |
| ASCII M | 1.173 | 1.164 | 1.592 |
| Woman technologist emoji | 2.611 | 2.583 | 3.017 |
| Box/block glyphs | 0.941 | 0.927 | 1.694 |

The unchanged warm-cache miss assertion passed for all three fixtures. These are one-worker CPU render_into timings, excluding GPU presentation, and provide no Gold performance comparison. The exact-source full release gate remains term-query-release-7a886e4f-a1 (487 tests, eleven existing ignores); scoped/foreign term-query-exact-7a886e4f-a1.

Private native.WZ9Pqm drives real Mix output and native local-history input against the same release bytes and canonical sequential Gold service. All 120 numbered fixture rows remain in matching full-history text. Fourteen states prove half-cell wheel accumulation, direction reversal resetting the fraction, live terminal.scroll.speed 20/40/80 on unchanged PIDs, Shift+PageUp/PageDown overlap by one row, Shift+Home/End history edges, and input returning a scrolled viewport to live output. Structured offsets and history counts match Gold; snapshots continue to describe the live screen while the viewport moves.

All 14 PNG pairs are byte-identical at 900×560, Studio Dark, scale 1. Whole/header/tabs/grid/content SSIM and RMSE distortions are zero. Six unique Gold originals and twelve full figures inspected; all other originals and figures have explicit file-hash/decoded-pixel inspection reuse. Header, tab geometry and each visible scroll position match; no unexplained region remains in these states. All seven owned PIDs stopped. Native scripts, settings/PTY/readbacks, binary receipts, comparison figures, benchmark metrics and closure remain private. No product change was required.

Remaining full parity work: Bus/action/topic/broker faults and deeper callbacks, issuer expiry/revoke, paired platform IME and remaining input/search/show faults, remaining Prefs/tune, wider conformance/representative performance and final acceptance. The earlier intermittent Gold input/startup observations remain retained and unclassified. No Fable question, install, desk/VT or merge work. Overall NOT ACCEPTED.

branch: lab/parity-term; head SHA:7a886e4fccbf4fa6bc772570bf5d201030f4d9dd (exact tested release source; receipt HEAD contains checkpoint); NOT ACCEPTED — remaining full Bus/authority/input/settings/conformance/performance and final acceptance open; CBC job ids:term-raster-bench-7a886e4f-a1 (green serial release benchmark), term-query-release-7a886e4f-a1 (green exact release 487 tests), term-query-exact-7a886e4f-a1 (green exact scoped/foreign).
