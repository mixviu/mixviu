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

## Native command path diagnosis

`term-actions-d855604-a1` passed five Mix suites, both bundle tests,
135 terminal tests/five ignores, command-search admission, dependency
gate and debug host. Complete log read; returned formatting changes
applied. Fetched/verified debug SHA256:
`94dfa290cf005964ef8dc0ce4eeb5b8ad3be28e4e5c9d0599dc7b996c93e96f2`.

Private native.zPpnfQ rendered menus but failed menu-click New Tab.
Diagnostic native.EjTjXv traces the shared adapter sending
`value.command={command:"tab.new",origin:"menu"}` instead of the flat
command payload already produced by the native menu. Generic repair
388b38a preserves that payload unchanged, matching the other menu path.
It also applies the full worker fmt patch. Native menu execution must
be repeated on the repaired executable.

The exact d855604 diagnostic capture versus native.gWIe4o is retained
in actions-first-regions: tabs/content/bottom SSIM/RMSE each 0/0;
header 0.0353355/0.0448267; full 0.00266545/0.0116771. Candidate PNG
SHA256 `55ec46ee594d3b3fd0ca8b704237ff22a87331ad7a0678dd4614df9535283c56`.
Originals, side-by-side and difference inspected. Menus/labels align;
action-icon/caption raster and position differences remain. Overall FAIL.

Separate native.LSmstT passed direct execute, Ctrl+Shift+T, targeted close,
dialog open/answer, command-search typing/selection, local light/session
theme, zoom/reset and normal close. These do not cover the faulty menu
adapter. The first direct variant used an invalid pointer button name;
the corrected primary-button harness and failed receipts are retained.
All tracked PIDs stopped; spawned action tabs were closed natively.

Inspection found an empty Keyboard Shortcuts body despite its bound rows
being present in the model. Native.eaUKMo retains the model/readback and
repeat screenshots; code 2b580ce adds a pure projection regression check.
Renderer diagnosis remains open, alongside About text, action-state
metrics, binding reload, zoom persistence and the broader inventory.
Next exact-source gate: term-menu-intents-2b580ce at
`2b580ce99bd8ea85049bebc444bcb38abeef22da`.

### Menu callback verified; dialog and observed-state repair checkpoint

`term-menu-intents-2b580ce-a1` passed at that exact source with no worker
diff: five Mix suites, two bundle tests, 135 terminal tests (five existing
environment/benchmark ignores), command-search admission, foreign gate and
debug host build. Fetched debug executable SHA256:
`b311f41117755b0ec08ab59c0faa4dcf48ae2431b17407ebb351136312fa3f56`.
Private run `native.h5uA6t` passed the actual File → New Tab callback,
targeted close, keyboard New Tab, command-search selection, dialog commands,
local mode, zoom/reset and normal close. Recorded processes all stopped.
This closes the menu payload defect; it does not grant visual acceptance.

The two-second settled shortcuts capture remained blank. Diagnosis found
the earlier grid renderer was unreachable: gold dialogs return through
`gold_dialog` before reaching it. Commit `301080a` moves the bounded grid
renderer into the active path and removes the dead branch. The new native
widget regression checks that 23 rows contribute to dialog height; gate
execution and matched candidate capture are pending. Gold's corresponding
capture is retained in private run `native.Dae992`, with stopped-process
receipt. Gold shows the scrollable shortcut rows.

Commit `598e6bd` reports actual logical cell size, device scale and unscaled
font size from the raster/session placement. Mix now supplies gold's
`term.actions.state.panes` tree order, focused flags, grid sizes and outer
rectangles. Unknown pre-draw metrics stay null. Terminal text scale now
rebuilds the raster without replacing its session. The owning native test
checks handle retention and observed metrics across font/scale changes;
these Rust changes await CBC. Commit `aaabb6e` routes live keymap updates
through the existing gold conflict resolver, refreshes native terminal
chords, menus and help, and follows native effective-theme notifications.
Pure tests cover multiple bindings, explicit unbinding and measured reply
composition. Native live-Prefs evidence remains required.

Next source: `9ea52c727cd2d7f4f602fc485e152664eb84edf1`, CBC job
`term-dialog-metrics-9ea52c7`. Overall **NOT ACCEPTED**: final native
dialog/metrics/keymap/Prefs witnesses, complete verified session/control
and shell handoff, full Bus/error/envelope parity, clipboard/input/IME,
search, zoom persistence, About provenance and release visual gate remain.

### Dialog/metrics native witness and seamless-frame diagnosis

`term-dialog-metrics-9ea52c7-a1` passed the five Mix suites, two bundle
tests, 135 terminal tests (five existing ignores), command-search admission,
new live-renderer shortcut-grid regression, foreign gate and debug build.
Read the complete log; formatting-only diff applied as `f55e499`.
Debug artifact SHA256:
`44770e2417b1347dc0bd4ecf1b521ab87203f92457daed3b887f85e098712dfe`.

Native `native.uIceLC` shows the repaired shortcuts dialog; its
454×458 region matches gold exactly (SSIM/RMSE distortion 0/0). Live
text scale 1→1.5 changes the measured cell 9×20→14×30 and grid
98×23→63×15 on unchanged PID 3815378. A settingsd keymap update binds
New Tab to F2; actual native F2 opens the second tab. Native readbacks,
replies and stopped-process receipt are retained.

Pinned gold view.rs explicitly calls `titlebar::seamless(ctx,true)`.
The candidate omitted this setting, causing its one-pixel header rule and
half-pixel action/caption glyph offset. Commit `e945bd9` supplies the same
setting through the existing shared frame facility. Composite diagnostic
`native.BvAeFe` uses the above executable with the e945bd9 Mix bundle;
it is explicitly not exact-source or release acceptance. Its real-session
and shortcuts images are byte-identical to the matched gold images at
900×560, Studio Dark, scale 1. All measured regions and whole images have
SSIM/RMSE distortion 0/0, and original/montage/diff figures were inspected.
Session PNG SHA256 `e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865`;
keys PNG `13f5f56e65ae1e832b737d388ed5bd879daa894ca80c34acd682342bc487df2b`.
The live metrics/keymap witness repeats successfully; recorded PIDs stop.

Next exact-source repeat: `term-seamless-e945bd9` at
`e945bd9815fadde1296715fa288db8b4129cdd5a`. Fetch its matching executable
and bundle and repeat these witnesses. **NOT ACCEPTED**: debug checkpoints
do not replace the full feature/Bus/Prefs inventory or release evidence.

### Exact frame checkpoint and platform clipboard repair

`term-seamless-e945bd9-a1` passed with no worker diff; complete saved log
read. Matching debug executable/bundle at e945bd9 verified, SHA256
`44770e2417b1347dc0bd4ecf1b521ab87203f92457daed3b887f85e098712dfe`.
Exact-source run `native.GDMFZD` repeats real Mix output, keys dialog,
measured cell/grid text-scale update on one PID, live F2 rebinding and
normal close; all recorded PIDs stop. Both real-session and keys PNG
hashes equal gold's hashes above. `session-e945-exact` and
`keys-e945-exact` retain inspected figures: all whole/region SSIM/RMSE
distortions 0/0. This is a passing bounded debug frame witness, not final
release or app acceptance.

Next code `327ab521c783643a6c7430c09e9c4641ef1b5382` restores gold
platform clipboard-event classification: without Shift, Copy/Cut/Paste
send shell control codes; Alt prefixes ESC; Shift requests the clipboard
operation, while image-paste remains Ctrl-V. A held clipboard chord fires
once until release/modifier reset. Reusable widget intents carry the
originating native handle and already-read paste text; Mix retains app
actions and avoids a second clipboard read or redirect to another pane.
Stale closed targets are ignored; stale Cut cannot close the new focus.
Menu paste retains asynchronous native clipboard read. Added control-byte,
Meta/Shift/release/image tests, a native widget event/owner test and Mix
supplied-paste effect tests. Five Mix suites pass; Rust/native runtime
validation awaits CBC `term-clipboard-327ab52`.

Remaining clipboard work includes primary selection, middle/right mouse,
copy-on-select, OSC52 window integration and full native clipboard-owner
tests. Verified session/control/shell, full Bus, IME/search/Prefs and final
release acceptance remain open. Overall **NOT ACCEPTED**.

Clipboard gate `term-clipboard-327ab52-a1` failed compiling the new test:
egui 0.37 has `ModifiersChanged` events rather than a `RawInput.modifiers`
field. The five Mix suites and two bundle tests passed before that failure;
terminal tests did not execute. Complete saved log read and returned fmt
patch applied. Repair `cf75bd9` uses the current native event API and keeps
modifier changes in event order in the runtime widget. A clipboard event
followed by Shift release in the same frame must retain its earlier Shift
meaning; the native test now includes that case. Next CBC:
`term-clipboard-order-cf75bd9`. No passing native clipboard claim yet.

`term-clipboard-order-cf75bd9-a1` subsequently passed: 137 terminal
tests (five existing ignores), both bundle tests, command-search/dialog
tests, five Mix suites, foreign gate and host build. Complete saved log
read; formatting-only patch applied in ff3161f8. Matching debug artifact
fetched/verified, SHA256
`fbcdfd9e4949f4a6d8b86de4d4adc8c15cc65291dde7a491fb00c5ddec6130a8`.

