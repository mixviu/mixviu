---
title: mix-mcp
description: MCP stdio access to the native MixViu Bus.
---

# mix-mcp

Runtime settings come from the matching installed `env.mx`, generated from
the repository's environment registry, or explicit registered overrides.
Paths are resolved by `paths`; no installation path or setting value is
compiled into this adapter. Script execution uses the prefix's `bin/mix`.

`mix-mcp` is an MCP stdio server for agents. Install the binary at
`<prefix>/bin/mix-mcp`. Keep the `mcpServers` key `mixos` at cutover:
renaming it changes every `mcp__<key>__*` permission rule.

## Connection

The server starts without a Bus connection. The first Bus tool connects
using `libraries/bus`. `MIX_NODED_URL` overrides the node configuration URL;
otherwise the native client's node config discovery applies, including
`MIX_NODE_CONFIG`, `MIX_ETC` and the user/system defaults.

The bridge registers under an anonymous name, `mcp-<pid>-<nonce>`. When an
agent gateway starts the agent that runs it, the gateway sets
`MIX_AGENT_SESSION` to that agent's session id: 4 to 12 lowercase letters
or digits. The bridge then registers as `mcp-s-<session>-<pid hex><4 hex>`
(the trailing hex keeps bridges in separate PID namespaces apart). The
broker stamps that name as `from` on everything the bridge sends, so the
gateway can recognise one of its own agents calling back. A value outside
that form, including an empty or non-UTF-8 value, is an error: every Bus
tool fails with that reason, rather than
the bridge quietly registering under an anonymous name.

Set `MIX_BROKER_ACCOUNT` to the broker's operating-system account and
`MIX_BUS_SOCKET` to the session's Unix socket when overriding discovery.
The account order is `MIX_BROKER_ACCOUNT`, node.conf `noded.account`, then
`noded`; socket ownership never selects the trusted account.

Unix candidates are tried in order: explicit `MIX_BUS_SOCKET`, node.conf
`noded.unix_socket`, this environment's `<run>/noded/bus.sock`, the system
`<run>/noded/bus.sock`, then the endpoint advertised by `noded.ping` at the
resolved URL. Only missing path components are skipped, with a warning.
An existing candidate must pass path, account, peer-credential and protocol
verification; any failure is fatal. A server-PID pin forbids skipping and ping.

Candidates are captured before connection. An unpinned client retries once
if a selected path disappears; a second disappearance follows the
missing-candidates policy below. Socket and account overrides are independent,
so broken lower-priority node settings cannot veto an explicit override.

Verified Unix ingress is required whenever any candidate exists. When all
candidates are missing, plain TCP requires `MIX_MCP_ALLOW_TCP=1`; the default
is to refuse it and list the candidates tried. Calls are never replayed after
reconnecting.

```json
{
  "mcpServers": {
    "mixos": {
      "command": "<prefix>/bin/mix-mcp",
      "env": {
        "MIX_NODED_URL": "ws://192.0.2.10:4200/ws",
        "MIX_BROKER_ACCOUNT": "user",
        "MIX_BUS_SOCKET": "<runtime>/session/4/noded/bus.sock"
      }
    }
  }
}
```

## Tools

With that key, each name below is exposed as `mcp__mixos__<name>` in Claude Code.

- `bus_call`, `bus_list_services`, `bus_node_info`, `bus_list_peers`, `noded_ping`
  (`bus_list_peers` leaves retired mesh members out unless `include_retired: true`)
- `term_list`, `term_snapshot`, `term_type`, `term_tab`, `term_pane`
- `log_tail`, `log_search`
- `app_describe`, `app_actions`, `app_execute`
- `desktop_windows`, `desktop_window_wait`
- `clipboard_get`, `clipboard_set`
- `context_search`, `index_workspace`, `knowledge_digest`
- `docs_feedback`, `journal_feedback`, `memory_feedback`, `journal_supersede`
- `mix_execute`, `mcp_status`
- `await_op {service, op, deadline_s?, note?}` registers a suspending await
  for `MIX_AGENT_SESSION`. It returns the await id and tells the agent to
  end its turn now: agentd resumes the session with the outcome; do not poll
  or sleep. Outside an agentd session it refuses. A deadline sends a notice
  but leaves the await pending for the eventual terminal outcome.

`clipboard_get {selection?, mime?, primary?, offset?}` and
`clipboard_set {text, selection?, mime?, primary?, sensitive?}` use the local
`clipd`. Selection is `clipboard` or `primary`; `primary` remains an alias.
Reads return encoding and continuation fields, or a blobd reference for
binary/large content.
See [clipd](../clipd/README.md) for limits and cross-session push semantics.

### Application commands

