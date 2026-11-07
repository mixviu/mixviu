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
