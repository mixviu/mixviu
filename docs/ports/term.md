# Term parity work

Gold is MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Branch starts at integration `4f2c0fd`. Method: [parity-method.md](parity-method.md).
Gold remains installed. This lab specimen is **NOT ACCEPTED**.

## Inventory and ownership

The earlier [native requirements](term-needs.md) are requirements, not a port
receipt. Gold's `apps/term/src`, `crates/term-core`, `crates/term-render` and
patched `vendor/teletypewriter` are the implementation oracle.

| Area | Owner | Acceptance still required |
| --- | --- | --- |
| Frame, menus, actions, shortcuts, dialogs, macros | Mix | Full action registry, native callbacks and matched captures |
| Tabs, pane tree, profiles, launch and close policy | Mix | Every mutation, focus, title, ordering and refusal |
| Parser, grid, alternate screen, scrollback, search | Viu compound widget | VT conformance, bounded history, incremental damage |
| PTY, resize, child lifetime and session handoff | Viu native facilities | Real Mix login, sealed-fd isolation, generations, reconnect, reaping |
| Glyphs, ligatures, attributes, colours, cursor | Viu compound widget | Gold raster/GPU tests and matched real-session captures |
| Selection, clipboard, mouse, paste, IME | Viu compound widget | Native input, ownership, cancellation and late delivery |
| Fonts, scale, theme and terminal settings | Mix + Viu | Live Prefs on unchanged PID, authority loss/recovery |
| Bus | Mix + native authority | Gold commands, props, topics, payloads, errors and async completion |

Gold core exposes 23 commands (including seven original macro commands);
frontend adds actions/execute/show/dialog, toolkit drive, debug and complete
macro editing commands. The verified native control lane is a separate
authority contract. It must not be replaced by ambient credentials or an
app-local imitation broker.

## Initial frame specimen

`apps/term` adopts the shared gold frame at gold's 900×560 initial size.
The scene is deliberately empty while the compound widget is extracted;
no terminal, menu, Bus or visual parity is claimed by this checkpoint.
The window follows the generic host's live settings path.

## Native extraction, round 2

Fable D6 permits rio-vt's `simdutf` as **Viu debt**, with no change to D2's
strict Mix/mixd tier. The pinned source has scalar fallbacks, but these do
not establish equal malformed-UTF-8/base64 behaviour. This extraction retains
the pinned gold implementation and adds the foreign policy/gate. Viu, Rio
and simdutf are explicitly forbidden in all strict dependency closures.
The exit path is behaviour-tested pure-Rust validation/transcoding/base64.

`libraries/viu/src/terminal` now contains the gold native model, PTY Machine,
damage tracking, cluster interning, selection, mouse protocols, bracketed
paste, clipboard request queue, glyph raster, atlas/GPU grid and pane-owned
IME cancellation machine. Original tests remain; self-exec names are qualified
under the Viu module. Provenance records original source hashes and adaptations.

Native lease and write-permit mechanisms use in-process callbacks. This
separates lifecycle from the original Rust-owned TabSet; it does **not** claim
verified supervisor attachment, reconnect or generation acceptance. Those are
open integration work. No gold application/tab/pane Rust policy is imported.

Added conformance witnesses cover cursor address/insert/erase/alternate screen,
all UTF-8 and escape chunk boundaries in a representative stream, scrollback
and selection, and real Mix PTY input/resize/exit/reaping. Gold's GPU/CPU parity
and IME tests are included in the native gate. Results remain pending.

## Native widget checkpoint: tested extraction, app load open

At `f8b976e2cea7cc664c849982c98632759ec464b5`, CBC
`term-widget-r2-view-repair-a1` passed 133 tests (five original ignores),
Mix frame and wire tests, the foreign dependency closure gate and generic
host debug build, with no worker diff. The complete saved log was read.
This is native extraction evidence, not app or release acceptance.