The existing generic `viu.ui.*` input driver lacked platform clipboard
event injection. Commit `06f6f251` adds `viu.ui.clipboard` with copy/cut/
paste kinds, existing modifier flags, bounded supplied text and a native
key-release/modifier-reset follow-up. It uses the existing host event
queue, not a separate transport or app shortcut. Tests check refusal
without partial input, Unicode text, modifiers and release. A private
native harness is prepared to verify actual shell Ctrl-C, Shift-paste,
plain-paste control-code refusal and Shift-Cut closing a split.
Next CBC `term-platform-clipboard-06f6f251`; runtime witness pending.

`term-platform-clipboard-06f6f251-a1` passed (137 terminal tests, seven
input tests, two bundle tests, five Mix suites, command-search/dialog,
foreign and debug build). Complete log read; fmt-only diff applied as
7dd0022d. Matching debug executable SHA256
`3511432b2aa848cf0cffe656f663e5b37c79bd26c7533ae5998a0f643f4a7db7`.
Native run `native.JxY16p` exposed missing host catalogue registration:
`viu.ui.clipboard` returned UNKNOWN_COMMAND before any platform event.
All recorded PIDs stopped. Commit b2a7b8aa adds the driver to the existing
generic host catalogue. Repeat gate/native witness required; unit tests
alone did not cover that host registration. Overall **NOT ACCEPTED**.

### Platform clipboard runtime checkpoint; cursor blink

`term-clipboard-catalogue-b2a7b8aa-a1` passed with no worker diff; full
saved log read. Verified matching debug executable SHA256
`b2ceef799e80f1069f85f7d8226750946530af9a2a950627efdc53ae0d7353d3`.
Native `native.fjLLL5` passes platform Copy→shell Ctrl-C, Shift-Paste
of Unicode supplied text, plain Paste→Ctrl-V without inserting the
supplied text, Shift-Cut closing the focused split while retaining its
sibling, normal close and all tracked PIDs stopped.

Two earlier runs are retained (`native.R56uYX`, `native.TkKsDs`): the
harness initially expected bracketed paste to execute immediately, then
expected a single Ctrl-C to clear Ctrl-V's literal-next-character state.
Snapshots demonstrated correct shell behaviour. The corrected witness
uses explicit Enter and a second Ctrl-C for recovery; no application code
was changed to satisfy those mistaken expectations. This does not yet
attest selection-copy readback, primary selection, OSC52 or every clipboard
ownership race.

Code `27c71218` wires live `terminal.cursor.blink` to a reusable native
530 ms phase, matching gold. Focus and typed input restart the shown phase;
unfocused, disabled or program-hidden cursors schedule no blink repaint.
Tests cover phase edges, focus recovery, activity and no-wake cases. CBC
`term-cursor-blink-27c71218` and native live-Prefs phase evidence pending.
Overall **NOT ACCEPTED**.

### Live terminal settings follower repair

`term-cursor-blink-27c71218-a1` passed: 138 terminal tests/five existing
ignores, seven input tests, both bundle tests, five Mix suites,
command-search/dialog/foreign and debug build. Complete saved log read;
formatting-only patch applied as 152a7dfe. Fetched artifact SHA256
`2e4cd45271fb78dba0da5700723c6e02793503a00453e8ceeaea338bd9b048a6`.

Native `native.yijAlb` exposed a shared follower defect: terminal-only
settings changes were not subscribed to and never reached the model.
The follower listened only to look and keymaps. Commit `21e53cc7` adds
gold's terminal/email/ced application settings namespaces to subscriptions
and native refresh hints. This repairs app-pref updates without requiring
an unrelated appearance change. Exact-source runtime repeat pending.

Diagnostic `native.g5bXp4` at the old 27c71218 executable uses an explicit
appearance co-change to trigger its existing follower. It passes two
pixel-distinct cursor phases and a steady cursor after disabling blink,
on an unchanged PID, with normal close. This verifies blink mechanics,
not the new terminal-only subscription. Earlier diagnostic `native.SXo6yj`
compared cropped PNG file hashes including ImageMagick date metadata;
raw RGBA hashes prove exactly two pixel states. The corrected harness
strips crop metadata. Failed runs are retained. About provenance remains
open; the live-Prefs defect took precedence.

Next gate `term-terminal-prefs-21e53cc7`, source
`21e53cc7f8c6f4767d59a2f49901acd5144898b0`. **NOT ACCEPTED**.

### Terminal-only live Prefs verified; About provenance checkpoint

`term-terminal-prefs-21e53cc7-a1` passed with no worker diff. Complete
saved log read in two bounded chunks; matching debug executable/bundle
fetched and verified, SHA256
`d6c3a997fe7a811d8f063beca525ace271df15913add3dced2529abda95725d2`.
Native `native.lrkN8M` changes only `terminal.cursor.blink`, with no look
co-change. Two cursor pixel phases appear; disabling blink produces four
identical whole PNGs. PID 3933235 remains unchanged, native settings and
Mix model both read back false afterwards, normal close and all tracked
PIDs stopped. Steady PNG equals gold's real-session PNG hash
`e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865`.

Code `43dcbf58` completes About content with actual compiled host
provenance, description, licence and live settings-source text. The
generic runtime context now uses the repository's buildinfo helper and
consumer build script rather than unrelated environment names that
returned unknown. It reports its own Viu version/SHA/build time; gold's
values are never copied. The actual-bundle test checks rendered About
model provenance and absent-authority text; pure tests cover line order
and connected/disconnected labels. Five Mix suites pass. Rust gate,
returned lock/fmt handling and matched About visual witness pending:
`term-about-provenance-43dcbf58`. Full parity remains **NOT ACCEPTED**.

`term-about-provenance-43dcbf58-a1` passed, including actual-bundle About
provenance checks. Complete saved log read in two chunks. Returned
Cargo.lock dependency and test-format diff applied/pushed as 8c5f6333.
Fetched matching debug artifact SHA256
`f9952a67a1143ee5dbe1305e274336585d09f5326cdc619f6a2dfbb52c13944d`.
Native candidate `native.U7W85I` and pinned gold `native.m4sBwh` About
captures have identical layout, title, fixed text, button and settings
source. All differences are confined to the actual compiled provenance
line. Its dirty marker truthfully reflects the worker's prepare patch.
Both tracked-process cleanup receipts pass.

Inspected `about-43dcbf58-regions`: header, above-provenance and
below-provenance SSIM/RMSE distortion 0/0; full .0050312/.0396607,
entirely differing data in the build row [230,255,440,18]. This follows
the standing truthful-runtime-data ruling, not a fabricated gold version.
Exact-source repeat `term-about-exact-8c5f6333` at
`8c5f6333c002bb1db8ad4684457394c06e60ef25` pending before final artifact
evidence. Overall **NOT ACCEPTED**; full remaining inventory applies.

### Exact About and Mix zoom-persistence checkpoint

`term-about-exact-8c5f6333-a1` passed with no worker diff; full saved log
read. Matching debug executable SHA256
`f361ee4f3e477144fb03b814a9d5d8fe6344b1533b015435eb058d36b04891c1`.
Native `native.3l2dVA` repeats About, normal close and stopped-process
receipt. Inspected exact-source About figures: header and regions above/
below provenance 0/0; whole SSIM/RMSE .0046487/.038541, entirely actual
version/SHA/time data. Authority-loss/recovery and release coverage remain.

The gold zoom-persistence machine is now Mix-owned: 400 ms debounce,
one read/write in flight, fenced settings.apply, fresh-read conflict
retry, event-driven failure retry, refusal clear/restore and reset's
unwritten cancellation or in-flight correction. It uses existing Viu
Jobs/timers and settingsd, with native process identity/counters for
operation IDs. Added a pure scenario suite and provenance source hash.
Six local Mix suites pass.

Composite diagnostic `native.mnoDiz` (8c5f6333 executable, 78d62995
bundle) passes saving a zoom and reset before writing; all recorded PIDs
stop. Earlier native.o6EiK2/native.pLLrd4 exposed uuid() being unavailable
in the embedded evaluator; replaced it with native identity/counters,
also fixing the clipboard-read token path. They are retained failures.
The final f32 size bridge reuses gold FontSize for exact persisted values;
that native change is not covered by the composite diagnostic.

Next gate `term-zoom-persist-f0d36085` at
`f0d3608585f7791e4b3f536a6a6c78237466f26d`. Exact-source zoom persistence
tests, native conflict/failure/reconnect/reset races and release evidence
are still required. Overall **NOT ACCEPTED**.

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

### Native layout context compile repair

CBC term-layout-context-f47e845d-a1 failed with Rust E0282 before tests: the local completion reply shadowed the optional supplied reply. Complete 26542-byte saved log read; returned formatting applied. Code a374b0974590c2257595f0a276d6022498c78e01 names the supplied argument separately, preserving the existing cancellation/commit rules. No test weakened and no acceptance claim.

