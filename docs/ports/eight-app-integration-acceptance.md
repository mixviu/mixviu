# Eight-app integration acceptance — 2026-11-08

Verdict: **ACCEPTED**. Tested runtime source:
`e1bbea99071c2fa8793a7e0e52aed381707d536f` on `lab/gold-parity`.
Later commits contain documentation receipts only. This tree merges Email
`70ea474` and Dopus `c66fe3c`, retaining both branches' shared facilities.
Gold is pinned MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
This is a source/artifact handoff; PR/CI builds the renamed cut-over bundle.
Gold term remains outside this port tree. Nothing was installed.

## Exact release bytes and gates

All four engines are together on **cbc2** at
`/build/es-artifacts/e1bbea99071c2fa8793a7e0e52aed381707d536f/`.
`sha256sum -c SHA256SUMS` verifies all four; the fetched Viu hash was checked
before preparing the native suite's exact-source bundle archive.

| File | SHA256 |
| --- | --- |
| mix | `fce979a2b83cc16b66445ea9ba2d8bab09d4b097895ce11467957e4c93e1ce6a` |
| mixd | `ca952cd5e7aa2199eacb7582f46afac221006d98eca2c8b359670b002768820a` |
| viud | `0b8277b44c7ce19e1440cd2beb496289df2177ccfa60ad9cb9695c0a2b1000ac` |
| viu | `3425765704e38b75898881c4e358f604d641f5e8c3f65e5af1baac4b05a109ad` |
| SHA256SUMS | `d108f8cc91af9fe650017265bd349c8324f468d673e562c3bd3921e4e698e545` |

Detached **gold-eight-full-r7-a1**, cbc2, returned rc 0. The complete
1,153,273-byte / 20,143-line log was read; SHA256
`aea9d0ebb6e341db86495115e4170c2ccf94535ae82bc2941dd6f2bc7aeb1f60`.
Saved basename: `20261108-080826-cbc2-e1bbea99071c-gold-eight-full-r7-a1-29c5ae28.log`.
Fmt, both isolated Clippy profiles, workspace/shell tests, all app Mix
policies, bulk primitives, P2 furniture, Track A (`SMOKE_FAILS=0`), four
release builds and empty worker diff pass. The 614 green reported test
batches include preparation repeats; existing ignores and warnings remain
visible. No failed result or incomplete stage is counted as a pass.

Native release execution uses detached **gold-eight-native-r7c** on the light
worker with the explicit native-only override: existing gold applications,
owned Bus/settingsd and headless compositor, no Rust build. Heavy builds
remain on cbc2/cbc3. All 42 execution groups, per-group original requests,
deadlines, binary/source identity and cleanup receipts must pass separately
from visual review. Image collectors run only after native execution.

## Revalidated behaviour and live settings

| App | Exact-release coverage |
| --- | --- |
| Monitor | Quick contract, 46/46 structured Bus replies, Processes and every Performance page; P1 PID-start-time fence retained |
| Ced | Eleven native suites: files/dirty gates, stable-caller edits/undo and second-caller refusal, recovery/notices, mail, 16/64 MiB, menus/input and live Prefs |
| Prefs | 69 aggregate checks including 40 functional cases, all 42 pages, drawers/catalogues, search, mutations/events, CLI/native input and authority recovery |
| Screenshot | Eleven suites, 55/55 Bus commands, annotation/save/export/clipboard callbacks, capture lifecycle, menus/dialogs and live Prefs/authority loss |
| Backdrop | Main, CLI, scale 1/1.5 and settled pipeline groups; native wallpaper formats/fit/slideshow/output/probes, train data and live settings |
| Workshop | 99/99 protocol replies, 25/25 macro cases, native menus/panels/Traffic, strict editor, live Prefs and saved selection/raw70 restoration |
| Email | Thirteen native groups; accounts/identities, MIME/reply/forward, independent composers and documents, attachment I/O, 1,200-message triage, PIM, native menus/controls, hosted framing and live Prefs. 86-command surface and 255 paired request probes plus 33 success records retained |
| Dopus | Thirteen groups including matched chooser; 44 gold command descriptors, 33 core actions, 90 paired requests, files/drag/preview, launch/close, hosting, authority loss/recovery, seventeen main and seventeen chooser live-Prefs states, real scan-budget refusals |

