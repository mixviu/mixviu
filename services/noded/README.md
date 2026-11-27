# noded

## TLS broker links

An absent `noded.listen` preserves the legacy listener. An explicit list is
complete, including an empty list; Unix ingress is configured separately.
Legacy listeners remain available under admission `enforce` during migration.
The first legacy (`tls: false`) entry supplies the SPEC 13 §9a bind self-check
and legacy route port, independent of TLS entry order. With no legacy entry,
`wg_bound` is false and the legacy port is zero: there is no legacy WG trust
boundary to assert. TLS sessions use verified TLS membership and bound D2
proofs instead of the legacy WG-bind check.

```mix
noded: {
  listen: [{addr: "192.0.2.1:4200", tls: false}, {addr: "192.0.2.1:4201", tls: true}]
  tls: {key: "mix:etc/noded/bus-tls.seed", handshake_timeout_s: 3, max_handshakes: 64, max_queue: 64, per_source_rate: 10, per_source_burst: 20, max_sessions_per_member: 8}
}
```

The TLS values shown are defaults. TLS requires an accepted verified inventory,
a current own `bus-tls` key and a secure seed. The handshake deadline starts
when a queued socket enters the active handshake set. Queued sockets hold no
handshake slot; a full queue refuses new arrivals. Refusals warn once per IP
address (IPv6 /64) and class, then coalesce for 60 seconds. Expired windows
report pending counts before removal. At most 4096 windows are retained;
additional sources share one overflow window and counter. `noded.info` exposes
`meta.tls_refusals`, including suppressed listener and dialler attempts.
`meta.tls_refusal_overflow` counts attempts assigned to the overflow bucket.

Source buckets start with 20 tokens and refill at 10 tokens per second. IPv4
addresses, including mapped IPv4, share one bucket; IPv6 uses /64 networks.
Buckets are shared across TLS listeners and checked before queuing or starting
TLS work. There are at most 4096 buckets; new sources are refused until a fully
replenished bucket can be reclaimed. Refusals use class `rate-limited`.
Each member may hold eight TLS sessions, counted across inbound and outbound
connections and all listeners. The cap is checked after TLS membership is
verified, before handing the transport to HTTP; excess sessions use class
`session-cap`. A lease lasts until the transport drops, including queued
accepted sockets and failed HTTP upgrades. Rate, burst and cap must be positive.
These settings do not affect legacy connections when TLS is absent.

Reading `noded.info` flushes expired TLS refusal summaries, including a burst
followed by silence. The first overflow for each class retains its peer address.
Existing aggregate counters retain their meaning. Additive `meta` fields are:

- `tls_inbound` and `tls_dial`: separate `classes` maps of lifetime totals,
  `total`, `overflow`, and `outcomes` (`admitted`, `refused`, `timeout`).
  Timeout is exclusive of refused. Pre-handshake rate/queue refusals and
  post-handshake capacity refusals count as refused attempts. Admission here
  means TLS transport handoff; D2 and frame admission remain separate checks.
- `tls_sessions_per_member`: current TLS transport leases by signed member.
- `reload_to_drop_ms`: null before an accepted reload; then elapsed local
  reload application time, excluding watch debounce. It waits for retired
  outbound TLS transports to drop and for revoked inbound outboxes to close. It does
  not measure remote TCP acknowledgement. Rejected reloads retain the last value.

Use top-level `mesh_addr` in `node.conf.mix`. The former `wg_ip` spelling is
accepted for one release and warns once per process when loaded. Setting both
names, even to the same value, refuses startup. `noded.peers` emits `mesh_addr`
for self and each peer, with `wg_ip` retained as a one-release JSON alias;
`noded.info` also preserves that alias.

A mesh peer's optional `tls_port` selects `wss://` with a mandatory member pin.
It is temporary until PR 4 adds inventory endpoints and DNS-SD. A failed TLS
dial never falls back to the peer's legacy `noded_port`. A node with configured
TLS peers also needs its own TLS credential, even without a TLS listener.

