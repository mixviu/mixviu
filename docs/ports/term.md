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