The original per-app suites retain their declared/default reply deadlines:
Prefs 95 seconds, Screenshot 10, Workshop 90 and the generic default for
the remaining bundles. Named and Calls witnesses also complete after 22
seconds while pings continue and cancellation works; Calls uses a one-second
ordinary endpoint deadline. Native deferred retention shares the original
request's flag. No refused application request is retried and no deadline,
assertion or required pair count was relaxed.

Workshop launches the **current generic Viu** with sibling Ced and Dopus
bundles. Its release witness checks New/title/Tab-to-Cancel, Unicode edits,
undo/redo/Save, Ced Bus/text/window and Dopus Bus/directory/window effects;
all thirteen owned PIDs stop. The old installed-native Dopus dependency is
removed. True Dopus source/build metadata is checked against this source.

The prior BUSY/Full admission defect was repaired before these merges at
accepted prerequisite `e2c3541a`: bounded runtime capacity credits and the
host's capacity check before removing native requests/notices leave the
original request queued. Original failures remain archived. The same-byte
release editor passed concurrently with fourteen other groups after Ced's
large-document suite. This final driver also runs the editor concurrently;
it does not use refused-request retries or serialise that witness away.
New child-window requests similarly retain their original request until
the first scene is drawn; Email's original failed initial-compose evidence is retained.

## Capture and region record

Method: [parity-method.md](parity-method.md). Gold first, identical fixture,
effective settings, viewport, focus and state; canonical native identities
and isolated homes. Most clients are 1040×680 at scale 1; original narrower,
composer, chooser and scale-1.5 sizes are retained without resizing. PNGs,
executed recipes, fixture hashes, native readbacks, original/side/diff figures,
region rectangles and both raw metrics are private evidence.

ImageMagick 7.1.2-35 reports normalised SSIM **distortion**: zero is exact.
No arbitrary score threshold grants acceptance. Every non-zero raw set is
listed below with its inspected disposition; every unlisted region is zero.
Diagnostic unmatched startup, scrolling, caret or transition frames remain
in the raw count and are superseded only by explicitly matched final states.

| App | Raw pairs | Exact raw pairs | Final disposition |
| --- | ---: | ---: | --- |
| monitor | 10 | 10 | All whole/region metrics 0 |
| ced | 35 | 31 | Every residual listed in the register below |
| prefs | 91 | 91 | All whole/region metrics 0 |
| screenshot | 47 | 44 | Every residual listed in the register below |
| backdrop | 44 | 35 | Every residual listed in the register below |
| workshop | 66 | 40 | Every residual listed in the register below |
| email | 44 | 31 | Every residual listed in the register below |
| dopus | 68 | 54 | Every residual listed in the register below |


### Matched final states and supplementary regions

Monitor's ten 1040×680 pairs have identical PNG hashes and full/header/options/content/status SSIM and RMSE **0**. Processes, TCP, CPU, Memory, three disks and Network are included. All 91 Prefs comparisons are also exact.

Ced retains the existing C1 Replace repair, whole 0.0111743/0.0324682, content 0.0120445/0.0356656, gutter 0.027558/0.0410017 and tabs/search 0.0138192/0.0083526. Its three other residuals are inspected native divider raster edges. Screenshot's About difference is its true source stamp (S2), and its guide arrow tip is antialiasing. No missing controls are accepted.

Backdrop's initial pipeline-clear scene is a transition diagnostic. The separately settled clear comparison is whole/all regions 0, as are scale-1.5 states. Native history shows actual readings under S2; the remaining history/Now legend edges are raster differences.