Both D2 proof directions run inside TLS and bind to its RFC 9266 exporter.
TLS admission always requires both credentials, including when legacy admission
is off or observe. The D2 node must match the TLS member. An accepted reload
checks both exact keys; revoking either closes only affected links and records
which credential failed. Legacy links retain their D2-only check.
Inbound teardown reasons are `d2-key-revoked` for the exact D2 credential,
`tls-key-revoked` for the exact TLS credential, and `unproven-link` when the
link lacks a valid bound proof. Overlapping current credentials remain admitted.

Refusal diagnostics emit the first occurrence, then at most one line per 60
seconds per peer and frame reason (or per peer for admission challenges).
`count` is the interval count and `total` is the lifetime count, including
suppressed occurrences across reconnects. A trailing burst needs no flush:
`noded.info.meta.refusal_totals` exposes `frame_refusals`, `challenge_refusals`
and `unauthenticated_frames` for the broker process lifetime.
`noded.peers.peers[].challenge_epoch` reports the last matching admission
challenge epoch observed for that peer, or null before one is seen. This is
diagnostic data, not authenticated inventory authority; epoch skew stays at debug.

The native ABP broker: service discovery, routing, subscriptions and authenticated local session grants.

Documentation: <https://mixos.dev/noded/> — source the component manual (not included in this source tree).

## Inter-node frame policy rollout

Set `noded.frame_policy` in the node's `node.conf.mix` (normally
`/opt/mix/etc/node.conf.mix`, or the path selected by `MIX_NODE_CONFIG`):

```mix
noded: {
  admission: "enforce"
  frame_policy: "observe"
}
```

`MIX_NODED_FRAME_POLICY=off|observe|enforce` in the broker service environment
overrides the file; an empty value is unset. Invalid values refuse startup.
The default is `observe`. Both settings are read at startup. Frame enforcement
requires an accepted signed inventory; otherwise startup reports "no identity
source". With an unverified inventory, observe reports `no-identity-source`
rather than claiming a known peer failed admission.

Before upgrading a node that runs admission `enforce`, move **every local
client to the Unix socket**. TCP loopback registrations are refused immediately
at broker upgrade, even with frame policy `observe`. Remove stale explicit
`MIX_NODED_URL=ws://127.0.0.1:4200/ws` values from session environments and
wrappers, or replace them with the configured Unix endpoint. Check the runtime
root and socket placement too; a default path cannot find a broker placed in
a different runtime. A broker-account setting does not make a stale TCP
override safe. This local-client migration is a prerequisite to installation.

Install this build on the brokers with frame policy at `observe`. Watch
`frame.refused` for registration, source-node, wrong-node and relay violations:
observe logs those violations while retaining legacy delivery, including
unadmitted sessions at the frame gate. Admission enforcement still refuses
registration without proof. Once every broker runs this build and normal traffic is clean, set frame policy to
`enforce` per node and restart that broker. Old brokers' bare or absent source
addresses are the main mixed-version risk; do not enable enforcement early.
`off` disables frame-policy checks. Explicit malformed or zero forwarding hop
limits remain refused in all postures.

