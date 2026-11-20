# toolsd

Mix wake citizen and separate unprivileged HTTPS queue drain. Ported from the
MIT OR Apache-2.0 Mix tools service, copyright 2026 Mark Constable; original
copyright and licence headers remain in each asset. No Rust personality.

`toolsd.wake` chooses no work: it only asks systemd to start
`toolsd-drain.service`. The `toolsd-wake` account has no credential. The polkit
rule grants it only that start action. The `toolsd` account claims authorised
work, rechecks the immutable registry, validates parameters and constructs
argv without a shell. External edge dependencies are `/usr/bin/flock`,
`/usr/bin/systemctl`, `/usr/bin/whois`, `/usr/bin/dig` and
`/usr/bin/traceroute`. Executable ownership, parent permissions and resolved
targets are checked before execution, including Debian alternatives chains.

Bundle scripts live at `/opt/mix/lib/toolsd`. The bundle takes its registry
from webd's `vhosts/apps/tools_registry.mix`; the source parity test prevents
drift. Scripts and their parents must be root-owned and not writable by either
service account (scripts may be 0444). The archive records owner 0.

The layout renderer ships all three units, sysusers, tmpfiles and polkit rule
under `<prefix>/share/toolsd/units`. Host installation links/copies these into
the matching systemd, sysusers.d, tmpfiles.d and polkit directories. Apply
sysusers before tmpfiles. The root-owned persistent runtime directory and
pre-created toolsd-owned lock inode must survive between oneshot drains.
The wake uses `MIX_NODED_URL=unix://<run>/noded/bus.sock`; the host's private
node policy must admit the toolsd-wake UID as service `toolsd`. Socket access
alone does not grant service registration authority.

Host policy stays outside the bundle: `<etc>/toolsd/env` supplies
`TOOLS_BASE_URL=https://queue.example.test` and `TOOLS_CA_FILE` (an absolute CA
file); `<etc>/toolsd/secret` is delivered only by `LoadCredential=tools_secret`.
No secret value belongs in an environment file, unit, argv or log. Configure
the precise web endpoint address in a private drain unit drop-in. The unit's
`IPAddressDeny`/`IPAddressAllow` use cgroup-BPF: they can silently do nothing
in unprivileged Proxmox CTs. They remain useful on VMs and privileged hosts.
`SystemCallFilter` is still enforced in those CTs. Keep the drain masked in a
CT until the hub has separately applied and proved a real egress guard.

The drain stops after eight runs or its 600-second monotonic wall budget,
reserving 100 seconds before each claim. Trace resolution, hops and progress
retries share one 60-second monotonic deadline; claim and final report have
15/20-second bounds. Eight worst-case envelopes fit the 900-second unit bound.
Resolved trace addresses must all be public unicast; private, special-use,
documentation, mapped IPv6 and transition targets are refused before exec.
Tools have fixed deadlines/output caps. A five-minute timer (two minutes after boot,
ten-second accuracy) covers lost wakes and truncated drains: queued work may
wait that full interval. Host systemd provides the 900-second final bound.

Run `mix services/toolsd/tests/all.mix` for every toolsd suite and the bundle
closure. Runtime edge fixtures never contact a
public queue or start host units. An isolated host rehearsal must separately
prove polkit, credential delivery, Unix registration, executable availability
and effective egress confinement. Installation, host policy and rollback belong
to the private hub; this port neither enables units nor changes a host.

The companion nftables guard is **opt-in**. Its renderer and probe ship at
`<lib>/toolsd/egress_render.mix` and `egress_probe.mix`; the pure rule renderer
is `lib/egress.mix`. Neither the bundle nor layout installer runs them or
loads firewall rules. The renderer runs only `nft --check --file -`, then
writes a private candidate file. The hub owns its application, persistence,
stop/start ordering and rollback (`delete table inet toolsd_egress` only).
It never flushes the host ruleset. Applying the candidate atomically replaces
only this service's table; its accept policy does not override other tables.