Workshop's first eight size frames and before-save wheel frame are diagnostics, preserved alongside the matched restored/raw70 frames, whose whole/all regions are 0. Both lanes start from the same saved 1040×680 viewport; restored selection offset is 809.5. All original passive/group/panel/selection and strict raw70 assertions pass. No scroll waiver is used. New-empty-tabbed is exact and proves Cancel owns focus after Tab. The earlier focused-title caret sample is retained. Help phases 4/5/6 and final menu Help are exact; phases 1/2/3 retain their different search-caret phase. Macro editor differences are the destination combo **chevron** raster, not the editor caret.

Workshop's final expanded Traffic crops have header/static content/status/controls 0. Empty bottom and right docks are whole/all regions 0. Populated bottom has whole 0.0183504/0.0705459 and data 0.082094/0.149212; right has whole 0.00496651/0.0319126 and data 0.0209246/0.0655036. Inspected differences are real Bus timestamps, sources, commands and durations (S2); static geometry is exact. Mesh's changing detail scrollbar is likewise live data, with static details 0.

Email's three initial extra-dialog comparisons retained widths from different first account draws. A detached same-release native supplement, **gold-eight-email-extra-matched-r7**, repeats the unchanged gold/candidate extra-dialog requests after matching the initial no-account state before window placement and restoring the exact saved account afterwards. Command search, delete and move are whole/header/content/status **0**; their original mismatches remain in the raw register. About/Shortcuts/contact/event retain only the same native status-divider raster as Calendar/Contacts. This is capture-state correction, not a new app deviation or a refused-request retry. The two expanded-attachment states retain the previously accepted C1 functional header repair.

Dopus's six matched chooser comparisons have header/navigation/pane/form **0**. Save's whole 0.000296392/0.0079461 and identity 0.00615716/0.0360983; short's whole 0.000260013/0.00934021 and identity 0.0137635/0.0675995 contain only the true broker caller (S2). External read-only phase samples independently confirm exact decoded form pixels for both save and short, preserving focus, text, all original actions and deadlines. The four original untuned chooser pairs remain diagnostic. Main live-Prefs, authority loss/recovery, native Macros popup and automatic day/night states are exact. Hosted mode retains its incoming C1 gold-refusal repair; About shows the true source stamp. Two initial baselines precede input/preview fixture insertion and remain diagnostics; subsequent matching feature frames are exact.

All original/side and non-zero diff sheets, the Monitor contact sheet, external chooser forms and expanded Traffic figures were inspected. Existing C1/P1, S2 and raster dispositions above are the complete remaining differences. There is no new design waiver or score-based acceptance.

### Complete non-zero raw comparison register

Numbers below are normalised SSIM/RMSE distortion pairs. Each row lists **all non-zero regions**, including full. Unlisted regions in that comparison are zero. Case numbers index the retained sorted raw register; the private record keeps original filenames, rectangles, hashes and figures. Diagnostic rows remain in the 405 count. No metric is replaced or omitted when a matched supplement supersedes a state.

