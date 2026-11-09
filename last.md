# Quoin P1 hand-off — 2026-11-09

Implemented P1 items 1, 2, 4 and 5. Item 3 remains the later lab.
Author: Codex (Sol).

- Branch: `feat/quoin-p1-dev-loop`
- Base: `0780b2ea` from `origin/docs/quoin-p0` at job start. P0 has since advanced; no other session's branch was merged.
- P1 PR: https://github.com/mixviu/mixviu/pull/38 (draft, base main)
- P0 dependency: https://github.com/mixviu/mixviu/pull/36
- P1 implementation head before this receipt: `90dad687e2c40dfc794573f7065e4eb1b62f5f02`
- Final code tested: `3835467abdcf6556f850bf37e833e54b6cdaa6af`; subsequent commit changes only the Bus registry documentation.

## Files

- `engines/viud/scripts/scenes.mix`: shared/custom library watches and quoin.reload handler. User lib is covered by the installed-root watch.
- `engines/viud/scripts/lib/scenes.mix`: conservative dependency fan-out, cached helper checks, library fingerprints/overflow recovery, group preflight and atomic host submission. Both discovery paths exclude the reserved lib directory.
- `engines/viud/scripts/scene_preview.mix`: passive use of real loader candidate, behaviour checks and recorded JSON model.
- `engines/viu/src/store.rs`: clone/stage/commit load envelopes with sources; refuses the entire batch on a bad candidate or seat collision.
- `engines/viu/src/store_tests.rs`: batch failure retains revisions/seats; successful fan-out publishes both notices.
- `engines/viu/src/dev_loop.rs`: first-diagnostic extraction and foreground error strip.
- `engines/viu/src/preview.rs`: plain watched specimen window, authored/recorded model, passive actions, last-good retention; discovers validator from current checkout or VIU_PREVIEW_VALIDATOR.
- `engines/viu/src/main.rs`, `lib.rs`: preview CLI/module wiring.
- `engines/viu/src/runtime/toast.rs`: toast refusal strip.
- `engines/viud/tests/scenes_test.mix`: helper failure/recovery, fan-out, quoin scopes and native inotify timing.
- `docs/scene-documents.md`, `docs/docs/spec/bus/commands.conf.mix`: P1 contracts and registered command/batch shapes.

## runtime.rs hunks for P2 coordination

Only State.errors and its initialisation; Program::show's diagnostic strip; worker's scenes.changed subscription/initial scenes.list; and its authenticated scenes topic receipt branch. No bindings, sources, comp subscriptions, output polling or structural rendering changes.

## Validation

- Mix loader: 400 assertions pass. A real fs_watch/fs_wait helper edit plus two-dependent headless loader fan-out is asserted below one second. Host/process adapters are fake in this transition test; real host atomicity is covered by the Rust store test.
- Mix lint: scenes.mix, lib/scenes.mix and scene_preview.mix pass with zero errors/warnings.
- Registry strict-data load and unique quoin.reload entry verified. Full source registry gate cannot start: pre-existing missing tools/lib/command_registry_source.mix.
- Existing scenes_io integration test: default invocation hits obsolete mix_path builtin; explicit MIX_TEST_NODED_EXE=/opt/mixos/bin/noded bypasses it but installed broker cannot establish owned private-bus readiness. No production configuration changed.
- git diff --check passes.
- No desk4, VT, /opt writes, production installs, behaviour state hand-off or visual acceptance performed. Opus's eye checks remain pending. Security is expected red until #21.

## CBC receipts

All Rust work was on CBC. Saved final log read in full.

| Job | Commit | Result |
|---|---|---|
| 20261109-153955-8005b781-a1 | 64588834 | BUILD_RC=101, initial egui App API mismatch, fixed before later passes |
| 20261109-154304-d021212d-a1 | 18be0c32 | BUILD_RC=0; 99 passed, 5 ignored |
| 20261109-154543-fa958fbe-a1 | d7442ba2 | BUILD_RC=0; 101 passed, 5 ignored |
| 20261109-154901-329f6c89 | d7442ba2 | BUILD_RC=0; CBC formatting diff applied locally; own worker changes restored in same job |
| 20261109-154948-87cffd24-a1 | 8438f7b0 | BUILD_RC=0; 101 passed, 5 ignored |
| 20261109-155106-f61e0c72 | 369309d4 | BUILD_RC=75; worker lock busy, command did not run; requeued |
| 20261109-155215-02577fd0 | 369309d4 | BUILD_RC=75; other worker lock busy, command did not run; requeued |
| 20261109-155336-3c23428c-a1 | 3835467a | BUILD_RC=0 on cbc3; fmt --check, 101 host tests (5 ignored), cargo check --locked -p viu --bin viu |

