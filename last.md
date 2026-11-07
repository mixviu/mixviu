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
Engine settings are all MIX_<ENGINE> names (Mark, Fable rule 5): the
former VIUD settings take the viud engine word (MIX_VIUD_XWAYLAND), their
test-only ones the test word first (MIX_TEST_VIUD_RENDER_NODE), and likewise
viu (MIX_VIU_SERVICE, VIU_BIN -> MIX_VIU_EXE), noded (MIX_NODED_SERVICE),
dlnad, hostd, indexd and maild. Labels and error codes that only share a
spelling (VIUD_READY, VIUD_PRESENTATION_PROBE) are not variables and stay.
`etc/env.conf.mix`: 198 names, loads with load_data, matches every
`MIX_[A-Z0-9_]+` token outside vendor/, provenance/ and Cargo.lock (a few
entries are Rust constants or a file name sharing the spelling; said so).

Owner subdirectories (Mark): every daemon, app and the viu runtime keeps its
files in `<dir>/mix/<owner>/` (one commit per owner; table below). The Mix
shell's own files stay at the top of the user tree (`~/.local/state/mix`:
history, stats). The top of the system tree holds only shared system
configuration (`node.conf.mix`, `theme.conf.mix`, `mesh.conf.mix`,
`session/<vt>.env`, `desk.env`, `/run/mix/mode`, `/run/mix/session/<vt>/`)
and the Bus socket, which already sits in noded's own `/run/mix/noded/`.
Logs: `~/.local/log/mix/<owner>/`, system tier `/var/log/mix/<owner>/`.

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
| `COMPD_*` runtime (`COMPD_XWAYLAND`, `COMPD_FRAME_TRACE*`, `COMPD_KEYMAPS`, `COMPD_CAPTURE_*`, `COMPD_TEARING`, ...) | `MIX_VIUD_XWAYLAND`, `MIX_VIUD_FRAME_TRACE`, ... (viud engine word) | no |
| `COMPD_*` test-only (`COMPD_TEST_RENDER_NODE`, `COMPD_TEST_BINARY`, `COMPD_WLCS_*`, `COMPD_NATIVE_TEST_BINARY`, ...) | `MIX_TEST_VIUD_RENDER_NODE`, `MIX_TEST_VIUD_EXE`, ... (EXE for binaries) | no |
| `VIU_SERVICE`, `VIU_BIN`, `VIU_COMP_SERVICE`, `NODED_SERVICE`, `DLNAD_CONF`, `DLNAD_STATE`, `HOSTD_SOCKET` | `MIX_VIU_SERVICE`, `MIX_VIU_EXE`, `MIX_VIU_COMP_SERVICE`, `MIX_NODED_SERVICE`, `MIX_DLNAD_CONF`, `MIX_DLNAD_STATE`, `MIX_HOSTD_SOCKET` | no |
| build env `MIXOS_GIT_SHA(_FULL)`, `MIXOS_GIT_DIRTY`, `MIXOS_BUILD_TIME`, `MIXOS_BUILDINFO_MARKER` | `MIX_*` | compile time only |

Data paths by owner (the list desk4 moves data by; `<etc>`, `<var>`, `<run>`
are the tier roots: `/etc/mix`, `/var/lib/mix`, `/run/mix` for the system
tier, `~/.config/mix`, `~/.local/share/mix`, `$XDG_RUNTIME_DIR/mix` for a
user):

| Owner | Old | New | Gold reads |
|---|---|---|---|
| maild | `<var>/jmap/{mail.db,blobs,mds,spamlite}`, unit `StateDirectory=... mix/jmap` | `<var>/maild/jmap/...`, `StateDirectory=mix/maild` | no |
| maild | `<etc>/jmap.conf.mix` | `<etc>/maild/jmap.conf.mix` | no |
| webd | `<var>/web.db` (default without --db-path) | `<var>/webd/web.db` | no |
| editd | `<var>/edit/recovery`, `~/.local/state/mix/edit/recovery` | `.../editd/recovery` | no |
| settingsd | `<etc>/settings/<profile>` | `<etc>/settingsd/<profile>` | no (term/portald use the Bus) |
| viud | `<etc>/comp/xwayland-enabled.<socket>` | `<etc>/viud/xwayland-enabled.<socket>` | no |
| viud | `~/.config/compd/*.json`, `~/.local/state/compd`, `~/.local/share/compd`, `$XDG_RUNTIME_DIR/compd/<socket>.xwayland.env` | `~/.config/mix/viud`, `~/.local/state/mix/viud`, `~/.local/share/mix/viud`, `$XDG_RUNTIME_DIR/mix/viud` | no |
| every config::store user | `<etc>/<service>.conf.mix` | `<etc>/<service>/<service>.conf.mix` | lib only at gold (no caller in term/portald/inputd) |
| logging, noded | `<var>/log/<identity>.log`, `<var>/log/bus.log` | `~/.local/log/mix/<owner>/` or `/var/log/mix/<owner>/` (noded: `noded/bus.log`) | term, portald, inputd write their own logs under `/var/lib/mixos/log` (gold libs/logging) |
| apps | `~/.local/state/mix/apps/<app>`, `<var>/apps/<app>` | `~/.local/state/mix/<app>`, `<var>/<app>` | term keeps its gold home (`~/.local/state/mixos/apps/term`) |
| indexd | `~/.config/mix/indexd.conf.mix` | `~/.config/mix/indexd/config.conf.mix` | no |
| hostd | `/run/hostd/hostd.sock` | `/run/mix/hostd/hostd.sock` (unit, hostd, mixtrayd, Quoin/scenes clients) | no |
| viu | `$XDG_RUNTIME_DIR/viu-captures` | `$XDG_RUNTIME_DIR/mix/viu/captures` | no |
| inputd keymap | gold inputd writes `~/.config/mixos/inputd/keymap.json` | viud reads `~/.config/mix/inputd/keymap.json` | yes: gold inputd is the writer; copy or point it at the new file |

