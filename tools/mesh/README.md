# Mesh tooling

Generators, signers and deploy scripts for a WireGuard mesh with a Bus broker
on each node. Nothing here names a project or a site: every name comes from
configuration, so a fork can relabel the whole tree.

## Configuration

| Setting | Where | What it names |
|---|---|---|
| `MESH_PROJECT` or `_etc/mesh/project.mix` | env or hub | private hub artifact namespace; units and groups keep their component names |
| `MESH_HUB` | env (required) | the directory that holds `_etc/mesh/` (inventory, epoch, zones) |
| `MESH_NODES_CONFIG` | env, optional | private node settings file, default hub `_etc/mesh/nodes.conf.mix` |

The signer is `mix mesh ...`, invoked through the running interpreter's physical
path (`/proc/self/exe`), including when testing an uninstalled binary. No binary
directory or source checkout setting is needed. Each SSH call passes its
node's explicit interpreter. Roots are queried using that interpreter's
`mix_path` with the system tier, including layout and environment overrides.
Settings are read once per invocation, and each node's first successful root
answer is kept for the whole run. An explicit `roots` setting must contain all
three absolute `prefix`, `etc` and `var` values. Partial, non-map and relative
settings refuse with `MESH_LAYOUT`; they never blend with interpreter defaults.
An older interpreter uses `legacy_defaults:true` with that complete triple.
Those explicit roots are returned directly, with no SSH layout query.

The hub machine has **two entries**: `local` supplies the roots used for local
anchors, seeds and freshness witnesses; its roster name supplies `mix_bin` for
tools that reach the hub through SSH. Filling only `local` is insufficient.
For example, this hub and its peer still use a legacy layout:

```mix
{nodes:{
  local:{roots:{prefix:"/srv/legacy",etc:"/srv/legacy-etc",var:"/srv/legacy-state"}},
  hub:{mix_bin:"/srv/legacy/bin/mix",legacy_defaults:true,
    roots:{prefix:"/srv/legacy",etc:"/srv/legacy-etc",var:"/srv/legacy-state"}},
  "node-a":{mix_bin:"/srv/legacy/bin/mix",legacy_defaults:true,
    roots:{prefix:"/srv/legacy",etc:"/srv/legacy-etc",var:"/srv/legacy-state"}}
}}
```

A MixViu hub and peer can instead use their own configured system roots:

```mix
{nodes:{
  local:{},
  hub:{mix_bin:"/srv/example/bin/mix"},
  "node-a":{mix_bin:"/srv/example/bin/mix"}
}}
```

Only a wholly absent `local.roots` queries the running interpreter's system
roots. Remote entries are keyed by admitted node name. Their interpreter is
explicit, and native jobs require the exact configured `<prefix>/bin/mix`.
Native worker binaries use physical, normalized installation paths with
root-controlled ancestors and regular root-owned files that are not writable
by group or others. The sole permitted symlink is the root-owned
`<prefix>/bin/noded -> mixd` (literal relative target), resolving directly to
the regular `<prefix>/bin/mixd`: releases ship noded as a personality of that
multi-call binary. Its resolved mixd file supplies the binary hash. Chains,
other targets, symlinked ancestors and a symlinked mix remain refused.
An absent entry refuses rather than guessing another installation.

The project names hub artifacts and legacy state paths, not the gold binary's
environment. Nodes using `legacy_defaults:true` keep the uppercase project
environment prefix. Gold nodes installed under `/opt/mix` without that flag,
and all pct desks, use `MIX_*`. System jobs use the node's configured etc/var
roots; desk jobs and service environments use the dedicated profile roots.
Preparation refuses an existing service.env with the old project prefix.
Preserve that file for inspection, remove only service.env, regenerate the
public plan and rerun explicit preparation with the reviewed tools. Matching
profile/config/drop-in/WG files are retained; no baseline or seed is reset.

`_etc/mesh/project.mix` is the hub's copy of the project name:

```
{project: {name: "example"}}
```

Each tool that needs a mesh name reads its own `_etc/<tool>.conf.mix`
(`expected_mesh`). The hub copies them from the `.example` files, which this
tree does not ship active.

## Layout

- `_bin/` the tools. Each is run as `mix _bin/<tool>.mix` with the variables above set.
- `_lib/deploy_lib.mix` the shared helpers: names, roots, roster, versions.
- `_lib/mesh_lib.mix` the inventory validator, the zone and WireGuard generators, the write lock.

## Retired members

