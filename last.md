# Four engines, compd -> viud, mixos -> mix: cut-over branch

Branch `cutover/four-engines`, from `lab/gold-parity` 4755e9e, with
origin/lab/gold-parity merged in four times (through 9bcfeee, not yet f1c5b71; merges only, no
rebase or force push). Move commits are separate from edit commits (7404a61,
eaf228c, 2fdb84c). Full package/path mapping and file renames:
`provenance/four-engine-layout.md`. Variable registry: `etc/env.conf.mix`.

## 1. Four engines (first checkpoint)

| Old | New |
|---|---|
| package `system` (bins `mix`, `mix-standalone`), `engines/mix` | package `mix`, bin `mix`, shell profile only |
| package `mix` (interpreter lib), `libraries/mix` | package `lib-mix`, lib name `mix` |
| package `viu` (Viu runtime lib), `libraries/viu` | package `lib-viu`, lib name `viu` |
| package `viu-engine` (`viu-client` in older scripts), `engines/viu` | package `viu`, bin `viu`, lib `furniture` |
| package `compd`, bin `compd`, `engines/compd` | removed; package/bin `viud` owns the sources in `engines/viud` |
| packages `scene-host`, `overlays` (in-process egui UI) | removed |
| `engines/mix-shell`, `engines/compd/crates/ui` | `libraries/mix-shell`, `libraries/ui` |

Scripts: `-p system` -> `-p mix`, `-p mix` -> `-p lib-mix`, `-p viu` ->
`-p lib-viu`, `-p viu-engine`/`viu-client` -> `-p viu`. Lab-only drivers carry
a LAB EVIDENCE ONLY note (any compd/oracle step needs gold compd from mixos
8779f33): p2_gate, p2_rehearsal, p2_prepare, p2_furniture_debug,
gate_furniture, gate_comp_split, engine_split_smoke,
engine_split_size_ablation, check_comp_profiles, check_native_chrome,
gold_parity_gate, integrated_gate, integrated_prepare, app_ports_gate/prepare,
email_list_gate, email_wire_prepare, bulk_prepare, backdrop_accept_gate,
backdrop_parity_gate, docs/ports/parity-method.md. `tools/cutover_gate.mix`
is the four-engine gate (modes gate, fmt, lock, builtins).

Checkpoint gate (cutover-gate-5d1ec61, cbc2): PASS, 10,484 passed / 0 failed /
90 ignored vs base 10,735 / 0 / 100 (integrated_gate 20261107-111516-437fd7b1).
The 251 + 10 difference is exactly the removed code (counted at base,
cutover-removed-4755e9e): scene-host 232 + 10 ignored and 3 registry,
overlays 1, compd bin unit tests 13 (duplicates; viud still runs them),
bus_wake 1 (inproc-ui only), mix personality_name 1.
Release sizes (bytes) base -> 5d1ec61: mix 20,521,752 -> 20,518,872;
mixd 65,896,480 -> 65,896,416; viu 55,726,024 -> 55,722,568;
viud 32,489,704 -> 32,489,144. viud tree: 0 egui/toolkit lines.

## 2. compd -> viud (Mark, 2026-11-07)

Everything compd became viud: units `viud@`/`viud-headless@` running
`bin/viud`, `COMPD_*` -> `VIUD_*`, user-facing `viud:` messages and log
targets, `libviud_wlcs.so` (lib `viud_wlcs`), `viud.ftl`, the settings and
Bus-registry owner `viud`, the `viud.truth` control, comments. No fallbacks.
The Bus service `comp` and its `comp.*` commands keep their names.
Gate after the rename (cutover-gate-43de632, cbc3): every test step green,
10,484 / 0 / 90; one rustfmt line, fixed in e32beb4.

## 3. mixos -> mix (Mark and Fable, 2026-11-07)

