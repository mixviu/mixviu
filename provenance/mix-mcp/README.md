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
logical indexd paths; sibling `current_exe().with_file_name("mix")` script execution; cleaned
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

Verification receipts are recorded in the PR. The suite retains the 59 source
unit tests and six stdio tests, including the exact 29-tool inventory, native
Bus, sibling execution, attribution and failed-Unix/no-TCP-downgrade checks.
A separate stdio regression covers rejection of an empty session override.
The SDK update leaves the existing stdio tests and inventory unchanged.

The 3.1.2 checkpoint was blocked by three RustSec advisories. The approved
3.9.0 candidate uses only stdio features and adds no advisory exception.
Its dependency delta is const-oid 0.10.3 (Apache-2.0 OR MIT), process-wrap
9.1.1 to 10.0.1 (Apache-2.0 OR MIT), rmcp 3.1.2 to 3.9.0 (Apache-2.0),
and rmcp-macros 3.5.1 to 3.9.0 (Apache-2.0). Other shared versions stay pinned.

The new const-oid edge cannot be removed through adapter features: rmcp
unconditionally enables sha2 defaults, which enable sha2/oid and digest/oid.
There is no newer stable rmcp pin at the registry audit. Round 4 accepts only
const-oid beyond the adapter, conditional on unchanged existing engine
versions, a pure-Rust dependency-free crate with no build script, release
size/startup evidence and the foreign/ELF gates. Both digest and sha2 already
come from the RustCrypto publisher organisation; registry trusted-publishing
records identify RustCrypto/formats, RustCrypto/traits and RustCrypto/hashes
respectively for const-oid, digest and sha2.

CBC-only tools: mcp_sdk_audit.mix prints licences, inverse and feature trees,
crate build-script/dependency checks and cargo-deny results; mcp_cost.mix
compares the merged main engine graphs, builds before/after release Mix,
measures 101 alternating fresh-process startup samples for each snapshot,
and checks the final ELF. File caches remain warm; every sample starts a
new process with a cleared environment and its own staged installed defaults.
The median growth limit is 1 ms. mcp_gate.mix runs focused tests, release
receipts or the full repository gate; temporary test files use a private
cleaned directory outside the worker build tree and /tmp.

Security remains the frozen adapter's surface: arbitrary Mix/Bus execution
with the caller account's permissions, bounded scripts/output and cancellation
without rollback. Inherited log_tail/log_search paths are unconfined (including
absolute and parent paths); path confinement and traversal/symlink coverage
are listed for follow-up in the PR. No install or live configuration change
is part of this port.

The audit checkpoint 3f924fae is the immutable 3.1.2 port closure required by the security-pin decision, not a claim that it is advisory-clean. It is intentionally separate from the moving main baseline used for engine closure and performance checks.