Already per owner, only the prefix changed: noded (`<etc>/noded`, `<var>/noded`,
`<run>/noded/bus.sock`), maild (`<var>/maild/{dkim,tls,rules}`,
`<etc>/maild/config.conf.mix`), webd (`<var>/webd/{session.key,acme}`), dnsd,
blobd, dlnad and indexd system paths, scenes (`~/.config/mix/scenes`,
`~/.local/share/mix/scenes`), macros (`~/.config/mix/macros`).

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
- User-visible app names say MixViu ("MixViu Editor"): a Mark-mandated
  difference from the gold apps.
- Category 2 wire names stay until one deliberate PR after desk4's gold
  binaries are gone (Mark).
- Mix-script tests and parity gates that run under the host's gold mix
  (`/opt/mixos/bin/mix`) now see MIX_* names and the `mix_path` builtin, which
  gold mix does not know; they need this tree's mix (indexd's Bus test now
  prefers `target/debug/mix`; others, such as dlnad's limits_test, still
  default to gold).
- Lab scripts (tools/bulk_native, p2_rehearsal, p2 furniture) still set the
  old VIU_COMP_SERVICE / COMPD_NATIVE_TEST_BINARY names.
- The furniture comparison (`COMPD_INPROC_TEST_BINARY`) still names the gold
  oracle; lab scripts pass the old `COMPD_NATIVE_TEST_BINARY`, now
  `VIUD_NATIVE_TEST_BINARY` in the test.

## Release set and apps.tar

The full cut-over gate (`tools/cutover_gate.mix gate`) ends by writing
`/build/cutover-artifacts/<sha>/{mix,mixd,viu,viud,apps.tar,SHA256SUMS}`:
the four release engines, then `tools/apps_bundle.mix` run with the release
mix, then SHA256SUMS over all five. `cutover_gate.mix bundle` is the light
check of the same step.

apps.tar is laid out as installed under `/opt/mix` (root 0:0, commit mtime,
sorted), with `share/mixviu/BUNDLE.conf.mix` listing every file's size and
SHA-256:

| Path | What |
|---|---|
| `share/mixviu/apps/<app>/` | the Mix apps backdrop, ced, dopus, email, gallery, monitor, prefs, quoin, screenshot, workshop (runtime files: no Rust, tests or gate drivers; gallery-reference is a Rust build and is left out) |
| `share/mixviu/libraries/viu/mix/` | the Viu Mix library the apps require (`../../libraries/...`) |
| `share/mixviu/units/` | unit templates with `@{prefix}`/`@{etc}`/`@{share}` placeholders: viud@, viud-headless@, apps@, scenes@, mix-headless.slice, clipd, clipd@, notifyd@, exportd-*, dlnad |
| `share/mixviu/{LICENSE-APACHE,LICENSE-MIT,NOTICE}` | licences |
| `share/scenes/`, `share/viud/scripts/` | scenes and the viud Mix scripts (apps@, scenes@) |
| `lib/{clipd,notifyd,exportd,dlnad}/` | the Mix services, where their units start them |
| `share/icons/`, `share/assets/`, `share/wallpapers/`, `share/layout/`, `share/daylight/` | icon theme (built by its offline installer), brand marks, the pinned font set `2026-10-14-core-5` with `current`, 12 wallpapers, layout and daylight data |

Light run (cachyos, fa950b2, debug mix): 360 entries, 45,785,739 bytes of
files, tar 46,161,920 bytes; top: share/mixviu 194, share/icons 49,
share/scenes 45, share/assets 22, share/wallpapers 15, lib/exportd 10,
share/viud 7, lib/dlnad 5, lib/clipd 4, share/layout 4, lib/notifyd 2,
share/daylight 2. Retired lab hosts (viu-client, viu-dopus,
viu-dopus-client) are not in it.

The migration's `extract_bundle` still expects the old repo-relative layout
(`apps/`, `libraries/viu/mix`, `share/scenes`, `engines/viud/scripts` at the
tar root): it needs to unpack this tar at `/opt/mix` instead, and it can drop
its `copy_tree` of gold `icons`, `wallpapers`, `brand` and `assets`.

