# Host udev forwarder

A Mix Bus citizen copies the host udevd's processed group-2 datagrams into
configured privileged LXC network namespaces. This lives under `services/`,
following the existing Mix citizens' `scripts/` and `units/` convention.
There is no second daemon, transport, udev database reader or wire encoder.

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

`udev_forwarder.actions` lists `post-start` and `pre-stop`;
`udev_forwarder.execute` accepts `{id, params:{ct:"102"}}`. Only configured
IDs are accepted. The Proxmox hook calls these synchronously over the Bus.
Startup enumerates CT init PIDs with `lxc-info`; it checks the PID again after
creating each socket. No polling. Pre-stop closes the old socket, post-start
opens the new namespace; restarting the service reconciles every configured CT.
Other send failures drop the socket and log once, until another start hook or
service restart. No-listener drops retain the socket so a compositor starting
later can receive future events. A socket pins its namespace: an empty multicast
group alone cannot establish that a CT has died.

Prerequisites: a PVE host with the new Mix binary at `/opt/mix/bin/mix`, the
native noded broker configured for root clients, `local` snippets storage at
`/var/lib/vz/snippets`, configured privileged CTs with separate net namespaces,
and the host `/run/udev` plus relevant device nodes mounted into them. libudev
can disable its subscription if `/run/udev/control` is absent and `/dev` is not
devtmpfs; forwarding cannot repair a monitor that never subscribed.

Run `mix services/udev-forwarder/scripts/install.mix` from this source bundle
on the authorised PVE node. `--check` writes nothing and verifies installed
files, executable hook, enabled/active unit, storage path and CT hook settings.
An existing `/opt/mix/etc/udev-forwarder.conf.mix` is preserved and validated;
otherwise the bundle default selects CTs 102, 103, 104 and input/hidraw/sound.
The installer preflights all CTs and refuses to replace another hookscript.
Snapshot sections are preserved. It installs scripts under
`/opt/mix/lib/udev-forwarder`, the unit under `/etc/systemd/system`, and
`local:snippets/mix-udev-forwarder.mix`, then enables/restarts the service.
It never starts or restarts CTs. Re-running a converged install does not
restart the service. When changing only config, restart the service explicitly.

Host edge programs: lxc-info (service); mkdir/chmod/test/systemctl/pvesm
(installer). Hooks call only Mix and the native Bus. Logs contain event action,
subsystem, device node name and target CT IDs, plus loss/lifecycle errors;
datagrams and other properties are never logged.

## Proof

Parser/filter and ordered Buffer/overflow tests live with lib-mix.
`mix services/udev-forwarder/tests/namespace.mix` is the root-only CBC proof:
it subscribes in namespace A, creates B with `run_argv`/managed spawn, triggers
a real input change through udevd, then copies those exact bytes into B.
A libudev monitor there must print the same SEQNUM. Its bounded retries handle
monitor startup only, not service lifecycle. It fails if sysfs writes or
unshare are denied, or no real udevd message arrives; there is no fake fallback.
Test-only edge programs: unshare, sleep, readlink, stdbuf, nsenter, udevadm.
Live installation, trigger and physical replug acceptance belong to the
orchestrator; no host or CT deployment is implied by these tests.