Prefix `/opt/mix`; `MIXOS_*` -> `MIX_*` by the Fable rules (test-only names
`MIX_TEST_*`, engine second word, `*_BIN` directory / `*_EXE` file); root
variable `MIXOS` -> `MIX_ROOT`; directories `/etc/mix`, `/var/lib/mix`,
`/run/mix`, `/var/cache/mix`, `~/.config/mix`, `~/.local/{share,state,log}/mix`,
`~/.cache/mix`, `$XDG_RUNTIME_DIR/mix`; systemd `*Directory=mix/<svc>`,
`User=mix`; `mix-headless.slice`; `[package.metadata.mixviu]`; the `mix:`
logical path scheme, `mix_path` builtin, `mix` cargo features (assets, dnsd);
prose MixOS -> MixViu. Nothing installs, writes or links under `/opt/mixos`.

No collisions with existing `MIX_*` names: `MIX_BIN` was only a placeholder
word in a lab usage line (reworded). Existing test names moved to
`MIX_TEST_*`; `MIX_UNDER_TEST` -> `MIX_TEST_ACTIVE`; `MIX_REMOTE_BIN` ->
`MIX_REMOTE_EXE` (a file). Four `MIX_TEST_*` delays/fault hooks
(`ADMIT_DELAY_MS`, `CLAIM_DELAY_MS`, `RESERVE_HOLD_MS`, `RESULT_TORN`) are
still read by runtime code without a test gate: the process tests drive the
built binary, so a cfg(test) gate would remove them; needs a decision.
`etc/env.conf.mix`: 133 names, loads with load_data, matches every
`MIX_[A-Z0-9_]+` token outside vendor/, provenance/ and Cargo.lock (a few
entries are Rust constants or a file name sharing the spelling; said so).

Shell versus product directories: the Mix shell keeps its own files in
`$XDG_STATE_HOME/mix` (`~/.local/state/mix`: history, stats). It has no files
in `~/.config/mix` or `$XDG_RUNTIME_DIR/mix`. viud's own files now sit in its
owner subdirectory (`~/.config/mix/viud`, `~/.local/state/mix/viud`,
`~/.local/share/mix/viud`, `$XDG_RUNTIME_DIR/mix/viud`). NOT yet done (needs
its own change): moving the other owners' top-level files under
`<dir>/mix/<owner>/` (config::store `<service>.conf.mix`, scenes, macros,
apps data); `node.conf.mix` and the Bus socket are shared system files.

`tools/command_registry.mix` cannot run on this branch or on the base: it
requires `tools/lib/command_registry_source.mix`, which was never imported.
The registry file itself only changed owner `compd` -> `viud`.

## Desk migration

Units and binaries (no gold binary reads these):

| Old | New |
|---|---|
| `compd@.service`, `compd-headless@.service` (`bin/compd`) | `viud@.service`, `viud-headless@.service` (`bin/viud`) |
| `mixos-headless.slice` | `mix-headless.slice` |
| `/opt/mixos/bin/{mix,mixd,viu,...}` for MixViu | `/opt/mix/bin/...` (gold stays in `/opt/mixos`) |
| `User=mixos` in viud@, viud-headless@, apps@, scenes@, editd@, settingsd@, noded@ | `User=mix` (account must exist) |
| `StateDirectory/RuntimeDirectory/ConfigurationDirectory=mixos/<svc>`, `%S/mixos/...` | `mix/<svc>` |
| `RuntimeDirectory=mixos/session/%i/desktop` | `mix/session/%i/desktop` |
| Bus control `compd.truth` | `viud.truth` |
| unit hint `mixos-desk-scenes.service` | `mix-desk-scenes.service` |
| Not in this repo, must follow on desk4: `session@.target` Wants/Upholds/After, `viu-host@` Requires/After, `backdrop@`, `dopus-*`, `viu-gallery`, `seatd@` if it names compd | name `viud@` / `/opt/mix` |

Variables (gold column: read by gold term / portald / inputd at mixos 8779f33,
directly or through libs bus, paths, config, settings, design, toolkit,
citizen, logging, buildinfo):

