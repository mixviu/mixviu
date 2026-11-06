# Monitor port inventory

Oracle: MixOS `8779f3328`, `apps/monitor/src`, `crates/vitals`, original
README and `docs/monitor/README.md`. Source review includes actions, model,
query construction and UI documentation. This lab specimen has no parity claim.

| Original capability | Round 1 status |
|---|---|
| Native monitord local/mesh snapshot and processes | Mix implementation; unavailable errors explicit; no fabricated readings |
| Process name, PID, user, CPU share, memory, I/O columns | Mix projection to shared virtual tree-table; 2000-row work budget with explicit refusal |
| Sort all six columns, reverse, filter, tree order, selection | Existing monitord query contract from Mix; native table intents; acceptance pending |
| Refresh, pause, one-second collection | Mix handlers and generic host timer; acceptance pending |
| Remote node selection | Text field / `monitor.node`; invalid node names refuse |
| CPU, core count, used/total memory and swap | Summary from real snapshot |
| CPU history | Bounded 60-sample Mix ring and shared canvas; initially empty; no invented history |
| Performance page/resource chooser, 64-core grids, memory details | Deferred: summary exists, detailed composition not yet ported |
| Disk/filesystem, network and TCP charts / connection table / sockstat | Deferred: snapshot retained for inspection, no equivalent UI yet |
| Server-seeded history, gaps, interactive plot hover, series toggles, zoom/pan/reset | Deferred: requires reusable graph widget; current canvas is explicitly limited |
| Peer discovery chooser and offline/reconnect UX | Deferred: explicit node field works without discovery, errors retained |
| End task / kill / command-line copy | Deferred: original local-only UID/start-time signal admission and confirmations need porting before enabling mutations |
| Action catalogue, menu/shortcuts, dialogs, theme axes, macros | Deferred until shared host actions/session integration; no application policy moved to Rust |
| Original HELP/app.describe/info/rows/show/quit and ui/window wire contracts | Partial info/rows/ping and generic `viu.*`; other compatibility envelopes deferred |
| Single instance raises existing window, CLI flags, logging registry | Deferred: generic bundle host currently refuses duplicate Bus identity but has no raise relay |

Per-app Rust added: **0**. All sampling, query, row formatting, state,
composition and history policy live in Mix. No new bulk builtin is justified
without measured row cost. Run/capture and exact-source CBC gates remain pending.
