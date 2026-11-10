# Mix Term beside Rust Term

The accepted `lab/parity-term` specimen is merged into main's generic host.
The bundle remains `apps/term`: the host separates the canonical command
namespace (`term`) from the registered service (`term-mix`). The native
adapter maps `term-mix.tabs` to the authored `term.tabs` handler; discovery
and topic publication use the registered name. The numeric `instance` in
terminal replies remains a fresh process token, used to reject stale input.

Launch from an installed tree:

```text
/opt/mix/bin/viu run /opt/mix/share/mixviu/apps/term --service term-mix --app-id mixviu.term-mix --title "Term (Mix)"
```

`/opt/mix/bin/term-mix` supplies those options and forwards further arguments,
including `--comp NAME`, `--noded-url URL` and `--headless`. Put application
arguments after the host options. The frame and native title both use
`Term (Mix)`. The distinct app id prevents compositor show/focus matching
from selecting the Rust terminal. Rust Term's executable, launcher, service
and app id are unchanged.

## Install recipe (operator action; not performed by this PR)

The normal gold path builds `tools/apps_bundle.mix` into `apps.tar`, rooted
at `/opt/mix`. It includes the runtime bundle at `share/mixviu/apps/term`,
the shared library at `share/mixviu/libraries/viu/mix`, executable
`bin/term-mix` and `share/applications/mixviu.term-mix.desktop`. Install the
whole reviewed release set through that path; no separate Term copy is needed.
The table below records the corresponding file destinations.

Build the exact reviewed commit on CBC. Retain the build receipt and hashes.
Build `mix` separately to preserve its shell feature profile; then build
`mixd` and `viu`. Use `--locked --release` for both commands. Preserve the
previous destination files and their receipts before copying anything.

| Source at the reviewed commit | Destination | Mode |
| --- | --- | --- |
| CBC `target/release/viu` | `/opt/mix/bin/viu` | 0755 |
| CBC `target/release/mix` | `/opt/mix/bin/mix` | 0755 |
| CBC `target/release/mixd` | `/opt/mix/bin/mixd` | 0755 |
| `apps/term/` (complete directory) | `/opt/mix/share/mixviu/apps/term/` | directories 0755, files 0644 |
| `libraries/viu/mix/` (complete directory) | `/opt/mix/share/mixviu/libraries/viu/mix/` | directories 0755, files 0644 |
| `apps/term/term-mix.mix` | `/opt/mix/bin/term-mix` | 0755 |
| `apps/term/mixviu.term-mix.desktop` | `/opt/mix/share/applications/mixviu.term-mix.desktop` | 0644 |

The relative `../../libraries/viu/mix` imports require that share-directory
layout. The host embeds the native terminal, shaders and fallback fonts;
there is no additional terminal shared library. Keep the installation's
existing asset set, icons, settings and broker configuration. Ensure the
desktop session includes `/opt/mix/share` in `XDG_DATA_DIRS`, or expose this
desktop entry through that session's existing launcher mechanism. Select
the desk's compositor and broker with the forwarded options if its defaults
differ. No compositor, systemd or Rust Term launcher changes are required.

Undo: close `term-mix`, remove only the new launcher and desktop entry, and
restore the backed-up binaries and shared Mix files. Remove the newly added
bundle only if no other launcher uses it. Rust Term continues independently.

## Checks

The CBC gate is `mix tools/ci.mix all`. It includes the Viu bundle/contract
tests and all thirteen Term Mix suites. `mix tools/term-smoke.mix` builds
the native executables, then explicitly runs the ignored `term_host` smoke:
two simultaneous services, HELP descriptors, real shell input/output and
last-tab reply delivery before process exit. This is headless integration
evidence; desk4 comparison remains a separate operator step.