Final log: `/home/user/.cache/mixos/cbc-build/20261109-155421-cbc3-3835467abdcf-20261109-155336-3c23428c-a1-93002c98.log`.
Queued formatter 20261109-154541-e5d7aa8a was superseded/cancelled; it was not a validation pass.

## Trade-offs

Computed require paths mean library dependency selection conservatively covers all enabled behaviour scenes, including transitive imports. Helpers are checked without executing them. Document acceptance is atomic; behaviour runtime faults after acceptance retain the existing restart/diagnostic semantics. Reload still kills/spawns behaviours; no state hand-off is claimed.

Preview has headless load/recorded-model/refusal/repair coverage and binary compilation coverage. Its live window and overlay pointer behaviour still need Opus's visual check. This receipt is prepended to the existing tracked hand-off history. The current PR head includes the receipt commit.

---

# Quoin P0 — 2026-11-09

PR: https://github.com/mixviu/mixviu/pull/36 (`docs/quoin-p0` -> `main`).
Head before the final receipt commit: `741001b802781bce46e02e3246fff91ab00b5b49`.
Rust rename-test source: `001c2a0b1907` (later changes are documentation and
the matching CI baseline path key and rustfmt line wrap, without semantic Rust changes). The receipt commit is a descendant of
those heads; the PR exposes its current head SHA.

Spec outline (`docs/scene-documents.md`): directory/catalogue/metadata and
behaviour lifecycle; envelope and strict-data grammar; all 19 widget families
and common ports; pure expressions and escaping; dependency/model patch path;
row templates and event routing; budgets, diagnostic catalogue and current
reload/output limits. Claims cite source path:line at baseline `881e4051`.

Registered 20 public loader commands: scenes.list, templates, install, remove,
enable, disable, reload, reset, open, close, toggle, ready, model, fork,
promote, move, editor.open, editor.close, editor.snapshot, editor.action.
Resolved pending schemas for all 36 host commands (scene.*, scenes.list,
dialog.*, toast.*, ping, info, props.get, panel.*, corner.*), 7 terminal
commands (status, execute, execute.result, execute.cancel, task.submit,
task.result, task.cancel), and 6 backdrop commands (ping, state, set, next,
view.set, view.cycle). Registry `contracts` contains typed documented params
and replies; `contract_types` supplies shared wire shapes. Dynamic handlers
use schema:none; terminal handlers name the existing Rust deserialisers.
These sections document actual code, not a new runtime schema validator.

Drift fixed: topbar uses desk.restart/desk.stop for the desk; absent Scene
Editor app distinguished from legacy share/scenes/editor; docs/cos links
replaced; omitted component manuals are stated as absent; current Mix source
paths and source links corrected; vendor README paths/licence links repaired.
Historical MixOS oracle paths and installed licence destinations are labelled.
The canonical "What MixViu is" text is unchanged; Quoin has its own section.

Rename: all 15 apps/quoin files moved unchanged to apps/volume-lab. Active
references updated in engines/viu/src/tests.rs,
libraries/viu/tests/acceptance.rs, tools/p2_rehearsal.mix,
tools/ci/mix-lint-baseline.json, runtime-lab docs, provenance README and this
session history. tools/apps_bundle.mix enumerates tracked app directories,
so no hard-coded install list changes. Bus identity quoin-volume is retained.
Immutable provenance JSON retains original import paths/hashes as history.

Code bugs found, deliberately not fixed:

- Default command_registry lint lacks imported helper modules. Supplying them
  reveals existing app lint failures and absent bridged family declarations.
- Host props.get returns a panels array; loader popup paths require a map.
- Advertised dialog/menu/panel.order operations return NOT_IMPLEMENTED;
  scene.layout returns SCENE_NOT_RENDERED.
- Family-less scene.describe enumerates ten families rather than nineteen;
  shell furniture layout also skips nine families accepted by the grammar.
