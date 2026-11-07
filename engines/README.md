# Engines

MixViu has exactly four engines, each its own package and binary:

| Engine | Package | Role |
|---|---|---|
| `engines/mix` | `mix` | The Mix shell profile (the login shell and script runner). |
| `engines/mixd` | `mixd` | The Mix service profile and the headless daemons (argv[0] personalities). |
| `engines/viu` | `viu` | The application and shell UI runtime (furniture host and Mix apps). |
| `engines/viud` | `viud` | The Wayland compositor, without the egui widget toolkit. |

Build each engine with its own `-p`; a workspace-unified build mixes the two
Mix profiles' features. Shared code lives in `libraries/` (`mix-shell`, the
interpreter `lib-mix`, the Viu runtime `lib-viu`, `ui`, ...). Mix application
bundles live in `apps/`.