CBC term-layout-reply-a374b097 is pending at that exact code, term_gate --prepare, await a8f168f1-32bc-4c66-aa26-7b98585d64d4. Read its complete saved log on resume, repair any failures, apply returned fmt/lock and repeat exact source as required. Protected layout producer and Mix affected-pane policy/reducer remain next; the full feature/Bus/Prefs, conformance and release visual gates remain open. No desktop or gold changes.

### Native layout bridge passes; protected Mix policy connected

CBC term-layout-reply-a374b097-a1 passed exact a374b0974590c2257595f0a276d6022498c78e01 with no worker diff: 171 terminal tests, five existing ignores, 13 runtime/calls/timers tests, seven input tests, two bundle tests, six Mix suites, dialog/search, foreign gate and debug build. Complete 42410-byte saved log read in two chunks. Four bridge tests prove owner-bound native scope, single commit, cancellation/abandonment, queue recovery and ordinary-event forgery rejection. No release or new visual acceptance claim.

Code be8edc5959b0666579eaaa55b29db641f2949c09 connects protected ManageLayout/Terminate delivery to the existing native-only runtime callback. Native guards validate target/sibling generations and current parent/connection permits, then check mutation epoch and reserve the native replay sequence. Native completion retains begun outcomes. The request authority stays in Rust closures, not callback JSON. Mix computes affected tabs/panes from pinned gold TabSet::control_affected and owns tab new/select/close and pane split/select/close, including replacement-tab focus and profile defaults. The actual Term handler publishes metadata/scene and settles native completion. No new transport, service or Rust application policy.

New pure Mix tests and lint pass locally: source/destination/replacement affected sets, explicit inactive targets, denied mutations without side effects, split directions, close cleanup, new tabs and gold focus retained on spawn failure. The CBC gate now includes this seventh Mix suite. Expanded real-Mix owner fixture drives verified protected selection, exact retry, invalid split, actual split/PTY creation, omitted/stale sibling authority and high-water retention through a fixture runtime pump loading the real authored reducer. These native cases and actual bundle loading are pending CBC, not yet proven.

Next CBC term-protected-layout-be8edc59 at the code SHA above, term_gate --prepare. Read the complete saved log, repair failures, apply fmt/lock and rerun exact source. Then exercise actual generic host protected layout and close/replacement/replay cases, broader enforcement/reconnect, remaining full Bus/input/IME/search/Prefs inventory and release matched visual acceptance. NOT ACCEPTED. No Fable deviation, install or desktop/gold changes.

### Protected Mix layout pipeline passes; close/replacement proof expanded

CBC term-protected-layout-be8edc59-a1 passed: seven Mix suites including affected-pane policy, 171 terminal tests with five existing ignores, 13 runtime/calls/timers checks, seven input checks, two actual bundle tests, search/dialog, foreign gate and debug artifact. Complete 51070-byte log read in three chunks. The close and natural-exit subprocess fixtures both proved verified native layout selection, exact retries, invalid-direction refusal, real split child creation, omitted/stale sibling-authority refusal and unchanged high-water on denial through the actual authored reducer and native runtime scope. Worker formatting applied in d16d82fd. No release or new visual claim.

Code d16d82fd586c030f47f66a0f29edb174655af9d9 extends those real-broker/real-Mix fixtures: protected new tabs and retries; closing an inactive singleton selects its following replacement without closing the former active child; closing the active last-position tab requires every leaf of the previous replacement tab; incomplete authority refuses without consuming the sequence; close retries and term.operation survive target removal; the original pane remains, metadata/high-water agree, removed handles fail and removed child PIDs disappear from /proc. These new cases are pending CBC. No implementation policy moved into Rust.

Next CBC term-layout-close-d16d82fd at that exact code, term_gate --prepare. Read complete saved log, repair and apply any fmt/lock diff, then repeat exact source if changed. Generic-host native layout captures, broader enforcement/reconnect, remaining full Bus/input/IME/search/Prefs inventory and final exact release visual acceptance stay open. NOT ACCEPTED; no Fable deviation or desktop/gold changes.

### Exact native close gate and matched protected-host captures

CBC term-layout-close-d16d82fd-a1 passed exact d16d82fd586c030f47f66a0f29edb174655af9d9 with no worker diff: 171 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input tests, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug artifact. Complete 42007-byte saved log read in two chunks. Both real-Mix close/exit fixture modes now pass protected new tabs/retries, inactive singleton close and following replacement, last-position close and previous replacement, incomplete authority refusal before reservation, retries/retained outcomes after removal, metadata/high-water and child-process cleanup.

Fetched and verified debug executable SHA256 fb737f382caf00458f40be9c6c84ccecb82de4ab86b79a04c5af2e9e0947e9d9, source as above. Gold stays pinned 8779f33284b7d3f1178c3d57a699efb9b0867b22, executable SHA256 153c369007f2da15bdb28fcd1dcbc93f511ff07834f11c2b046650d4c1c51bf6. Private archive native.8ss83B contains executed harness, exact command/function fixture, binary/capture/native readbacks, protected request/reply log, comparisons, inspection and closure receipt. Owned native Bus/settingsd/headless compositor; sequential canonical term identity, matched 900x560 clients, scale 1, Studio Dark, Inter 400/14.667, terminal 400/14.667, CSD right captions, bar menu and row actions.

Real host endpoints on BOTH apps passed protected split and exact retry, native Tab/Shift+Tab focus, a new enrolled shell and natural exit, protected pane close and retry after removal, retained term.operation and frame readback with only the intended survivor. All 11 tracked PIDs, including broker and six shell children, were stopped. Discovery uses native noded.session.list through Mix send with the strict bus/native-session envelope; no additional transport or automation language.

The same short pair() command invokes an identical private .mixrc function which clears the display and prints red/bold, a constant marker, wide and combining text. No fabricated terminal output. Original gold/candidate session PNGs share SHA256 e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Protected split PNGs share SHA256 165ac3aed587310d3180dd5bb870986f7c66bb7d5dd770954dcf209912b1f75d. Fresh parity_compare output directories compare-session and compare-protected-split contain full/cropped figures. ImageMagick 7.1.2-35 SSIM is distortion (zero identical): full [0,0,900,560], header [0,0,900,38], tabs [0,38,900,32] and terminal [0,70,900,490] all have SSIM 0 and RMSE 0 in both states. Inspected both originals, both full montages and both full difference figures; no differences remain in these states.

Earlier native.5IDCZa retains unequal long synthetic-command echo: full session SSIM distortion 0.0135522/RMSE 0.055697, split 0.0084703/0.0445039; header/tabs zero. Images inspected: extra echo line on gold shifts output; no acceptance claimed for that mismatched-data run. An alternative protected-evaluation recipe native.PhXX9p stopped when local gold shell.status advertised evaluation UNSUPPORTED and term.execute relayed UNSUPPORTED; its five tracked PIDs stopped. That local capability differs from the newer worker shell used by positive forwarding tests. It is recorded separately, without claiming forwarding support in the local gold shell or weakening positive worker checks.

This is debug functional/visual evidence for the tested states, not release acceptance. Remaining mouse/selection/primary/OSC52, IME/search/Prefs, full Bus/topics/errors/enforcement/reconnect, full conformance inventory and exact release matched acceptance remain open. NOT ACCEPTED. No install, gold change or Fable deviation.

### Shared native clipboard and primary-selection path checkpoint

Code 13b95e5ced6018a1502ce8ea2ec7af702a5a5f22 adapts pinned gold apps/term/src/selection.rs and clipboard.rs into reusable Viu terminal-native facilities. Source hashes and adaptations are in provenance/terminal-native.json; original licences retained. Existing smithay-client-toolkit/Wayland/raw-window-handle dependencies are reused, with no new foreign dependency. The display-owning eframe host constructs the worker; a layer-only host without a display refuses reads as gold does. The worker uses a separate event queue on the same native window connection, matching gold; it is not a service or parallel Bus.

viu_platform retains its bounded in-process queue, with a validated optional primary boolean. Host writes/reads go through the gold clipboard/primary worker; original Mix request IDs are mapped to the worker completion owner. Four original tests remain: read-busy/nonblocking write and refusal isolation in the wrapper; byte/UTF-8/newline bounds and actual pipe busy/timeout/oversize/ordinary-read recovery in the worker. The existing terminal scoped gate includes them. Removed the former viewport RequestPaste wait path so native clipboard reads have one path. No application tab or paste-target policy moved into Rust.

This checkpoint adds native platform capability only; terminal OSC52 draining/replies and mouse selection/copy-on-select/middle-paste ownership still need their Mix/native adapter and real Wayland evidence. Ordinary platform Paste input events still follow the existing native input route. Build/runtime/clipboard readback not yet claimed. Next CBC term-native-selection-13b95e5c, term_gate --prepare at the exact code above. Read its full saved log, repair failures, apply fmt/lock and repeat exact source; then verify actual clipboard/primary ownership, empty/refused/late/busy reads, original paste-owner survival, and connect OSC52/live policy.

The previous exact native lifecycle and matched protected-host evidence at d16d82fd remains recorded above; it does not attest these new bytes. Overall NOT ACCEPTED: remaining input/clipboard/IME/search/Prefs, complete Bus/enforcement/conformance and release visual acceptance open. No desktop/gold change or Fable deviation.

### Native selection wrapper import repair