- Binding coercion rejects length ports accepted by the literal schema.
- port-max is emitted but missing from ALL_CODES.
- Duplicate envelope headers are overwritten by the core but rejected by
  the loader.

Validation:

- Registry data/canonical lint PASS: 790 commands, 6 families, schema 5.
  Ran unchanged tools/command_registry.mix from a temporary directory, with
  its two missing helper modules supplied from MixOS, explicit registry FILE
  and no source scan. Default source-mode check is not claimed green.
- mix lint on the modified registry and rehearsal: zero errors/warnings/notes.
- Mix path audit: zero broken Markdown local links across README/docs source,
  including vendor READMEs; code path references reviewed separately for
  historical oracle paths, installed destinations and external source links.
- All scene-reference path:line citations checked for file existence and line
  bounds. git diff --check PASS.
- CBC rename tests PASS: BUILD_RC=0, job `20261109-152512-58ecf1c3-a1`,
  source `001c2a0b1907`, cargo test -p viu -p lib-viu --lib --tests.
  371 passed, zero failed, 6 existing environment-dependent tests ignored.
  Full 907-line saved log read, including existing dependency warnings.
  Final CI fmt diff (longer include_str path) applied exactly.
- 124 scene-reference citations checked for path/line bounds.
- Observed PR security checks are green; the task's #21 caveat is retained
  as context rather than reported as an observed failure.

No desk4/VT interaction, production deployment or /opt writes. Author: Codex <codex@openai.com>.

---

# Security Hickory split — gates green, awaiting review

Mark split this work from fix/security-advisories. Original branch history remains unchanged. All previously uncommitted adapters were committed first at 2dfcb234. The branch intentionally retains the full original history so no work is lost. The smaller non-Hickory PR is fix/security-tls.

2026-11-08: merged origin/main (b94c725a; apps/dopus deletion accepted, this file kept both sides). Finished the migration:

* mail-auth stays exactly =0.11.0 (0.11.3 is yanked); Cargo.lock now records 0.11.0, so --locked resolves. A brief =0.11.2 detour (16a83504) was reverted in a8a51330.
* Test-only TXT caches use Box<str> keys and Error::Dns(DnsError::InvalidRecordType).
* Hickory 0.26: maild envelope uses proto::rr::Name; mix DNS test sets Message.metadata.id directly.
* rustfmt output applied from CBC; logging caps system_configuration(_sys), pulled in by Hickory 0.26 (macOS).
* .gitleaksignore copied verbatim from PR #7: gitleaks scans every fetched ref, and fix/mix-hygiene's terminal help line is a false positive that failed main and this PR.

**DKIM behaviour (decided by Fable, 2026-11-08):** mail-auth 0.11 rejects a DKIM-Signature missing d/s/b/bh/h as MissingParameters at parse, and verify_dkim surfaces only Error::Dkim parse errors, so such a header would add no dkim= result. Verdict: report it as `neutral` (RFC 8601 §2.7.1, "signed, but the signature contained syntax errors"; matches 0.7), not permerror. `mapping::surface_dkim_parse_errors` adds a neutral DkimOutput for every other DKIM-Signature parse error right after verify_dkim, under the existing signature cap. Tests pin the upstream behaviour, maild's neutral result and no double reporting.

Next: fresh independent reviews (3-round budget), code-owner approval, undraft. No production installation or deployment.

Branch fix/security-hickory; head/source 76857d20 (this status commit follows); gate: CBC tools/ci.mix fmt/env/viud/mix-lint/clippy/test green at fa52b33e except the logging cap fixed in 76857d20 (job 20261108-173323-19ccf584-a1), logging --lib green at 76857d20 (job 20261108-180037-1f154069-a1); GitHub security workflow green.

# Email desk4 launch repair — ready for hot-apply

Branch `fix/email-desk4-tree-table`, based on `origin/cutover/four-engines`
`b601bf828c13e9d9a82bff59c7bedd2409be7687`. Tested implementation commit:
`1e38766b9a8dfcc260a8f377162aaaa3c37ec19d`; later receipt commits change only this document.

Root cause: `mail.accounts()` accepted the legacy accounts.json catalogue and
returned `label` but no `name`. `navigation_rows()` uses `entry.name`, so the
first account-tree row was `{key:"local",cells:[nil]}` for its one column.
The native validator correctly rejected that row before drawing any chrome.
This happens before the email table receives messages; neither pim-list nor
maild's string `from` caused the startup failure. The existing protocol already
normalises string senders into address lists.

