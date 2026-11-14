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
The MixViu config store uses `<etc-root>/knowledge/knowledge.conf.mix` and
`<etc-root>/domains/domains.conf.mix`. Moving existing files into those service
directories is an operator cutover step; this port changes no configuration.

`knowledge_digest {}` returns global statistics. It refuses a non-null `domain`
argument because indexd cannot scope these queries. Stale-query failures preserve
statistics with `stale: null` and a `stale_error` message; see the manual for the
explicit thresholds and the observation purge procedure.

Run `mix tools/mcp_gate.mix focused` through CBC to build the test interpreter,
run tests and collect dependency/release receipts. The integration tests embed
the real noded using `test-broker`, with isolated socket, config and home paths.

## Worker and settings contract

The interpreter is `mix` beside the adapter's resolved `current_exe()`.
`MIX_PREFIX` relocates runtime data, never the executable. Tests use the
same production rule; no interpreter override is provided.

`mix --internal-mix-worker` is a public protocol entry, despite its name.
It accepts one JSON object on stdin: `script` (string, at most 256 KiB)
and optional `cwd` (string). Unknown fields are rejected. It skips ordinary
shell startup hooks, captures stdout/stderr (at most 1 MiB each), and returns
the script status. It adds no authority beyond a user running `mix`; this
is not a sandbox. Bus provenance reports `binary: "mix"`, with the MCP
session's named attribution retained.

The evaluator uses the committed frozen builtin inventory in
`libraries/mix-shell/src/mcp_builtins.txt`, derived from the two builtin
registries and feature-gated dispatch arms at `8779f33`. Only core plus
json, regex, toml, datetime, url and crypto calls are admitted. Shell-only
extensions such as `tar_list`, feature-gated HTTP client and SQLite calls are refused. Core
process and file-write builtins were already in the original surface.

Log-filter and deadline overrides use the exact process environment and
fall back to the original code defaults (baseline filter, 120 seconds,
600-second script floor). The script override is read only for scripts.
Missing or malformed installed knowledge/domain/indexd defaults fail the
affected tool with the variable and `<prefix>/env.mx` in the error; they
never prevent startup or tool discovery. `Default` implementations are pure.