Render explicitly with `mix <lib>/toolsd/egress_render.mix --config PRIVATE.conf.mix
--out PRIVATE.nft`. The private strict-data config contains a non-root `uid`,
`node_public` (a list of public IP literals), and `mode` (`auto`, `cgroup` or
`skuid`). Auto prefers `socket cgroupv2 level 2
"system.slice/toolsd-drain.service"` when the hub sets `refresh_cgroup: true`
and nft's kernel check succeeds; otherwise it checks the `meta skuid` fallback.
Explicit cgroup mode fails closed if unsupported. If neither selector checks,
no file is written. nft needs appropriate in-CT privileges even for check-only
operation; an unsupported/denied nft setup is a blocker, never a guard.

The cgroup must exist when checked/applied. nft resolves its path to an ID:
the hub must refresh the rule **before any network work on every new drain
cgroup**, including after restart/reboot, and prevent work until refresh and
probe succeed. `refresh_cgroup: true` acknowledges that lifecycle obligation.
Without a proven refresh hook, use the dedicated UID fallback, which survives
cgroup recreation. Resolve the UID on the destination; do not share that UID
with other services. The renderer reports the selector and refresh obligation.

Both modes deny 0/8, 10/8, 172.16/12, 192.168/16, 169.254/16, 100.64/10,
127/8, multicast 224/4 and ff00::/8, reserved 240/4, limited broadcast,
unspecified/loopback IPv6, fc00::/7 and fe80::/10. IPv4-mapped IPv6 is denied
as ::ffff:0:0/96 when visible as IPv6; mapped IPv4 sockets also hit the IPv4
deny rules. Named `deny4`/`deny6` counters record rule hits.
Exceptions precede the deny: only DNS port 53
(UDP and TCP) to 127.0.0.53 and the configured node public addresses. Native
Unix Bus traffic bypasses inet hooks, so needs no IP exception. A legacy TCP
Bus link can supply `noded_tcp: {address: "127.0.0.1", port: 4200}` for an
exact address/TCP-port exception; it does not open the whole noded subnet.
The UID guard affects all IP sockets owned by that account, not toolsd-wake.

After authorised application, the hub runs the **root supervisor**:
`<prefix>/bin/mixd run <lib>/toolsd/egress_probe.mix --config PRIVATE.conf.mix
--user toolsd`. It reads named counters through an absolute nft executable,
checks the account UID, then uses absolute `runuser` to launch a credential-free
worker as the drain. The worker verifies identity, creates live IPv4/IPv6
loopback listeners, attempts five-second connects, and checks A answers over
UDP/TCP via 127.0.0.53 (`dns_name`, default `example.com`). A pass requires both
connects to fail **and each corresponding deny counter to rise**, plus both DNS
checks. Errno is diagnostic only: IPv4 rejection commonly gives 113, IPv6 13.
Missing/reset/unchanged counters, successful connects, failed listeners and DNS
errors fail closed. The worker alone never attests a pass. The drain gains no
nft privilege or extra polkit authority; counter access stays with the hub.

Set config `mode` to the selected mode, not `auto`, and make this non-secret
config readable by toolsd. Quiesce other drain/UID traffic and do not reload
rules during the probe, so counter deltas belong to the test. UID mode needs
no cgroup movement. Cgroup mode requires the supervisor/worker to be in the
actual drain cgroup; any temporary hub test unit override must permit the root
supervisor's netlink counter reads, preserve the worker UID, and omit all queue
credentials. Do not widen the production drain's hardening for this check.
Record selector, counter deltas and result before unmasking; validate Bus parity
in the same isolated rehearsal. No credentials or tool queue are involved.
Rendering/unit tests do not establish CT enforcement. Applying this firewall
on a guest requires separate operator approval; this PR authorises no deployment.

Selector syntax and check-only semantics follow the
[nftables manual](https://netfilter.org/projects/nftables/manpage.html).
Trace special-use exclusions follow the IANA
[IPv4](https://www.iana.org/assignments/iana-ipv4-special-registry/) and
[IPv6](https://www.iana.org/assignments/iana-ipv6-special-registry/) registries,
conservatively excluding transition/protocol blocks. Documentation literals
are accepted for offline node-rule examples, but never as trace targets.
shwho revalidates MX-derived names and uses dig's explicit `-q` query slot:
[BIND dig's parser](https://github.com/isc-projects/bind9/blob/main/bin/dig/dig.c)
does not support a `--` terminator. Reverse lookup `-x` consumes a validated IPv4.
