# inputd

The input broker: keys become Bus commands, the user keys stay the user's, and the keyboard grab follows the VT.

Documentation: [`docs/inputd/README.md`](../../docs/inputd/README.md).

- `src/`: the daemon (Bus service, keymap file, evdev and uinput, the VT-aware grab).
- `../../libraries/inputd-schema`: the wire contract and user-key definition.
- `../../libraries/inputd-core`: the pure resolver and press-latched interceptor.
- `units/`: `inputd.service` (Bus only) and an example drop-in that turns the grab on.

Tests: `cargo test -p inputd -p inputd-core -p inputd-schema`. None needs a device.
