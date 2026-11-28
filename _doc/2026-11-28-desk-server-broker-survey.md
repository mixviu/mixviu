# Desk server broker survey

Survey of main commit `736c7abbf4585b5aa24d763e56b0145d6e50608e`, 2026-11-28.

**Disposition: stop before implementation.** One node broker is mechanically
possible with the existing native transport. Sharing its legacy service namespace
between desktop processes and server daemons needs an explicit ownership and
authorisation decision. Running noded under a separate UID protects its files;
it does not reserve server service names for their service accounts. This is the
brief's survey stop condition.

## Session broker discovery

`services/noded/units/noded@.service` is part of `session@%i.target`, runs
as `mix` in the template, and reads `@{etc}/session/%i.env`. Installers
substitute the desk account. That environment places `MIX_RUN` at
`@{run}/session/N` and selects the desk's profile node configuration.

There are two related lookup paths:

- `libraries/bus/src/client/locate.rs::noded_url_with_config` resolves
  `MIX_NODED_URL`, then absolute `MIX_BUS_SOCKET`, then
  `noded.unix_socket` in the selected node configuration, then
  `config::Dir::Run/noded/bus.sock`.
- Native verified clients use `libraries/bus/src/client_helpers.rs`
  (`BrokerEnvironment`) and `libraries/bus/src/native_client/unix.rs`.
  Their Unix candidates are the explicit `MIX_BUS_SOCKET`, configured
  `noded.unix_socket`, and runtime defaults. `MIX_NODED_URL` supplies
  a separate URL; changing it alone does not replace those Unix candidates.
  An explicitly placed session runtime excludes the system runtime fallback.
  Mix's lazy verified connection in `libraries/mix-shell/src/bus.rs`
  uses this path and disables ping discovery.

The paths and environment contracts are in `libraries/paths/src/lib.rs`,
`libraries/paths/src/env_names.rs`, `libraries/config/src/node.rs`, and
`etc/env.conf.mix`. No new environment variable is needed.

**The single session configuration selector should be `MIX_NODE_CONFIG`.**
For desk-server, select a public client configuration containing
`noded: { unix_socket: "/opt/mix/run/noded/bus.sock", account: "noded" }`
and the chosen node identity. This selects the same endpoint for both paths
and supplies the trusted broker account. Keep `MIX_RUN`, `MIX_VAR`,
`MIX_ETC`, XDG runtime, Wayland and D-Bus files session-scoped.
Remove contradictory inherited endpoint/account overrides when generating the
environment. If explicit endpoint overrides are retained for particular units,
use consistent `MIX_BUS_SOCKET`, `MIX_NODED_URL` and
`MIX_BROKER_ACCOUNT`; that is not a one-variable solution.

Plain desks retain their existing profile configuration and runtime socket.
Launch handoffs under `MIX_SESSION_FD` continue to use their issuing broker's
stamped endpoint; they must not be redirected after allocation.

## Sharing the node broker

| Item | Finding and disposition |
| --- | --- |
| Broker identity and stamping | No new stamping design is needed. `services/noded/src/noded.rs::unix_ws_handler` stamps the real peer UID/GID/PID from SO_PEERCRED, the broker's node, epoch and connection ID. Desktop clients remain the desk UID; they do not become `noded`. Client verification checks the configured broker account, socket ancestors, server credentials and PID. Fix: public client config trusts `noded`; private broker state, seed, inventory baseline and runtime parents remain broker/root-owned. Never recursively chown desktop state into broker ownership or expose the broker's private config as the client config. |
| Local authorisation and service ownership | **Blocker.** `libraries/bus/src/native_session.rs::PeerContext::is_local` accepts any Unix peer. `register_gate_decision` in `services/noded/src/noded.rs` leaves local Unix citizens ungated even in mesh admission enforce. Legacy `noded.register` reserves broker/native-session name shapes and rejects an occupied name, but does not bind ordinary names to Unix UIDs. A desktop process can claim an unoccupied `maild`, `settingsd` or other server name before its real daemon starts. Peer stamping records this; it does not authorise the name. Decide whether all desktop processes deliberately share the server's trusted local namespace, or specify UID/service ownership and command access rules. The brief specifies broker ownership, not that policy. Adding such rules here would invent a security model. |
| Service collisions | `settingsd` connects as `settingsd` (`services/settingsd/src/service.rs`); notifyd starts with `--name notify`. `services/maild/src/bus.rs` fixes `BUS_SERVICE = "maild"`, so `maild@N` and system maild collide despite distinct systemd unit names. The second live connection is refused; startup order cannot resolve ownership safely. Fix requires the service-owner decision: choose one settings authority/mirror and one notify service per node, and decide whether session mail is replaced by node mail or needs a separately designed instance contract. Do not silently rename services and break callers. Multiple desktop sessions need the same explicit ownership rule. |
| Desk inventory and tooling | Bounded implementation after the policy decision: add desk-server to `tools/mesh/_lib/targets.mix` parsing, reconstruction and planning; represent a node broker with guest transport and system state roots. Extend `desk_worker.mix` and `desk_ops.mix` rather than treating a desk-server as an ordinary desk: their guards require kind desk and derive session unit/profile paths. Update `_bin/mesh_join.mix` and tests. Never permit one node identity to target both system and session brokers. Schema 2's separate-principal preview still uses `noded@N`; it is not the desk-server implementation. |
| Mesh membership and identity | Option C gives the session broker the node identity. Desk-server must give that same single identity and signed authority to system noded, with the session a local client. Do not create a second member to represent the desktop. Keep production identity transfer, keys, membership changes and listener activation outside this PR's authority; the separate cutover must select the one retained identity before startup. Settings mirrors derive their local node from the broker, so assess authority/mirror bindings against that identity. Changing the broker process alone is not an inventory migration. |

