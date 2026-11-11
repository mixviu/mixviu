# mix-mcp

MCP stdio bridge to the MixViu Bus. See the [manual](../../docs/mix-mcp/README.md)
for connection settings and the complete tool list.

Connections require verified Unix ingress whenever a socket candidate exists.
Only `paths::env::MCP_ALLOW_TCP set to 1` permits plain TCP when all Unix candidates are
missing; existing socket verification failures remain fatal.

Application tools are `app_describe`, `app_actions` and `app_execute`.
`app_actions {service}` lists the application's registry rows;
`app_execute {service, id, params?}` sends the command id and optional
parameter object to the application's execute command.

`clipboard_get {selection?, mime?, primary?, offset?}` reads the local `clipd`.
`clipboard_set {text, selection?, mime?, primary?, sensitive?}` writes UTF-8 text.
See the [frozen clipboard manual](https://github.com/markc/mixos/blob/8779f33284b7d3f1178c3d57a699efb9b0867b22/docs/clipd/README.md) for size limits,
binary encoding and explicit cross-session push.

The skills loop is retired. At deployment, remove `trust_weight_graduated`,
`trust_weight_skill` and any old `skills`/`llm` sections from `knowledge.conf.mix`,
and remove the unused `skills.conf.mix` and `llm.conf.mix` files from the MCP
config directory (`paths::env::ETC`, normally `<etc-root>`). The config loader
tolerates unknown keys, but they have no effect; no aliases are provided.
Generated knowledge defaults omit them. Existing configs are not rewritten.

`knowledge_digest {}` returns global statistics. It refuses a non-null `domain`
argument because indexd cannot scope these queries. Stale-query failures preserve
statistics with `stale: null` and a `stale_error` message; see the manual for the
explicit thresholds and the observation purge procedure.

Run `mix tools/mcp_gate.mix focused` through CBC to build the test interpreter,
run tests and collect dependency/release receipts. The integration tests embed
the real noded using `test-broker`, with isolated socket, config and home paths.
