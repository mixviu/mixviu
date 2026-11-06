# Monitor parity inventory

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`,
`apps/monitor/src`, `apps/monitor/crates/vitals`, and the installed gold
Monitor. The acceptance procedure is [parity-method.md](parity-method.md).
Implementation and acceptance are recorded separately: the current acceptance
receipt is in `last.md`.

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

Run `viu run apps/monitor [--node NAME]`. The generic host's environment and
manifest select its native Bus/compositor connection; no Monitor Rust host is
used. Gold's app ID is `dev.mixos.monitor`, service `monitor`, native window
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

## Evidence and acceptance

Private receipts retain scripts, fixture hashes, native replies, paired PNGs,
whole/region figures, raw SSIM/RMSE, source/binary hashes and PID cleanup.
No private captures or machine configuration belong in this repository.

Baseline 1040×680 Processes and all seven Performance sections have
byte-identical gold/Mix witnesses, including populated TCP. Additional
matched witnesses cover action states, menus, dialogs, SSD/hosted furniture
and live Prefs. These individual results do not replace the final exact-source
CBC gate and acceptance record in `last.md`. No visual deviation has been
proposed or approved; P1 is the sole approved non-visual deviation.