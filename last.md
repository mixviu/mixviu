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