Native sessions and the terminal widget are wired through generic Services
and SceneView. Mix has explicit selector validation and bounded replay for
synthetic input. A pure Mix tab/pane reducer now has checked tree traversal,
sibling focus, selector agreement, reorder revisions and the 32-slot cap.
It is not yet connected to the full app Bus/menu surface. Mix-authored split
composition is supported by the compound widget; its latest source is queued
for validation.

A private headless run captured pinned gold at 900×560 after deterministic
real Mix command output. The candidate failed initial behaviour loading, so
no matched pair or visual metric exists. Gold FileTerminal is now correctly
named `file-terminal`. The remaining first load error is obscured by later
callbacks; a new isolated actual-bundle test retains it for diagnosis.
The debug artifact and diagnostic bundle variants are not accepted bytes.
All recorded run processes were stopped; failed runs remain private.

## First native diagnostic pair (FAIL)

The actual-bundle test identified a scene/control ID mismatch: root was
a control whose definition ID was terminal. The scene now contains a
matching terminal node. A real candidate host then produced the same Mix
command output as pinned gold and a 900×560 capture at scale 1. The
candidate used the verified f8b976e debug executable plus the repaired Mix
bundle; this mixed-source repair witness cannot attest release bytes.

Both originals, full side-by-side and diff were inspected. Private receipt:
`term-parity-20261107/first-diagnostic-regions`. Gold run native.gWIe4o,
candidate native.iceFXQ; capture/readback/command and cleanup receipts retained.
Gold PNG SHA256 e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865;
candidate b7aa2953967f204666e64843cb61d47c5c804befcba05b5bf53db81a34f508f0.

ImageMagick 7.1.2-35 normalised **distortions** (zero identical):

| Region | x,y,w,h | SSIM | RMSE | Open difference |
| --- | --- | ---: | ---: | --- |
| Full | 0,0,900,560 | 0.0483143 | 0.055971 | Incomplete app furniture |
| Header | 0,0,900,38 | 0.0749878 | 0.0752184 | Menus and actions missing |
| Tabs | 0,38,900,32 | 0.167079 | 0.0876784 | Tab strip missing |
| Content | 0,70,900,460 | 0.0388236 | 0.0536959 | Grid shifted up 32px; surrounding inset/band |
| Bottom | 0,530,900,30 | 0.0798319 | 0.0308504 | Extra status bar/plug icon |

These are unfinished implementation items, not approved deviations. All
tracked candidate-run PIDs were stopped. The capture succeeded before the
generic close request exposed the missing app.quit handler; that handler
is now added and still requires the next gate/native-close witness.

## Exact widget and live tab/pane checkpoint

At exact code `b095d72222c3c6b018f59edf2d2be2eac483401a`, CBC3
`term-widget-r2-node-fixed-a1` passed 134 native tests, five retained ignores,
two actual-bundle tests, all three Mix suites, the foreign dependency gate
and generic host debug build. No worker diff. Debug SHA256:
`08375b79ec45e965bd258a1660080f572d3ec9dbf2b75ad27db545abae3b5990`.
Private `term-parity-20261107/native.uqPes6` used that executable and the
same archived source, executed the matched real Mix command output, captured
900×560 and passed normal close. All tracked PIDs were stopped. This closes
the first-load/quit defects; it does not change the visual FAIL verdict.

Code `39e1208653a54e0f0bc1b0a040713abc112fa721` connects the Mix reducer
to real session creation/close, tab selection/title/reorder, pane splits and
focus, explicit snapshot/type targets, and scroll commands. Native metadata
and output/title/exit changes reach Mix in process. The reusable document
strip now supports canvas attachment and leading status markers; Term uses
it above its compound surface. The shared frame supports omitting the status
footer, with the existing default preserved for other apps.

Four local Mix suites pass, including real selector disagreement, retryable
validation, request-key ordering, original-outcome replay and UTF-8 byte
bounds. The actual-bundle test now exercises native tab/split sessions,
inactive-pane input without focus movement, retry after close and final
shutdown. CBC `term-live-tabs-39e1208` is queued for this source; native
compile/runtime and new captures remain unverified until its result.

