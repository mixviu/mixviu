# Furniture input ownership: lab design stop

The client port has a concrete input-contract gap. No runtime or test source
was changed at this checkpoint.

The in-process scene host installs a synchronous button hook in
`engines/compd/src/scenes.rs`. Seat invokes it in
`crates/seat/src/pointer/input/button.rs` before decoration release handling,
native hot corners, popup grabs, world input and client hit-testing.

`scene-host/src/panels.rs::pointer_button_at` takes ownership while the corner
detector reports either a candidate or an engaged corner. Dwell is not needed.
It remembers the button across pointer departure, consumes its release and
only then toggles a panel or invokes its popup route. With a furniture menu
open, an outside press closes it and consumes the entire click too.

The native path in `world/src/comp/corners.rs::Corners::press` instead supplies
`engaged_corner()` and `engaged_dwell_ms()` to its press tracker. Before dwell,
it passes the press. Two existing tests make the distinction explicit:

- scene-host: `hotspot_click_is_consumed_before_dwell_and_fires_on_release`;
- world: `an_unengaged_press_is_not_consumed_and_another_output_resets`.

Concrete regression: press and release in the launcher corner before dwell
with an application below it. Today the application sees neither event and
the launcher pins on release. Simply replacing the scene hook with a client
subscribed to `corner.entered/left/clicked` lets that press reach the
application. A later Bus message cannot retract an already delivered event.
An unmapped launcher cannot intercept it through a layer-shell input region.
`comp.panel.*` manages panel membership/conceal and reservations; it does not
provide the early synchronous button veto currently installed by scene-host.

Options requiring an ownership decision:

1. Keep early hotspot ownership in viud. Add a session/owner-bound furniture
   claim, preserve the pre-hit-test press/release tracker and publish plain
   events to viu. Define claim loss, reconnect, session lock, popup priority
   and multi-button behaviour. This is the recommended parity route; panel
   contents and settings policy remain in viu.
2. Authorise the native dwell-only behaviour for client furniture. Smaller
   port, but changes fast corner clicks and fails the current scene contract.
3. Add always-mapped client hotspot surfaces. Changes input ordering against
   popup grabs/chrome and introduces lifetime/ownership surfaces. It needs
   explicit parity decisions and is not an equivalent mechanical port.

The existing pure panel clock can be driven by client frame callbacks, but
that does not resolve this earlier input ownership gap. No panel cold-start
or idle CPU result is claimed before a client implementation and runtime gate.
Task tooltip thumbnails remain deferred to B7; a future client must state
that previews are unavailable in its tooltip UI.

B5 remains unimplemented at this preflight stop. Its smaller tooltip route
would be a Bus hover topic emitted from decor, retaining presentation in viu;
the CPU route needs another output composition path in viud. No tooltip
behaviour or titlebar/theme ownership has been changed without the port.