Local ingress uses the Unix WebSocket endpoint (`noded.unix_socket`, otherwise
the configured runtime's `noded/bus.sock`). Only kernel SO_PEERCRED grants local
origin. TCP, including loopback and this node's own address, never does. The
existing plaintext mesh listener remains for the D2 migration; its off/observe
frame postures allow legacy delivery without local authority. Enforce rejects
unproved TCP application clients. There is no additional trusted loopback lane.

The shared Bus client accepts `unix:///absolute/path` in `MIX_NODED_URL` and
uses Unix discovery by default. `MIX_BUS_SOCKET` and `noded.unix_socket` select
the endpoint. Existing explicit TCP URLs remain legacy, unverified clients.
Monitor and logger use the broker's Unix endpoint. Desktops with a configured
broker account retain the existing verified Unix client path.

With a verified inventory the broker now challenges TCP peers even when
admission is off: that setting controls registration refusal, not whether an
identity can be learned. Diallers additionally challenge the broker and pin
its proof to the member named by the endpoint. Old brokers cannot answer this
reverse proof. Waiting for that answer adds a bounded **one second per dial**
to an old broker, including a re-dial following inventory reload. Off/observe
permit legacy fallback after that wait; enforcement refuses the unproved
connection before registration, publication or application writes.
Observe permits their ordinary traffic but supplies no
authenticated mesh-service assertion; protected cross-node deliveries can
therefore fail during a mixed-version upgrade. Keep enforcement disabled until
both ends support mutual proof. Legacy D2 links remain plaintext and origin-only;
TLS links use the channel-bound authentication described above.
The D2 signing seed is loaded once at startup for both directions. Restart the
broker after changing its seed file, before the old credential window closes.

Accepted reloads retain connections only while their exact proven key remains
current and uniquely assigned. Overlapping current keys coexist. Inbound
tracking starts at proof completion, before registration, in every posture;
outbound attempts recheck the live key authority before publication. The
immutable key lookup is built per accepted revision. Teardown notification and
transport cancellation target less than one second after accepted reload.

## Offline broker principal preparation

`scripts/broker_profile.mix` is an opt-in module for a fresh, explicitly marked
offline rootfs. It never calls a service manager or changes the running system.
`plan(spec)` describes a separate desktop client and broker UID/GID, protected
profile, native Unix endpoint, public client config, dedicated broker environment
and a per-instance `noded@N.service` override. `prepare(root, spec)` requires
namespace root, a canonical root-owned rootfs and an inode-bound
`.broker-rootfs.json` marker. It refuses `/`, existing broker profiles, account
collisions, mismatched desktop accounts and symlink paths. It creates the broker
only in that rootfs's passwd/group/shadow databases. Desktop state is never
recursively chowned. Existing system units and single-account desks are unchanged.

The override retains the existing unit/dependency name, resets inherited desktop
environment files and supplementary groups, and selects the private broker
account, runtime and environment. Client config/environment is separate and
does not change application VAR or the desktop runtime. Preparation creates no
authority baseline, seed or trust anchor. Its receipt starts incomplete and
becomes prepared only after files are synced. No production image consumer or
mesh distributor is enabled by this module; activation still requires effective
unit verification, signed authority and connection identity gates.

`tests/broker_namespace_test.mix` uses real subordinate UIDs in private user,
mount, PID and network namespaces, with native Mix and a candidate noded copied
into a disposable chroot. It preserves the enclosing container's
masked proc mounts. Disposable signing material stays in its temporary root and
is removed. It checks a signed authority query as the desktop UID, repeats the
query after isolated TCP becomes unreachable, verifies private witness/lock
ownership and mode0600, requires actual desktop write denials, and checks parent
mount/loopback state and temporary-root cleanup. With a separately built Mix
candidate that includes the server-PID pin, it also proves that a fresh
correct-PID query succeeds and a wrong-PID query refuses while TCP is unreachable.
It does not boot systemd.

Run through native Mix; `MIX_TEST_NODED_EXE` may select a CBC-built candidate file
for this fixture only. It is copied and hash-checked, never installed to gold.
The candidate must include the explicit private witness/lock mode invariant.
Set `MIX_TEST_MIX_EXE` to an absolute candidate Mix path plus
`MIX_TEST_MIX_SHA256` and `MIX_TEST_NODED_SHA256` to exercise the opt-in
PID-pin proof. The fixture verifies each hash before copying that binary;
without these variables, `pid_pin_tested` stays false. Both candidates remain
uninstalled.
The fixture needs Linux subordinate UID/GID mappings and native `unshare`,
`chroot`, `mount`, `ip`, `readelf`, `ldconfig` and `pidwait` facilities; unsupported
namespace policy is a fixture environment failure, not a noded regression.