`app_describe {service}` calls `app.describe` on that Bus service.
`app_actions {service}` calls `<service>.actions`. Its `result` contains
the application's command rows unchanged: `id`, `label`, `menu`, `shortcut`,
`more` (when present), `enabled` and `checked`, plus any further metadata.

`app_execute {service, id, params?}` calls `<service>.execute` once with
`{id}` or `{id, params}`. `service` is a Bus service name such as
`"workshop"`, `id` is a dotted command id such as `"file.refresh"`, and
`params` is an optional JSON object. Parameters are forwarded unchanged as
a sibling of `id`; the application owns validation and may refuse unsupported
parameters. Successful calls return `{service, command, result}`; application
error messages from the Bus client pass through unchanged as tool errors.
The same error-text pass-through and prefix removal apply to `app_describe`,
`desktop_windows` and `desktop_window_wait` through their shared native helper.

Retired tools: `app_actions_list`, `app_action_invoke`, `app_controls_list`,
`app_control_get`, `app_control_set` and `app_control_wait` have been removed
without aliases. Use `app_actions` and `app_execute` for registry commands,
and `app_describe` to discover an application's state commands. These retired
tool names must not be reused.

### Knowledge tools

`context_search` returns documents, journals, specs, plans, notes, memory,
source comments, observations and scripts, grouped by source. It applies
source trust weights and journal age limits. `index_workspace` indexes workspace
content. `knowledge_digest {}` returns global index statistics under
`statistics` and bounded staleness samples under `stale`, including legacy rows.
It explicitly queries chunks never retrieved for 90 days, low-value chunks
retrieved more than 3 times, and chunks dormant for 180 days, with 5 samples per
bucket. If the stale query fails, statistics remain available, `stale` is `null`
and `stale_error` contains the failure message (otherwise it is `null`). The
optional `domain` argument is recognised but any non-null value is refused
before connecting: indexd's stats and stale APIs cannot filter by domain.
Omit `domain` for a global digest; it is never silently ignored or inferred.
These tools send `indexd.<action>` requests to the configured indexd transport.
An absent backend returns an explicit tool error. Settings are MixViu strict-data
files: `<etc-root>/knowledge/knowledge.conf.mix` and
`<etc-root>/domains/domains.conf.mix`. An operator must move existing files
into these service directories when cutting over; this port moves none.

The knowledge skills loop was retired on 2026-10-27. The six `skills_*` tools,
`knowledge_brief`, skill refinement and graduation to `CLAUDE.md` are removed
without aliases. `context_search` neither queries skills nor returns a `skills`
group. Existing stored skills remain untouched; these retired tool names must
not be reused. The server no longer instructs clients to capture, refine or
rate knowledge after tasks.

At deployment, remove `trust_weight_graduated` and `trust_weight_skill` from
`knowledge.conf.mix`, along with any old `skills` or `llm` sections there.
Remove the retired `skills.conf.mix` and `llm.conf.mix` files from the MCP
configuration directory (`$MIX_ETC`, normally `<etc-root>`). They are
no longer read or generated. No compatibility aliases are provided.
The `config::store::load_service` loader tolerates unknown keys for
`KnowledgeSettings`: the retired keys are ignored, not rejected. A fixture test
exercises its directory-explicit core with those keys and invalid retired files,
and verifies that generated defaults omit them. Existing config files are not
rewritten; the coordinator removes the obsolete entries at deployment.