| Old | New | Gold reads |
|---|---|---|
| `MIXOS` (root) | `MIX_ROOT` | term, portald, inputd (paths) |
| `MIXOS_PREFIX`, `MIXOS_SHARE`, `MIXOS_ETC`, `MIXOS_VAR`, `MIXOS_RUN`, `MIXOS_CACHE` | `MIX_PREFIX`, `MIX_SHARE`, `MIX_ETC`, `MIX_VAR`, `MIX_RUN`, `MIX_CACHE` | term, portald, inputd |
| `MIXOS_NODED_URL`, `MIXOS_NODE_CONFIG`, `MIXOS_BUS_SOCKET`, `MIXOS_SESSION_FD`, `MIXOS_BROKER_ACCOUNT` | `MIX_NODED_URL`, `MIX_NODE_CONFIG`, `MIX_BUS_SOCKET`, `MIX_SESSION_FD`, `MIX_BROKER_ACCOUNT` | term, portald, inputd (bus) |
| `MIXOS_APP_HOME`, `MIXOS_APPS_HOME` | `MIX_APP_HOME`, `MIX_APPS_HOME` | term, portald, inputd (config) |
| `MIXOS_SETTINGS_INSTANCE`, `MIXOS_SETTINGS_PROFILE` | `MIX_SETTINGS_INSTANCE`, `MIX_SETTINGS_PROFILE` | term, portald |
| `MIXOS_DECORATIONS` | `MIX_DECORATIONS` | term, portald |
| `MIXOS_COMP_SERVICE` | `MIX_COMP_SERVICE` | term |
| `MIXOS_COMPD_CONFIG` | `MIX_VIUD_CONFIG` | no |
| `MIXOS_OUTPUT_SCALE`, `MIXOS_HEADLESS_SIZE`, `MIXOS_CONTAIN_CHILDREN`, `MIXOS_USER_KEYMAP` | `MIX_VIUD_OUTPUT_SCALE`, `MIX_VIUD_HEADLESS_SIZE`, `MIX_VIUD_CONTAIN_CHILDREN`, `MIX_VIUD_USER_KEYMAP` | no |
| `MIXOS_VT`, `MIXOS_INSTANCE`, `MIXOS_SCENE_SERVICE`, `MIXOS_SCREENSHOTS_DIR` | `MIX_VT`, `MIX_INSTANCE`, `MIX_SCENE_SERVICE`, `MIX_SCREENSHOTS_DIR` | no |
| `MIXOS_EDIT_RECOVERY(_DIR)`, `MIXOS_INDEXD_SEARCH`, `MIXOS_WEBD_SESSION_KEY_PATH`, `MIXOS_SPEC_DIR`, `MIXOS_SPEC_RELEASE_DIR`, `MIXOS_SPEC_RELEASE_SHA256`, `MIXOS_NODED_EXPECTED_SERVER_PID` | `MIX_EDITD_RECOVERY(_DIR)`, `MIX_INDEXD_SEARCH`, `MIX_WEBD_SESSION_KEY_PATH`, `MIX_NODED_SPEC_DIR`, `MIX_NODED_SPEC_RELEASE_DIR`, `MIX_NODED_SPEC_RELEASE_SHA256`, `MIX_NODED_EXPECTED_SERVER_PID` | no |
| `MIXOS_MACRO_{APP,ID,PATH,TARGET,TIER}`, `MIXOS_MIX` | `MIX_MACRO_*`, `MIX_EXE` | no |
| `MIXOS_BIN`, `MIXOS_SRC`, `MIXOS_TERM_BIN`, `MIXOS_AGENT_CLI_MIX`, `MIXOS_MAN_SOURCE`, `MIXOS_MAN_URL` | `MIX_BIN`, `MIX_SRC`, `MIX_TERM_EXE`, `MIX_AGENT_CLI_EXE`, `MIX_MAN_SOURCE`, `MIX_MAN_URL` | no |
| `COMPD_*` (`COMPD_XWAYLAND`, `COMPD_FRAME_TRACE*`, `COMPD_KEYMAPS`, `COMPD_CAPTURE_*`, ...) | `VIUD_*` | no |
| build env `MIXOS_GIT_SHA(_FULL)`, `MIXOS_GIT_DIRTY`, `MIXOS_BUILD_TIME`, `MIXOS_BUILDINFO_MARKER` | `MIX_*` | compile time only |