| App/case and state | Non-zero regions: SSIM / RMSE | Inspected disposition |
| --- | --- | --- |
| ced/29 prefs-light-large-compact | status: 0.00355067 / 0.00564538; full: 0.000187397 / 0.00129694 | Raster: native one-pixel status divider |
| ced/33 search-replace | content: 0.0120445 / 0.0356656; gutter: 0.027558 / 0.0410017; tabs_search: 0.0138192 / 0.0083526; full: 0.0111743 / 0.0324682 | C1: existing functional Replace repair; gold defect retained in comparison |
| ced/34 view-output | content: 0.00000257402 / 0.00060235; gutter: 0.00000343203 / 0.000695534; full: 0.00000215932 / 0.000551697 | Raster: thin panel separator |
| ced/35 view-problems | content: 0.00000257402 / 0.00060235; gutter: 0.00000343203 / 0.000695534; full: 0.00000215932 / 0.000551697 | Raster: thin panel separator |
| screenshot/31 diff-about-dark | canvas: 0.000898965 / 0.0104964; full: 0.000717316 / 0.00937611 | S2: truthful own source stamp, identical placement |
| screenshot/37 diff-guide-dark | canvas: 0.00000000223216 / 0.0000082542; full: 0.00000000178112 / 0.00000737325 | Raster: guide arrow-tip antialiasing |
| screenshot/43 overlay-diff-about-dark | overlay: 0.00553999 / 0.0260568; full: 0.000717316 / 0.00937611 | S2: truthful own source stamp, identical placement |
| backdrop/10 compare-native-2d | history: 0.00014116 / 0.00427989; full: 0.0000394417 / 0.00226233 | S2: actual native history readings |
| backdrop/11 compare-native-3d | history: 0.000261134 / 0.00609691; full: 0.0000729639 / 0.00322279 | S2: actual native history readings |
| backdrop/14 compare-pipeline-alert | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; full: 0.000000430271 / 0.0000666045 | Raster: history/Now legend glyph edges |
| backdrop/15 compare-pipeline-alerts | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; full: 0.000000430271 / 0.0000666045 | Raster: history/Now legend glyph edges |
| backdrop/16 compare-pipeline-clear | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; scene: 0.00599815 / 0.0240108; full: 0.00201158 / 0.0139035 | Diagnostic: transient pipeline clear; settled clear whole/all regions 0 supersedes scene; legend raster retained |
| backdrop/17 compare-populated-2d | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; full: 0.000000430271 / 0.0000666045 | Raster: history/Now legend glyph edges |
| backdrop/18 compare-populated-3d | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; full: 0.000000430271 / 0.0000666045 | Raster: history/Now legend glyph edges |
| backdrop/29 compare-train-2d | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; full: 0.000000430271 / 0.0000666045 | Raster: history/Now legend glyph edges |
| backdrop/30 compare-train-3d | history: 0.00000153992 / 0.000126003; legend: 0.0000322338 / 0.000576486; full: 0.000000430271 / 0.0000666045 | Raster: history/Now legend glyph edges |
| workshop/1 integration-figures-macro-open | details: 0.0000896649 / 0.00425608; details_static: 0.0000918062 / 0.0043066; full: 0.0000495769 / 0.00316474 | Raster: destination combo chevron only, not editor caret |
| workshop/2 integration-figures-macro-saved | details: 0.0000896649 / 0.00425608; details_static: 0.0000918062 / 0.0043066; full: 0.0000495769 / 0.00316474 | Raster: destination combo chevron only, not editor caret |
| workshop/3 integration-figures-macro-typed | details: 0.0000897161 / 0.00425684; details_static: 0.0000918586 / 0.00430737; full: 0.0000496052 / 0.00316531 | Raster: destination combo chevron only, not editor caret |
| workshop/5 integration-figures-new-empty | dialog: 0.00104317 / 0.0170049; title_field: 0.0106866 / 0.0548226; tree: 0.000293388 / 0.00901815; full: 0.0000837101 / 0.00481709 | Diagnostic: focused caret phase; matched Tab-to-Cancel whole/all regions 0 |
| workshop/7 integration-figures-size-1 | details: 0.0211603 / 0.0783777; details_static: 0.0215954 / 0.0792658; traffic: 0.0746699 / 0.144306; tree: 0.00244009 / 0.0293152; full: 0.017008 / 0.0701996 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/8 integration-figures-before-save | details: 0.00706613 / 0.0443495; details_static: 0.00674027 / 0.0438146; traffic: 0.0197566 / 0.0396625; tree: 0.102031 / 0.136179; full: 0.0343549 / 0.0804732 | Diagnostic: in-progress smooth wheel; matched restored and raw70 whole/all regions 0 |
| workshop/11 integration-figures-size-1 | details: 0.0221006 / 0.0801664; details_static: 0.0226283 / 0.081118; traffic: 0.0743877 / 0.135935; tree: 0.00402438 / 0.0367723; full: 0.0179625 / 0.0712777 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/12 integration-figures-help-1 | header: 0.000354783 / 0.00478142; tree: 0.000166452 / 0.00832929; full: 0.0000588537 / 0.00459047 | Diagnostic: menu search caret phase; Help 4/5/6 and final repaired Help whole/all regions 0 |
| workshop/13 integration-figures-help-2 | header: 0.000354783 / 0.00478142; tree: 0.000166452 / 0.00832929; full: 0.0000588537 / 0.00459047 | Diagnostic: menu search caret phase; Help 4/5/6 and final repaired Help whole/all regions 0 |
| workshop/14 integration-figures-help-3 | header: 0.000354783 / 0.00478142; tree: 0.000166452 / 0.00832929; full: 0.0000588537 / 0.00459047 | Diagnostic: menu search caret phase; Help 4/5/6 and final repaired Help whole/all regions 0 |
| workshop/18 integration-figures-size-1 | details: 0.0219217 / 0.0798931; details_static: 0.0224118 / 0.0808366; traffic: 0.0765603 / 0.138489; tree: 0.00326275 / 0.0339308; full: 0.017782 / 0.071009 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/22 integration-figures-menu-help | header: 0.000354783 / 0.00478142; tree: 0.000166452 / 0.00832929; full: 0.0000588537 / 0.00459047 | Diagnostic: menu search caret phase; Help 4/5/6 and final repaired Help whole/all regions 0 |
| workshop/26 integration-figures-size-1 | details: 0.0221644 / 0.0802172; details_static: 0.0226234 / 0.0811281; traffic: 0.0765322 / 0.137623; tree: 0.00365739 / 0.0355889; full: 0.0180255 / 0.0713379 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/28 integration-figures-traffic-bottom | details: 0.0230684 / 0.0781254; details_static: 0.0236193 / 0.0790528; traffic: 0.0715844 / 0.139589; tree: 0.00414318 / 0.0368578; full: 0.0183598 / 0.0704688 | Diagnostic: earlier unmatching populated Traffic state; final expanded static regions 0 supersede; actual rows S2 |
| workshop/29 integration-figures-traffic-narrow | details: 0.00932675 / 0.0452645; details_static: 0.00939434 / 0.045739; traffic: 0.000823663 / 0.0138807; full: 0.00521551 / 0.0338342 | Diagnostic: earlier unmatching populated Traffic state; final expanded static regions 0 supersede; actual rows S2 |
| workshop/31 integration-figures-traffic-right | details: 0.00917006 / 0.0442959; details_static: 0.00934942 / 0.0448099; traffic: 0.000823663 / 0.0138807; full: 0.00512887 / 0.0331177 | Diagnostic: earlier unmatching populated Traffic state; final expanded static regions 0 supersede; actual rows S2 |
| workshop/35 integration-figures-mesh | details: 0.000285207 / 0.00680472; full: 0.000157695 / 0.00505986 | S2: live mesh detail scrollbar; static details 0 |
| workshop/37 integration-figures-size-1 | details: 0.021156 / 0.078451; details_static: 0.021591 / 0.07934; traffic: 0.0739101 / 0.142974; tree: 0.00242946 / 0.0292413; full: 0.0169556 / 0.0700676 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/40 integration-figures-macro | details: 0.0000896649 / 0.00425608; details_static: 0.0000918062 / 0.0043066; full: 0.0000495769 / 0.00316474 | Raster: destination combo chevron only, not editor caret |
| workshop/42 integration-figures-size-1 | details: 0.0211107 / 0.0784981; details_static: 0.0216149 / 0.0794298; traffic: 0.0741488 / 0.142895; tree: 0.00298205 / 0.0324246; full: 0.0171121 / 0.0704853 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/48 integration-figures-prefs-fonts-scale | details: 0.0000456998 / 0.000657549; details_static: 0.0000467911 / 0.000665354; full: 0.000025268 / 0.000488941 | Raster: native thin divider/control edges |
| workshop/53 integration-figures-size-1 | details: 0.0211574 / 0.078472; details_static: 0.0215924 / 0.0793612; traffic: 0.0761758 / 0.145792; tree: 0.00398442 / 0.0369102; full: 0.01754 / 0.0714584 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/61 integration-figures-size-1 | details: 0.0211308 / 0.0782516; details_static: 0.0215652 / 0.0791381; traffic: 0.0755231 / 0.145155; tree: 0.0037005 / 0.0363035; full: 0.017404 / 0.0711553 | Diagnostic: unmatched first-frame data/layout; later settled states retained |
| workshop/63 integration-figures-traffic-bottom | details: 0.0231729 / 0.0785473; details_static: 0.0237262 / 0.0794797; traffic: 0.0705162 / 0.138972; tree: 0.00413924 / 0.0361343; full: 0.0183504 / 0.0705459 | S2: true Traffic rows; expanded static/header/controls/status 0, both empty docks all 0 |
| workshop/64 integration-figures-traffic-narrow | details: 0.00913195 / 0.0439226; details_static: 0.00919489 / 0.0443792; full: 0.00505336 / 0.03266 | S2: true Traffic rows; expanded static/header/controls/status 0, both empty docks all 0 |
| workshop/66 integration-figures-traffic-right | details: 0.00897486 / 0.0429175; details_static: 0.00914956 / 0.0434147; full: 0.00496651 / 0.0319126 | S2: true Traffic rows; expanded static/header/controls/status 0, both empty docks all 0 |
| email/1 compare-r5-attachments-expanded | reader: 0.00368646 / 0.0150809; full: 0.00126013 / 0.00881719 | C1: existing expanded-attachment header functional repair |
| email/2 compare-r5-attachments-no-inline | reader: 0.00368646 / 0.0150809; full: 0.00126013 / 0.00881719 | C1: existing expanded-attachment header functional repair |
| email/10 compare-extra-about | status: 0.00034897 / 0.00121496; full: 0.0000151023 / 0.000255193 | Raster: native one-pixel status divider; content/header 0 |
| email/11 compare-extra-command-search | content: 0.00168675 / 0.0175478; full: 0.00151807 / 0.0166473 | Diagnostic: unmatched first account draw; same-byte symmetric-initial-state supplement whole/all regions 0 supersedes |
| email/12 compare-extra-contact-form | status: 0.00034897 / 0.00121496; full: 0.0000151023 / 0.000255193 | Raster: native one-pixel status divider; content/header 0 |
| email/13 compare-extra-delete-form | content: 0.00168677 / 0.0175478; full: 0.0015181 / 0.0166473 | Diagnostic: unmatched first account draw; same-byte symmetric-initial-state supplement whole/all regions 0 supersedes |
| email/14 compare-extra-event-form | status: 0.00034897 / 0.00121496; full: 0.0000151023 / 0.000255193 | Raster: native one-pixel status divider; content/header 0 |
| email/15 compare-extra-move-form | content: 0.00168677 / 0.0175478; full: 0.0015181 / 0.0166473 | Diagnostic: unmatched first account draw; same-byte symmetric-initial-state supplement whole/all regions 0 supersedes |
| email/16 compare-extra-shortcuts | status: 0.00034897 / 0.00121496; full: 0.0000151023 / 0.000255193 | Raster: native one-pixel status divider; content/header 0 |
| email/19 compare-calendar | status: 0.00034897 / 0.00121496; full: 0.0000151023 / 0.000255193 | Raster: native one-pixel status divider; content/header 0 |
| email/21 compare-contacts | status: 0.00034897 / 0.00121496; full: 0.0000151023 / 0.000255193 | Raster: native one-pixel status divider; content/header 0 |
| email/31 compare-prefs-font-scale-compose | header: 0.00000000111203 / 0.0000113206; full: 0.000000000158862 / 0.00000427878 | Raster: two compositor header glyph-edge cases |
| email/33 compare-prefs-icon-mono-tune-compose | header: 0.00000000111203 / 0.0000113206; full: 0.000000000158862 / 0.00000427878 | Raster: two compositor header glyph-edge cases |
| dopus/2 integrated-matched-options-passive | identity: 0.005647 / 0.0429797; full: 0.000269661 / 0.00946086 | S2: true broker caller identity only; header/navigation/pane/form 0, exact caret supplements retained |
| dopus/3 integrated-matched-options | identity: 0.005647 / 0.0429797; full: 0.000269661 / 0.00946086 | S2: true broker caller identity only; header/navigation/pane/form 0, exact caret supplements retained |
| dopus/4 integrated-matched-replace | identity: 0.00528972 / 0.0417812; full: 0.000252867 / 0.00919705 | S2: true broker caller identity only; header/navigation/pane/form 0, exact caret supplements retained |
| dopus/5 integrated-matched-save | identity: 0.00615716 / 0.0360983; full: 0.000296392 / 0.0079461 | S2: true broker caller identity only; header/navigation/pane/form 0, exact caret supplements retained |
| dopus/6 integrated-matched-short-passive | identity: 0.0137635 / 0.0675995; full: 0.000260013 / 0.00934021 | S2: true broker caller identity only; header/navigation/pane/form 0, exact caret supplements retained |
| dopus/7 integrated-matched-short | identity: 0.0137635 / 0.0675995; full: 0.000260013 / 0.00934021 | S2: true broker caller identity only; header/navigation/pane/form 0, exact caret supplements retained |
| dopus/8 integrated-compare-baseline.png | panes: 0.00716812 / 0.0371828; properties: 0.00699557 / 0.0467956; full: 0.00549227 / 0.0349188 | Diagnostic: input/preview fixture added between lanes; matching operation states retained |
| dopus/15 integrated-compare-hosted-mode-result.png | header: 0.418108 / 0.705453; panes: 0.329948 / 0.756498; places: 0.440045 / 0.873724; properties: 0.341922 / 0.753471; status: 0.453948 / 0.878322; full: 0.338059 / 0.753376 | C1: gold refuses hosted mode change; port follows mandated live mode |
| dopus/22 integrated-compare-about.png | panes: 0.00122489 / 0.0122535; full: 0.000717623 / 0.00937904 | S2: own true source stamp |
| dopus/61 integrated-legacy-chooser-options | full: 0.000479281 / 0.00984662 | Diagnostic: untuned original chooser geometry/caret; six explicitly matched chooser pairs supersede |
| dopus/62 integrated-legacy-chooser-replace | full: 0.000354427 / 0.00859309 | Diagnostic: untuned original chooser geometry/caret; six explicitly matched chooser pairs supersede |
| dopus/63 integrated-legacy-chooser-save | full: 0.000475218 / 0.00936353 | Diagnostic: untuned original chooser geometry/caret; six explicitly matched chooser pairs supersede |
| dopus/64 integrated-legacy-chooser-short | full: 0.000449117 / 0.0105822 | Diagnostic: untuned original chooser geometry/caret; six explicitly matched chooser pairs supersede |
| dopus/67 integrated-compare-baseline.png | panes: 0.00548826 / 0.0316899; properties: 0.00581951 / 0.0414829; full: 0.00429181 / 0.0301665 | Diagnostic: input/preview fixture added between lanes; matching operation states retained |