CBC term-native-selection-13b95e5c-a1 failed before tests with E0433: the imported clipboard wrapper still used the gold sctk alias without importing the existing smithay_client_toolkit crate under that name. Complete 9771-byte saved log read. Worker formatting applied and the missing alias added in e59f321259ef1606c166b8d2b0cb7d015c1bec61. No dependency, test or runtime behaviour weakened.

Next CBC term-selection-alias-e59f3212 at that exact code, term_gate --prepare. Read the complete log, repair further failures and apply fmt/lock changes before exact-source repeat. Native clipboard/primary readback and terminal OSC52/mouse adapters remain unproven/open; prior matched protected-host debug evidence applies only to d16d82fd. NOT ACCEPTED. No install, gold change or Fable deviation.

### Native selection gate passes; OSC52/live read policy wired

CBC term-selection-alias-e59f3212-a1 passed exact e59f321259ef1606c166b8d2b0cb7d015c1bec61 with no worker diff: 175 terminal tests, five existing ignores, four imported clipboard/pipe tests, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug artifact. Complete 42772-byte saved log read in two chunks. Actual compositor-backed clipboard/primary readback remains to be driven; unit success does not claim it.

Code 0c68440672ce66d600f337d713b8048b093e75d9 connects program OSC52 effects to the native window selection worker. Mix forwards the live terminal.clipboard.osc52_read flag to the in-process terminal binding. Existing and newly created panes obey it; reads default off and writes remain allowed. Native host drains hidden as well as visible owned panes in bounded fair turns, retaining any remaining queued requests for another frame. Native reply formatter closures stay outside JSON.

Selection completion has explicit typed owners for a Mix platform request versus an OSC52 query. Each OSC52 read retains its original immutable terminal identity/formatter with weak ownership; close or handle replacement refuses the delayed reply rather than retargeting another pane. Concurrent refused reads retain distinct IDs, so an OSC52 completion cannot fall through into ordinary paste. The ownership and terminal locks cover the bounded queue write without retaining a pane across the asynchronous read.

Four pending native regressions cover hidden-pane request order and queue remainder, fairness under a busy low handle, successful original-owner reply versus close/replacement, and live permission changes including new-pane inheritance while writes remain enabled. Existing parser/model OSC52 tests and the full scoped gate remain unchanged. Next CBC term-osc52-owner-0c684406, term_gate --prepare at exact code above. Read complete saved log, repair any failures, apply fmt/lock and repeat exact source. Then fetch verified bytes and test real clipboard/primary/OSC52 queries, busy/refused/late reads and matched native captures.

Prior protected-host matched debug evidence applies to d16d82fd only. Mouse selection/middle-paste/copy-on-select, complete IME/search/Prefs, full Bus/enforcement/conformance and release acceptance remain open. NOT ACCEPTED. No gold/desktop changes or Fable deviation.

### OSC52 native gate and real compositor clipboard/primary witness

CBC term-osc52-owner-0c684406-a1 passed: 179 terminal tests, five existing ignores, including the four new ownership/policy/draining regressions; 13 runtime/calls/timers checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug artifact. Complete 64314-byte saved log read in three chunks. Returned formatting applied and committed as c1afd7011c3315b635cf06e2c485355a9884cef1; an exact-source repeat at that revision is still required.

Fetched/verified debug artifact source 0c68440672ce66d600f337d713b8048b093e75d9, executable SHA256 8c74fe7214626ac3f3e232c0da74876518af0697563543c73c45f6fc17ae7f47 (worker formatting present; diagnostic, not release acceptance). Private native.Hr6vCc contains the executed fixture/harness, all requests/replies, snapshots, fenced settings receipts, binary receipt, witness.json and closure.json. On sequential canonical gold/candidate hosts over an owned Bus/settingsd/headless compositor, both real Mix shells emitted OSC52 clipboard writes and the normal Paste action read back TERM_OSC52_CLIPBOARD through native Wayland selection transfer into the original shell. Only encoded fixture text was emitted by the copy command; the visible marker came from actual paste readback.

Both also emitted an OSC52 primary-selection write containing TERM_OSC52_PRIMARY. Live read permission false -> true -> false, without restarting either app or shell, produced exact PTY reply-written deltas 0 -> 32 -> 0. The enabled response placed the exact expected primary base64 token in the originating shell; the 32-byte OSC52 envelope matches the parser formatter. Disabled queries produced no reply bytes. This covers native primary read transfer and query permission, independently of the still-open mouse middle-paste gesture. All seven tracked processes stopped; no live clipboard, desktop or Prefs were touched.

Earlier native.2NLAyw passed clipboard roundtrip plus the protected split/retry/close/exit sequence, with all 11 tracked PIDs stopped. Longer native.yQYxUE passed the clipboard and primary subcases on both apps, but a later candidate split had no usable ready grant and entered the existing gold actor 60-second mint-floor graphics-only fallback; the combined run failed and all nine tracked PIDs stopped. That failure is retained and is not a full-run pass. Matched long-idle grant-pool/enforcement/reconnect behaviour remains open; no mint security floor was changed.

Next CBC term-osc52-exact-c1afd701 at exact c1afd7011c3315b635cf06e2c485355a9884cef1, term_gate --prepare. Read complete saved log and require no worker diff; fetch its verified bytes for the native repeat before advancing acceptance. Then complete mouse reporting/selection/middle-paste/copy-on-select, native IME/search/Prefs, full Bus/topics/errors/enforcement/conformance and exact release matched visual acceptance. NOT ACCEPTED. No Fable deviation or install/gold change.

### Exact OSC52 witness; ordered pointer/selection checkpoint

CBC term-osc52-exact-c1afd701-a1 passed exact c1afd7011c3315b635cf06e2c485355a9884cef1 with no worker diff: 179 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug build. Complete 43670-byte saved log read in two chunks. Fetched and verified executable SHA256 19bd7ef38e2cc9b2daa09447c67ed74bb14c62218473c85da4e3d4c259fbb7fe. Private native.31X4Jv repeated actual gold/candidate OSC52 clipboard write/menu paste, primary transfer and live false-true-false permission at unchanged processes: exact reply-written deltas 0/32/0 and expected primary base64 text in the original shell on both. All seven tracked PIDs stopped. Harness, all receipts, snapshots and witness.json retained. Debug functional evidence only, not release acceptance.

Code 0bffd1a7b7e486e94bf77cca8c90a4a20fbe0689 ports gold mouse.rs gesture decisions into the reusable compound widget. Source hash/adaptations recorded in provenance. Events retain their own coordinates; a grab latches its original native terminal, button and local/reporting ownership. Matching releases use the press modifiers even after Shift changes; unrelated releases cannot end a grab. All three buttons and hover motion report to the program. Half-cell anchors and gold same-cell 300ms simple/word/line click cycles replace the prior delayed response-based selection. Local selection clears other owned panes, writes PRIMARY on finish and optionally clipboard via live terminal.mouse.copy_on_select. Local middle press requests native primary paste through Mix with its immutable handle. These raw selection/paste effects do not invent registry action-executed events.

The generic widget records geometry before dispatching pointer and keyboard input together in original event order, so a same-frame click changes the native input owner before the next key rather than typing into the former pane. Focus/clipboard application effects remain authored Mix. Wheel ownership follows ordered pointer positions, including exclusive pane edges; existing wheel zoom/scroll mechanics retained. Disappeared widgets, hidden/closed grab owners and focus loss cancel the old grab once. Popup layers block underlying pointer handling. Shared Widgets begin/finish hooks only manage native widget gesture lifetime; no app policy was added there.

Seven new primitive regressions cover matching/latched release and clamping, three buttons/hover, hidden/focus cancellation, half-cell selection/primary/copy-on-select, Shift middle-read ownership, click timing plus actual double-click selection, and event-position wheel ownership. A real-widget regression sends text before and after a click in one frame and requires each byte sequence on its correct native input channel. Existing input/wheel/clipboard/IME tests remain. Mix core/model/actions/Bus/protected suites pass locally, including primary read owner/effect validation. Rust compilation, new native tests and actual mouse/selection captures are pending.

Next CBC term-pointer-order-0bffd1a7 at exact code above, term_gate --prepare. Read full saved log, fix failures, apply fmt/lock and repeat exact source. Then fetch verified artifact and drive native primary middle-paste, drag/word/line/copy-on-select/reporting/cancellation and matched captures against gold. Full input/IME/search/Prefs, Bus/topics/errors/enforcement/conformance and release acceptance remain open, including matched long-idle grant-pool behaviour noted above. NOT ACCEPTED. No Fable deviation, install or gold/desktop changes.

### Pointer gate compile repair

CBC term-pointer-order-0bffd1a7-a1 failed before tests with E0027: the new MouseWheel match omitted egui phase/source fields. Read the complete 71956-byte saved log in four chunks and applied its formatting patch. Code dd61895fb832f36b39769134f371da7072e00673 ignores those fields in the match and explicitly supplies Move/Unknown in the wheel regression fixture. No behaviour assertion was weakened. Seven Mix suites passed before the compile failure; the new pointer and ordered-focus tests have not run yet.

