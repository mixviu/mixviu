# Workshop parity — implementation in progress

The current parity job uses pinned gold
`8779f33284b7d3f1178c3d57a699efb9b0867b22`. Gold wins where the earlier inventory
disagrees. The bounded specimen described below is historical, not the current
behaviour or an acceptance claim.

| Gold capability | Current parity work |
| --- | --- |
| Shared frame, action registry, menus, keymaps, About/Shortcuts and per-window themes | Wired; native and visual matrix pending |
| Nodes, identities, remote routes, description fallback and progress | Implemented with separate local/remote read lanes, 64 descriptions per node, queued/phase deadlines, stale generation checks and last-good retention |
| Command body, Format, Clear, Call and reply | Gold 65,536-byte body bound, 30-second raw native call, deferred outcome, no retry, return code/error/body preservation and 1,000,000-byte UTF-8 reply bound |
| Traffic observation and dock | Bounded/redacted ring, correlation, loss, pause/filter, Auto/Bottom/Right and native table/card presentation implemented; native matrix pending |
| Macro manager | Storage, running-app routing, groups, creation, source/drafts, save-before-run, run/stop/copy/move/delete confirmation, target/destination and Ced/Dopus effects implemented; shared editor integration and native matrix pending |
| Mesh | Bounded totals, time windows, probes and observer replacement/stale fencing implemented; radial canvas and inspector composed with reusable Viu controls; native matrix pending |
| Agents | Node/session lifecycle, trusted MCP refusal, transcript catch-up/follow, configuration, forms/composer and permission-card policy implemented; native matrix pending |
| Saved arrangement, complete Bus equivalence, single-instance launch and live Prefs | Completion and acceptance work active |
| Macro recording/generated forms, SSH/WireGuard pages | Earlier roadmap inventory, absent from pinned gold; not a parity requirement |

W1: Format pretty-prints JSON exactly as gold; reply display is bounded at
1,000,000 UTF-8 bytes. No proposed deviation is pending. Intra-app traffic stays
in process. Application policy is Mix; native changes are reusable facilities
and widgets, with no application reducers in Rust.

CBC `20261107-080757-7c328b78-a1` passed at `b4b4488581258996f484d97e3926acd1b6f0d56d`,
including embedded Workshop policy tests, launch tests, native bridge tests and
72 scene tests. The full log was read and its formatting patch applied. The
debug artefact is exploratory. Exact release, full native, live Prefs and visual
receipts remain pending. Overall **NOT ACCEPTED**.