A retired member keeps its inventory entry for ever with `status:
"tombstoned"`: its name, address and closed keys stay reserved, and
validation, allocation, admission and drift checks keep seeing it. It is never
**shown** unless asked for. `mesh -a` and `mesh -j` list active members only,
and `mesh --retired` adds the retired ones. `mesh_reconcile` still reports a
retired member that is on the wire (`STALE-ON-WIRE`), but prints the
"absent (expected)" row only with `--retired`. Apps that read `noded.peers`
(the Monitor, the tray, the MCP `bus_list_peers` tool) drop
`routing_view` entries whose class is not `active-bus`.

## External binaries

The tools call native project components:

- `mix mesh` signs the inventory and prints its public key.
- `noded` is the broker each node runs.

`noded` is found under the remote node's reported prefix.

## Checks

DNS overlays accept `published` (A, MX, mail SRV and PTR) and optional
`published_address_only` (A and PTR only). Owners must be unique across both
lists, declared active inventory members and have numeric entries in `serials`.
Only `published` requires `bus:true`, because it advertises mail services.
An `extra` entry for an address-only owner is refused, not silently dropped.
For example: `published_address_only: ["desktop"]` with `serials: {desktop: 1}`.
`mesh_gen preview --out DIR` writes unsigned DNS output to `DIR/zones.mix` when
the overlay and header exist; seal and `mesh_zones_check` share the same recipe.

System-target sync verification uses the local broker first. Only a member
new in this epoch (no d2 credential admitted it at the previous epoch), when
the local broker answers "Unknown mesh node", is verified on the target over
the existing SSH/Mix transport instead; any other failure, including no route
to a known member, stays a failure. Both paths require verified posture and
the exact epoch and canonical inventory hash, and each target logs its path.
Before publishing the canonical inventory, every member verified over SSH is
checked again over the Bus, once every node runs the new epoch.
Run `mix tests/new_members_test.mix` with the mesh environment set for offline
policy, probe and DNS fixtures. It never contacts a broker or SSH host.

`mix _bin/mesh_targets.mix TARGETS.conf.mix` previews explicit system and
desk target paths without contacting a host or writing anything. Optional
`--probe` uses the existing native SSH/Mix path and a namespace stdin pipe to
query only noded.info and noded.peers as the session account. It requires an
`MESH_HUB` and the explicit per-node interpreter/root settings above; missing
node settings raise `MESH_LAYOUT`. Process success is reported separately from Bus
reply codes; neither is an admission or deployment verdict. The version1
schema has `project` and a nonempty `targets` list. Each target has `node`,
`host` (an SSH alias) and `kind` (`system` or `desk`). A desk also names its
`machine`, `account`, integer `session` from1 to6, and `profile` as one direct
child of `/var/lib/<project>`. See `tests/targets.conf.mix`.

The model distinguishes the host SSH endpoint from the admitted node and the
container namespace. It derives profile-local genesis, seed, inventory and
anti-rollback baseline paths and `noded@N.service`. The distributor integration
was parked behind `desk_ops.ROLLOUT_READY` until the reviewed live proof on two
Proxmox CT desks (2026-11-17); the switch is now on. Existing
read-only probes remain available. Never use a path-only override to provision
a desk. The guard must stay closed until the complete integration is reviewed.
Run `mix tests/targets_test.mix` for the hermetic model checks.

### Separate broker principal preview (schema 2)

Schema 1 retains the existing single-account description. Schema 2 is an
opt-in, read-only identity contract: desk entries use `client_account` and
`broker_account` instead of `account`. Both must be unprivileged and different.
The desktop account remains the citizen execution identity. The broker account
is resolved independently by native clients; it is never inferred from the
socket's owner. System entries retain their existing description and transport.

```mix
{schema_version:2,project:"example",targets:[
  {node:"desk-a",host:"host-a",kind:"desk",machine:"desk-a",
   profile:"/var/lib/example/broker-a",session:3,
   client_account:"desktop",broker_account:"noded",
   admission:"enforce"}
]}
```

The desktop runtime remains `/run/example/session/3`. The explicit broker
endpoint is `/run/example/broker/3/noded/bus.sock`, reached through root-owned
ancestors and a broker-owned 0711 leaf. The broker's private witness directory
is owned by its dedicated UID with mode 0700. The desktop must not share that
UID or its private group. The public `etc/client.conf.mix` contains only node,
mesh, WireGuard address, broker port and explicit Unix endpoint; the private
broker node configuration and seed are separate. A citizen job sets the public
client config and broker account, and does not set the broker's private VAR.
Existing application state/runtime environment is not migrated by this model.