## 2,000-row measurements

Verified release bytes; native probes run alone before capture work, three warmups and 21 samples. Email fixture SHA256 `1ca642d313c13d9a6eb58c086f51336650076ed2897d2a18f0e94b43e36cd780`. Callback/projection stages remain distinct from Bus ACK and applied readback.

| Stage (ms) | Accepted median / p95 | Integrated median / p95 |
| --- | ---: | ---: |
| Email projection |13.056 /16.967|12.347 / 12.607|
| Dopus sort callback |67.138 /68.418|66.088 / 66.769|
| Dopus sort native ACK |111.892 /129.111|111.419 / 112.158|
| Dopus refresh callback |67.651 /68.971|66.473 / 67.115|
| Dopus refresh native ACK |113.293 /129.114|111.704 / 127.833|

Email projection p99 16.325 ms. Callbacks/projection meet the roughly 100 ms target. The real 65,537-name refusal preserves rows, location and model serial; no truncated successful scan. Detached **gold-eight-release-profile-r7-a1** passes the unchanged release profile; full log read.

**gold-eight-perf-e1bbea99071c-a1** passes 510 same-worker correctness samples against 4c1d715, thirty Latin-rotated rounds per action. Historical ACK timings are **materially slower**, not unchanged:

| Action | 4c1d715 median / p95 ms | Integrated median / p95 ms |
|---|---:|---:|
|view.sort-size|115.073 / 132.256|165.024 / 173.110|
|view.sort-name|98.505 / 122.015|165.239 / 172.766|
|view.refresh|101.973 / 110.358|148.247 / 160.059|
|selection.first|52.990 / 71.955|103.050 / 119.575|
|selection.last|62.724 / 72.072|100.370 / 118.559|
|email.list|114.052 / 118.272|168.634 / 187.960|

