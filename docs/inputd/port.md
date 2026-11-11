# MixViu inputd port

Derived from frozen MixOS revision `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
The shared schema and core Rust sources match that revision byte for byte;
no library changes are needed. Their manifests already use MixViu paths.

The daemon adds no foreign dependencies: its Linux device implementation
calls libc ioctls directly, without libinput, libudev or xkbcommon crates.
The compositor's existing device dependencies are unchanged.

The default keymap is `<etc>/inputd/keymap.json`, resolved by paths.
An explicit `--keymap` wins; the standard STATE_DIRECTORY override remains
for existing callers. The unit passes the rendered etc path explicitly.
An installer must move the old state keymap directory into the chosen etc
root, preserve existing data/backups and remove the old directory compatibility
link only after reader/writer acceptance. A file symlink is unsuitable:
atomic save replaces the file. viud watches the containing directory.
The unit creates that directory before starting and permits writes to the
etc root under ProtectSystem=strict; no systemd-managed state/run directory
is used. The shipped unit serves Bus only, without grabbing a device.

## Protocol

The service remains `inputd`. Commands are `HELP`, `input.query`,
`input.bind`, `input.unbind`, `input.mode`, `input.reload`, `input.key`,
`input.pointer.move`, `input.pointer.button`, `input.pointer.scroll`.
Dispatch and injection code, descriptors, JSON keys, errors and return codes
(0 success, 10 error) are unchanged. Keymap schema remains version 1.
The returned keymap path changes to the resolved etc location.

## Reserved-key finding

The definition currently lives in `libraries/inputd-schema/src/lib.rs`:
`is_user_key` reserves bare F1–F12 and arrows with only nonempty combinations
of RightCtrl, RightAlt, RightShift. `user_keys` enumerates that set.
`inputd-core::Resolver::resolve` swallows unbound user keys in normal mode;
transparent mode forwards everything, and explicit passthrough rows can
forward keys too. `Interceptor` latches the press decision through release.
inputd's grab/reader code omits swallowed events from its virtual keyboard;
observe and Bus-only modes cannot suppress physical events.

viud's `policy::user_keys::UserKeys::edge` uses the same Resolver/Interceptor,
before normal compositor bindings. It takes swallowed edges; the compositor
does not forward those to clients. Its loader admits only `is_user_key` rows,
and the locked resolver suppresses user actions. Both components agree with
each other on the old set, including its forwarding exceptions.

They disagree with PR #68 (`fix/reserved-desk-keys`, inspected at e17bb8b7),
whose `action::keymap::reserved_key` reserves F1–F12 regardless of modifiers,
and either-side Ctrl/Alt arrows without Shift. Examples: Ctrl+F3 and
LeftCtrl+Left are reserved there but not by the old predicate;
RightShift+Left is swallowed by the old predicate but not reserved there.
Neither this port nor that PR changes inputd's old predicate. This port
does not alter policy; app admission alone does not prove physical suppression.

## Headless coverage

All daemon tests are retained: fake grab devices/console/emitter, event
encoding, injection validation, command dispatch, persistence, recovery,
permissions (subprocesses drop uid when run as root), routing and CLI.
No test opens an input device or needs a seat. Real kernel grabs, uinput
creation/injection, VT switching, live Bus delivery and viud forwarding on
a real seat remain unverified. No install or desktop action is part of this port.

CBC cbc2 job `20261112-070225-65912e13`: 99 daemon tests, 26 core tests,
14 schema tests passed (139 total), zero ignored or failed; zero doctests.
