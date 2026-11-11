# mix-mcp derived port

Source: frozen MixOS commit `8779f33284b7d3f1178c3d57a699efb9b0867b22`,
`cli/mixos-mcp` and `docs/mixos-mcp`, read with `git show`. File hashes and
destination mapping are in `imports.json`. Existing SPDX/copyright notices
and workspace MIT OR Apache-2.0 licensing are retained.

Neither layout guide assigns command-line edge adapters a directory, so the
brief's fallback `services/mix-mcp` is used. This is a derived build, with no
lab merge, install or service change.

Adaptations: crate/binary/server/log identity; existing MixViu shared crates;
registered environment constants and installed settings; paths-root logging;
logical indexd paths; sibling `paths::bin("mix")` script execution; cleaned
manual and helper; all source tests retained, plus a committed 29-name tool
inventory and stdio status/Bus/execution test. The frozen evaluator worker
moves into the sibling Mix binary; process-group cancellation and byte bounds
remain unchanged.

Runtime defaults moved to the registry retain ranking, transport, deadline
and logging values. The domain map retains all four frozen entries and adds MixViu and its hub. As prescribed
by `paths::env::raw`, empty environment overrides count as unset (the source
treated an empty agent session override as invalid). Nonempty malformed agent
session IDs still refuse the Bus. No compatibility reads of retired names.

MCP tool names, parameters and result field shapes stay unchanged; inventory
is `services/mix-mcp/tests/tools.txt`. Human descriptions mentioning the old
installation/environment identity are updated. The new server identifies as
`mix-mcp`. Operator migration: rename the agent's `mcpServers` key from
`mixos` to `mix-mcp`, change the command to the installed prefix's `bin/mix-mcp`,
and translate legacy environment overrides to their registered current names.
No agent configuration is changed by this port.
The shared MixViu config store places each file in its own service directory:
`<etc-root>/knowledge/knowledge.conf.mix` and
`<etc-root>/domains/domains.conf.mix`. Operator relocation is documented;
fixtures use that store layout without changing their source assertions.

`rmcp = "=3.1.2"` is crate-local, with the original server feature set and
client/child-process features confined to dev-dependencies. The source lockfile
seeds resolution, with unused packages removed by Cargo on CBC.

The frozen isolated evaluator worker now lives in `mix-shell::script_worker`,
entered by the sibling `mix` binary's private worker flag. The frozen named
connection policy is shared through `bus::agent_client`; MCP keeps its original
connection tests. This avoids ordinary Mix's anonymous/TCP-fallback semantics
and preserves session attribution, strict Unix failures, lazy local execution,
return-code mapping and skipped shell hooks. Neither native library imports the
MCP SDK or any package added by it. Normal Mix invocation keeps its existing
handler. Additional stdio tests cover attribution and a failed Unix socket
with TCP explicitly enabled. `adaptation_targets` records these source moves.

Verification remains incomplete. CBC's foreign gate passes, and the inverse
`rmcp` tree contains only `mix-mcp`. The required `cargo deny` check rejects
the authorised pin for RUSTSEC-2026-0457, RUSTSEC-2026-0461 and
RUSTSEC-2026-0462; bans, licences and sources pass. These advisories concern
OAuth and Streamable HTTP features outside this stdio adapter's feature set,
but the crate-level advisory gate still fails. No advisory exception or pin
change has been made. The focused job stops there, before tests and release
size measurement; the final full gate has not run.

