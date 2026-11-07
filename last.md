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
