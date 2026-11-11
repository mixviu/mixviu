# Mix service derivation

Source: `markc/mixos`, frozen `origin/main` commit
`8779f33284b7d3f1178c3d57a699efb9b0867b22` (2026-11-11).
Mode: derived build. MIT OR Apache-2.0; original notices retained.
`files.mx` lists every imported file, its source Git blob and derived SHA-256.
Source paths and destination paths are identical. No Rust was imported.

The import includes each component manifest, all scripts, tests and fixtures,
units, daylight's zone-generation tool and releasesd's README. The shared
private-broker test helper and daylight zone data already exist in MixViu.

Changes from the oracle:

- Replace old environment names with registered `MIX_*` names, including
  `MIX_TEST_MIX_EXE`, `MIX_TEST_NODED_EXE`, `MIX_TEST_SETTINGSD_EXE`.
- The round-two naming audit reuses `MIX_SETTINGS_INSTANCE` and
  `MIX_SETTINGS_SERVICE` for daylight instead of adding aliases. The remaining
  28 additions have matching constants in `paths::env` (22 runtime, six test).
  Agentd-specific settings use `MIX_AGENTD_*`, including the MCP child's
  `MIX_AGENTD_SESSION`; releasesd uses `MIX_RELEASESD_*`; daylight fault
  injection uses `MIX_TEST_DAYLIGHT*`. The forwarded `MIX_MCP_ALLOW_TCP`
  belongs to the MCP edge. Registry-only constants add no Rust reader.
- Replace path calls with `mix_path`; child interpreters use the running
  interpreter (`/proc/self/exe`), with test executable overrides where needed.
  The agent CLI no longer accepts a separate child interpreter override.
- Default the MCP executable to `<prefix>/bin/mix-mcp`. Its separate port is
  required for real agent sessions; these tests use fake ACP agents.
- Remove script shebangs. Units invoke the prefix's interpreter explicitly.
  Remove systemd StateDirectory directives; user units create their existing
  private XDG state directories explicitly with umask 0077. Desk units retain
  their session environment file and account contract.
- Sanitise the recorded build completion fixture's worker, checkout and log.
  Use a neutral MCP executable path in the unit fixture.
- Document lint's inability to analyse the two shared-scope handler files.
  Their dynamic handler loading is unchanged from the oracle.
- Use MixViu in service descriptions; retain compatibility tokens below.

Integration: bundle scripts in `lib/<service>`, render all five unit templates
through `share/layout/templates.mx`, and run all 17 test scripts from
`tools/ci.mix test` (also `all`) using `tools/mix_services_test.mix`.
The `mix-services` CI stage runs these checks separately. Bundle creation
copies the layout manifest's template closure into `share/layout/sources`
and rewrites its bundled manifest, so installed rendering does not depend on
repository source directories. A headless regression check removes the source
fixture before rendering all five installed service units at a relocated root.
Tests create owned loopback brokers and local HTTP fixtures. No public network,
real agent CLI, service installation or desktop is used by these tests.

## Retained `mixos` tokens

- `mc@mixos.dev`, documentation URLs: original copyright/contact and the
  currently retained documentation domain.
- `package.metadata.mixos`: component manifest comments describe the existing
  metadata schema; no new key or schema was introduced.
- `User=mixos` in agentd/release desk units and the matching agentd comment:
  existing desk account, pending the separately scoped account migration.
- XDG `mixos/agent.conf.mix`, `mixos/agentd`, `mixos/peers`,
  `mixos/peers.conf.mix`, `mixos/releases.mx`, `mixos/releases`: existing user
  config/state and per-target ownership/lock records. Renaming would hide
  sessions, peers, catalogue entries or installation ownership. Tests and
  comments retain the same spellings.
- MCP server name `mixos` (including reservation errors and test expectations),
  ACP client/agent identity `mixos-agentd`: on-wire compatibility and resume.
- `mixos-mcp` in explanatory comments/test labels: names the oracle bridge
  contract. The executable default is now `mix-mcp`.
- `mixos-desktop`, `mixos-server`, their `-v` tag prefixes, `markc/mixos`:
  releasesd's existing catalogue/category and published-release contracts.
- HTTP User-Agent `mixos-releasesd`: retained identity of the release client.

Bus names remain `agent`, `peers`, `releases` and `daylight`; all commands,
topics, await/resume handlers and permission policy are inherited intact.

## Investigate-only services

**toolsd: recommend a separate port if the web tools feature is retained.**
Both frozen source and MixViu contain `services/webd/vhosts/apps/h_tools.mix`:
it enqueues authorised jobs, exposes claim/report endpoints and nudges
`toolsd.wake`. `tools_lib.mix` owns the queue, progress and completion state;
`tools_registry.mix` defines validation/roles shared with the drain.
`h_biz_domains.mix` exposes Shwho/Trace only under explicit `tools: true`;
`h_ssh_hosts.mix` shares that opt-in for Uptime/Ping. The Rust
`services/webd/src/bus_call_handler.rs` allows the authority-free wake and
`handlers_namespace.rs` tests that contract. These callers exist in MixViu.
Without a drain, enabled whois/dig/shwho jobs can remain queued; losing the
optional wake alone would be harmless only if a scheduled drain still ran.
Disabled desk units do not prove the feature has no callers. No service or
vhost configuration was changed; whether any deployed vhost enables it was
not tested.

**bridged: recommend drop, subject to Mark's decision.** The frozen service
is a Bus adapter for an external application's line-delimited JSON control
channel, one `<app>` citizen per instance, rather than a noded mesh link.
No direct bridged service calls or instance units were found in MixViu.
The cited `services/settingsd/src/mirroring.rs` hit is a test local named
`bridged` holding a `Sender` with `bridge-alpha` identity; the logging
`init.rs` hit describes log records bridged into tracing. Neither starts or
calls the bridged service. The webd handler hit describes async dispatch onto
its main runtime. Removing the unported service breaks no identified caller
in this source tree. External applications configured outside this repository
were not exercised; no claim of fleet-wide absence is made.

Neither investigated service was imported. No hooks, live agentd or other
running services were changed.