Next CBC term-pointer-fields-dd61895f runs term_gate --prepare on that exact code. Read its full saved log, repair any failures and apply returned diffs before an exact-source repeat. Native pointer/selection captures, remaining full functional/Bus/Prefs/conformance checks and the release visual gate remain open. NOT ACCEPTED; gold and desks remain untouched.

### Ordered focus surrender repair

CBC term-pointer-fields-dd61895f-a1 compiled and passed both bundle tests, 13 runtime/call/timer tests and seven Mix suites. Terminal results: 186 passed, one failed, five existing ignores. All seven new pointer controller tests passed, as did existing wheel, Tab, clipboard and live font view tests. Read the complete 30426-byte saved log in two chunks; no worker diff. The real-widget same-frame test failed because bytes preceding a click were lost: egui surrendered the old pane focus while registering its response from the frame-wide pointer state, before the ordered input loop ran.

Code f6c849a24dfffbd1caa7f22234804d1243b41e3a suppresses that automatic surrender only during terminal response registration, immediately restores the shared option, and handles outside/blocked pointer presses in event order. Existing before/after-click assertions remain; a new outside-click case requires preceding text to reach the old pane and subsequent text to reach neither pane. Warm-up explicitly asserts the original pane owns focus. No test expectation was relaxed.

Next CBC term-focus-order-f6c849a2 runs term_gate --prepare at that code. Its tests, any formatting return and exact repeat are pending. Then fetch verified bytes for native mouse/selection/primary/copy-on-select/reporting/cancellation captures. Full functional, Bus, Prefs, conformance and release visual acceptance remain open. NOT ACCEPTED; no Fable deviation or desk/gold change.

### Pointer gate and native matched selection witness

CBC term-focus-order-f6c849a2-a1 passed: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search checks, foreign gate and debug build. Both ordered-focus tests passed without weakened assertions. Complete 47979-byte saved log read in three chunks. Applied its formatting patch in 155d3c1ceb06b8f7613e257b6d99a34062492152; exact repeat pending.

Fetched and verified f6c849a2 debug executable SHA256 eaf0eae488d0a9dc9577bc1eacc39ed06dd54c6d4f0bda80648eb43445f605af. Private native.cVYXIK exercised gold then candidate on owned Bus/compositor/settings authority: actual OSC52 PRIMARY write followed by middle-button paste; native drag of TERM_PAIR_NATIVE and PRIMARY pasteback into its original real Mix shell; live terminal.mouse.copy_on_select false then true; true additionally copied to clipboard and menu-paste read back the selected text. Each app retained its process during live changes. All seven tracked processes stopped. Script, calls, snapshots, captures, cleanup and witness.json retained privately.

Three 900x560 capture pairs at scale 1, Studio Dark/Inter 400 14.667, terminal regular 14.667, CSD right captions/bar menus/row actions: baseline session, drag with copy-on-select false, drag with true. All full/header/tabs/terminal SSIM and RMSE distortions are zero (ImageMagick 7.1.2-35, zero means identical). Rectangles: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. Both baseline PNG hashes e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865; all four selection PNG hashes 78897e18d3b10893a22c29e98408adbe20c513df3b09ea5b2c089c24f2d60fb9. Inspected originals, montages and differences: no region differences. These are worker-formatted debug diagnostic witnesses, not release visual acceptance.

Earlier native.tlv09y failed on gold because the harness compared clipboard paste against a snapshot taken before an intervening PRIMARY paste and Ctrl-C output. The real clipboard text was present; corrected the assertion to compare immediately before/after that paste. Retained failed run, all five tracked processes stopped. No application change or expectation relaxation.

Next CBC term-pointer-exact-155d3c1c at exact 155d3c1ceb06b8f7613e257b6d99a34062492152. Require complete log/no worker diff, fetch matching verified bytes and repeat native proof. Remaining: native program mouse reporting/cancellation, word/line selection, IME/search/full Prefs, clipboard failure notices, complete Bus/topics/errors/enforcement/conformance, long-idle grant-pool parity and scoped release visual matrix. NOT ACCEPTED; gold and desks unchanged.

### Exact pointer witness; clipboard refusal and notice checkpoint

CBC term-pointer-exact-155d3c1c-a1 passed exact 155d3c1ceb06b8f7613e257b6d99a34062492152 with no worker diff: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, seven Mix suites, dialog/search, foreign gate and debug build. Complete 49451-byte saved log read in three chunks. Fetched and verified executable SHA256 a8a4d9a56a1fec7a2a9f3cee3e3f9e3c9efe46c09633f85c460e24044e66a345.

Private native.m9EJYm repeated actual gold/candidate middle PRIMARY paste, drag selection/PRIMARY pasteback and live copy-on-select false/true with clipboard readback on true. All seven tracked processes stopped. Three matched 900x560 capture pairs, same settings/regions as the preceding checkpoint: all full/header/tabs/terminal SSIM and RMSE distortions zero; baseline hash e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865, selection hash 78897e18d3b10893a22c29e98408adbe20c513df3b09ea5b2c089c24f2d60fb9 on both apps. Originals, montages and differences inspected; no differences. Private witness.json includes exact bytes/source/profile, region figures and cleanup. This attests debug bytes and these states only, not final release acceptance.

Code 050e935002a810c4b3804cdabcd9e8d7c7b1d86e restores gold clipboard completion policy in Mix: Empty is silent, TooLarge/TimedOut/Busy/NoFocus/Failed produce the pinned labels, read tokens retire once, successful paste clears the notice, stale or closed owners never retarget another pane, and refused copies do not end pending reads. Generic host forwards native write refusals over existing in-process platform completions; OSC52 query failures remain native and never ordinary paste. No broker or application policy added to Rust. Supplied platform text uses the same success/refusal policy. Mix publishes changed notices immediately.

Gold also renders clipboard refusal and unsaved zoom rows above the terminal. The port now composes those rows from existing scene columns and warning labels, card background and 8x3 margins, independently visible in gold order. Zoom label is exactly Zoom is not saved. No new native presentation widget was needed. Eight Mix suites pass locally, including new completion ownership/refusal regressions and frame notice visibility/labels. App lint passes with its five embedded host functions declared; plain standalone lint otherwise correctly reports those embedding-only names. Rust/scene integration and native notice captures pending. Source hashes and adaptations recorded in provenance.

Next CBC term-clipboard-notices-050e9350 runs term_gate --prepare at code above. Read complete saved log, apply any returned formatting and require an exact repeat; fetch verified bytes for native clipboard refusal/zoom notice and unchanged baseline captures. Then continue program mouse reporting/cancellation, word/line selection, native IME/search/full Prefs, complete Bus/topics/errors/enforcement/conformance, long-idle grant-pool parity and scoped release visual acceptance. NOT ACCEPTED; no Fable deviation, install or gold/desk changes.

### Native clipboard refusal notice witness

CBC term-clipboard-notices-050e9350-a1 passed: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, eight Mix suites, dialog/search, foreign gate and debug build. Complete 50419-byte saved log read in three chunks. Applied the returned host formatting in 7b97f07344d0cc1fffd44a093b470887069f7311. Exact repeat pending. Verified worker-formatted 050e9350 debug binary SHA256 1af77fecdee94bd015e8088846160089576086308ad672eee073b485f6b897d1.

Private native.W8jl8A passed actual gold/candidate NoFocus clipboard refusal and success clearing. An owned ced window takes keyboard focus, the real Mix shell emits an ordinary OSC52 clipboard write, and the terminal reports the exact gold notice. After closing ced, both apps are restored/focused through PID/generation-matched native compositor commands. A real focused clipboard write/menu paste clears the notice. Captured baseline, refusal and cleared states on unchanged term processes. All nine tracked processes stopped. Private script, request/reply logs, snapshots, captures, comparison.json files and witness.json retained.

All three 900x560 pairs are byte-identical. Full/header/tabs/terminal and refusal notice region SSIM/RMSE distortions are zero (ImageMagick 7.1.2-35; zero identical). Refusal rectangles: header 0,0,900,38; tabs 0,38,900,32; notice 0,70,900,24; terminal 0,94,900,466; full 0,0,900,560. Baseline/cleared use preceding checkpoint regions. Refusal PNG SHA256 on both apps 87f8f03b1d6be4b6580ec1bf95c63fc4d9aa7bb015ce53214a7db6fba789d794; baseline/cleared e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, montages and differences inspected: notice text, warning colour, card background, margins and grid displacement match; no region differences. Debug witness only.

Retained probe failures: native.7iLuOu and native.tYTO51 wrongly expected a notice for unfocused PRIMARY writes, which gold intentionally keeps silent; native.BmdeZd used window action focus, correctly refused UNSUPPORTED by gold. Corrected the probe to ordinary clipboard write and native compositor focus; no app expectation was weakened. Native.dwBKOc passed gold notice/clearing but candidate returned UNKNOWN_COMMAND for term.show. All six/six/six/nine tracked PIDs respectively stopped.

Concrete open Bus gap: gold term.show restores/focuses its own compositor window and returns shown plus target; candidate has no term.show command. This is required parity work, not a proposed deviation. Gold implementation is libs/citizen/src/lib.rs show: native wait, app-id and PID lookup, generation-bound restore, then raise/focus; no viewport-focus substitution. Port through existing Mix jobs/native Bus ownership and test matched refusals/results. Final native.W8jl8A uses the documented native compositor alternative for both apps and does not claim term.show passed.

