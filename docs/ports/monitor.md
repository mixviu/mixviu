<!-- Inventory paths apps/monitor/src and apps/monitor/crates/vitals below refer to the historical MixOS oracle, not this checkout. Current Mix app: apps/monitor. -->
# Monitor parity inventory

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`,
`apps/monitor/src`, `apps/monitor/crates/vitals`, and the installed gold
Monitor. The acceptance procedure is [parity-method.md](parity-method.md).
Implementation and acceptance are recorded separately. Monitor is **ACCEPTED**
at the tested source below; `last.md` carries the handoff receipt.

| Gold capability | Implementation and checks |
| --- | --- |
| Shared window frame, menus, icon actions, options and status | Mix frame with native design metrics, menu renderer, captions, title, options and status widgets; identity-preserving content mount |
| Live Prefs | Gold session resolver and fonts; live mode, scheme, style, accent, density, roundness, fonts, scale, icon stroke, app overrides, daylight, tunables, framing/alignment and key bindings; authority loss/recovery |
| Processes | Native striped, resizable virtual table; Name, PID, User, CPU, Memory, Disk and Command line; gold widths, alignment and selected-row text colours |
| Query and selection | Six sort columns, reverse, name/command-line filter, tree ordering, selection and command-line copy; queries use monitord's unlimited-list contract rather than the former 2,000-row port cap |
| Sampling | Mix per-kind asynchronous jobs, node/ticket fences, one-second clock, refresh while paused, gold paused navigation and independent history/connection requests |
| Performance | CPU overview and logical-processor grids, Memory/swap, disks/filesystems, Network and TCP; Mix composes the gold regions and formats their readings |
| Charts | Reusable native time-series widget; 60 aligned samples, missing-data gaps, server seeding, legends, hover, pan/zoom/reset and native capacity/rate scales |
| TCP connections | Top 20 throughput order, known/unknown rates, listener backfill, IPv6 formatting, RTT/retransmits, native table proportions and tooltips |
| Nodes and errors | Peer discovery, This node/mesh chooser, node switch clears old readings, stale replies are discarded; Bus connection and daemon errors appear in gold's status layout |
| Process actions | End task, Kill and copy; local selected-row/dialog admission, kernel errors and the approved P1 identity fence below |
| Registry and overlays | All 45 gold actions, dynamic availability, exact modifiers, live rebinding, native menus/search and About/Shortcuts/macros overlays |
| ABP | Gold's 50 command descriptors plus P1 `monitor.signal`; gold HELP/describe, compact info/rows, edits, refusals, show/quit, native UI/window, macros and diagnostics |
| Native hosting | Generic `viu run`, single-instance activation, logic while hidden, compositor restore/focus, native menu/action-bar publication and completion topics |

Application layout, sampling, history ownership, queries, admission, state,
formatting and command handlers are Mix. Rust additions are reusable Viu
facilities: tables, dashboards/charts, native menus, process identities,
local owner calls and native Bus/drive/macros integration. Two existing gold
toolkit functions are exported for embedding frames (compact-menu toggle and
title painting); their implementations are unchanged.

## Bus and launch contract

Run `viu run apps/monitor [--node NAME] [--service NAME] [--comp NAME]
[--noded-url URL] [--debug[=LEVEL]]`. Common native flags are consumed by the
generic host; Monitor receives its node argument. Registration overrides keep
the canonical Monitor command, Prefs, macro and diagnostic identity. The host
also supplies help/version and environment/manifest defaults. No Monitor Rust
host is used. Gold's app ID is `dev.mixos.monitor`, service `monitor`, native window
title `Monitor`; the client header includes the watched host. A second launch
activates the existing instance. Framing, native input, capture, macro and
diagnostic commands are generic opt-in host facilities.

`monitor.info` reports the compact gold state; `monitor.rows` reports gold's
row schema, without adding command lines to that wire response. The full
command catalogue is app-authored data in `contract.json`; native facilities
append the original gold descriptors. Accepted command execution publishes
`monitor.action.executed`; macros publish `monitor.macros.finished`; hosted
chrome publishes `monitor.ui.titlebar.changed`. Native FIFO publication keeps
loss accounting, and macro completion preserves its boolean output-loss flag.
Models, widget intents, callbacks and owner operations stay in process.

## Fable P1: approved non-visual deviation

Fable ruled on 2026-11-07 that signals preserve gold's exact admission:
local node, no open dialog, selected row and positive PID; SIGTERM/SIGKILL
run as the app user via `kill(2)`, without confirmation or a UID fence.
Kernel EPERM/ESRCH are surfaced. The earlier inventory's claim that gold had
UID checks and confirmations was incorrect.

The sole approved addition is an invisible identity fence. Refresh records
`/proc/PID/stat` field 22; immediately before signalling the native primitive
re-reads it. Mismatch refuses with `pid reused`, refreshes the list and
returns an error without a prompt. `monitor.signal {pid, signal, start_time?}`
uses the supplied token when present, otherwise the selected process's
snapshot token. Decimal-string tokens preserve exact ticks. Mix owns
admission, snapshots, refresh and replies; the primitive owns stat parsing and
the guarded syscall. Tests signal disposable children only.

## Protected session processes

Monitor refuses session infrastructure with rc 10 and
`{error_code: "PROTECTED", message, recovery}` on direct signals, command
execution, widget intents and frame intents. Procfs identifies a main process
(ppid 1) whose cgroup ends in `<unit>@N.service`, where N comes from
`MIX_RUN=/run/mix/session/N` or `MIX_VT`. `apps@N.service` is excluded.
Process names and Bus registration do not grant or remove protection.
If neither environment value identifies the session, Monitor protects main
processes of all numeric session units and says that the session is unknown.
The protected set includes viud, noded, mixd services and the shell host when
they meet this procfs rule.

Known gap: the apps registry main process is also in `apps@N.service` and
is **not protected**, because launched apps share that cgroup and are also
reparented to PID 1. Ending the registry can terminate all launched apps,
including Monitor, and leave app launching unavailable. Follow-up: put each
launched app in its own sub-cgroup, then remove the `apps` exclusion so the
registry main can be protected without blocking individual apps.

For recovery, use the **host's hostd** `desk.restart {desk: <desk name>}`
command, as the top bar Restart action does. This restarts the whole desk
and ends its applications. The implementation is
[`services/hostd/src/desk.rs`](../../services/hostd/src/desk.rs);
hostd inside a container refuses desk power operations. Monitor never
performs this recovery itself.
For a non-container session, use `sudo systemctl restart <unit>@N.service`
from another VT, substituting the affected unit and session number (for
example, `viud@2.service`). The session unit templates live under
[`engines/viud/units`](../../engines/viud/units). Restarting a unit can end
its clients; save work first.

Disabled End task/Kill actions expose their reason; the status and End task
tooltip show it too. Credentials and ancestry are read again at admission;
temporary procfs failures fail closed with a retry reason and are not cached.
No confirmation override exists: intent origin is caller supplied, not
authenticated. The native start-time fence remains; pidfd signalling to close
the remaining identity race is deferred.

## Evidence and acceptance

### Tutorial corrections: 2026-11-13

Monitor now defines `reverse: true` as descending for all six sort columns,
with a down arrow; false is ascending with an up arrow. The initial CPU view
remains busiest first. Both Bus requests and header clicks use the same
transition: an omitted direction toggles the current column, or starts a new
column ascending. Tree order remains parent first, sorting roots and siblings.
The Mix query adapter translates this direction to monitord's unchanged
natural-order contract (CPU/Memory/Disk descending, Name/PID/User ascending).
The pinned gold view used `reverse: true` to show an up arrow for every column;
its numeric order was ascending. This correction deliberately changes that
public Monitor convention and fixes gold's inconsistent text-column arrows.

At Mark's request, signal admission supersedes the P1 gold admission above:
End task, Kill and `monitor.signal` require a selected local row, no dialog,
PID greater than 1, and a known matching effective UID unless Monitor runs as
root. Monitor itself and its ancestors are always protected. Credentials and
ancestry come from local procfs, independently of filters/tree order. Unknown
ownership fails closed. The native start-time fence and kernel refusal remain.
`policy-test.mix`, the embedded `monitor_policy` test and monitord's direction
matrix cover these corrections without sending signals.

Private receipts retain scripts, fixture hashes, native replies, paired PNGs,
whole/region figures, raw SSIM/RMSE, source/binary hashes and PID cleanup.
No private captures or machine configuration belong in this repository.

### Acceptance: 2026-11-07

Verdict: **ACCEPTED**, following [parity-method.md](parity-method.md).
Tested source: `5ab4207ab7a0c74aaf4b40592370a69b875e5eed` on
`lab/gold-parity`. The commit containing this acceptance record is receipt
only; it does not change the tested application, widgets or build recipe.
Gold source is the pinned revision above. Gold Monitor binary SHA256:
`59d016a2eadf7107e5b8ccbbe32c4cf006c9616b4a26316ade5c20ebeb3d93f9`.
The candidate is the CBC **release** binary, SHA256:
`3ba7412344bcd3c110164d762c0812359ba6fc2908f2943558383883a7bbb046`.

The release artifact and JSON hash receipt are saved at
`/build/es-artifacts/5ab4207ab7a0c74aaf4b40592370a69b875e5eed/`.
Use that source checkout with `viu run apps/monitor`; the reusable host accepts
`--service`, `--comp` and `--noded-url`, and preserves canonical app identity.
The native window title is the static `Monitor`; the client header remains
node-aware. Override, singleton activation, hidden-window restore and quit
were exercised against an isolated native broker/compositor.

The private receipt identifier is
`gold-parity-20261107/round5/5ab4207ab7a0c74aaf4b40592370a69b875e5eed`.
Its `index.json` indexes 12 passing runs and 55 comparison sets, including
additional region crops of the same pairs. It retains the release bytes,
complete CBC log, scripts, input hashes, copied bundle, native replies,
original PNGs, side-by-side/diff figures, inspection and cleanup receipts.
Every owned application, service, broker and disposable child exited.

#### Matched visual evidence

Both clients are 1040×680, scale 1, paused, using the same native fixtures,
Studio Dark, Inter 400 at 14.667, comfortable density, round controls, CSD,
right captions, bar menus and row actions. Processes uses CPU sort, no filter
or selection. Performance uses the same constant 60-sample histories and
20-row TCP fixture, including IPv6 and a listener. Explicit refresh while
paused is exercised. The pinned snapshot/process fixture SHA256 values are
`e027e9d09aa4647385b65a16ef7306e9d8b0d048c27b0d0573d3c9f1b82f6df2`
and `0ef528fe92e7aab5f3931adc73c9171239565ad341305bb6e78cbc988591fbb5`.

ImageMagick 7.1.2-35 reports normalised **distortion**, with zero identical.
For Processes and **each** of CPU, Memory, three disks, Network and populated
TCP, the measured figures are:

| Region | Rectangle x,y,w,h | SSIM distortion | RMSE distortion | Inspection |
| --- | --- | ---: | ---: | --- |
| Whole window | 0,0,1040,680 | 0 | 0 | Byte-identical PNG pair |
| Header/menu/actions/captions | 0,0,1040,38 | 0 | 0 | Exact |
| Options row | 0,38,1040,42 | 0 | 0 | Exact |
| Table or Performance content | 0,80,1040,570 | 0 | 0 | Exact |
| Status | 0,650,1040,30 | 0 | 0 | Exact |
| Performance resource rail | 0,80,258,570 | 0 | 0 | Exact on all seven pages |
| Performance dashboard | 258,80,782,570 | 0 | 0 | Exact on all seven pages |
| Processes command-line column | 713,80,327,570 | 0 | 0 | Exact |
| CPU overview chart | 266,150,758,150 | 0 | 0 | Exact |
| CPU logical-processor grid | 266,327,758,196 | 0 | 0 | Exact |

Originals, full pairs and perceptual figures were inspected, with detailed
region receipts retained. Processes PNG SHA256 on both sides is
`800c28506c24d9bc9c755c740d4cbca72486ce67aee48597e548461d12da19fc`;
populated TCP is
`19cab727a53c59d336c628c232798f96bc355cf44a53123b4535aa0a79766c5b`.
The 11 selected/tree/tab/menu/About/Shortcuts states are byte-identical and
all 45 action availability records match gold in every state. Both waiting
screens are exact. The compositor-hosted client and 1120×788 furniture pair
are exact too, including the static native title.

#### Live Prefs and remaining differences

Nineteen cumulative settingsd changes were captured on one unchanged PID per
app: baseline, light, accent, compact/square, fonts/scale/stroke, dark Pro,
SSD, left/centre CSD captions, live keymap, compact menu, app override/clear,
auto day/night, style tunables and three menu/action/title alignments.
Live F8 and Ctrl+P rebinding worked. Seventeen pairs are byte-identical.

| Remaining raster case | Region | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| Compact/square | Whole 1040×680 | 9.74852e-8 | 8.50963e-5 |
| Compact/square | Chrome 0,0,1040,120 | 5.52416e-7 | 2.02570e-4 |
| Fonts/scale | Whole 1040×680 | 7.93036e-8 | 8.14068e-5 |
| Fonts/scale | Chrome 0,0,1040,120 | 4.49387e-7 | 1.93787e-4 |

Full pairs and amplified RGB differences isolate these to antialiasing at
the node chevron and Close icon. Geometry, typography and colours match;
content (0,120,1040,500) and status (0,620,1040,60) both measure zero in
both cases. These raster edges meet the standing brief's antialiasing
allowance; no visual deviation is proposed. A separate live run passed
settings authority loss/restart/recovery and owner-close without app restart.

P1 above remains the sole Fable-approved non-visual deviation. There are no
open parity items or unanswered Fable questions.

#### Behaviour, ABP and build evidence

The release-byte native run matches **46/46** structured gold request/reply
cases, including malformed arguments, refusals and atomic theme changes.
The complete 51-command catalogue (50 gold plus P1 signal) and 45 actions
pass, together with native tree/input/menu/capture/window/debug commands,
macro mutation/run/stop/output, action event order and exactly one macro
completion. Separate witnesses pass native caption close, registration and
broker/compositor overrides, canonical Prefs/macros identity, second-launch
activation, node argument handling and quit. Per-kind asynchronous fetch,
paused navigation, TCP ordering, history gaps/seeding and stale-node/ticket
fences remain covered by the owning tests.

API SIGTERM/SIGKILL, native End task and action Kill each refused a stale
start token with `pid reused`, refreshed without a dialog and then signalled
only its disposable child. Real monitord sampling/resource navigation and
the same disposable-child signal path passed separately.

CBC job `20261107-053306-0497e9c3-a1` passed at the exact source above:
four Mix contract tests; rustfmt check; locked all-target check, tests and
clippy for `viu`/`viu-engine`; debug and release builds. The full 2,614-line
saved log was read. Rust results: 226 passed, zero failed, six existing
environment-dependent tests ignored. No test was weakened and the worker
returned no formatting/lock diff. Warnings remain visible in the saved log.
The installed-Mix pure 2,000-row projection took 42.5 ms; that measurement
does not claim embedded Viu event latency.

### Integration revalidation

The [desk4 integration receipt](desk4-integration.md) records the full 55-figure, 46/46 Bus and 19-state live Prefs revalidation at 9643f92, and the release-byte quick recheck after merging accepted Ced at `4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0`. The latter retains zero whole/region distortion for Processes and all seven Performance pages and matches 46/46 structured replies. Exact release hashes, worker paths, CBC jobs and private evidence identifiers are in that receipt. P1 and the previously accepted raster edges are unchanged; no new Monitor deviation is proposed.

## Four-app release integration at 1804bb9

Release-byte revalidation **PASS**. Exact four-engine hashes, complete native/visual results, all residual figures, deferred-reply witnesses, two same-worker timing comparisons and provenance debt are in [four-app-integration-acceptance.md](four-app-integration-acceptance.md). The runtime source is `1804bb98a5ab305111bd35412bf6f3902bc24013`; following commits are documentation receipts.