These execute ACKs and separate verified readbacks are not presentation times. Generic hosts return full state where the older endpoint accepted compact state; that changes readback work but does not explain the ACK regression by itself. No causal claim or timing equivalence follows from green correctness. Raw samples, profiles, load and final all-seven-process exit audit are retained.

## Provenance, history and handoff

The [22 inherited Mix import-hash mismatches](import-hash-debt.md) remain
provenance debt before production derivation. The current audit confirms
the same inventory: source-pin replacement and recorded Track A/bulk
adaptations predate this round. The linked table names every file, first
divergence, expected/current hash and reason. This acceptance does not turn
`verify_runtime` green or erase that debt.

Rejected source `317eea1a` and its originals remain NOT_ACCEPTED. Actual
image inspection drove the shared bottom-dock, header-spacing, macro-anchor,
padding, empty-error, authority fallback/theme-notice, short-scroll and
fractional table-row repairs. Tests retained their original assertions;
new native widget regressions cover the observed failures. Both sides'
Email counted outline and Workshop recursive/reveal layout are retained.

The private archive `gold-parity-20261107/eight-app-integration/e1bbea99071c2fa8793a7e0e52aed381707d536f`
contains all final raw pairs, external Monitor runs, matched caret supplements,
expanded Traffic regions, recipes/support fixtures, native request/reply and
cleanup receipts, complete CBC logs, raw performance and failed preflights.
The worker HANDOFF files are written only after the native audit and visual
acceptance agree. The four-engine set and SHA256SUMS remain together for
the PR/CI cut-over session; this job does not create apps.tar or install it.