Next CBC term-notices-exact-7b97f073 at exact 7b97f07344d0cc1fffd44a093b470887069f7311. Read complete saved log/require no worker diff, fetch verified bytes and repeat notice proof. Continue term.show/full Bus, native zoom-unsaved row, transfer-limit/busy/timeout cases, program mouse/word/line/cancellation, IME/search/full Prefs, enforcement/conformance/long-idle grant-pool cases and scoped release visual acceptance. NOT ACCEPTED. Gold and desks unchanged; no Fable question.

### Exact notice witness; Mix window activation checkpoint

CBC term-notices-exact-7b97f073-a1 passed exact 7b97f07344d0cc1fffd44a093b470887069f7311 with no worker diff: 188 terminal tests, five existing ignores, 13 runtime/call/timer checks, seven input checks, two bundle tests, eight Mix suites, dialog/search, foreign gate and debug build. Complete 49758-byte saved log read in three chunks. Fetched and verified exact debug SHA256 f1a5d9c12def5989ed5022b5f015df20769bd1f67366449634b0c1d3162177f6. Native.d1YCMr repeated actual gold/candidate clipboard refusal and successful paste clearing; all nine tracked processes stopped. Baseline/refusal/cleared 900x560 captures retain the preceding checkpoint hashes; all full/header/tabs/notice/terminal SSIM and RMSE distortions zero, with the same rectangles and ImageMagick distortion convention. Originals, montages and differences inspected: no differences. Private witness.json records exact source/bytes/profile and closure. Debug proof only.

Code bdf9de1a9bd17277b941375e9aa24749bedd4af7 implements term.show in Mix through the existing bounded Jobs/Calls facilities. Shared reusable window-show.mix follows pinned citizen::show_with: wait mapped (10s compositor budget), list windows, select the first matching real app-id and host PID, restore its exact id/generation, require minimized false, then raise/focus and require focused true. Returns the real shown/target response or gold ACTIVATION/show_failed refusal. Each native compositor call has the gold 15s bound. User body cannot supply compositor, app-id, pid or target. A held native reply is completed once from its matching job owner/id; cancelled receipts retire the waiting map, with bounded native work retaining ownership until completion/drop. No application window policy moved into Rust and no transport was added.

Shared launch code now resolves one actual window app-id for both the viewport and injected app_launch context, including service overrides and explicit app-id. Term uses its resolved compositor and actual host PID. The manifest declares term.show, its descriptor and existing-instance activation; launch callers allow the pinned gold twenty-second show budget. A second launch should activate rather than create another session, but native proof is pending. App-id override/service tests added to the CBC gate. Nine Mix suites pass locally. New show policy tests verify exact stage order/arguments, app-id plus PID ownership, generation preservation, missing windows, stale restore, unsuccessful focus, transport failure and native completion/refusal shape. CBC/runtime/native show and launch-repeat tests remain pending.

Next CBC term-window-show-bdf9de1a runs term_gate --prepare at code above, now including launch regressions and nine Mix suites. Read complete saved log, apply formatting/lock and exact-repeat if needed. Fetch verified bytes and run gold/candidate term.show from another focused app, minimized restore, stale/refused compositor replies, caller cancellation, --comp/--service targeting and second-launch PID/session invariance. Update the missing-show case only after actual native proof passes. Continue native zoom-unsaved row, transfer limits/busy/timeout, mouse reporting/word/line/cancellation, IME/terminal search/full live Prefs, full Bus/topics/errors/enforcement/conformance, long-idle grant-pool and scoped release visual acceptance. NOT ACCEPTED; no install, gold or desk changes and no Fable deviation.

### Worker-compatible activation test repair

CBC term-window-show-bdf9de1a-a1 failed before Rust checks: eight existing Mix suites passed, then the new show test hit a parse error at global declarations. Read the complete 2722-byte saved log; no worker diff. The worker-installed Mix predates the local global syntax used only by this test.

Code afe888d652bb97f95a24054330a3165ebad2d480 replaces that test-only global mutation with an owned mktemp fixture recording ordered compositor requests and remaining replies. The same success, ownership, exact stage ordering, stale-generation, refusal and completion assertions remain. Cleanup runs in finally. Local lint and the full show test pass. No application, host, transport or expected outcome changed; no worker software upgrade was attempted.

Next CBC term-show-fixture-afe888d6 runs term_gate --prepare at that exact code. Read the complete log; repair failures and apply returned changes, then require an exact repeat before native activation acceptance. Existing exact notice evidence remains valid at 7b97f073; new window-show/activation, runtime, native targeting/second-launch proof and all previously recorded full parity/release cases remain open. NOT ACCEPTED; gold/desks unchanged.

### Activation descriptor repair

CBC term-show-fixture-afe888d6-a1 passed all nine Mix suites, including the worker-compatible ordered show fixture, and compiled Viu. The real bundle-load test then refused the new show descriptor because CommandDescriptor requires args. Read the complete 9972-byte saved log; applied its launch/native-bus formatting return.

Code 27f1a82644f433424985bd30ab77e8a2d54fc72e adds the required empty args array for the no-argument term.show descriptor. Confirmed the exact descriptor schema in libraries/bus/src/protocol.rs. Local show tests and diff check pass; no assertion changed. Remaining gate stages and native activation checks have not passed yet.

Next CBC term-show-descriptor-27f1a826 runs term_gate --prepare at that exact code. Read complete log, repair failures/apply returned changes and require an exact repeat. Then fetch verified bytes and exercise term.show, minimized restore, service/compositor overrides, refusals/cancellation and second-launch PID/session invariance against gold. All earlier full parity/release open items remain. NOT ACCEPTED; no gold, install or desk changes.

### Activation launcher regression repair

CBC term-show-descriptor-27f1a826-a1 passed all nine Mix suites, both real bundle-load checks, thirteen runtime/call/timer checks and 188 terminal tests (five existing ignores). The launch suite then failed an incorrect new assertion: Monitor explicitly declares dev.mixos.monitor, so its service override must preserve that window app-id. Read the complete 35080-byte saved log in two chunks; no worker diff. No debug artifact was emitted by this failed gate.

Code 06ff3077e731c4bcafe3b67e7107cc1c3720e757 corrects the declared-id assertion and checks the service-derived default separately against the Term bundle before applying an explicit override. The production resolution policy is unchanged: explicit window identity wins, otherwise use mixviu plus the resolved service. This retains checks on the injected Mix launch context as well as native app-id resolution. Diff check passes; Rust tests remain CBC-only.

Next CBC term-show-launch-06ff3077 runs term_gate --prepare at that exact code. Read the complete saved log, repair any failures and apply returned changes, requiring an exact repeat when necessary. Then fetch verified bytes for native term.show/minimized restore, compositor/service targeting, refusal/cancellation and second-launch host/session invariance against gold. Earlier full input/Bus/Prefs/enforcement/conformance and final release visual acceptance items remain open. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

### Native show witness and repeated-launch inventory correction

CBC term-show-launch-06ff3077-a1 passed exact 06ff3077e731c4bcafe3b67e7107cc1c3720e757 without worker diff: nine Mix suites, two bundle checks, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher tests, seven input tests, dialog/search checks, foreign gate and debug build. Complete 52533-byte saved log read in three chunks. Verified debug binary SHA256 2eea193b53a169204c44302c0efcd390dfec220bd87d814ea974e241d0cdad99. Strict Mix/mixd dependency tiers remain clean.

Private native.uHavh6 passed real gold/candidate term.show with ignored body targeting and generation-bound restore/focus, then native minimize followed by term.show restored/focused the same window. Clipboard refusal and success clearing still pass. All nine tracked processes stopped. Three fresh matched 900x560 captures of baseline/refusal/cleared sessions are byte-identical. All full/header/tabs/terminal and refusal-notice SSIM/RMSE distortions are zero (ImageMagick 7.1.2-35, zero identical). Same rectangles and PNG hashes as the exact notice checkpoint: baseline/cleared e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865; refusal 87f8f03b1d6be4b6580ec1bf95c63fc4d9aa7bb015ce53214a7db6fba789d794. Originals, montages and differences inspected; no region differences. Private witness includes executed script, exact artifact receipt, calls, comparisons and closure. The script's inherited printed result mentioned second-launch invariance, but that block was removed before this run; witness explicitly corrects the label. No repeat-launch claim derives from this run. Debug witness only, not release acceptance.

Gold inventory correction: native.riTOpe disproved the preceding single-instance assumption. A repeated gold invocation stays alive as a separate window and reports serving as term-<pid> after the base registration is refused. Pinned apps/term/src/main.rs and apps/term/crates/term-core/src/bus.rs confirm that startup does not call show: it retries a refused base registration once using the PID suffix. A second refusal or a transport/authentication failure is not another fallback rung. This supersedes earlier notes proposing second-launch activation and unchanged parent/shell identity. Gold wins; no Fable deviation proposed.