The security decision needed to resume is concrete: may any process running as
the desk account register server names and use the existing local server
surfaces? If yes, record that accepted trust boundary and select the sole owner
of each colliding service. If no, provide the service-name/UID and command
authorisation contract before implementation. Native session leases already
bind their generated identities to UIDs; they do not assign the fixed legacy
daemon names above.

## Kinds and unit installation

There is no single existing public host-kind switch controlling installation:

- `tools/mesh/_lib/targets.mix` validates `system` and `desk`, derives
  `noded.service` or `noded@N.service`, and rejects duplicate broker
  targets/profile stores. Its `account: root` for system targets is the
  remote administration account, not the broker process user.
- `tools/mesh/_lib/desk_worker.mix` checks kind, account, profile/runtime
  paths and `noded@N.service`. `desk_ops.mix` and
  `_bin/mesh_join.mix` also branch on desk.
- `tools/apps_bundle.mix` gathers unit templates into the flat desk catalogue.
  `share/layout/files.mix::desk_unit` filters the exact
  `# mixviu-install: host-only` marker. System noded currently has no such
  marker. This filter is not a kind guard and does not remove a stray installed
  system unit.
- `services/noded/units/noded.service` owns the node broker as `noded`,
  with `MIX_RUN=@{run}`, private node config/service environment and
  `WantedBy=multi-user.target`. It is independent of the session target.
- The private hub owns the real CT image installer
  (`_lib/desk_ct_install.mix`), CT validation
  (`_lib/desk_ct_policy.mix`), desk configuration
  (`_etc/desks.conf.mix`) and mesh target configuration
  (`_etc/mesh_targets.conf.mix`). Its CT installer creates session env/config
  and the session target's Wants/Upholds, including noded@N, and explicitly pins
  hostd to that broker. These are not public repository files.
- The release entry points `_bin/gold_ship.mix` and
  `_lib/ct_release.mix` use
  the private release directory’s `upgrade-local.mix` and `release_upgrade.mix`. The upgrade re-renders already installed units,
  preserves their installed User/Group and leaves new templates inert, apart
  from explicit session-member additions. It has no desk-server kind switch.
  Service-host installation is separately owned by `_lib/host_install.mix`
  and `_lib/node_install_tx.mix`.

After the blocker is resolved, use one validated host-kind decision for image
creation and upgrade. Plain desk must stop/disable and remove or mask any
stray system noded, including an already installed unit. Desk-server must
stop/disable and mask its session broker before enabling system noded; remove
session Wants/Upholds and the hard noded@N requirement in maild@. Update hostd's
explicit pin and all session consumers, not just viud. Preserve normal
service-host installation. Refuse inconsistent kind/unit/config combinations
before mutations. A template marker alone cannot enforce either direction.

## Design as built and validation

Survey only. No runtime, installer, kind or authorisation implementation was
made, and no host configuration changed. The two brokers were not started.

Source inspection confirms the discovery and registration branches above.
No Rust crates or Mix tools changed, so worker fmt, clippy, crate tests and Mix
tool tests are not applicable to this documentation-only stop. There are no CBC
job IDs or BUILD_RC results. The requested kind/installer/lookup tests remain
unimplemented pending the security decision; existing desk tests were not run.
No CI gate, review, queue, deployment or desktop proof was invoked.