The lab's connected UI suites created accounts with `email.account.set` through
settingsd, or supplied both name and label in runtime tests. They exercised real
maild but missed the positional legacy accounts-file startup path. The new
regression loads that actual file shape, validates/mounts the initial navigation,
then feeds both supplied messages through the existing JMAP normaliser and
validates all three tables. It also covers oversized multi-byte subjects/sender
names without altering the full server message or weakening native validation.

## Files for Opus

| Changed repository file | Installed destination / use |
| --- | --- |
| `apps/email/mail.mix` | `/opt/mix/share/mixviu/apps/email/mail.mix` — add canonical name from the validated legacy label; fixes the reported startup failure |
| `apps/email/surface.mix` | `/opt/mix/share/mixviu/apps/email/surface.mix` — bounded string display cells and sender rendering; preserve full server values, bulk row projection and the native 4096-byte cell bound |
| `libraries/viu/tests/email_frame.rs` | Regression test source only; no installed Email file |
| `last.md` | This handoff only; no installed Email file |

Apply the two Mix files from the tested commit together. No engine or validator
change is needed. Nothing was applied to desk4; main and infra/pr-ci were not
changed. This branch is for Opus hot-apply and PR/CI landing.

## Checks

* `email-desk4-scope-r4-a1`, cbc3, **BUILD_RC=0**: `cargo fmt --all` produced an
  empty patch; `cargo test --locked -p lib-viu --test email_frame --test email_wire`
  passed **17/17** tests, including the desk4 regression and 2,000-row projection.
  Complete saved log read: `20261108-124907-cbc3-1e38766b9a8d-email-desk4-scope-r4-a1-fd116d2f.log`.
* `email-desk4-native-r5`, native-only CBC light override, **BUILD_RC=0**: same
  previously verified release Viu, old b601 bundle versus fixed Mix bundle,
  isolated Bus and headless compositor with a real release maild process.
  The supplied accounts.json fields, HTTP origin 127.0.0.1:4310 and schema-2 UI
  config were reproduced. Maild submit seeded Inbox(1)/Sent(1); its own private
  store was given the supplied receivedAt instant. Raw HTTP Email/get confirms
  string from, empty keywords, Welcome subject and `2026-11-06T14:26:33.705Z`.
  Old bundle: ready=false, revision=0, exact cell/column mismatch. Fixed bundle:
  ready=true, account-tree label and three email cells valid, chrome/list captured
  and inspected. All five owned PIDs stopped. No fake HTTP/JMAP service was used.
  Complete saved log read: `20261108-125021-cachyos-1e38766b9a8d-email-desk4-native-r5-132b0730.log`.

Earlier attempts are retained privately: missing worker script and renamed Cargo
package invocation were harness preparation errors; the first display helper
inside rows_project was rejected and repaired with a native jq preparation pass.
The regression's protocol module path was corrected. Native harness checks were
corrected to read viu.state on the deliberately broken old app, accept its
documented empty error string on success, and preserve the supplied millisecond
instant when seeding the private real store. No application assertion or native
validator was loosened and no refused input request was retried.
SHA256 `apps/email/mail.mix`: `deb91fa76ae98a6e5da067035dff1cf97a67fd129a3d6e386fdfe072a4f1b0de`.

SHA256 `apps/email/surface.mix`: `2e1be2b720b0bae3da39212bfd7658f2216b08cc1661177635c2adb90880c54b`.

## Earlier integration record (historical)

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
`etc/env.conf.mix`: 200 names, loads with load_data, matches every
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

Asset store: `tools/apps_bundle.mix` rebuilds it from the pins. Fonts are
downloaded by `share/assets/install.mix` from each pinned upstream URL at its
revision, checked by BLAKE3 and SHA-256. Wallpapers download from the pinned
GitHub release `assets-wallpapers-2026-10-14` (recorded in
`share/wallpapers/manifest.conf.mix`; asset name = path with `/` as `--`),
following at most three redirects and stored only when size and SHA-256 match
the pin; the URL is never trusted. The gold install, read only, is the last
resort for both. Proof at aa1ee08, empty scratch store (cachyos
20261107-213836-eb7e5600): 17 fonts downloaded, 12 wallpapers from the
release, none from gold.

