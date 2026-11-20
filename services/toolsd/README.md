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
the precise web endpoint address in a private drain unit drop-in. The bundled
deny rules retain private/management network confinement and permit only the
resolver stub exception; do not broaden those ranges for deployment.

The drain stops after eight runs or its 600-second wall budget. Tools have
fixed deadlines/output caps. A five-minute timer (two minutes after boot,
ten-second accuracy) covers lost wakes and truncated drains: queued work may
wait that full interval. Host systemd provides the 900-second final bound.

Run `mix services/toolsd/tests/toolsd_test.mix` and
`mix tests/mix_services_bundle_test.mix`. Runtime edge fixtures never contact a
public queue or start host units. An isolated host rehearsal must separately
prove polkit, credential delivery, Unix registration, executable availability
and effective BPF confinement. Installation, host policy and rollback belong
to the private hub; this port neither enables units nor changes a host.
