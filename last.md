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