This is partial Bus implementation. Topic publication after props.watch,
the full action/menu/dialog/macro surface, raw-envelope validation and exact
large integer handling remain open, alongside verified native authority,
complete input/clipboard/search/Prefs and release visual acceptance.

CBC `term-live-tabs-39e1208-a1` compiled the Rust changes and passed all
four Mix suites, then failed actual scene validation: the dock root had
unsupported padding/gap ports. Repair `849b2aec8e4d99291108a7671625354c0d7910f3`
removes those ports and applies the full worker fmt patch. Lifecycle
assertions remain intact. Next gate: `term-live-tabs-dock-849b2ae`.

## Native lifecycle witness and pane repair

`term-live-tabs-dock-849b2ae-a1` passed at exact source
`849b2aec8e4d99291108a7671625354c0d7910f3`: four Mix suites, both
actual-bundle tests (including real split/tab sessions and replay), 134
native tests, five retained ignores, dependency gate and debug host build.
Complete log read; no worker diff. Fetched and verified debug SHA256
`56d0f178788e13a333e733f2a9e037058cda8378c20368b5b1626addb8a29e1b`.

Private `term-parity-20261107/native.sf7min` runs the matching source and
artifact. Native Bus exercise passed split output, split replay, inactive
pane input without focus movement, tab close, shell exit preserving its
sibling and normal window close. Every tracked PID was stopped. Retained
captures include the original matched command output and a live split.

Inspected originals, full side-by-side/difference and the split capture.
Private `lifecycle-diagnostic-regions` retains figures and inspection.
Candidate PNG SHA256
`fa3de4df1adba1ed987db9ab00c108f22642f09368a6b422b15fac737cd6c2a4`;
gold and rectangles match the earlier diagnostic. ImageMagick 7.1.2-35
normalised distortions (zero identical):

| Region | SSIM | RMSE | Finding |
| --- | ---: | ---: | --- |
| Full | 0.0337594 | 0.0253796 | FAIL: header and pane surround |
| Header | 0.0749878 | 0.0752184 | Menus/actions absent |
| Tabs | 0 | 0 | Pixel-identical |
| Content | 0.0141672 | 0.0128128 | Grid output/origin matches; black surround |
| Bottom | 0.274618 | 0.0483722 | Black remaining-cell band, missing pane border |

Repair code `0a34c03037ca0e3bdda34f2b87208c93868b5bec` paints the native
pane canvas/border from gold theme roles, adds its split focus ring and
live border/focus settings, and applies gold's physical-pixel grid fit,
texture ceiling and minimum dimensions. Representative fractional-scale,
small-pane and texture-limit cases are retained as tests. Native validation
and new captures are pending `term-pane-paint-0a34c03`. NOT ACCEPTED.

## Pane arithmetic gate and change topics

`term-pane-paint-0a34c03-a1` passed four Mix suites, both actual-bundle
tests, 135 native tests (five retained ignores), the dependency gate and
generic debug build. Complete log read. It returned a formatting-only patch
for the terminal view; this patch is applied in the next code checkpoint.
No new pane-paint capture or exact-source acceptance is claimed yet.

Code `9b53f0d9f50eda3b337d05ed4be636bf2b5d4fdb` adds gold's separate
monotonic change sequence and ordered tab/pane/title changes. Unwatched
changes advance the sequence without accumulating. After props.watch,
the queue holds at most 256 events; overflow drops new deliveries while
sequence gaps remain detectable. Existing Viu background jobs publish
bounded batches, so terminal mutations do not wait on broker delivery.
Tests cover order, unchanged layout revision on retitle, focused versus
unfocused close, watch revision, replay without duplicate events and queue
bounds. All local Mix suites pass. Native delivery/reconnect tests are
still required; this remains partial Bus parity.

Next exact-source gate: `term-change-events-9b53f0d`. After its complete
receipt and any returned patch are handled, fetch its matching artifact,
retake the pane painting captures and exercise actual topic delivery.