Asset store: the bundle copies the font set and wallpapers from a store and
checks every byte against `share/assets/core.conf.mix` (SHA-256 and BLAKE3)
and `share/wallpapers/manifest.conf.mix`. cbc2 and cbc3 have it at
`/build/mixviu-assets` (staged from the host's gold install); elsewhere the
gold `/opt/mixos/share` serves, read only.

## Ported Mix services

clipd, notifyd and exportd come from mixos 8779f33 (verbatim import, then
renames; provenance `provenance/gold-mix-services.md`). New variables are
registered: MIX_CLIPD_CONFIG, MIX_CLIPD_MODE, MIX_CLIPD_SESSION,
MIX_CLIPD_TARGET, MIX_CLIPD_WATCH_TOKEN, MIX_EXPORTD_STATE,
MIX_EXPORTD_COSMIC_ROOT, MIX_EXPORTD_GTKQT_HOME, MIX_EXPORTD_DRY_RUN,
MIX_EXPORTD_KDE_STATE, MIX_EXPORTD_KDE_DRY, MIX_EXPORTD_KDE_RECONFIGURE,
MIX_EXPORTD_KDE_KWRITECONFIG, MIX_EXPORTD_KDE_BUSCTL, MIX_NOTIFYD_STATE, MIX_SETTINGS_SERVICE, MIX_TEST_BLOBD_EXE,
MIX_TEST_EXPORTD_REAL_KDE. clipd's config moves to
`<etc>/clipd/clipboard.conf.mix`; units run as `User=mix`. Exported KDE/Qt
names (MixOS-Dark/Light, MixOSDark/Light, the `[MixOS]` kdeglobals stamp) stay:
they name files gold exportd already wrote. notifyd's default app name is
MixViu. tests/cbc_checks.mix (a MixOS crate check) is not imported.

Lint with this tree's mix (`cutover_gate.mix lint`, cachyos
20261107-211822-f6329f7d): 34 files, 0 errors, 0 warnings (kde_test 2 notes).
Unit tests with this tree's mix (`cutover_gate.mix mixtest`, cachyos
20261107-212515-4a70d7c5): clipboard_test, notify_test, gtkqt_test,
cosmic_test, busctl_test pass; kde_test passes its unit checks and then
needs a built noded for its Bus part (not run). The gold mix cannot lint or
run them (no `mix_path`).

## Shared assets

`provenance/shared-assets.md`: the 87 share files imported from mixos 8779f33
(icons: Lucide ISC plus MIT OR Apache-2.0 caption glyphs; brand: MIT OR
Apache-2.0; assets installer and pin manifest; layout; daylight from tzdb,
public domain), the 17 pinned files of font set 2026-10-14-core-5 (all OFL-1.1:
Inter, JetBrains Mono, Noto Sans, Noto Serif, Quicksand, Noto Color Emoji,
with their licence texts; each with upstream URL, revision and hashes) and the 12 wallpapers: 6 Wikimedia Commons photos
(CC0, CC BY 4.0 x2, CC BY-SA 4.0 x3) and 6 generated (MIT OR Apache-2.0).
The JPEGs and fonts are not committed. The photos' CREDITS still asks for the
Nunivak author to be named before a public release.

## Gate status

No full gate on cbc2/cbc3 for this round (Mark: light worker only until the
tree is final; parity has priority). Last full run, cutover-gate-2d7e1c7:
10,514 passed, 3 failed, 90 ignored; one failure since fixed (indexd), two
inherited from lab 9bcfeee (workshop_frame; the lab round is fixing them).

Light checks on cachyos at 34a8d52 (one or two crates per job):
- config 61 passed, 1 failed; editd + daemon 144/0/2; settingsd + hostd 165/0/6;
  maild 1,651/0/4; webd 490/0/0; noded + mixtrayd 277/0/2;
  policy-host + comp-model 106/0/0; lib-viu 105 passed, 3 failed;
  viu + viud 118/0/6; clippy lib-viu + viud clean.
- The config and lib-viu failures (non-UTF-8 names, links) are the cachyos
  filesystem: the same config and viu file_data tests fail at base 4755e9e
  on cachyos (20261107-185148-df07c276, 20261107-185233-ce550d01) and pass on
  cbc2 in every full gate. raster_decode's folder test (merged lab code) is
  the same kind.
- At 7fddb05: rustfmt clean (20261107-185355-f8c3d8cd); logging + indexd
  green (20261107-185418-c5fe3256).

Branch cutover/four-engines; head is this last.md commit (source 6fb20c11); gate: full run held for parity (the next full gate also writes the release set with apps.tar); light checks green apart from cachyos filesystem cases; CBC light bundle 20261107-212351-63549e61, lint 20261107-211822-f6329f7d, mixtest 20261107-212515-4a70d7c5, last full cutover-gate-2d7e1c7, earlier cutover-gate-5d1ec61, cutover-gate-43de632, cutover-removed-4755e9e, 20261107-111516-437fd7b1.
