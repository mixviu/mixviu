# Host udev forwarder

A plain long-running Mix program copies the host udevd's processed group-2 datagrams into
configured privileged LXC network namespaces. This lives under `services/`,
following the existing services' `scripts/` and `units/` convention.
There is no second daemon, transport, udev database reader or wire encoder.
Run `mix scripts/forwarder.mix [config.conf.mix]`; there is no `--serve`,
mixd or broker dependency. Config defaults to
`/opt/mix/etc/udev-forwarder.conf.mix`. An explicit path lets a lab copy run
without editing source or installing configuration.

Plain Mix enters the native event pump after script initialisation when `on`
handlers exist. The uevent socket keeps it alive and parks it on readiness.
The pump now retires a closed Bus stream in plain scripts while native sources
remain, rather than terminating the script or spinning on an exhausted stream.
Serve-runtime transport-close behaviour stays unchanged. No new receive
primitive is required; the uevent primitives are unchanged in this round.

## Primitives

Linux only, implemented in `libraries/mix/src/uevent.rs` using existing libc.
All handles belong to the evaluator generation and retire with it.

| Call | Contract |
| --- | --- |
| `uevent_watch({group:2, subsystems:["input"]})` | Watch handle; group 1 also supported. Default group 2, empty subsystem list accepts all. |
| `uevent_unwatch(handle)` | Discard pending records, cancel and join reader. |
| `uevent_sender(netns_path)` | Sender handle; dedicated short-lived thread opens the namespace, calls setns, creates socket, then exits. Requires uid 0 and namespace capabilities. |
| `uevent_send(handle, group, buffer)` | Send unchanged bytes. True on send, false for ECONNREFUSED (no listeners); other errors raise. |
| `uevent_close(handle)` | Close sender immediately. |

`on uevent.received` gets `$event.args`: `{watch, raw:buffer, properties:map,
uid:0, overflow, dropped}`. The receive side requires SCM_CREDENTIALS and uid
0, rejects truncated control/data, validates property ranges and UTF-8, then
filters SUBSYSTEM. It never changes the original datagram. Rejections produce
`{drop:reason}`, terminal failure `{closed:reason}`. Excluded subsystems are
intentional filtering, not loss. ENOBUFS reports overflow with unknown kernel
loss count; queue overflow reports an exact `dropped` count. Records remain
ordered, without deduplication or coalescing. Limits: 16 watches, 64 pending
records per uevent watch, 256 KiB datagrams, 128 senders per evaluator.
The existing string-only IncomingEvent envelope carries a private hex copy;
the evaluator restores Buffer before dispatch. Outbound bytes are that copy,
not a new libudev message or a serialization of the properties map.

## Lifecycle and installation

The Proxmox hook synchronously runs `systemctl restart udev-forwarder.service`
on post-start and pre-stop. It also does so on post-stop: pre-stop runs while
the CT init may still exist, so that first reconciliation can reattach it;
post-stop reconciles after init exits, leaving stopped CTs without a socket.
Startup enumerates configured CT init PIDs with `lxc-info`; it checks each PID
again after creating its socket. No polling or interprocess transport.
Each restart closes all old sockets and reopens the running CT namespaces.
It can lose events during the restart, normally about a second while a CT is
starting or stopping, including events for other running CTs. Slow startup or
failed lxc-info calls can extend that gap; events are not replayed.
Other send failures drop the socket and log once, until another start hook or
service restart. No-listener drops retain the socket so a compositor starting
later can receive future events. A socket pins its namespace: an empty multicast
group alone cannot establish that a CT has died.

Prerequisites: a PVE host with the new Mix binary at `/opt/mix/bin/mix`, the
new brokerless native-pump fix, `local` snippets storage at
`/var/lib/vz/snippets`, configured privileged CTs with separate net namespaces,
and the host `/run/udev` plus relevant device nodes mounted into them. libudev
can disable its subscription if `/run/udev/control` is absent and `/dev` is not
devtmpfs; forwarding cannot repair a monitor that never subscribed.

Run `mix services/udev-forwarder/scripts/install.mix` from this source bundle
on the authorised PVE node. `--check` writes nothing and verifies installed
files, executable hook, enabled/active unit, storage path and CT hook settings.
An existing `/opt/mix/etc/udev-forwarder.conf.mix` is preserved and validated;
otherwise the bundle default selects CTs 102, 103, 104 and input/hidraw/sound.
The installer checks all five uevent builtins in `/opt/mix/bin/mix`, preflights
all CTs and refuses to replace another hookscript. A binary missing a builtin
is refused before any writes, with an instruction to install a suitable gold.
Snapshot sections are preserved. It installs scripts under
`/opt/mix/lib/udev-forwarder`, the unit under `/etc/systemd/system`, and
`local:snippets/mix-udev-forwarder.mix`, then enables/restarts the service.
It never starts or restarts CTs. Re-running a converged install does not
restart the service. When changing only config, restart the service explicitly.

Host edge programs: lxc-info (service); mkdir/chmod/test/systemctl/pvesm
(installer); systemctl (hook). Logs contain event action,
subsystem, device node name and target CT IDs, plus loss/lifecycle errors;
datagrams and other properties are never logged.

## Proof

Parser/filter and ordered Buffer/overflow tests live with lib-mix.
`mix services/udev-forwarder/tests/check.mix` is the CBC root-only regression
harness. It copies only the built Mix executable into an isolated PATH (no
mixd), starts the actual forwarder in a private network namespace with an
explicit lab config and a stand-in lxc-info, and requires consumption of a
synthetic netlink fixture with no broker. It also exercises the real hook with
a stand-in systemctl; it never restarts a worker service or contacts a CT.
This checks the plain native event path, not real libudev acceptance. Library
regressions cover native progress after Bus closure and unchanged serve closure.
`mix services/udev-forwarder/tests/namespace.mix` is the root-only CBC proof:
it subscribes in namespace A, creates B with `run_argv`/managed spawn, triggers
a real input change through udevd, then copies those exact bytes into B.
A libudev monitor there must print the same SEQNUM. Its bounded retries handle
monitor startup only, not service lifecycle. It fails if sysfs writes or
unshare are denied, or no real udevd message arrives; there is no fake fallback.
Test-only edge programs: unshare, sleep, readlink, stdbuf, nsenter, udevadm.
Live installation, trigger and physical replug acceptance belong to the
orchestrator; no host or CT deployment is implied by these tests.
