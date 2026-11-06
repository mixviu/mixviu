# Furniture boundary (Track B lab)

This branch is a discovery specimen and never lands on main. Runtime receipts
and measured values live in the worktree's `last.md`.

## Authority

Viud owns input routing and ownership, focus, grabs, hit-test order, reserved
space, output geometry, global corners/edges and session lock. Viu draws
furniture and owns widget state. Existing Mix sources in `share/scenes` retain
application behaviour. No code from `lab/viu-runtime` is imported here.

The resolved design stop was a fast corner click before dwell. The former
scene-host hook consumed both button events before chrome, popup grabs and
application routing. A client cannot retract an already delivered press.

The native shared seat button path now takes corner presses before those
routes. Its existing tracker keeps each matching release across pointer
departure, output loss, lock and renderer departure. Leaving the hotspot
cancels the action, never ownership of the release. The in-process default
retains its existing scene hook and legacy dwell behaviour.

Native global hotspots also own presses while furniture starts or disconnects.
An action requires the same live panel reporter, generation, namespace and
Wayland client claim at release. A hidden panel may bind its first layer role
during a press. Missing or replaced claims cancel actions; clicks never leak.

Viud publishes plain `corner.input` records: output, corner, phase (engaged,
dwelled, consumed, clicked, released), optional button, consumed flag and event
sequence. Existing corner topics remain, including modifier-bearing
`corner.clicked.v2`. Two extension bits leave the frozen base catalogue intact. The imported noded
reservation helper now reserves and attests these authority streams; integration
must carry that native broker change into mixd. A real broker test rejects
foreign publishers and verifies owner stamps on deliveries.

The changed production-source copy is
`libraries/test-broker/noded/props_reservation.rs::publisher_owner`. It now
reserves corner.input/entered/left/clicked/clicked.v2, panel.command and
chrome.action.hover for the service named by their topic prefix. Noded's
existing publication path then stamps broker_service and broker_origin.
`engines/viu/tests/authority.rs::furniture_events_refuse_foreign_publishers_and_attest_the_compositor`
uses the embedded real broker to reject a foreign publisher and check the
owner's delivered stamps, including replacement of forged inner headers.
P2 must carry this change into mixd's noded personality: changing the lab's
embedded broker does not update an installed broker. Without the reservation,
real corner/hover deliveries lack the owner stamp and viu rejects them.

Viu reports `comp.panel.mode` with output, edge, namespace and generation.
Viud computes panel holds and sends `comp.panel.command` verdicts. Exclusive
zones are layer-shell requests arranged by viud. Viu changes no compositor
world coordinates, reservations or routing hooks.

## Native chrome

The compositor-only service uses its existing authenticated Bus connection
for settingsd, service discovery and visible targets' titlebar forms.
`titlebar.changed` and connection generations fence refreshes. Menu/action
emitters preserve the old `menu.get` and `action.execute` request bodies.
`decor::appearance` prepares egui-free chrome from accepted settings and core
theme/design values. A parity test covers all shipped modes, schemes and
chrome styles. The default retains its in-process appearance push.

Action tooltips use the smaller Bus hover route. Retained
`chrome.action.hover` carries output, opaque identity, title, pressed state,
output-local anchor and output size, or null (also during session lock).
Viu keeps the old 500 ms delay, 400 ms sticky interval, press suppression,
card renderer and placement. Its tooltip layer has an empty input region and
no keyboard interactivity. Viud supplies geometry; viu owns timing and pixels.

## Client specimen

`engines/viu` builds binary `viu`, serving the `shell` protocol through the
copied transactional scene store and renderer on `toolkit::layer`. Production
code has no compositor Loop, typed registry or synchronous compositor callback.
Each edge has an output-bound layer. The pure panel model drives presentation
and motion; Wayland frame callbacks drive authored scene frame requests.
Hidden layers unmap buffers and wait for configure when remapped. Authored
extents are retained, including the shipped taskbar's 52 px. Accepted panel
preferences use the copied, tested preparation module. Default output comes
from viud's Bus state.

Viu consumes `comp.props.watch`, corners and panel verdicts. Mix taskbar
behaviour retains `comp.window.*` calls; widget events return to their authored
Mix citizen. Page selection, mode and resize requests live in the client;
viud retains final layer layout and input authority. Pinning requests focus once
after the reporter-bound layer maps. Locks and grabs win; repeated reports
never steal focus back from an application.

The first target is panels/taskbar and launcher. Dialog/toast/corner-menu
presentation, carousel ordering, layout readback and persistence of client
mode changes are follow-up work; unsupported commands refuse explicitly.
Thumbnails stay off until B7. Task tooltip cards say “Window previews
unavailable until B7”. This is not an accepted complete desktop or installer.

Toolkit layer adds dynamic size, margin, stratum, visibility, input transparency
and frame notifications, with unchanged defaults for existing programs.
Nested viud advertises its actual EGL render node when available. The old
measurement-only tooltip context explicitly discards unused texture deltas;
otherwise egui 0.37 panicked during startup.

## Gate method

`tools/gate_furniture.mix` builds and fingerprints exact native/default binaries
on CBC. Owned processes use private brokers and Wayland sockets. Native viud
plus viu must map a panel layer, reserve space and answer `shell.ping`.
An owned nested native compositor under a headless parent exercises real
human corner injection and requires consumed/clicked/released events at viu.
The headless human seat's intentional refusal is unchanged.

Each headless profile receives the same idle panel fixture and 180 paced shm
damage frames. An opt-in CLOCK_THREAD_CPUTIME_ID span covers the full headless
draw call; /proc CPU ticks provide a coarser whole-process measure. Client
first commit and first frame callback include GPU/Wayland startup. The prior
58 ms in-process figure is context, not an equivalent end-to-end sample.
No performance improvement is inferred from the architecture.

The measured a2de606 sample reached its first panel commit at 2705.825 ms
and first frame callback at 2709.018 ms from viu runtime startup. The timer
includes Bus setup, five independent layer connections/threads, EGL/wgpu
initialisation, font setup, scene acceptance and first presentation. Those
stages were not separately timed, so no numeric GPU-versus-host split is
claimed. The test loads a scene fixture over the Bus; it starts no Mix
behaviour process. Mix startup is therefore unmeasured, not part of a
measured 2.7-second decomposition. Multiple GPU contexts are a plausible
contributor, not a measured attribution. Startup tuning is outside this
close-out round.