Current apps.tar at aa1ee08: 387 entries, 45,809,153 bytes of files, tar
46,213,120 bytes; share/mixviu 221 (apps, Viu Mix library, 39 unit templates
and snippets, licences, manifest), share/icons 49, share/scenes 45,
share/assets 22, share/wallpapers 15, lib/exportd 10, share/viud 7,
lib/dlnad 5, lib/clipd 4, share/layout 4, lib/notifyd 2, share/daylight 2.

Unit templates: apps.tar carries every unit under engines/ and services/
(viud@, viud-headless@, viu-host@, apps@, scenes@, mix-headless.slice, noded,
noded@, settingsd@, editd@, monitord, dnsd, blobd, hostd + socket, mixtrayd,
maild + sockets, webd, indexd, dlnad, clipd, clipd@, notifyd@, exportd-*) and
the sysusers/tmpfiles snippets, owner-prefixed. Desk-specific drop-ins and
session@/seatd@/session-dbus@ (written by the desk image) are not in it,
nor pipewire@/wireplumber@ (desk audio). Added since: monitord@ (per-session
history in mix/monitord/%i), backdrop@ and viu-gallery@ (the gallery app
ships in apps.tar); app unit templates come from apps/*/units. At 37e7f3b:
390 entries, share/mixviu 224, 45,810,831 bytes (cachyos
20261107-221523-0bbe5b3a; fonts upstream, wallpapers from the release).

Directories per owner (37e7f3b): maild@ is one instance per session broker,
so each keeps its own state: StateDirectory/RuntimeDirectory mix/maild/%i,
MIX_VAR/HOME/WorkingDirectory %S/mix/maild/%i. maild.service RuntimeDirectory
mix/maild; maild sysusers home @{var}/maild. hostd StateDirectory mix/hostd and
code default /var/lib/mix/hostd. monitord StateDirectory mix/monitord, code
default /var/lib/mix/monitord, sysusers home @{var}/monitord. dnsd tmpfiles
creates @{etc}/dnsd (was @{prefix}/etc/dnsd). All other Directory= and
sysusers homes were already mix/<owner>; noded@ and viud keep their
session-scoped mix/session/%i/... runtime dirs; mixos-tls stays.
viud@ and viud-headless@ no longer pass `--scene-service shell`: viud parses
the flag (name check only) and its scene stub ignores it, because the shell
furniture now runs in viu-host@ (`viu --host`), a new template in
engines/viu/units.

quoin: `apps/volume-lab` is not an app bundle: the runnable app is
`apps/volume-lab/volume` (contract.json with behaviour viu.mix, scene.mix). It
needs no launch arguments or host initialise. The migration's generated
launcher `viu run /opt/mix/share/mixviu/apps/volume-lab` must become
`viu run /opt/mix/share/mixviu/apps/volume-lab/volume`.

exportd and real homes: GTK/Qt already refused to write when XDG_CONFIG_HOME
is (or defaults to) the home's own .config unless MIX_EXPORTD_GTKQT_HOME=1.
COSMIC and KDE wrote there by default; now they run dry and report
`home_guard` unless MIX_EXPORTD_COSMIC_HOME=1 / MIX_EXPORTD_KDE_HOME=1 (an
explicit MIX_EXPORTD_COSMIC_ROOT or XDG_CONFIG_HOME elsewhere is allowed). KDE
state moved to ~/.local/state/mix/exportd/kde. Lint with this tree's mix: 0
errors (cachyos 20261107-213810-b6eb81ec); unit tests pass
(20261107-213742-29994673). Registry: 200 names.

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

Final lab merge (2026-11-08): origin/lab/gold-parity at 01912877 (eight accepted apps; Email 70ea474, Dopus c66fe3c; runtime e1bbea9). Conflicts took the lab side of libraries/viu (launch, app_host, chooser_host, Cargo.toml) with lib-viu/lib-mix put back, the Dopus Rust crates deleted as the lab retired them, and the lab Cargo.lock with the four-engine renames reapplied. Renames applied to the merged files: package names in the new Dopus/Email gate scripts, MIXOS_INSTANCE -> MIX_INSTANCE, MIXOS_COMP_SERVICE -> MIX_COMP_SERVICE, VIU_SERVICE -> MIX_VIU_SERVICE, /opt/mix, Dopus home ~/.local/state/mix/dopus, MixViu prose in the Dopus and Backdrop contracts. No new registry names. Kept as wire/data: theme value mixos and its MixOS label, the email token label mixos-email. apps.tar at 7bde4c0: 428 entries (share/mixviu 262), 46,138,946 bytes; fonts upstream, wallpapers from the release, none from gold (cachyos 20261108-105441-54343276). Light checks: fmt clean; cargo tree --locked -p viud ok; lib-viu --tests 263 passed, 3 failed (the known cachyos filesystem cases); dopus_contract, dopus_bundle, backdrop, app_ports pass at db16141.

Branch cutover/four-engines; head is this last.md commit (source db16141); full gate: the coordinator runs it on infra/pr-ci; CBC light 20261108-105611-7534e7c2, 20261108-104904-e4317f0f, 20261108-104841-c85e2532, 20261108-105441-54343276.

Branch fix/email-desk4-tree-table; tested commit 1e38766b9a8dfcc260a8f377162aaaa3c37ec19d; Email legacy account-tree startup FIXED; hot-apply apps/email/mail.mix + apps/email/surface.mix to /opt/mix/share/mixviu/apps/email/; CBC email-desk4-scope-r4-a1 rc0 (17/17), email-desk4-native-r5 rc0 (real maild before/after); no desk4/main/infra/pr-ci changes.

## P1 rebase onto merged P0 — 2026-11-09

This receipt supersedes the original P0 dependency/base above.

- New base: `cd85e2a3024f27bd997f8787e50d59d349b3ab67` (`origin/main`, P0 squash PR #41).
- Old P1 head: `5cb266763fd5b4ea811c10b9284e0b494dfcfbca`; retained locally as `backup/quoin-p1-before-main-20261109`.
- Replayed only `0780b2ea..feat/quoin-p1-dev-loop` onto main. All eight commits rebased automatically, with no conflict stops. Reviewed the resulting main diff: only P1 files and this session's harness/receipt changes remain.
- P0's name-limit/header-only panel text, registry draft statuses, backdrop.view.list contract and viu registry test owner filter are preserved. Neither document required manual conflict resolution; the auto-merge retained P0's reviewed text plus the P1 additions.
- New rebased source head tested and pushed: `75255585405b5ab511f59032dbda47b48f88e45b`. The subsequent receipt commit changes only last.md; its hash is the final branch head returned in the session hand-off.
- Bot push used `gh_agent.mix git -C WORKTREE push --force-with-lease origin feat/quoin-p1-dev-loop`, on this branch only. No PR opened or reopened; Opus handles the bot PR.

Harness work:

- Separate commit `75255585`: scenes_io_test.mix now uses `mixos_path("prefix")` instead of obsolete `mix_path("prefix")`. Local syntax/lint pass. The default integration invocation now gets past that builtin and reaches the already-known private broker readiness failure; it still does not pass end to end with the installed noded.
- Registry source helper deferred as larger than a small fix. The original MixOS checker is 390 lines and requires another omitted 173-line command_registry_embed.mix. It traverses the old apps/services/cli/libs layout; MixViu needs engines/libraries ownership coverage, embedding-context/provenance assessment and fixture tests. Merely copying the missing file would leave another missing require and incomplete source coverage. The gate remains blocked at its missing require, with no bypass or stub added.

CBC, exact source `75255585405b5ab511f59032dbda47b48f88e45b`, cbc2:

| Job | BUILD_RC | Outcome |
|---|---|---|
| 20261109-174228-83fb5387-a1 | 1 | 400 loader assertions and three P1 script lint checks passed; stopped at the worker's older linter not knowing mixos_path in the harness |
| 20261109-174321-7bf300df-a1 | 0 | 400 loader assertions; P1 script lint; harness lint with --allow-function mixos_path for the older worker binary; cargo fmt -p viu -- --check; cargo test -p viu --lib (101 passed, 5 ignored); cargo check -p viu --bin viu --locked |

Final saved log was read in full: `20261109-174321-cbc2-75255585405b-20261109-174321-7bf300df-a1-ef6cb226.log` under the maintainer's CBC cache. Harness lint has one existing literal-$tmp note, zero errors/warnings. Other P1 lint checks have zero errors/warnings/notes. Existing dependency warnings remain in the Rust log.

No desk4, VT or /opt changes. No additional runtime.rs hunks beyond those listed in the original P1 hand-off.