`_lib/broker_identity.mix` validates resolved identity metadata, exact protected
directory snapshots and matching broker/public-client configurations without
performing I/O. `tests/broker_identity_test.mix` uses synthetic distinct IDs;
it is not an OS ownership or monotonic-witness isolation test. Schema 2 remains
`mutation_supported:false` and the remote worker explicitly refuses it. No
account, unit override, endpoint or profile is provisioned by this source.

Activation still requires separate UID/group provisioning, a dedicated effective
broker environment and unit contract, native namespace ownership/adversarial
replay tests, the independently reviewed Bus server-PID pin, and one complete
reviewed distributor path. Existing system units and single-account desks are
unchanged. Neither this preview nor schema 1 establishes resistance to a
compromised desktop account until those isolation gates pass.

`_bin/name_gate.mix` fails when a project or site name appears as a literal in
this tree. Run it before a merge.

The pure `stdin_job` helper describes namespace invocation and profile context.
It separates source/bindings from process argv and distinguishes administrator
from session-account jobs. It performs no execution and does not enable a
distributor write path. Its namespace stdin, account and profile boundaries
are covered by hermetic tests.

## Guarded desk distributor draft

Both `mesh_sync` and `mesh_d2_seed` now consume the same optional private
`$MESH_HUB/_etc/mesh_targets.conf.mix`. Without it, existing system-node dispatch
is preserved. When present it must describe **every** fleet roster node exactly
once; system entries retain `host == node`, and desk entries explicitly state
`admission` (`off`, `observe` or `enforce`). Omission or
malformation cannot silently fall back to system paths. For example:

```mix
{schema_version: 1, project: "example", targets: [
  {node: "hub", host: "hub", kind: "system"},
  {node: "desktop", host: "host", kind: "desk", machine: "desktop",
   profile: "/var/lib/example/desktop-bus", session: 3, account: "desktop",
   admission: "observe"}
]}
```

This describes prepared targets; it does not create accounts, profiles, units,
network routes or membership. Use a **fresh dedicated noded profile**, preserving
the desk's old anchor/baseline and all app/settings data. Profile, etc, etc/noded
and var directories must be root:session-group0750; var/noded must be
session-account:session-group0700. Existing distributor-owned files must be
root:session-group0640. A baseline, when present, stays account-owned0600 and is
never copied, deleted or reset by these tools. Root-controlled ancestors and
native binaries are mandatory.

The namespace-local `noded@N.service` must retain one native `Type=simple` start,
`KillMode=control-group`, its session runtime directory, and the declared account.
After guarded activation its **last mandatory** EnvironmentFile is the root-owned0640
`<profile>/etc/noded/service.env`, containing only these literal assignments:

```text
EXAMPLE_ETC=/var/lib/example/desktop-bus/etc
EXAMPLE_VAR=/var/lib/example/desktop-bus/var
EXAMPLE_NODE_CONFIG=/var/lib/example/desktop-bus/etc/node.conf.mix
EXAMPLE_RUN=/run/example/session/3
EXAMPLE_BROKER_ACCOUNT=desktop
EXAMPLE_NODED_URL=
```

The empty broker URL prevents inherited host/desk overrides; native clients read
the explicit node configuration. That root-owned0640 config must match the
authenticated node, mesh, WG address/port, roster path, Unix socket and explicit
policy. Service environment, static executable/argv, runtime uid/environment and
binary hashes are checked independently. Runtime pid/timestamp fields do not
change the static unit fingerprint. Installed native Mix must expose all needed
filesystem-event and transaction primitives; no alternate interpreter is used.

Seeds travel as data through native SSH/Mix stdin and a namespace pipe, never
source argv or a second broker. `mesh_d2_seed` keeps its verified-inventory,
coherent secrets snapshot and current-public-key checks; it writes a root-owned
seed provenance receipt and **never restarts**. The receipt is deployment
provenance, not an independent authority: `mesh_sync` checks its seed hash/key
against the current authenticated D2 credentials before installing inventory.
Existing system seed dispatch retains its private file transport.

The worker snapshots preimages in protected `<profile>/.mesh-rollout/<token>`
directories, records absent files, uses exclusive staging beside destinations,
rechecks preimages and verifies publication. Failure logs identify the proposed
receipt path even if transport dies. Seed/roster preimages can be restored only
if no concurrent update occurred; once inventory was published, recovery requires
a higher signed epoch. The existing single-writer proposal, hubs-last order and
canonical-cache rules remain with the owning distributors.