Directories (gold term, portald and inputd all resolve these through
libs/paths and libs/config; portald's unit reads the session env file):

| Old | New | Move data? |
|---|---|---|
| `/etc/mixos` (incl. `session/<vt>.env`, `node.conf.mix`) | `/etc/mix` | yes: copy config; gold keeps reading `/etc/mixos` |
| `/var/lib/mixos/<desk>` and `/var/lib/mixos/<svc>` (settings stores, noded, editd recovery, maild/jmap, webd, indexd, dnsd, dlnad, blobd) | `/var/lib/mix/...` | yes: move the stores |
| `/run/mixos`, `/tmp/mix-run`, `$XDG_RUNTIME_DIR/mixos` | `/run/mix`, `/tmp/mix-run`, `$XDG_RUNTIME_DIR/mix` | no (runtime) |
| `/var/cache/mixos`, `~/.cache/mixos` | `/var/cache/mix`, `~/.cache/mix` | no |
| `~/.config/mixos` (node.conf.mix, `<service>.conf.mix`, scenes, macros) | `~/.config/mix` | yes |
| `~/.local/share/mixos` (apps data, scenes lib, assets) | `~/.local/share/mix` | yes |
| `~/.config/compd` (settings.json, preferences.json, keybinding.json, experimental.json) | `~/.config/mix/viud` | yes |
| `~/.local/state/compd` (world document store), `~/.local/share/compd` (shaders) | `~/.local/state/mix/viud`, `~/.local/share/mix/viud` | yes |
| `$XDG_RUNTIME_DIR/compd/<socket>.xwayland.env` (launcher read `mixos-comp/`) | `$XDG_RUNTIME_DIR/mix/viud/<socket>.xwayland.env` | no |
| `/usr/local/lib/mixos` | `/usr/local/lib/mix` | if present |

## What keeps an old name (rg -i mixos, compd)

1. Host interpreter paths: `/opt/mixos/bin/mix` in tools/*.mix, apps/*/gate.mix,
   capture-build.mix, apps/gallery/README.md and Mix-script test defaults
   (MIX_TEST_MIX_EXE falls back to the host's gold mix on CBC).
2. Wire, data and protocol identifiers (Mark's call whether to break):
   - Bus service `comp` and `comp.*` commands.
   - App ids `dev.mixos.*` (Wayland app_id, desktop ids, icon names; gold term uses `dev.mixos.term`).
   - Desktop name `MixOS` (XDG_CURRENT_DESKTOP, CLIENT_SIDE_DESKTOPS) and `MIXOS` (OnlyShowIn/NotShowIn).
   - Theme, scheme and window-style value `mixos` (`Scheme::Mixos`, `ChromeStyle::Mixos`, `WindowStyle::Mixos`, `.mixos()`/`mixos_only` registry flag, `view.theme.mixos` action ids, label "MixOS"), icon theme `icons/mixos`: stored settings values shared with gold term.
   - HTTP headers `X-Mixos-*`/`x-mixos-*` (blobd lanes, maild), cookies `mixos_session`, `mixos_csrf`, `mixos_flash`, `mixos_login_pending`, URL path `/_mixos/assets/`.
   - Prometheus metrics `mixos_process_*`, `mixos_stats_*`.
   - Native-session transcript labels `mixos.native-session.proof/allocate`; mesh admission domain `mixos-d2-admit-v1`; binary marker `MIXOS-BUILDINFO:1`.
   - Data formats `mixos.static-assets.v1`, `mixos-mds-set-export`; DAV properties `mixos:rawicalendar`/`mixos:rawvcard`; service binary name `mixos-jmap` in service_info.
   - System group `mixos-tls`; CA names `MixOS Internal Root/Issuing CA`.
   - Software groups `mixos-desktop`/`mixos-server` (app manifests); webd vhost variable `$mixos_delegation`.
3. Provenance and gold: provenance/, vendor/ patch markers (`compd:` notes
   attribute MixOS patches), docs/ports/ evidence, CHANGELOGs and
   docs/mix/changelog, tools/ lab and oracle scripts, the gold oracle names
   `compd-oracle`/`COMPD_INPROC_TEST_BINARY`, references to MixOS 8779f33 and
   github.com/markc/mixos, `~/Projects/mixos`, gold crate and daemon names in
   comments and the Mix manual (`mixos-lib-*`, `mixos-mds`, `mixos-editd`,
   `mixos-webd`, `mixos-dnsd`, `mixos-claude`, `mixos-mcp`, `mixos_maild` log
   targets, `mixos-comp` as the manual's hyphenated-command example, generated
   site `mixos-build` meta tags), the brand logo file name, domains
   mixos.nexus and mixos.au, and every mixos.dev URL and the mc@mixos.dev
   address (copyright lines; the manual's links: mixos.dev/, /mix/, /maild/,
   /dlnad/, /indexd/, /webd/, /compd/ and the per-page manual URLs).
4. Not renamed, outside the three categories (about 280 hits): lowercase
   mentions of the `mixos` theme in comments and test names (the theme value
   itself is category 2), a few local identifiers (`mixos` field in
   mix-shell, `mixos_dbkey_test`). Cosmetic; a follow-up can take them.
   `rg -i compd` outside provenance, vendor, tools and docs/ports: only the
   gold oracle (`COMPD_INPROC_TEST_BINARY`), the `provenance/compd-oracle`
   include paths and the viud README naming gold compd.

Other decisions for Mark:
- Fable rule 5 suggests engine runtime variables be `MIX_VIUD_*`; Mark's compd
  instruction named them `VIUD_*`. Left as `VIUD_*`.
- User-visible app names now say MixViu (e.g. "MixViu Editor", "About MixViu
  Editor") in both the Mix apps and their contract JSON, so they differ from
  the gold apps' "MixOS" text in parity captures.
- The furniture comparison (`COMPD_INPROC_TEST_BINARY`) still names the gold
  oracle; lab scripts pass the old `COMPD_NATIVE_TEST_BINARY`, now
  `VIUD_NATIVE_TEST_BINARY` in the test.

## Gate status

No full gate yet on the renamed tree at head: the coordinator holds cbc2/cbc3
for the parity jobs and will call one full cutover gate on the final
integrated tree. Latest full run, cutover-gate-2d7e1c7 (cbc2, before the last
rustfmt and indexd commits): 10,514 passed, 3 failed, 90 ignored; clippy
clean; viud egui lines 0; release sizes mix 20,518,680, mixd 65,895,968,
viu 56,745,544, viud 32,489,056 (viu grew with the merged Workshop/Backdrop
work). Failures: indexd native_contract (the Bus script got MIX_* env but
ran the host gold mix; fixed in c027612, passes on cachyos); the two
workshop_frame tests, which fail identically at lab head 9bcfeee (light job
20261107-171825-83bb29fb), so they come with the merge. rustfmt is clean at
c027612 (cachyos 20261107-173952-9448392a). The test count rose from 10,484
because the merges added the six-app work.

Branch cutover/four-engines; head is this last.md commit (source c027612); gate: full run pending (last full 2d7e1c7: 10,514/3/90, 2 failures inherited from lab 9bcfeee, 1 fixed); CBC cutover-gate-5d1ec61, cutover-gate-43de632, cutover-gate-2d7e1c7, cutover-removed-4755e9e, 20261107-111516-437fd7b1.
