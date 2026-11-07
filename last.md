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