Before activation, effective EnvironmentFiles must contain only the release's
session environment. The worker requires complete seed, genesis, roster and
inventory placement before activating the parked drop-in. It reloads, restarts
and checks local node/WG identity and verified mesh/epoch/hash in the same
guarded operation, using a fresh session-account interpreter. Socket readiness
uses a native filesystem-event child with a bounded deadline, not polling.
Failure parks the drop-in first, reloads systemd, validates release-only
environment, clears the unit's failed/start-limit state and restarts the
release broker. Accepted inventory and its baseline are retained. Recovery
failure is reported explicitly rather than claimed successful. This is local rollout
verification; production acceptance still requires addressed cross-node ABP and
the separately reviewed WG, service/client environment, canary and rollback plan.

Run `tests/desk_ops_test.mix` for real native filesystem failure/restoration
fixtures and `tests/desk_flow_test.mix` for typed executor order/failure, unit
binding and generated-source/RPC fixtures. Neither contacts production SSH,
services or secrets. A real disposable namespace exercise remains a separate
proof requirement before claiming deployed readiness.

### Proxmox CT preparation

Schema 1 desks may instead specify `transport:"pct", ct:123` with no
`machine`. `host` is the PVE SSH alias. Their dedicated profile is a direct
child of `/opt/mix/var/lib/<project>`; runtime is `/opt/mix/run/session/N`.
Configure each CT's explicit native roots and interpreter in nodes.conf.mix
(prefix `/opt/mix`, etc `/opt/mix/etc`, var `/opt/mix/var/lib/<project>`).
Never query CT layout by running a layout probe on its PVE host. The PVE host
must also have the same native Mix interpreter path for the SSH stdin adapter.
CT jobs use `pct exec ID -- systemd-run --pipe ...`, with the same transient
unit deadline and control-group kill policy as nspawn; citizen commands then
use `runuser -u ACCOUNT --`. Bindings and source remain on stdin.

Prepare explicitly, separately from sync/seed, using a reviewed public plan:

```text
# Hub, with MESH_HUB and node settings pointing to the intended inventory:
mix tools/mesh/_bin/mesh_desk_prepare.mix --plan SIGNED NODE 192.0.2.1:51822 /tmp/desk-plan.mix
# Copy the public plan and this reviewed tools/mesh tree to the PVE host.
# PVE root (does not use SSH):
/opt/mix/bin/mix tools/mesh/_bin/mesh_desk_prepare.mix --apply /tmp/desk-plan.mix
# Separate, explicit network activation inside the CT:
pct exec 123 -- systemctl enable --now wg-quick@wg1.service
```

Preparation creates root:session-group0750 profile/etc/etc-noded/var and
account:session-group0700 var/noded; existing directories must already match.
It writes root:session-group0640 service.env, node.conf.mix and a per-instance
`zzzz-mesh-profile.conf.parked` drop-in. It validates the release-only effective
unit and does not reload systemd. An identical already-parked file is retained;
no second copy or active `.conf` is created. An existing active drop-in refuses
preparation, because service activation belongs to guarded sync.
No seed, genesis, baseline or inventory is created or reset. Existing files
must match exactly, making completed preparation idempotent and conflicting
state an explicit refusal. It does not restart noded or start WireGuard.

Reuse the release's `noded@N` to preserve User, PartOf, native argv and session
cleanup. A second unit would compete for the same socket and lifetime. The
drop-in adds the profile environment last only when guarded sync activates it;
later overrides cause refusal. Until then a reboot or broker restart continues
to use the release's own session environment.
Unlike nspawn's draft contract, this release creates its absolute runtime path
through ExecStartPre and RequiresMountsFor, leaving RuntimeDirectory empty.

The preparation plan derives the address, current desk/hub WG keys and subnet
from authenticated inventory; the endpoint is explicit private operator input.
The CT key stays in `/etc/wireguard/wg1.key`, is checked with `wg pubkey`, and
is used only inside the CT to write root:root0600 `/etc/wireguard/wg1.conf`.
This is the desk-side generator; `mesh_gen seal` retains its existing hub WG
generation. No private key enters the hub plan or process argv.

`desk_ops.ROLLOUT_READY` remains the sole sync/seed rollout switch.
Explicit preparation is a separate administrative tool, not a sync override.
Live proof must establish WG routing, effective unit/session environment,
cross-node addressed ABP and recovery before the orchestrator reviews/flips
that one line. Schema 2 remains preview only.
