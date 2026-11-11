# inputd

The input broker: keys become Bus commands, the user keys stay the user's, and the keyboard grab follows the VT.

Documentation: [`docs/inputd/README.md`](../../docs/inputd/README.md).

- `src/`: the daemon (Bus service, keymap file, evdev and uinput, the VT-aware grab).
- `../../libraries/inputd-schema`: the wire contract and user-key definition.
- `../../libraries/inputd-core`: the pure resolver and press-latched interceptor.
- `units/`: `inputd.service` (Bus only) and an example drop-in that turns the grab on.

Tests: `cargo test -p inputd -p inputd-core -p inputd-schema`. None needs a device.

The keymap lives in the **configuration directory**, `<etc>/inputd/keymap.json`.
The root unit pins that path with `--keymap`. Manual privileged runs still
honour `STATE_DIRECTORY`, the paths root overrides and an explicit `--keymap`.

**Existing bindings are not carried automatically.** Before enabling the new
unit, the installer must copy the old state directory's `keymap.json` and
backups into `<etc>/inputd`, retaining the old data until acceptance. The
directory must be root-owned and mode 0700; the keymap must be mode 0600.
The scoped tmpfiles rule creates/corrects the directory idempotently. The
daemon's writable sandbox is that directory only. If no new keymap exists,
the daemon seeds the original built-in defaults, without personal bindings.

Logging is optional: the standard `RUST_LOG` filter takes precedence over
the inputd override. Missing or invalid filters fall back to the original
`info` behaviour; ignored invalid overrides print one stderr line. No
installed defaults file is needed for logging or headless startup.