## Native fixture history and closure

Final native CBC gold-eight-native-r7c returns BUILD_RC=0; the complete saved log was read. All 42 groups pass, with zero refused-request retries. The preceding r7/r7b attempts each passed 41 groups: Prefs' gold lane failed before candidate launch, first because the private HTTP fixture port was already bound, then because its readiness caller used an unsupported fractional timeout. The fixture now reserves a kernel-assigned port and requires HTTP 200 before launching gold, using the existing integer timeout contract. A focused original install/rollback/remove witness passed before the full 42-group rerun. Original failures, unchanged assertions/deadlines, and partial fixture preparation are retained.

The final read-only closure audit covers 71 closure receipts and 557 PID records, including external Monitor and the Email supplement; every owned PID is stopped. The separate performance audit confirms all seven benchmark PIDs stopped. Workshop's strict concurrent editor witness checks generic Ced/Dopus effects and all thirteen owned PIDs stopped.

Final native applied-model readback medians/p95: Dopus sort 127.218/129.370 ms, refresh 127.153/142.920 ms. Final Email projection is 12.347/12.607/16.325 ms (median/p95/p99). All modern probes use 21 samples after three warmups. The historical 510-sample comparison and release callback profile are the same frozen-source jobs, not repeated after private fixture-only corrections.

The final whole-manifest audit also records 49 inherited vendor entries incorrectly declared removed while their tracked bytes match the imported hashes. These are stale retention flags, separate from the 22 Mix hash mismatches; the linked provenance debt record lists every file and origin. Both kinds must be reconciled before production derivation.