## Exact pane pixels; native topic repair witness

CBC `term-change-events-9b53f0d-a1` passed all four Mix suites, both
actual-bundle tests, 135 native tests/five ignores, the dependency gate and
host build at `9b53f0d9f50eda3b337d05ed4be636bf2b5d4fdb`, without worker
diff. Full saved log read. Verified debug executable SHA256:
`ee189b5838344d4e988d34c2c4655fdc5ca5a9ba13b55f591502e1e2460f085e`.

Private native.dtQmFV uses those exact executable/source bytes and repeats
the real Mix session/lifecycle witness. It retains a failed topic probe:
jobs.start was passed an incorrectly nested body. This is an application
submission defect, not evidence that the native Bus cannot publish.
The failed run and its cleanup receipt are retained.

Its matched 900×560 capture is compared with native.gWIe4o in private
pane-painted-regions. Originals, full side-by-side/difference inspected.
Candidate PNG SHA256:
`d9e33f4a2d258739995714eaa678373a9422fdde7e76528939584b7382fb1782`.
At the same rectangles and ImageMagick convention as above, tabs, content
and bottom each have SSIM/RMSE distortion **0/0**. Header remains
0.0749878/0.0752184; full 0.00535614/0.0195939. Missing menus and actions
keep the visual gate at **FAIL**. This single debug state does not accept
all sessions, inputs, fonts, themes, scales or release bytes.

Repair `ca019e33aada1112b34cacdbf4351ffa14f12eb4` passes the job fields
directly, uses gold's suffix inner commands and retains final removal
records with a total two-second drain bound. Private native.HINPJ0 is a
composite repair witness (9b53f0d executable, ca019e3 Mix sources), explicitly
not exact-source acceptance. It passed native lifecycle and delivered 22
events, revisions 6..27 without gaps/reordering, including retitle and
final pane/tab removals before exit. All tracked PIDs stopped.

Next exact-source gate: `term-events-drain-ca019e3`; then repeat native
topic/lifecycle evidence on its verified artifact. Full gold shell/native
authority, menus/actions/Bus, input/clipboard/search/IME, live Prefs and
release visual acceptance remain open.

## Exact topic drain; gold command registry checkpoint

CBC `term-events-drain-ca019e3-a1` passed at
`ca019e33aada1112b34cacdbf4351ffa14f12eb4`, with no worker diff: four
Mix suites, both bundle tests, 135 native tests/five ignores, dependency
gate and debug host. Complete log read. Verified debug SHA256 remains
`ee189b5838344d4e988d34c2c4655fdc5ca5a9ba13b55f591502e1e2460f085e`
(the source change was entirely Mix). Private native.BG1pld now repeats
the topic/lifecycle/final-close witness with matching executable/source,
without composite repairs. All tracked PIDs stopped.

Pinned gold native.oBJRB9 supplies its complete term.actions and native
menu-list replies, archived privately. Its normal Quit again exceeded
the eight-second harness bound; finally stopped the tracked processes.
This repeated gold shutdown issue is retained as evidence, not accepted
as a port requirement to hang or a new Fable request.

Code `d85560415c58ce2ae023edc12dcf2461063e5e42` adopts all 61 registry
entries, six menu trees, four toolbar actions and command chords as data.
Mix owns dynamic enablement, target resolution, tab/pane navigation,
directional geometry fallback, themes, zoom steps, clipboard effects,
macro/help/search state and execute replies. Command search is a reusable
Viu control; clipboard operations use the shared in-process platform
facility, and existing native macro ownership is enabled. Five local Mix
suites pass. New native command-search admission tests are in its owning
library and added to the scoped gate.

This checkpoint does not claim action parity. Native callbacks and menu
pixels await `term-actions-d855604`. In particular zoom persistence,
full action-state metrics, live binding resolution, complete About text,
action-topic overflow and the broader Bus contract need further work.
The verified session supervisor, input/clipboard/IME/search completeness,
full live Prefs and exact release acceptance remain open. NOT ACCEPTED.