The optional `docs_feedback`, `journal_feedback` and `memory_feedback` tools
remain because indexd consumes their scores when ranking search results and
selecting low-value staleness candidates. `journal_supersede` hides an obsolete
chunk while retaining it for audit. To purge old CMM observations, use the
existing [indexd filtered delete](../indexd/README.md#purging-retired-cmm-observations).

### Host indexd from a desk

When a desk cannot reach the host's Bus, select SSH in
`$MIX_ETC/knowledge/knowledge.conf.mix` (normally
`<etc-root>/knowledge/knowledge.conf.mix`):

```text
indexd: {
    transport: "ssh",
    target: "indexd",
    ssh_host: "user@host.example",
    ssh_command: ["<prefix>/bin/indexd", "bridge"],
    call_timeout_secs: 120
}
```

Install indexd on the host and configure key-based SSH access for the desk's
user. The client runs `ssh -T -o BatchMode=yes -o ServerAliveInterval=30`
with the configured host and command. The host command runs in the remote
login shell; use trusted command arguments. `indexd bridge` uses the host's
local Bus; `target` selects the address only in Bus mode.

One bridge is shared by knowledge tools for the MCP server's lifetime,
with one request in flight at a time. Each call has a deadline, including
queueing and retries. A broken pipe, exited child or bridge transport failure
gets one restart and retry; an application error is returned immediately.
A timeout discards the child and returns an error without replaying the call.
A retried mutation may already have executed, so retry is not an exactly-once
guarantee. Normal MCP shutdown closes stdin and reaps the bridge.
Restart the MCP server after changing transport settings.

Missing settings default to `transport: "bus"`, `target: "indexd"`, the
command above, an empty `ssh_host` and a 120-second call deadline. SSH mode
requires `ssh_host`. When a Bus route is available, select
`transport: "bus", target: "indexd.host.example.bus"` instead. Presence is
checked with `indexd.stats` at the configured target, including routed targets.

Logical SSH command entries use the `mix:` scheme and resolve remotely with
`mix_path`. Update existing command vectors explicitly when moving to MixViu;
the adapter does not rewrite configuration files.

The installed client domain map retains the frozen project/hub mappings and adds MixViu and its private hub. Custom
`domains.conf.mix` maps override these defaults. `index_workspace` expands
only a leading `~` or `~/`, resolves the workspace to an absolute path, and
sends absolute file paths and content to indexd.

Terminal tools preserve explicit pane selection and instance checks. Native
application tools require an exact service and semantic target. Read-only
tools do not invoke mutations; arbitrary `bus_call` and `mix_execute` can.
Mix execution runs in an isolated worker with bounded output.

`mix_execute` is a trusted-agent tool. It runs as the adapter's user with that
user's permissions and is not a sandbox. Scripts can run programs, write files
and mutate Bus services. The builtin fence does not restrict those powers.

Log tools reject absolute paths, parent components and symlink targets outside
the log directory. Tail seeks into an 8 MiB suffix; search streams lines with a
64 KiB line limit and retains only the last requested matches. Both refuse
requests above 10,000 results or their 8 MiB byte budget rather than truncate.
Indexing requires a configured domain root, never enters directory symlinks,
and accepts markdown symlinks only within the scanned root.

### Capability dispatch audit

The fence controls builtin availability, not operating-system permissions.
The evaluator keeps the policy in shared globals; module and handler bodies
execute with those globals. Source references in `libraries/mix/src/`:

| Path | Enforcement |
| --- | --- |
| Generic builtin | `evaluator.rs:15179` dispatches through `call_policy_builtin` at line 16431, which owns the named policy check before `builtins::call_builtin`. The generic caller does not repeat it: host policies may be stateful. |
| HOF | `eval_expr` at line 15088 gates the HOF; `builtins_hof.rs:200` invokes callback bodies through `call_function_public`, returning to the same evaluator. Sync callback eligibility excludes builtin calls. |
| Mutating builtin | `call_mutating_builtin` at line 17563 checks before mutation or fallback. |
| Row plans | `rows.rs:53` checks during compilation; `Plan::run` at line 112 uses `bulk_call`, which checks again at `evaluator.rs:16578`. |
| Files and modules | `exec_include` at line 18169, `exec_file_in_scope` (including source) at line 18208 and `exec_require` at line 18363 gate FsRead before reading; bodies use the caller's evaluator and policy. |
| Async special cases | `eval_numeric_socket` at line 13434, `eval_socket_source` at line 13613, `eval_native_source` at line 13793, and async arms in `eval_expr` (sleep at line 14321, HTTP at line 14643, database at line 14701, JMAP at line 14758, Bus at line 15002) check before dispatch. FIFO I/O checks at line 18845. |
| Shell and Bus syntax | Class checks cover `send`/`emit` at lines 17862/18063, source shell fallback at line 18771, `sh` at line 18821, command substitution at line 18911 and pipes at line 18938. |

The other direct `call_builtin` calls in the evaluator decode clipboard
base64 or internal directory-entry bytes; they are fixed pure conversions,
not caller-selected builtin dispatch. HTTP's synchronous fallback is reached
after the HTTP capability check. Regressions cover the wrapper itself,
generic/closure/HOF/row dispatch and existing FsRead/module denials in
`libraries/mix/tests/limits.rs`.

## Logging and tests

Stdout is reserved for MCP. Logs go to stderr, journald and
`<var-root>/log/mix-mcp.log`. Arguments and results are redacted by
default; `RUST_LOG=mix_mcp=trace` explicitly enables full arguments.
`--version --json` reports build provenance without starting MCP or the Bus.

Package tests cover parameter translation, target guards, output schemas,
redaction and retirement of the skills protocol. The stdio integration test
uses the real fixture broker, including verified Unix ingress and lazy reconnection.
An in-process fake application checks command listing, execute request bodies
with and without parameters, and unchanged application error messages.
It also checks configured knowledge targets, paginated re-indexing and
absolute file metadata. SSH client tests use a local Mix fake bridge through
a test-only command hook; no SSH connection or indexd process is required.
