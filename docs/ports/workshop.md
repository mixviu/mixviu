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
