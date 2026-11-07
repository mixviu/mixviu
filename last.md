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