Code 1891af2f3f11959937d19ee4be7fdf6d0a39a163 removes the incorrect term activation manifest entry and corrects provenance. term.show remains the independent window command. The generic host's registration-refusal collision-name path still needs implementing and native multi-window proof; removing activation alone does not complete that behaviour. Existing generic activation support remains available to apps whose gold contract uses it.

Next CBC term-show-gold-launch-1891af2f checks the corrected manifest at exact code above. Read complete log/no worker diff, fetch verified artifact for native repeat if needed, then implement gold's registration-refusal-only PID suffix ladder with command namespace/application identity preserved and real two-window/session/cleanup proof. Native show refusal/cancellation, zoom-unsaved notice, clipboard transfer limits/busy/timeout, reporting/word/line/cancellation, IME/grid search/full live Prefs, full Bus/topics/errors/enforcement/conformance, long-idle grant pool and final release visual acceptance remain open. NOT ACCEPTED; no install, gold or desk changes.

### Refused-registration PID fallback implementation checkpoint

CBC term-show-gold-launch-1891af2f-a1 passed exact 1891af2f3f11959937d19ee4be7fdf6d0a39a163 without worker diff. Complete 52534-byte saved log read in three chunks: nine Mix suites, two bundle checks, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher tests, seven input checks, dialog/search, foreign gate and debug build. Artifact exists at the worker's term-artifacts source directory; it has not been fetched or claimed as a new native witness.

Code 7b1599af6aabb552668afa85253cfa1f6133cad8 adds app-authored registration_pid_fallback, enabled only in Term's contract. The reusable native adapter attempts the base name and, only following an initial registration rejection, retries exactly once as base-PID. Both attempts have the pinned gold two-second deadline. Second refusal, transport, authentication and timeout errors preserve their own diagnostic and do not earn further names. The selected supervised client owns reconnects; no reconnect reruns this ladder. With the opt-in policy, failed initial Bus setup leaves the window alive as gold's graphics-only path; other apps retain their existing close-on-failure behaviour.

Wire command descriptors are renamed with the actual selected service, and incoming commands map that exact dot-delimited prefix back to the original Mix handler namespace. Initial bus.connected reports the selected service. This is native transport adaptation; Term tab/pane/actions remain Mix. Added CBC regressions for base success, one refusal, second refusal, disabled policy, non-registration failures and namespace dot boundaries. No native broker, external transport or daemon added. Diff check and Mix gate lint pass locally; Rust compilation and tests remain CBC-only.

Next CBC term-registration-7b1599af runs term_gate --prepare at this exact code, now including native_registration tests. Read complete saved log, repair failures/apply worker fmt, then exact repeat if changed. Fetch verified matching artifact and drive actual gold/candidate two-window registration collision: fallback HELP descriptors and snapshot/type/show/UI commands under term-PID; independent native parents/shells; close one without affecting the other; second refusal and unavailable broker graphics-only cases. Audit suffixed service identity in menus, macro/outbox/topics/full Bus replies before calling fallback parity complete. No new native acceptance is claimed at this checkpoint.

Earlier native.uHavh6 show/minimized restore and zero-difference captures attest only 06ff3077 debug bytes. Native show cancellation/refusals, complete global Bus surface/topics/errors/enforcement/conformance, clipboard transfer limits/busy/timeout, zoom-unsaved notice, mouse reporting/word/line/cancellation, IME/grid search/full live Prefs, long-idle grant pool and final release visual acceptance remain open. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

### Two-window native witness and selected-service routing repair

CBC term-registration-7b1599af-a1 passed nine Mix suites, two bundle checks, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher tests, three registration ladder/namespace tests, seven input tests, dialog/search, foreign gate and debug build. Read the complete 63293-byte saved log in four chunks. Applied its formatting-only return in c90ea9d72cbeb10480fd992ef2a6a63da8f889fb; exact repeat pending. Verified worker-formatted 7b1599af debug executable SHA256 1318939350a4f89be54886006f21bc1e941d8dad68151109b0b19c8fbd3134bb.

Private native.eHaAXB passed actual gold/candidate simultaneous-window checks: the first owns term, the second term-PID; suffixed HELP describes its snapshot command; distinct native parents and Mix shells are attached; each window executes the same pair() output; show matches each host PID and compositor generation; suffixed native UI capture works. Closing the second stops its host/shell and leaves the first host/shell unchanged and accepting new command output. All eleven tracked processes stopped. Both gold and candidate logged unconfirmed parent revoke and existing lease/window expiry on close; this witness proves process cleanup, not immediate broker revocation.

The first and second window pairs are both byte-identical 900x560 captures, Studio Dark/Inter400/14.667/scale1, existing deterministic real-Mix command. All full/header/tabs/terminal SSIM and RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Regions: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, full montages and differences inspected: no region differences. Private witness.json records artifact, executed script, native calls/discovery, comparisons and cleanup. Diagnostic debug evidence with returned formatting; not exact release acceptance.

The service-identity audit found static canonical names in native UI advertisement and Mix change-topic replies/publication. Code 08b17569a63561da033a34f50f2860d1bee07ff3 makes native UI use the actual supervised service, and Mix retain bus.connected's selected service for props.watch and change publication. Pure Mix regression checks suffix watch topics and exact namespace boundaries; bus-test and event-job lint pass locally. Code dce0310e08bef48e676149ea0a40a11581b070b8 also includes the actual service in every reconnect notice, preventing Mix from reverting its topic namespace after reconnect. These routing changes are not yet native-attested.

Next CBC term-registration-routing-dce0310e checks exact code above, including applied formatting. Read complete saved log, apply any returned changes and exact-repeat if needed. Private prepared fetch script and registration-dce0310e harness repeat two-window native cases plus each window's props.watch and titlebar service readbacks. Continue actual topic subscriptions/delivery, hosted menu execution, macro/outbox/service identity and whole HELP/app.describe/global Bus parity; real second-refusal/unavailable broker graphics-only/reconnect cases remain open. Do not duplicate earlier diagnostic artifact builds.

Remaining full parity/release items from prior checkpoints still bind: native show cancellation/refusals, clipboard limits/busy/timeout, zoom-unsaved row, mouse reporting/word/line/cancellation, IME/grid search/full live Prefs, full topics/errors/enforcement/conformance, long-idle grant pool and final scoped release visual matrix. NOT ACCEPTED; no install, gold or desk changes, no Fable deviation proposed.

### Clipboard fixture descriptor isolation repair

CBC term-registration-routing-dce0310e-a1 failed with rc101. Read the complete 32375-byte saved log in two chunks; no worker diff and no artifact emitted. Nine Mix suites (including selected-service topic regression), two bundle tests and thirteen runtime/call/timer checks passed. Terminal suite: 187 passed, five existing ignores, one failure in reads_are_one_at_a_time_bounded_and_time_out. Read id4 returned TimedOut instead of pasted-newline. Remaining gate stages did not run. This is a red test result, not a queue/admission failure.

Inspection found the test fixture creates libc::pipe without CLOEXEC, unlike the production smithay-client-toolkit 0.19.2 receive path, which uses rustix pipe_with(PipeFlags::CLOEXEC). The scoped suite concurrently launches real child processes; inheritance of a fixture writer can postpone EOF after its local owner closes it. That is an inferred cause of this observed intermittent failure; no production regression is established by this log.

Code c727969315cab9fb0819df7e1c38e0a5ea438c82 makes the fixture create both ends atomically with pipe2(O_CLOEXEC), matching the production pipe contract. Ordinary read, single-reader Busy, actual oversized pipe transfer and timeout assertions are unchanged, including their five-second budgets. No ignore, serialisation, retry loop or production timeout change. Diff check passes; Rust checks remain CBC-only.

Next CBC term-clipboard-fixture-c7279693 runs term_gate --prepare at exact code above. Read the complete log, repair actual failures/apply returned formatting, require an exact repeat if source changes, then fetch the artifact from the actual reported worker. Use the prepared registration-dce0310e harness with its artifact path/source updated to the new verified bytes: repeat both-window parent/shell/show/capture/type/close checks plus selected-service props.watch and titlebar readbacks. The older native.eHaAXB witness remains valid only for its worker-formatted 7b1599af diagnostic artifact. All full Bus/topics/menu/macros/reconnect/refusal/graphics-only/input/Prefs/enforcement/conformance and release acceptance open items remain. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

### Exact selected-service witness; authored discovery checkpoint

CBC term-clipboard-fixture-c7279693-a1 passed exact c727969315cab9fb0819df7e1c38e0a5ea438c82 with no worker diff. Complete 52912-byte saved log read in three chunks: nine Mix suites, two bundle tests, thirteen runtime/call/timer checks, 188 terminal tests (five existing ignores), three launcher and three registration tests, seven input tests, dialog/search, foreign gate and debug build. The clipboard pipe regression passes with unchanged deadlines. Verified exact debug SHA256 e03318875820f61252ef35c6bb299a4d13bdd385c5e627e96c1c44b0287647c9.

Private native.1tuEgl repeated two-window native parent/shell independence, actual suffixed snapshot/type/show/capture and independent close/survivor input on those exact bytes. Both windows report props.watch topics in their actual service namespace. All eleven tracked processes stopped. First/second matched 900x560 pairs are byte-identical, same settings/command/regions as native.eHaAXB: all full/header/tabs/terminal SSIM/RMSE distortions zero (ImageMagick 7.1.2-35; zero identical). All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, montages and differences inspected; no region differences. Private witness.json records exact bytes, calls, readbacks and closure. Debug witness only.

