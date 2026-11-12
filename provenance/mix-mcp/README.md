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
by `paths::env::os`, an empty agent session override remains present and
is rejected, preserving the source policy. Malformed agent session IDs refuse
the Bus. The exact presence-preserving reader is shared with the paths follow-up. No compatibility reads of retired names.

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

The candidate `rmcp = "=3.9.0"` is crate-local, with defaults disabled and
only `server`, `transport-io`, `macros`, `schemars`; client/child-process features
are confined to dev-dependencies. `ServerInfo` is renamed to its SDK replacement
`ServerConfig`. The source lockfile
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

Verification remains incomplete. At source commit
`109f3efca2b3e0b2f0c4a2eed4321b0c0a3ec339`, CBC passes all 65 adapter tests
(59 unit, six stdio), with none ignored. Worker formatting is unchanged. Both
debug and release Mix pass the foreign gate; release Mix has no test-hook
marker. The inverse `rmcp` tree contains only `mix-mcp`, and none of the seven
added lock packages reaches a strict engine. The release adapter is 6,094,608
bytes. Package licences are confirmed by Cargo metadata.

The required `cargo deny` check rejects the authorised pin for
RUSTSEC-2026-0457, RUSTSEC-2026-0461 and RUSTSEC-2026-0462; bans, licences and
sources pass. These advisories concern OAuth and Streamable HTTP features
outside this stdio adapter's feature set, but the crate-level advisory gate
still fails. No advisory exception or pin change has been made. The final full
gate has not run. The empty-session compatibility difference above remains an
open decision because a presence-preserving reader would exceed the brief's
permitted `paths` edits.


## Round 3: dependency-isolation stop

The approved candidate compiles on CBC. Resolve job
`20261112-100430-22ef7d43-a1` passed (including the foreign gate, 16 listed
foreign crates, no policy change) and supplied the applied lock/fmt patch.
Compared with the 3.1.2 closure, resolution adds const-oid 0.10.3 and updates
process-wrap 9.1.1 to 10.0.1, rmcp 3.1.2 to 3.9.0, and rmcp-macros 3.5.1
to 3.9.0. Existing shared package versions remain locked.

The exact-source audit at `d820ad5d7dad` then failed:
`20261112-101225-50ff9ff4-a1`, BUILD_RC=1, STRICT_DEPENDENCY const-oid.
Cargo metadata reports its licence as Apache-2.0 OR MIT. The workspace inverse
tree contains both rmcp/mix-mcp and this second path:

```text
const-oid 0.10.3 <- digest 0.11.3 <- sha2 0.11.0 <- sha-crypt 0.6.0
  <- lib-mix <- mix / mix-shell / mixd
```

Workspace feature unification makes this new package reachable beyond the
adapter. This violates the approved isolation condition even though shared
versions did not change and the foreign gate passed. Work stops here under
the explicit round-3 instruction; the remaining licence/graph/advisory checks,
66-test candidate suite, final full gate and new release-size receipt have not
run. The previous 65-test/size receipts above cover 3.1.2 only.
The new separate empty-session stdio regression is committed but unrun; the
existing stdio tests and 29-name fixture remain unmodified in round 3.
No ready PR, merge, install or running-adapter change was made.
