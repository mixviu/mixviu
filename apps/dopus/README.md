# Dopus on generic Viu

Run the Mix file manager with `viu run apps/dopus [LEFT [RIGHT]]`. It uses the native application config directory and forwards paths to a running instance. `--headless` serves the core without a window; `--print-config` resolves config before connecting. Tests can isolate persistence with `--profile PROFILE_DIR`. Use disposable files for mutation tests. The full inventory and acceptance state are in [the port record](../../docs/ports/dopus.md).

The generic executable also hosts the reusable file chooser: `viu run apps/dopus --choose-service PROFILE_DIR --service files`. Application behaviour and composition remain in `chooser.mix`, `chooser-engine.mix` and `chooser.scene.mix`; Viu owns native viewports, bounded runtime delivery, captures and reply fences.

`core.mix`, `operations.mix`, `reducer.mix`, `presentation.mix` and `app.mix` own file-manager policy. The app uses the shared gold frame, navigation rail, virtual tree tables, dialogs, metadata and preview facilities. Intra-app traffic stays in process; live settings use the native authority.

The former `viu-dopus` executable and Rust comparator are retired. Their source remains in git history; clean pinned MixOS gold is the oracle. Mix chooser and raw-name contract tests now live with Viu.

Agents can choose the active pane with `dopus.execute {id:"nav.pane-left"}` or
`dopus.execute {id:"nav.pane-right"}`. These idempotent actions are listed by
`dopus.actions` and work headlessly. They preserve locations and selections;
subsequent selection and file actions use that pane. `nav.switch-pane` still
toggles. `dopus.open` with a pane target changes its location and preserves
the active pane.

Rust checks and builds run through CBC. `apps/dopus/gate.mix` checks the bundle, scan refusals, byte-preserving operations, chooser contracts and release 2,000-row paths, then saves the generic release executable and its source/hash receipt. A scoped gate alone does not grant parity or authorise installation.