Retained failed probes native.eLacC0 and native.NtWP1z incorrectly treated ui.titlebar.form.service as a readback of drive's MenuService registration. The second window reports null/hosted=false in this CSD hosting form on BOTH gold and candidate; this is not a discovered gold defect or approved deviation. Corrected the probe to preserve the CSD readback and check routing through actual suffixed commands/watch topics. Seven/eleven tracked processes stopped respectively. Actual hosted menu execution and topic subscription delivery remain open.

The same oracle probe found gold app.describe rc0 with schema/app_id/commands, versus candidate UNKNOWN_COMMAND. Code dc9fbd01319e78f62f5c75c1cfd2934b2902b105 adds immutable pinned discovery data and Mix-owned handlers: full HELP (56 descriptors), lowercase help (core-only 36 descriptors), app.describe (53 command entries), INFO/info/term.info usage and the port's own actual instance/PID. Names are rewritten only at exact command namespace boundaries; descriptors preserve gold order, argument shapes, read-only flags and descriptions. INFO's canonical command occurrences follow the actual service. This is a discovery implementation checkpoint, not proof of every advertised command's behaviour.

The generic host gains an opt-in authored_discovery switch so the SDK does not pre-empt Mix's HELP reply. Term enables the existing shared native macro session for its advertised macro commands. Explicit window_app_id chooses dev.mixos.term for both viewport and injected launch context, including service overrides; the shared resolver retains legacy/default behaviour for other apps. Launcher regression now checks declared identity, fallback and explicit override separately. Ten local Mix suites pass, including discovery schema/namespace/identity checks. Actual bundle tests now invoke HELP/app.describe/INFO. No runtime PID/version was copied from gold; immutable contract data came from the pinned native oracle and INFO source literal. Provenance records sources and catalogue hash.

Next CBC term-discovery-dc9fbd01 runs term_gate --prepare at that exact code. Read complete log, repair failures/apply returned fmt/lock, then exact-repeat if needed. Fetch from the actual reported worker (prepared fetch script's worker must match), verify bytes/source/profile and run discovery-dc9fbd01 native harness. Compare complete HELP/help/app.describe replies on first and fallback services after canonicalising only service names; compare INFO/info/term.info layout/text while checking each process's own identity. Repeat two-window/watch/show/captures/cleanup. Full hosted menus/macros/outbox/topics, raw payload/refusal/enforcement/conformance, broker failure/reconnect and remaining input/Prefs/search/release acceptance items still bind. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

### Discovery reply test representation repair

CBC term-discovery-dc9fbd01-a1 failed rc101 after all ten Mix suites passed and Viu compiled. Read the complete 10697-byte saved log. The actual bundle loaded and HELP returned rc0; the new Rust assertion then called as_str on its already-decoded array. Confirmed Runtime's reply adapter parses valid JSON into typed Json before storing Completion.reply. This was a test representation mistake, not an application refusal. Later scoped checks and artifact build did not run.

Code 5c69b0becfa305b71e72f8bde890df757681808c checks HELP's typed array and app.describe's typed object directly, retaining rc, term.show descriptor/flag, schema/app-id and actual INFO PID assertions. Applied the returned app-host/test formatting in the same code checkpoint. No application policy, expected Bus payload, descriptor or timeout changed. Diff check passes.

Next CBC term-discovery-typed-5c69b0be runs term_gate --prepare at that exact source. Read complete log, repair failures/apply formatting and exact-repeat if needed. Fetch the reported worker's verified artifact; adapt the prepared discovery-dc9fbd01 harness/fetch to the tested SHA before native comparison. Match complete HELP/help/app.describe and INFO identity formatting on canonical and fallback services; repeat unchanged capture/watch/show/two-window cleanup. The exact c7279693 native witness remains the latest completed baseline; discovery and all previously recorded full parity/release items remain open. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

### Discovery module import repair

CBC term-discovery-typed-5c69b0be-a1 passed all ten Mix suites, the real bundle HELP/app.describe/INFO assertions, thirteen runtime/call/timer checks and 187 terminal tests. Read complete 32605-byte log in two chunks; no worker diff. One native owner/control integration test failed: its embedded Mix reducer imports the real Bus policy from a different program entrypoint, and discovery's read_file of bus-contract.json resolved from that entrypoint rather than the module. The reducer load fault caused the control request to refuse FORBIDDEN. No authority assertion was removed; the catalogue import must work in this reusable policy path. No artifact or later gate checks ran.

Code abfb41c50000d60fcad593f6de2f4f9a29ced51b converts immutable bus-contract.json into bus-contract.mix and imports its catalogue using relative require. data_encode preserved every value exactly (deep equality checked before removing the JSON form); the module now requires no entrypoint-relative file I/O. Provenance destination/hash updated. Local discovery and Bus suites plus catalogue lint pass. A private probe whose entrypoint and working directory are outside the app imports the full nested Bus policy/catalogue and passes. This fixes module use in both app and embedded native recipient paths without changing any authority, descriptor or diagnostic expectation.

Next CBC term-discovery-import-abfb41c5 checks term_gate --prepare at this exact source. Read complete saved log, repair failures/apply fmt and require exact repeat if changed. Then fetch verified matching bytes from the reported worker and update the prepared native discovery harness's artifact source. Compare canonical/fallback HELP/help/app.describe and INFO own-identity formatting, repeat two-window/watch/show/captures/cleanup. The latest exact native baseline is still c7279693; discovery native and all earlier full parity/release open items remain. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

### Exact native discovery witness; namespace and text-wire checkpoint

CBC term-discovery-import-abfb41c5-a1 passed exact abfb41c50000d60fcad593f6de2f4f9a29ced51b with no worker diff. Complete 53599-byte saved log read in three chunks: ten Mix suites, two real bundle tests including discovery, thirteen runtime/call/timer checks, 188 terminal tests (five retained ignores), three launcher and three registration tests, seven input tests, dialog/search, foreign gate and debug build. Verified debug binary SHA256 436c81936a0862d13ca5092887f27752cb2d572d6988843d4e96b8c623fbfad6.

Private native.lQTiN0 matched all twelve discovery comparisons: HELP/help/app.describe/INFO/info/term.info on both canonical and fallback services. Whole replies retain descriptor order, argument shapes, descriptions, flags, schema and app-id; only service prefixes and final live instance/PID values were normalised. Each reported PID independently matched its actual spawned host. All twelve per-command comparison files and raw native replies retained. Two-window independent parents/shells, suffixed snapshot/type/show/capture, watch topics and close/survivor input also passed; all eleven tracked processes stopped.

First/second real-session 900x560 pairs are byte-identical with matched settings and command output. Full/header/tabs/terminal SSIM/RMSE distortions zero, ImageMagick 7.1.2-35 zero-identical convention. Rectangles unchanged: full 0,0,900,560; header 0,0,900,38; tabs 0,38,900,32; terminal 0,70,900,490. All four PNG SHA256 values e31900b2f10ca55d792ca7556b5f19da05f151dcbb4ba000d334f9834a994865. Originals, montages and differences inspected; no region differences. Private witness.json records exact source/bytes, twelve comparisons, readbacks and cleanup. Debug acceptance of these cases only, not final release/full Bus acceptance.

Code e10885b2fa8cfea189f20b40a1c6c24e4e3a0eda adds app-authored command_namespace, text_replies and unknown_reply contract data. The reusable native adapter maps only the actual registered prefix into the authored namespace; another instance's prefix is refused before invoking its reducer. The same mapping covers explicit generic service overrides. Reserved generic host controls retain their route. Unknown reply shape comes from app data. Text replies emit original string bytes rather than extra JSON quotes, while structured maps/arrays retain JSON. Other apps retain defaults. This follows pinned core canonical_command/dispatch/plain peer.reply behaviour; no authority or replay gate relaxed.

Added native Bus regressions cover fallback/override namespace mapping, dot-boundary refusals, framework controls, exact multiline/quoted/Unicode text bytes, and preserved structured JSON/default encoding. The scoped gate now includes native_bus tests. Diff check and Mix gate lint pass locally. CBC/native namespace refusals and raw-wire acceptance are pending. Exact raw request-envelope size/parsing/duplicate-key behaviour remains open; no full wire parity claimed by this patch.

Next CBC term-wire-policy-e10885b2 runs term_gate --prepare at exact code above. Read full saved log, repair failures/apply fmt and exact-repeat if changed. Fetch verified bytes from the reported worker; prepared wire-e10885b2 harness repeats discovery and two-window captures and records unknown/foreign-prefix read and mutation refusals. Compare refusal bodies with gold, verify no foreign request mutates another instance, and attest raw encoding through native transport. Continue service overrides, macro/hosted-menu/outbox/topic delivery, broker refusal/graphics-only/reconnect, full payload/errors/enforcement/conformance, remaining input/Prefs/search and release visual acceptance. NOT ACCEPTED; no Fable deviation, install, gold or desk changes.

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
