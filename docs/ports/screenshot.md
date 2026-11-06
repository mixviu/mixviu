# Screenshot parity inventory and acceptance record

Gold is MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`. Branch:
`lab/parity-screenshot`. Tested candidate code: `5c3f3852bdd5d0705881a266a20db30c833f6f72`; receipt-only
documentation and provenance changes follow that code commit.

Overall verdict: **ACCEPTED**. Functional, Bus, live Prefs and visual parity
are complete. Opus resolved S2 on 2026-11-07: the truthful About source stamp
is runtime data, with gold's layout, format and position preserved.

## Inventory

Mix owns application layout, policy, storage, launch and compatibility dispatch.
Reusable Viu facilities own native raster documents, canvas input, controls,
clipboard, macros and deferred Bus replies. Intra-app traffic stays in process.

Launch: `mix apps/screenshot/launch.mix [OPTIONS] [IMAGE]`. The generic host
also accepts `viu run apps/screenshot [COMPOSITOR_SERVICE [SAVE_FOLDER
[RAW_FOLDER]]]` for isolated tests.

| Gold feature | Closure evidence |
| --- | --- |
| Screen/output, explicit and interactive region, fenced window | Native compositor; actual 300×200 human-seat selection, request id/generation, settled 800×600 target client readback and decoded capture extent checked |
| Delay, cursor, hide/restore, cancellation and caller loss | Original completion reply, selector cancellation after selection, delayed caller departure, restored window and quit-during-capture CANCELLED; host and launcher retire |
| Capture/open/export/save completion replies | Identity-bound deferred replies; retained-request, duplicate/stale/fault/reload/caller-loss/capacity tests; native request retained beyond 22 seconds on the final release |
| Arrow, line, rectangle, ellipse, pen, highlighter, redact, text, number | Equal document replies and byte-identical final PNG; native rectangle pointer callback and scroll witness |
| Selection, move/delete, crop, undo/redo, zoom/pan/fit and guides | Native document/viewport tests, compatibility replay and matched selection/guide/narrow captures |
| Open, Save As, discard, About and shortcuts | Native keyboard path entry and caption-close prompt; matched dialogs; About source stamp is truthful runtime data under S2 |
| PNG/JPEG/WebP and exclusive PNG export; clipboard | Native decoders, overwrite refusal and actual Wayland clipboard PNG readback |
| Paths, XDG Pictures, collision names, recent ledger and pruning | XDG user dirs, `_3` no-clobber save, newest-20/seven-day raw policy, protected symlink, >1 MiB ledger, typed/malformed/UTF-8-prefix cases and bounded compaction |
| HELP, describe, actions, execute and deprecated edit aliases | All 55 command descriptors and initial registry match; 53 structured compatibility cases plus lifecycle/headless/files/macros/debug/UI/window witnesses |
| Macro library, execution and popup | CRUD, built-in copy/refusal, move out of app, output, stop/exit 143, finished topic, native Escape and anchored popup |
| Debug, UI/menu/window drive and topics | Live debug controller on canonical and aliased services; received/answered history; native saved/action-executed/macro-finished subscribers; maximize/restore/minimize/show/dirty-close |
| CLI/headless, image start, Print handoff and namespaces | Actual Mix launcher, tracked child hosts, restart persistence, compositor mismatch refusal, same-PID handoff and canonical app Prefs/keymaps |
| Live Prefs and content contrast | Twelve variants, invalid value refusal without revision/value mutation, F8 remapping, authority loss/recovery and unchanged app PID |

Gold corrects the earlier inventory: capture/save/export answer when finished;
cancellation waits for cooperative cleanup. Bus file paths may be relative;
dialog entry also trims and expands a leading home-directory abbreviation.
Unannotated raw captures do not prompt for unsaved annotations. Fixed annotation
inks remain independent of chrome appearance. Moving a macro to a global tier
or another app removes it from Screenshot's catalogue; subsequent get is
NOT_FOUND. Workshop is a reserved destination, not an ordinary app tier.
Gold's generic `view.mode` action is always unchecked and toggles the local
choice, falling back to the theme file. The toolbar's explicit mode action
uses the effective live look. Both paths are preserved and driven separately.

## Exact bytes and CBC

| Component | SHA256 |
| --- | --- |
| Gold Screenshot | `f3b5bad3b0f43a3183a9810fe9178a9f445801c63c5e06e313bd4caaa2e3f932` |
| Candidate release Viu | `2a6db756d90172ce84348c55cbc4c90a7c583780088fa9f9e0a0d8ede88e351b` |
| Native compositor | `6e22d073fba0011f9ea3bddc4e6bf406c457e7d5fb257001fc85b74f40a16180` |
| settingsd | `26edf2810b5c48f9fa72c839b5eb5de09508afc4da3deb5c5c8fa88da84f47fd` |
| noded | `004d4e6f31036b52be057106c04caae0fd632a5c1b2e14e9d44668798b30e61c` |
| Mix launcher | `ad5e6c50a511909e3d600c01a0a3b8d5b76101e257352fc75f97001dc219729a` |

CBC `20261107-083150-1cb4a226-a1`, `cbc3`, **BUILD_RC=0**, ran
`mix apps/screenshot/gate.mix --release`: policy, formatting check, locked
all-target check, all scene/Viu/viu-engine tests, isolated engine shell-profile
test, clippy and release build. Worker diff empty; complete saved log read:
`20261107-083150-cbc3-5c3f3852bdd5-20261107-083150-1cb4a226-a1-d0c7d2c7.log`. This is the complete scoped app/host gate, not an integration claim
about other apps or engines. Existing warnings are retained in the full log.
Six existing environment-specific tests are ignored (installed-icon lookup,
generic engine GPU snapshots/process gate); Screenshot's native GPU evidence
below runs separately.

The Viu engine declares the lib-mix shell profile with defaults disabled:
json, regex, toml, datetime, url, crypto, http, sqlite, ws and archive. The
isolated engine test prevents feature unification from masking an omission.

Round-four generic repairs include the native dialog identity/path label and
shortcut aliases, canonical debug dispatch and completion history, canonical
Prefs/keymaps across service aliases, native macro anchoring/active marker,
and gold's menu-search hint. The vendored eframe CloseRequested path now exits
after destroying the app on the active event loop: Wayland need not send a
further window event. Reusable in-process `native.theme.file {}` reads the
native theme file without applying it, so Mix can preserve gold's toggle
fallback independently of live Prefs. Native quit-during-capture verifies
cancellation and process exit. Its file hash/adaptation is recorded in runtime-imports.json.

## Native evidence and capture recipe

Private evidence root: hub `_var/screenshot-parity-20261107/`. Frozen scripts,
their hashes and service hashes are in `release-evidence-5c3f3852bdd5d0705881a266a20db30c833f6f72/manifest.json`.
Evidence index SHA256:
`dc7aa613884ba7406de4ea240ac7140f7fcc092b897e7a8709358946004fdffb`;
frozen manifest SHA256:
`10374311c309e247b633e2cf4962ad2b24f27a4d87a612fc56517c7f6ec514dc`.
Capture filenames are `gold-STATE.png` and `mix-STATE.png`; comparison figures
are in `diff-STATE/` and `overlay-diff-STATE/` within the indexed run.
Native harness revision: `884fdd8ef2c548cc42017b9170593ec67ed30e57`;
`nested_host_lib.mix` SHA256
`af373c0b767ef73048bb29276277d5092836d0423325cb801ace49ad4ea7a317`.
The deterministic input is a 320×240 gradient PNG, hash `987ed01a449855d67ddcfb9b355582bcd9af241149ada1c2fe38a24977ebbd20`.
The base settings fixture hash is
`a14e96d0c1153e1a7008fe27d7149bc98434b4ccbfffad57c48114eef8037d08`.

Each run owns its broker, compositor, settings root, home/runtime/config/data/
cache and service identities. Gold runs first, then the candidate against the
same services. Client captures use CSD, right captions, bar menus, row actions,
scale 1 and 1040×680; the narrow pair is 650×480. Effective forms, native
readback receipts, request/reply traffic and source/binary hashes are retained.
The window-capture fixture respects gold's minimum client size; it separately
records settled client dimensions and compositor capture extent, including
any decoration, instead of conflating them. CLI fixtures track launcher and
actual host PIDs. All final closure receipts confirm every owned PID stopped;
the private brokers were stopped in finally. Failed diagnostic runs remain.
No desktop, VT, live settings or installation was changed.

| Native witness | Private directory |
| --- | --- |
| matched | `matched.AVcnx9` |
| prefs-menus-native-input | `advanced.LJ37jD` |
| capture-lifecycle | `lifecycle-nested.Rn9YNN` |
| debug-topics-authority | `extras.wWFTO6` |
| files-history | `files.MwgRg5` |
| cli-headless | `cli-headless.2zMYIM` |
| cli-window | `cli-window.u0SjFR` |
| abp | `abp.x92JBC` |
| annotations-gold | `release-functional-gold.R1BfyZ` |
| annotations-mix | `release-functional-mix.nB7p8e` |
| deferred-primitive | `deferred-native.aiMlim` |


The live Prefs sequence covers Studio Light; orange/compact/square controls;
Noto Sans and JetBrains Mono, weight 600, size 16, scale 1.25 and icon stroke 2;
Pro Dark; Classic spacious; app light/Studio/scale-1.1 overrides;
`tune.style.radius=12`; auto/daylight/day-night override states; high contrast
and reduced motion. Every transition retains the same app PID. Native F8
remapping also runs through the compositor-specific service alias.

The functional nine-kind export SHA256 is `f0ea386d7e8aff6f85a731b384d05f174e103be80b6cdf895e4e6069f0aee9f3`. Native pointer
export SHA256 is `5499c226d54cdc40c78efce83b94dfe1fee05aab19d08b7db6d3173c71b87980`; both match gold byte for byte. Clipboard bytes
are recorded and compared independently from the application's copy reply.
The final deferred-reply primitive witness holds the original native request
for 22.041999015025795 seconds while answering 73 pings, then
separately cancels a retained request with rc 10 / CANCELLED.
Compatibility normalisation covers generated paths, timestamps, PIDs and
compositor identities; it preserves rc, errors, payloads, action state and
side effects. Debug history additionally normalises clocks and broker RPC ids.
Action topics normalise per-process run UUIDs and wall-clock `at_ms`; sequence,
origin, drop counts and result payloads are compared unchanged.

## Visual inspection

`tools/parity_compare.mix` produced full and region comparisons without
resizing captures. ImageMagick **7.1.2-35 Q16-HDRI** reports SSIM as distortion:
zero is identical, lower is better. Raw SSIM/RMSE, normalised values, original
hashes, tool version, side-by-side figures and difference figures are in each
`diff-*/comparison.json`. No numerical pass threshold was used.

At 1040×680 the regions are header `[0,0,1040,38]`, options
`[0,38,1040,42]`, rail `[0,80,50,570]`, canvas `[50,80,990,570]` and status
`[0,650,1040,30]`. At 650×480: header `[0,0,650,38]`, options
`[0,38,650,42]`, rail `[0,80,50,370]`, canvas `[50,80,600,370]`, status
`[0,450,650,30]`. The figures include originals and separately cropped regions.

Prefs crops follow the corresponding gold density/style, measured at its
native separators and rail edge. Each state retains its own region JSON:

| Prefs layout | Header bottom | Options bottom | Rail width | Status top |
| --- | ---: | ---: | ---: | ---: |
| Studio comfortable | 38 | 80 | 50 | 650 |
| Studio compact, including font variant | 32 | 68 | 43 | 655 |
| Pro comfortable | 32 | 68 | 40 | 656 |
| Classic spacious and later overrides/menus | 46 | 96 | 60 | 644 |

| Capture state | Full SSIM | Full RMSE | Header SSIM / RMSE | Other differing regions (SSIM / RMSE) |
| --- | ---: | ---: | --- | --- |
| about-dark | 0.000487896 | 0.00796244 | 0 / 0 | canvas: 0.000611448 / 0.00891379 |
| discard-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| document-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| document-light | 0 | 0 | 0 / 0 | All other regions 0 |
| empty-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| empty-light | 0 | 0 | 0 / 0 | All other regions 0 |
| guide-dark | 0.00000000178112 | 0.00000737325 | 0 / 0 | canvas: 0.00000000223216 / 0.0000082542 |
| narrow-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| open-dialog-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| save-as-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| selection-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| shortcuts-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| menu-annotate | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| menu-capture | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| menu-edit | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| menu-file | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| menu-help | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| menu-view | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-app-override | 0.0000916474 | 0.00422342 | 0.00135479 / 0.0162383 | All other regions 0 |
| prefs-auto | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-auto-day | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-auto-night | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-auto-override | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-classic-spacious | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-contrast-motion | 0.0000936303 | 0.00474771 | 0.0013841 / 0.0182541 | All other regions 0 |
| prefs-fonts | 0.0000000202248 | 0.0000409199 | 0.000000429776 / 0.000188631 | All other regions 0 |
| prefs-orange-compact-square | 0.0000000202248 | 0.0000409199 | 0.000000429776 / 0.000188631 | All other regions 0 |
| prefs-pro-dark | 0 | 0 | 0 / 0 | All other regions 0 |
| prefs-studio-light | 0 | 0 | 0 / 0 | All other regions 0 |
| prefs-tune | 0.0000916474 | 0.00422342 | 0.00135479 / 0.0162383 | All other regions 0 |
| authority-lost | 0 | 0 | 0 / 0 | All other regions 0 |
| authority-present | 0 | 0 | 0 / 0 | All other regions 0 |
| authority-recovered | 0 | 0 | 0 / 0 | All other regions 0 |
| macros-menu | 0 | 0 | 0 / 0 | All other regions 0 |
| cli-light | 0 | 0 | 0 / 0 | All other regions 0 |

Tight overlay crops are the union of gold accessibility descendant bounds plus
16 pixels, clamped to the original image; each retained region JSON records
the rectangle. The following are additional comparisons of those same pairs:

| Overlay | Rectangle | SSIM | RMSE |
| --- | --- | ---: | ---: |
| about-dark | `[284,245,472,194]` | 0.00376813 | 0.0221282 |
| discard-dark | `[284,266,472,152]` | 0 | 0 |
| open-dialog-dark | `[115,248,721,187]` | 0 | 0 |
| save-as-dark | `[204,248,632,187]` | 0 | 0 |
| shortcuts-dark | `[284,111,472,569]` | 0 | 0 |
| menu-annotate | `[190,29,252,457]` | 0 | 0 |
| menu-capture | `[120,29,252,374]` | 0 | 0 |
| menu-edit | `[80,29,252,187]` | 0 | 0 |
| menu-file | `[41,29,252,198]` | 0 | 0 |
| menu-help | `[315,29,264,179]` | 0 | 0 |
| menu-view | `[267,29,252,148]` | 0 | 0 |
| macros-menu | `[587,24,252,113]` | 0 | 0 |


Originals, side-by-side figures and difference figures were inspected. The
spacious rail/options/content/status are exact. Remaining accepted raster
differences lie on header glyph/icon/caption and rounded frame-corner edges,
with native control widths differing by about 1/32 logical pixel; there is
no missing control, changed
label or shifted content. The native drag guide has only edge anti-aliasing.

## S2 — About build provenance (resolved)

Gold displays `0.1.2 (8779f33284b7)`; the port displays
`0.1.2 (5c3f3852bdd5)`, its actual built source. Layout and the rest of the dialog
match. The source stamp region is runtime data; its raw metrics above remain
unaltered and unmasked.

Decision: **Opus, 2026-11-07 — keep the truthful About source stamp**.
The standing brief now classifies build stamps, versions, SHAs, timestamps,
PIDs, hostnames and live readings as runtime data, not parity differences,
when their layout, format and position match gold. Never fake gold's values.
The original options were to retain truthful provenance or hold the visual
gate pending a provenance policy; this ruling resolves that policy and closes
S2. Evidence is the unaltered About pair and its full/region figures above.
No acceptance items remain open. This documentation-only update reuses the
exact-source release gate and native evidence recorded above.
