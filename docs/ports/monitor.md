# Monitor port inventory

Oracle: MixOS `8779f3328`, `apps/monitor/src`, `crates/vitals`, original
README and `docs/monitor/README.md`. Source review includes actions, model,
query construction and UI documentation. This lab specimen has no parity claim.

Round-two source audit: the signal-policy description in the table below
conflicts with the pinned implementation. Gold's `Engine::can_signal` checks
local node, no open dialog and a selected row. Its shell validates a positive
PID and calls `kill(2)` with SIGTERM/SIGKILL; it has no UID/start-time identity
fence or confirmation. The earlier inventory instead requires those guards.
The Fable question and source evidence are recorded in `last.md`; neither
policy is claimed implemented or accepted by the Mix app yet.

| Original capability | Round 1 status |
|---|---|
| Native monitord local/mesh snapshot and processes | Ported in Mix; native read/capture verified; unavailable errors explicit; no fabricated readings |
| Process name, PID, user, CPU share, memory, I/O columns | Ported Mix projection to shared virtual tree-table; 2000-row work budget with explicit refusal |
| Sort all six columns, reverse, filter, tree order, selection | Ported existing monitord query contract from Mix; native input and semantic checks in headless harness |
| Refresh, pause, one-second collection | Ported Mix handlers and generic host timer; real native reads captured |
| Remote node selection | Ported text field / `monitor.node`; invalid node names refuse |
| CPU, core count, used/total memory and swap | Ported summary from real snapshot |
| CPU history | Ported bounded 60-sample Mix ring and shared canvas; initially empty; no invented history |
| Performance page/resource chooser, 64-core grids, memory details | Deferred: summary exists, detailed composition not yet ported |
| Disk/filesystem, network and TCP charts / connection table / sockstat | Deferred: snapshot retained for inspection, no equivalent UI yet |
| Server-seeded history, gaps, interactive plot hover, series toggles, zoom/pan/reset | Deferred: requires reusable graph widget; current canvas is explicitly limited |
| Peer discovery chooser and offline/reconnect UX | Deferred: explicit node field works without discovery, errors retained |
| End task / kill / command-line column and copy | Deferred: command-line projection/copy still needs wiring; original local-only UID/start-time signal admission and confirmations must be ported before enabling mutations |
| Uptime, load/task counts and CPU frequency/per-core detail | Deferred: present in the retained snapshot; detailed Mix projection not yet authored |
| Action catalogue, menu/shortcuts, dialogs, theme axes, macros | Deferred until shared host actions/session integration; no application policy moved to Rust |
| Original HELP/app.describe/info/rows/show/quit and ui/window wire contracts | Ported info/rows/ping and generic `viu.*`; other compatibility envelopes deferred |
| Single instance raises existing window, CLI flags, logging registry | Deferred: generic bundle host currently refuses duplicate Bus identity but has no raise relay |

Per-app Rust added: **0**. All sampling, query, row formatting, state,
composition and history policy live in Mix. No new bulk builtin is justified
without measured row cost. Headless captures inspected against the original. Detailed parity is explicitly deferred; final exact-source gate is recorded in last.md.
