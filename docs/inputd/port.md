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
Existing bindings are not carried automatically: the installer must copy the
old state directory's keymap and backups before enabling the new unit.
The unit invokes a scoped tmpfiles rule before starting, creating/correcting
the root-owned configuration directory to 0700 idempotently. Only that
directory is writable under ProtectSystem=strict. Atomic writes always use
0600, including replacements of existing files with broader permission bits.
No systemd-managed state/run directory is used. The shipped unit serves Bus
only, without grabbing a device. Manual privileged runs still honour
STATE_DIRECTORY and paths root overrides; the unit pins its --keymap path.

Logging uses an optional raw override and the original non-fatal info fallback.
No installed defaults file is read by inputd logging. Invalid overrides are
reported in one stderr line; even a failed stderr write is ignored.

Startup audit against the frozen main: the two fallible logging operations
introduced by round 1 are removed. Remaining `?` operations (VT validation,
optional device discovery, runtime creation, incoming stream and Bus serving)
are unchanged. Keymap preparation was extracted for a pre-Bus test, without
changing its recovery or in-memory fallback. No other setting() call exists
in inputd. Manual runs using default_path still require a valid paths layout;
invalid root/layout configuration can fail resolution, while the unit's
explicit --keymap bypasses that lookup. Missing env.mx is not a startup error.
Provisioning failures (permissions, read-only storage, absent systemd-tmpfiles)
can fail the pre-start step just as failed StateDirectory provisioning could;
the deployed systemd role must include its tmpfiles tool. No live unit start
or seat acceptance is claimed here.

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

Round 2 adds optional/invalid logging override tests, empty-environment
pre-Bus startup through CLI/keymap/resolver/runtime, real executable help
startup, and scoped tmpfiles ownership/0700/idempotence checks in a synthetic
root. The atomic-save test now verifies new files and replacements are 0600.
CBC job `20261112-082710-2857184e` passed 102 daemon unit tests, 2 integration
tests, 26 core and 14 schema tests (144 total), none ignored or failed.
The worker refreshed the workspace lockfile offline: zero package changes,
so the existing committed lockfile remains byte-identical. Test temp roots
are under the worker target directory; no input device or seat is accessed.
The actual systemd mount namespace and deployed reader access still need
installer/desk acceptance; the test does not start a live unit.
