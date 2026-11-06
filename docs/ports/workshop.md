# Workshop port inventory

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`,
`apps/workshop`: inspector model/bus/engine/workshop/mesh/agents, application
actions/view/shell/saved and discovery/drive/mesh/macros/registry tests.
The round-3 inventory preceded implementation; these are round-4 dispositions.
This is a bounded local Bus browser, not full Workshop parity.

| Original capability | Disposition |
|---|---|
| Local membership and Refresh | Ported via noded.list; at most 512 services, sorted/deduplicated; malformed/oversized results refuse, never silently omit entries |
| Node tree, identities and remote routed discovery | Deferred: local selector replaces tree; per-node generations, progress, identity badges and parallel capacity policy need Mix composition |
| HELP with app.describe fallback | Ported on demand for selected service; string/object entries, array/commands envelopes; at most 512 commands, names deduplicated |
| Arguments/descriptions and service search | Ported bounded metadata and case-insensitive local service filter; filtering leaves chosen target intact |
| Advertised command selection | Ported: command must belong to selected service's catalogue; changing service clears command before discovery |
| JSON body editing, clear and format | Ported shared editor, 65,536 UTF-8 bytes; blank body or valid JSON; format emits compact JSON rather than original indentation |
| Explicit command call and reply | Ported one native call, 8-second deadline, body/selection held while busy; peer rc/structured refusal retained; transport uncertainty never auto-retried |
| Reply display bounds | Ported smaller budget: 900 Unicode scalars with explicit truncation; parsed JSON is serialised again, raw whitespace/error headers are not preserved |
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

CBC job `20261106-221954-40c3a368-a1` passed 111 tests and the scoped full gate
at source `087b891e47f15fc65ef249d71267db7903acd3ae`. Private native comparison
is pending. No parity or performance acceptance is claimed.