## Historical bounded specimen (2026-11-06)

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`,
`apps/workshop`: inspector model/bus/engine/workshop/mesh/agents, application
actions/view/shell/saved and discovery/drive/mesh/macros/registry tests.
The round-3 inventory preceded implementation; these are round-4 dispositions.
This is a bounded local Bus browser, not full Workshop parity.

| Original capability | Disposition |
|---|---|
| Local membership and Refresh | Ported via noded.list; at most 512 services, sorted/deduplicated; malformed/oversized results refuse, never silently omit entries |
| Node tree, identities and remote routed discovery | Deferred: local selector replaces tree; per-node generations, progress, identity badges and parallel capacity policy need Mix composition |
| HELP with app.describe fallback | Ported on demand for selected service; string/object entries, array/commands envelopes; at most 512 commands/256 KiB description, names deduplicated |
| Arguments/descriptions and service search | Ported bounded metadata and case-insensitive local service filter; filtering leaves chosen target intact |
| Read-only declaration badge | Deferred metadata presentation; target admission stays authoritative for every call |
| Advertised command selection | Ported: command must belong to selected service's catalogue; changing service clears command before discovery |
| JSON body editing, clear and format | Ported shared editor, 65,536 UTF-8 bytes; blank body or valid JSON; W1 corrected: Format pretty-prints JSON and preserves the body if expansion exceeds 65,536 bytes |
| Explicit command call and reply | Ported one native call, 8-second deadline, body/selection held while busy; peer rc/structured refusal retained; transport uncertainty never auto-retried |
| Reply display bounds | W1 corrected in round two: gold headers and JSON pretty-printing, raw non-JSON text, bounded at 1,000,000 UTF-8 bytes with gold marker; native acceptance pending |
| Own call history | Ported last 12 target/command/outcome metadata records in memory; visible history bounded to 900 scalars; full retained metadata via info |
| Reply clipboard copy | Deferred shared clipboard selection/ownership contract |
| Live Traffic dock, pause/filter, redaction, drop reporting and request/reply correlation | Deferred: observation ownership, loss and redaction-before-retention must be ported together; own history is not broker observation |
| Traffic dock Auto/Bottom/Right, resizing and saved layout | Deferred with observation dock; shared dock/split controls already exist for later Mix composition |
| Macro groups/catalogues, new/title/save/run/stop/copy/move/remove and Ced/Dopus open | Deferred dedicated workflow; generic calls to advertised macro commands are not a port of its manager |
| Macro recording, generated command forms and Bus-message documents | Deferred original roadmap features; no new implementation claim |
| Mesh map, peer traffic totals/top commands, ping, age/loss indicators | Deferred original topology/observation policy in Mix |
| SSH/WireGuard pages | Deferred original roadmap work; no new implementation claim |
| Agent sessions across nodes, transcript catch-up/follow, new/prompt/cancel/close/resume/set/answer | Deferred: preserve session ownership, pagination/gaps and lifecycle semantics together |
| Action registry/menu/shortcuts/keymaps/Help/About | Deferred shared action integration; visible Refresh/Format/Clear/Call implement this specimen's path |
| Settings-follow theme and per-window appearance | Deferred generic host integration; bundle reads no private settings/files |
| Single-instance activation, original workshop.v1/drive compatibility | Deferred full compatibility; bounded surface documented below; generic viu.* handles native window input/capture/quit |

## Runtime and placement

Run `viu run apps/workshop`, click Refresh, select a service, wait for HELP,
select a command, edit JSON and Call once. No argument starts a call.

`workshop.refresh` starts membership discovery. `workshop.select {service}`
starts command discovery; wait for `workshop.info.busy == false` before
`workshop.select {command}`. `workshop.body {text}`, `workshop.filter {text}`
and `workshop.call {}` share UI policy. Call answers an acceptance receipt;
eventual peer outcome is in info/visible reply. Target validation remains
authoritative. Failed calls never retry automatically. No durable queue or
history is introduced. Discovery uses bounded serial jobs (3-second calls).

All policy is Mix. Per-app Rust added: **0 lines**. Shared native extension:
editor `properties.rows` (integer 1..100, default 24), so any app can size its
text buffer. No text limit, existing control admission, dependency or embedded
interpreter version changes.

## Verification status

CBC job `20261106-223216-699f3e94-a1` passed 111 tests, fmt-check,
all-target check, clippy and the generic host build at source
`e330c14a5c7725804ef62614badd7c8b08dd1b69`. The full saved log was read;
worker diff empty. Existing clippy warnings remain.

Private broker/nested compositor verification passed eight operation records:
native editor typing and Call, structured peer rc 17, invalid JSON/unadvertised
command/oversized Unicode refusal before dispatch, bounded reply/history,
responsive ping and held request while busy, timeout uncertainty with no retry,
offline call and the original Workshop sending the same Unicode echo request.
Five captures were inspected: echo, refusal, truncated reply, history/filter
and original. The new layout is visibly simpler; no pixel or feature parity
claim follows. The bounded emoji fixture exposes the current font's missing
glyph fallback; Unicode bytes survive the request/reply path.

The final CBC binary hash matches the native witness. All seven owned harness
processes were confirmed stopped. Captures and receipts stay in the private
hub. Shared production Rust is +7/-2 lines; shared tests +121/-1 lines.
No dependency, lockfile, engine or embedded Mix source changed. Email remains
inventory-only at this checkpoint. Performance comparison is unmeasured.


## Acceptance attempt — 2026-11-07, round 4

Overall verdict: **OPEN**. This record distinguishes completed contract checks
from remaining integration and visual work; no unapproved difference is accepted.

The oracle is MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, executable
SHA256 `21303df6158fbc7c856aa39525a9bacff31a258f05d03f1b6602de6aeb8e52e6`.
The verified release code is `cc43ae6825511b907db43f932ba12b349fc479d4`.
The later Traffic repair and its focused gate are identified below. Captures, scripts, fixtures, calls,
metrics, original PNGs, comparison figures and closure receipts are retained in
the private `workshop-parity-20261107` archive.

The isolated recipe uses owned native noded/settingsd/compd, separate runtime
and application state, canonical `workshop` sequentially, 1040×680 client
pixels at scale 1, Studio Dark, CSD/right captions, bar menus and row actions.
Baseline typography is Inter 400 at 14.667. No live desktop settings, VT or
installed executable changed. Every native run retains its failed evidence
and a `finally` cleanup receipt.

### Functional and Bus evidence

`release-bus.Qvag7k` matched **99/99** structured cases against gold, including
complete HELP/descriptors, payload/null/type refusals, Format, raw return codes,
errors and whitespace, Unicode byte bounds, observation provenance and quit.
Generated IDs, sequence numbers, timestamps, elapsed time and wire sizes remain
in the raw receipts and are normalised only for comparison. Filtered Traffic
retains payloads, outcomes and request/reply correlation.

`release-macro.7TtyCS` matched **25/25** macro protocol cases. The independent
target is the pinned native Dopus, running on the owned broker. Checks cover
global and app builtin copy, user-tier shadowing, moving to a running app,
execution and exact UTF-8 output, stop/exit 143, offline catalogue/get/put,
move back to global, removal and rc10 UNAVAILABLE for offline execution.
Only private root path names and run-start timestamps are normalised.
Duplicate headless registration now exits 1 without a texture-delta panic;
the original primary PID survives. CLI malformed arguments exit 1.

Native panel evidence includes form selection/typing/Start, nine identical
agent lifecycle/configuration/permission backend calls, automated permission
refusal without dispatch and rc17 USE_AGENT for an attested MCP caller.
Opaque tool JSON retains numeric u64 digits and numeric-looking strings.
The native clipboard contains identical reply bytes. Menu Clear and
Ctrl+Shift+F dispatch the gold operations. In-process policy remains in Mix.

The 2,024-row installed-Mix benchmark records 27.7–28.4 ms tree processing and
80.1–81.3 ms full projection. This is pure projection timing, not embedded Viu
end-to-end latency. The seed scene hundred-row timing assertion is unchanged.

### Persistence and remaining integration

Native restart checks preserve selected command and expanded parents,
collapsed Details, panel shares, Pause, per-window theme and native placement.
Bodies and replies remain passive. Selection visibility takes priority over a
saved raw scroll position, using the gold forget-scroll operation. A separate
seeded-offset probe returned zero on gold; it is retained as diagnostic evidence
and does not establish a port difference.

The accepted shared native_document editor and New macro field support have
not been merged into this branch. The standing job assigns checkpoint merges
to Opus. The consumer and editor height reservation are present, but complete
macro UI/New-dialog and Ced/Dopus window effects remain open until integration
and native repetition. No duplicate editor was introduced.

### Visual and live Prefs evidence

Debug pair `pair-debug.mcrgH4` has whole-image and all-region SSIM/RMSE
distortion 0 for Command, About, Shortcuts and Agents. Mesh static details,
header/tree/Traffic/status are also 0. Its scrollbar follows truthful live
counter rows below the viewport (S2): details SSIM distortion 0.000285207,
whole 0.000157695. No gold readings were fabricated.

Native menu recapture after repair has exact File/Edit/View/Bus/Macros images.
The Help difference in the earlier capture is the blinking search caret;
the release recipe resets its input phase before capturing.

Live Prefs variants cover mode, scheme, style, accent, density, roundness,
font family/weight/size, mono family, text scale, stroke, application overrides,
auto daylight state and tune.title_control_spacing. Accepted revisions,
unchanged PIDs, invalid refusal and authority loss/recovery are recorded.
Release repetition and inspected region metrics are appended below.

Metrics use ImageMagick 7.1.2-35 **distortion**, lower is better and 0 means
identical. Regions are header [0,0,1040,38], tree [0,38,354,570], details
[354,38,686,570], Traffic [0,608,1040,42], status [0,650,1040,30]; Mesh static
details [354,38,670,570] exclude its data-dependent scrollbar. No arbitrary
numeric acceptance threshold is used.

### Exact release checkpoint

CBC `20261107-105013-9650fd5f-a1` passed at `cc43ae6`: fmt, all-target
check, tests, clippy, debug and release builds, with no worker diff. Its full
saved log was read: `20261107-105101-cbc3-cc43ae682551-20261107-105013-9650fd5f-a1-924dfcdb.log`
in the private CBC log directory. Release SHA256 is
`c696f723eb33d5517c6de48941c36147ecec12b34755c8d0950238c9a2b461b5`;
the source/profile/hash receipt under `/build/es-artifacts/<sha>/viu.json`
was fetched and verified before these runs.

| Release evidence | Result |
| --- | --- |
| release-bus.Qvag7k | 99/99 structured comparisons match |
| release-macro.7TtyCS | 25/25 macro comparisons match, including execution/cancellation/offline |
| release-pair.nCJfXw | Command/About/Shortcuts/Agents: whole and every region 0; Mesh static regions 0, S2 scrollbar as above |
| release-native.WX8KZr | File/Edit/View/Bus/Macros: whole and every region 0; native clipboard, menu Clear, shortcut, Pause and placement callbacks pass |
| release-help.6MpXkw | Gold/candidate phase 4 hashes match; the earlier Help difference is only caret phase |
| release-panels.rTqIoa | New/session/expanded u64 tool/refusal: whole and every region 0; windowed activation keeps the existing PID |
| release-prefs.OkkQg5 | Compact diagnostic absent; seven of nine captured states are exact in every region; invalid input and authority restart checks pass on unchanged PIDs |

The two live Prefs raster cases were inspected on release originals, montage,
diffs and native UI trees. Font-scale details SSIM/RMSE distortion is
0.0000456998/0.000657549 (full 0.000025268/0.000488941); the Clear body edge
is at 682.5 versus 682.46875 logical pixels. Auto overrides header distortion
is 0.00172234/0.0164557 (full 0.0000962486/0.00389003); the affected icon
rectangle differs by 1/32 pixel and caption height by 0.00625 pixel.
Other regions are exact. These are fractional raster-edge differences within
the method's anti-aliasing allowance, not omitted controls or an arbitrary
score waiver. Compact, base, auto-night, both caption sides and authority
absence/recovery are exact. No C1 deviation is claimed.

All six principal release suites confirmed all 44 owned PIDs stopped; the
additional Help run has its own cleanup receipt. Final scripts are copied into
their run directories. The agent fixture SHA256 is
`cc0b5f74c401dd7df96a5bed93a8568a9ca53edc66b126a2f8355152e344b211`;
the Bus fixture is `118fed3aec43fa0202a2db11e2575667ca0ac33fe2de5abdffbd3c56d397185a`.

### Traffic repair checkpoint

The expanded-bottom capture exposed a sizing error: its fraction used content
height instead of gold's full-window height. The reusable `reference_extent`
placement input now supplies the full height, while narrow windows retain the
gold available-height rule. The right-docked capture also exposed the frame
clipping a native panel's separator paint margin. An explicit root dock
`clip:false` lets native panels apply their own clipping; other applications
retain the existing frame clipping. Gold's cosmetic divider endpoint is copied.

Tested repair source: `5484446d1a990f81bc4367260aae6948e79cd42c`.
CBC `20261107-113242-1ad7b79e` passed with no worker diff: Workshop and
ports_round4 tests, native Bus and launch unit tests, all 72 scene tests,
format check and the Viu debug build. The unchanged hundred-row timing test
passed. The saved complete log was read:
`20261107-113242-cachyos-5484446d1a99-20261107-113242-1ad7b79e-91ef16f5.log`.
This used the documented one-app visual-check light override, with six jobs,
nice 15 and idle I/O in the CBC shadow checkout. The exact worker command was
`mix light-visual-check.mix`, which invokes `mix apps/workshop/gate.mix` and
`cargo build --locked -p viu-engine --bin viu`; the executed helper is retained
in the private archive. Earlier compile failures are retained, not counted as
passes. The seed timing assertion was never relaxed.

The verified **debug** executable SHA256 is
`1fc5f449c10aff7764d30060c53bd4784aa301d60defb8f6d6aed5736f0f5bf9`.
It does not attest new release bytes. `repair-native-margin.1y7BRh` has
whole-image and every-region SSIM/RMSE distortion **0** for both
`traffic-right-empty` and `traffic-bottom-empty`. Originals, montages and
diffs were inspected. `repair-pair-margin.JrPbFw` repeats all five principal
states: Command/About/Shortcuts/Agents remain 0 throughout; Mesh retains only
the same S2 scrollbar difference. All 14 owned PIDs were confirmed stopped.

Capture names are `gold-<state>.png` and `candidate-<state>.png`; readback
receipts use the same stems with `.json`. Comparison sets are
`figures-final-<state>/comparison.json`, with `<region>-gold.png`,
`<region>-mix.png`, `<region>-side-by-side.png` and `<region>-diff.png`.
The private `round4-evidence-index.json` indexes eleven release/repair runs,
all generated comparison receipts and cleanup records. Each final run keeps
its executed `harness.mix`; prior failures remain separate.

Overall acceptance remains **OPEN**. Opus owns the checkpoint merge of the
accepted shared native_document editor and New macro field support. After
that merge, macro editor/New-dialog and Ced/Dopus window effects need native
repetition, followed by the full exact-source release gate and native release
verification including both Traffic repairs. The release Bus, macro storage
and execution, repaired menus, live Prefs, compact diagnostic and two raster
edge-case checks above are complete. No proposed deviation or C1 waiver is
pending; the missing integrated capability is not labelled accepted.

## Integrated shared editor checkpoint

Merged into the desk4 integration branch after four-app runtime 1804bb9. The macro-source editor uses the accepted shared native_document editor. New macro uses the reusable gold dialog title field presentation, leaving Ced path dialogs unchanged. The generic host now bridges native numeric deferrals through the existing Calls receipts rather than replacing the accepted apps' deadline/cancellation path. Workshop declares its job and reply-envelope limits in Mix bundle configuration. Integrated release-byte acceptance remains pending.

The integrated native editor witness now passes New macro, Unicode edit, undo/redo and Save. The Ced effect uses the accepted editor bundle and its native ced.open Bus contract; the file is opened once after read-only readiness probes. The Dopus effect resolves the installed application through the active prefix, matching gold. Dopus generic-host migration remains a separate open port, not part of Workshop acceptance. Full release-byte repetition is still pending.
