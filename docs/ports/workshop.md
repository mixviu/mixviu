# Workshop port inventory

Oracle: MixOS 8779f3328 apps/workshop shell/view/workshop/mesh/actions and
registry/traffic/macros/agents tests. Workshop follows the Ced checkpoint.

| Original capability | Disposition |
|---|---|
| Node/service registry and command discovery/forms | Deferred until Ced checkpoint; Mix composition over native Bus |
| Traffic dock, filtering and output history | Deferred; requires bounded native subscriptions/readback |
| Macros create/edit/run and recording | Deferred; existing macro primitives are available, policy remains Mix |
| Mesh peer pages, SSH/WireGuard and agent views | Deferred backend workflows; no app-specific Rust authorised |
| Saved dock layout, actions/keymaps/theme and original wire compatibility | Deferred shared host integration |

Round 4 scope, before implementation: local membership from `noded.list`,
on-demand HELP/app.describe command discovery, service filtering, advertised
command selection and metadata, editable JSON body, format/clear, one explicit
call with the peer's return code and a bounded reply/history. All application
policy stays in Mix. The existing shared editor provides request text input.

Remote discovery, observation and macro/agent management remain deferred;
calling their advertised Bus commands is possible through the generic form,
but is not a port of their dedicated workflows. No acceptance is claimed yet.
