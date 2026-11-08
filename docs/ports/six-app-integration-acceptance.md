# Six-app integration acceptance — 2026-11-08

**ACCEPTED for the lab integration handoff.** Tested runtime source
`0124e5c58c889f1d93c730b92993e2e74b707464`, branch `lab/gold-parity`.
This combines Monitor, Ced, Prefs, Screenshot, Backdrop, Workshop and the
bulk primitives from `4c1d715`. Later documentation receipts do not change
the tested code or binaries. No installation or desktop/VT work occurred.

The [22 inherited import-hash mismatches](import-hash-debt.md) remain open
before production derivation. This acceptance covers application behaviour,
Bus compatibility, appearance and the integrated release set. Generic Dopus
migration remains separate.

## Source, gates and handoff

Gold is MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, launched through
explicit `/opt/mixos/bin/<app>` paths. Candidate bundles came from the exact
frozen source. The release viu was fetched and hash-verified before native
testing. The checkout stayed clean throughout all sixteen execution groups.

| Pinned gold executable | SHA256 |
| --- | --- |
| monitor | `59d016a2eadf7107e5b8ccbbe32c4cf006c9616b4a26316ade5c20ebeb3d93f9` |
| ced | `d83f730e3d69b10f7bd1ab0e4cec5cd7bcf299b4705f864393c8cea96e4aa328` |
| prefs | `18590ca486b4b417b9de5a762f2dcf31b6b13d2351bb3eddd46d66e783fe3c55` |
| screenshot | `f3b5bad3b0f43a3183a9810fe9178a9f445801c63c5e06e313bd4caaa2e3f932` |
| backdrop | `3501c2d6d867689d873758b84284e777fa5f68a4db094c5fae36bfba83c01bac` |
| workshop | `21303df6158fbc7c856aa39525a9bacff31a258f05d03f1b6602de6aeb8e52e6` |

All four engines are together on **cbc2** at
`/build/es-artifacts/0124e5c58c889f1d93c730b92993e2e74b707464/`:

| Engine | SHA256 |
| --- | --- |
| mix | `68856fdceb020681257ebe07b6d35975b56fb1089071d6a2b823aa2e78100a8f` |
| mixd | `e28cd8cd8c6e3fa4e71d12fdcea335bcc173c8c7315399358eb04d1b57df5cad` |
| viud | `c6bcb6047af1a2d13c5ed97178a2520028f597ce6e5a4b830dc708abd51d9ffd` |
| viu | `90bcb22785a223291d0c2152eef2c78ccc63e2abb96dc2cf6c52583ef7f769bf` |

`SHA256SUMS` SHA256:
`c3d7694fce426855334dc622c97958f9cdaa2790ba84159280ce7b8dad0e6e9a`.
Accepted `HANDOFF.json` and `HANDOFF.md` sit beside the engines. Their SHA256s
are `c113aa3b050f968704ae6f4addcb9d61162aabcc68ececa056167f252d37b714`
and `1b1b11c6c4af334bf7bf818721cd6b400888efeadd855f476f2306279c4ee533`.
All engine and manifest hashes were checked again when writing the handoff.

| CBC job | Result and scope |
| --- | --- |
| `gold-six-full-20261107-r8-a1` | PASS, detached cbc2: fmt, workspace and isolated-shell Clippy, shell/workspace tests, source-built Mix app policies, bulk tests, P2 furniture, Track A `SMOKE_FAILS=0`, four release builds, empty worker diff |
| `gold-six-perf-0124e5c58c88-a1` | PASS, detached cbc2: same-worker 2,000-row comparison, 510 checked native samples |
| `gold-six-focus-test-r8b-a1` | PASS, scoped/light: all seven text-input tests, including title/path Tab retention and Enter cancellation |
| `gold-six-focus-debug-r8b-a1` | PASS, detached cbc2: debug repair/native Tab witness; final acceptance uses the release bytes above |

The complete full-gate log is retained: **1,138,382 bytes / 19,797 lines**,
SHA256 `fbf2680daba0993899d1be42b5d5b83d1eb838c1cc85123d9a68916efe2b399a`.
Its stages and test results were reviewed. Existing warnings and 100 ignored
entries remain visible; ignores are not counted as passes. The complete
performance log and raw distributions are retained too.

## Release-byte native revalidation

Original assertions and deadlines, request/reply transcripts, fixture hashes
and owned-process closure receipts are retained. The final audit verifies
all sixteen native groups, byte identity and these capture counts:

| App | Final release evidence | Raw comparison sets / exact sets |
| --- | --- | ---: |
| Monitor | Quick Bus **46/46**, Processes and every Performance page; ten 1040×680 pairs, whole/header/options/content/status SSIM and RMSE all **0** | 10 / 10 |
| Ced | All **eleven** suites: actions with stable caller and cross-caller undo refusal, native input, files/prompts, mail, notices, recovery, 16/64 MiB and live Prefs | 35 / 31 |
| Prefs | Ten groups; **69/69** aggregate checks including the **40-case** functional matrix; all **42 pages**, drawers/catalogues, events, mutations, CLI and live authority | 91 / 88 |
| Screenshot | **55/55** Bus cases, all **eleven** suites, native input/export/clipboard/lifecycle/CLI/deferred replies and live Prefs | 47 / 42 |
| Backdrop | Four groups: main, scale 1.5, settled scenes and CLI; placement/click-through, image formats/fits, slides, refusals and appearance authority | 44 / 35 |
| Workshop | **99/99** protocol and **25/25** macro replies; native menus/clipboard, agents/permissions, mesh, editor, persistence, live Prefs and authority loss | 66 / 39 |

Raw counts include overlay crops and the explicit startup/scroll/blink
diagnostics below. They are not a claim that every raw frame is a matched
acceptance state.

The strict Workshop editor ran alone before the other groups. New/title,
Unicode typing, undo/redo, Save, integrated generic Ced Bus/window/text and
installed native Dopus directory/window effects pass; all thirteen PIDs
exited. Added native Tab assertions require Cancel focused and Title
unfocused. The new 1040×680 tabbed pair is whole/all-region **0 / 0**.
The reusable gold dialog now requests field focus only when no widget owns
keyboard focus, exactly as gold does.

The earlier sixteen-app concurrent editor probe's `BUSY/Full` is preserved
alongside its unchanged same-byte isolated pass. Final interop evidence uses
isolated paired runs per [parity-method.md](parity-method.md). No refused
request was retried or ignored.

App-native asynchronous suites and shared deferred witnesses pass. Named-token
95-second and Calls 180-second configurations each retain a request for 22
seconds, answer 73 intervening pings, deliver the original reply and return
`CANCELLED` on explicit cancellation. Screenshot's `keep` also passes beyond
its ordinary ten-second response deadline. Application operation deadlines
and cancellation policy remain unchanged.

Live Prefs captures cover mode/scheme/style/accent/density/roundness, fonts,
weight/size/mono family, text scale/stroke, app overrides, auto-mode, title
controls and authority loss/recovery on unchanged PIDs. Monitor's final run
is the requested quick recheck; its earlier complete 19-state live-Prefs
acceptance remains the reference. The other final native suites repeat their
live settings matrices. Background appearance retention is opt-in for
Backdrop. Ordinary windows recover gold's file palette; Workshop and
Screenshot use gold's automatic window fallback on authority loss.

## Visual inspection and remaining differences

The recipe is [parity-method.md](parity-method.md): equal initial viewport,
state/data/settings, no image resizing, native readback and separate region
figures. Most client captures are 1040×680 at scale 1; Ced uses its accepted
1000×720 fixture; Backdrop repeats at scale 1.5. Every decoded size, PNG hash
and crop rectangle is in the private index. ImageMagick 7.1.2-35 reports
normalised **distortion**, zero identical. No score threshold grants acceptance.

All 293 paired sets were inspected through the paired sheets, with originals
and full differences for nonzero regions. Expanded Traffic was recropped
separately. Final index SHA256:
`980abc493431dbaf61c549bbf2ef9da76b1db9df5f5c4964db58d944c3bf4f71`.
The appendix lists every raw nonzero set and region. Its numbers are **SSIM /
RMSE distortion**; unlisted regions are zero.

- **Monitor:** all ten pairs and every region 0. Existing invisible Fable P1
  process-start-time fence retained; no new visual deviation.
- **Ced:** C1 Replace deliberately exposes the controls gold clips. Canonical
  whole 0.0111743 / 0.0324682; content 0.0120445 / 0.0356656; gutter
  0.027558 / 0.0410017; tabs/search 0.0138192 / 0.0083526. Header/actions/status
  0. Fresh raw pair whole 0.0112466 / 0.0328173 changes only gold caret phase:
  candidate PNG is byte-identical to the preceding release. The two gold PNGs'
  inspected diff contains only Find/editor carets. Comparing canonical gold
  with the **current release capture** reproduces the accepted C1 figures.
  Output/Problems retain their separator raster edge; light/large/compact
  retains Search-icon and separator raster edges. No new C1 repair is claimed.
- **Prefs:** three nonzero states, only the Macros header glyph raster;
  sidebar/content 0. All 42 pages and remaining pairs exact.
- **Screenshot:** About shows its true own source stamp (S2), with matching
  format/position. Two header glyph raster cases and the guide edge remain.
  Open dialog whole and overlay are now **0 / 0** after actual left/right
  cursor motion resets the blink phase. No caret waiver. Its own debug-target
  list remains truthful S2; shared directives/levels/clamps match.
- **Backdrop:** two native train-history pairs contain S2 live sample extent;
  seven populated/train/pipeline pairs retain trace/legend raster edges.
  Scene geometry is 0, including main pipeline-clear. All scale-1.5 and all
  three settled scene pairs are **0 / 0**. Its recorded Mark-mandated live
  Prefs/new-output improvement remains where gold ignores the setting.
- **Workshop:** macro open/saved/typed/panel differences are the destination
  combo **chevron** raster, not editor carets. Two live-Prefs cases retain
  Macros glyph/button-edge raster. Mesh differs only at the S2 live scrollbar;
  static details and other regions 0. Command/About/Shortcuts/Agents, final
  menus, Tabbed New/title, restored selection/raw70 and authority loss are
  exact. Populated Traffic differences are S2 data; correct expanded crops
  prove all static regions exact as shown below.

### Workshop expanded Traffic

| State | Whole SSIM / RMSE | Data SSIM / RMSE | Header/static content/controls/status |
| --- | --- | --- | --- |
| Bottom, populated | 0.0187066 / 0.0714922 | 0.0836876 / 0.151214 | **0 / 0** |
| Right, populated | 0.00606559 / 0.0378831 | 0.0255551 / 0.0777585 | **0 / 0** |
| Bottom, empty | **0 / 0** | **0 / 0** | **0 / 0** |
| Right, empty | **0 / 0** | **0 / 0** | **0 / 0** |

Only actual observed row membership, timestamps, commands, readings and wire
sizes differ. Reply comparisons still check payloads and correlation. The
collapsed crop map crosses the data panel when expanded, so its raw
`details_static` name does not establish an expanded static-region difference.
Separate expanded crops and full differences establish the boundary above.

### Temporal diagnostics and matched closure

| Raw set(s) retained | Difference | Matched closure |
| --- | --- | --- |
| Eight Workshop `size-1` frames | Initial live Traffic/pre-settled layout | Same runs' settled states; initial frames are diagnostic |
| Persistence `before-save` | Fractional in-progress wheel offset/live Traffic | Equal saved 1040×680 viewports before restart; restored selection (809.5px) and strict raw70 both whole/all-region 0 |
| New macro `new-empty` | Title caret phase only | Added Tab pair exact, with real Cancel/Title focus assertions; typed-title state exact |
| Help phases 2/3 and original native Help | Search-menu caret only | Help phases 1/4/5/6 and final-native Help exact; all phases retained |
| Original native right-empty Traffic | Filter caret only | Final-native empty dock every expanded region exact |
| Original populated Traffic and final-native narrow | S2 observed rows; collapsed crop names cross expanded data | Correct final bottom/right crops above; separate collapsed narrow-window layout exact |

No C1 scroll waiver or blanket caret tolerance is used. P1 and Ced C1 are
the existing approved deviations. S2 preserves truthful runtime data; the
remaining matched cosmetic differences are inspected raster edges.

## Same-worker 2,000-row comparison

Exact release versus `4c1d7153f6e13da0e1f67317af9c3b4518ff5e09`, cbc2,
30 warm Latin-rotated rounds, 2,000 actual files, alternating Email order and
an actual 2,001st refresh witness. All **510** native samples pass row counts,
sorting/selection/revision and applied-model assertions. All four hashes
verify; no compiler active. ACK and model readback are separate, with no
physical-presentation claim. Original gold timings and full distributions
remain retained; the generic host is still slower than original gold.

| Operation | ACK median before → after, ms | ACK p95 before → after, ms | Applied-model median before → after, ms |
| --- | ---: | ---: | ---: |
| view.sort-size | 114.931 → 113.656 | 123.937 → 119.756 | 131.037 → 131.032 |
| view.sort-name | 108.682 → 111.060 | 124.682 → 116.604 | 125.418 → 126.104 |
| view.refresh | 101.000 → 103.123 | 111.804 → 106.434 | 115.622 → 119.426 |
| selection.first | 64.006 → 66.174 | 70.976 → 70.845 | 78.773 → 81.955 |
| selection.last | 62.545 → 64.049 | 70.142 → 69.835 | 77.935 → 79.099 |
| email.list | 115.892 → 115.284 | 135.892 → 140.686 | 133.003 → 133.793 |

Bulk correctness/hot paths are retained. Median shifts span −1.1% to +3.4%;
timings are not identical. Email median decreases 0.5%, p95 increases 3.5%.
No equivalence threshold or new latency target is claimed. Initial cleanup
caught the broker still exiting; the separate final audit confirms all seven
PIDs gone, preserving the original receipt.

## Provenance and retained evidence

The fresh audit confirms the same **22** files explained in
[import-hash-debt.md](import-hash-debt.md), with expected/current hashes,
first divergence and subsequent adaptations. Twenty arise from A1 importing
newer `8779f33` Mix under an inventory pinned to `884fdd8`; A5's host-header
parser and the later bulk HOF import explain the other two. They predate this
work and are unchanged from accepted `4f2c0fd`. Reconcile pins/adaptations
before updating inventory and deriving production. `verify_runtime` stays red.

The private source-keyed archive contains executed recipes/support fixtures,
all originals/regions/figures, native transcripts, cleanup, raw performance,
complete CBC logs, inspected sheets, and outside-root Monitor runs with a path
mapping. Previous failed source-73 appearance retention, source-2894 chrome
loss, source-5bd Tab focus, concurrent admission and fixture-preparation
attempts remain retained. Source-5bd's handoff remains NOT_ACCEPTED.

Workshop opens Ced through the generic host. Dopus interop uses the installed
native dependency with explicit `MIXOS_PREFIX=/opt/mixos`, as gold does. This
does not attest the separate unfinished generic Dopus port.

## Raw nonzero comparison appendix

This exhaustive table retains the diagnostics as well as accepted raster,
C1 and S2 differences. The dispositions and canonical replacements above
govern acceptance. Values are normalised SSIM / RMSE; absent regions are 0.

| App / capture | Whole | Other nonzero regions |
| --- | --- | --- |
| ced / accepted-integrated/prefs-light-large-compact | 0.000301469 / 0.00405315 | actions 0.0147586 / 0.0436787; status 0.00355067 / 0.00564538 |
| ced / accepted-integrated/search-replace | 0.0112466 / 0.0328173 | content 0.0120445 / 0.0356656; gutter 0.0292901 / 0.0471239; tabs_search 0.0142043 / 0.0138868 |
| ced / accepted-integrated/view-output | 0.00000215932 / 0.000551697 | content 0.00000257402 / 0.00060235; gutter 0.00000343203 / 0.000695534 |
| ced / accepted-integrated/view-problems | 0.00000215932 / 0.000551697 | content 0.00000257402 / 0.00060235; gutter 0.00000343203 / 0.000695534 |
| prefs / pair/authority-recovered | 0.0000927844 / 0.00306816 | header 0.00166035 / 0.012979 |
| prefs / pair/search-edited | 0.0000927844 / 0.00306816 | header 0.00191192 / 0.0139276 |
| prefs / pair/variant-compact | 0.0000927844 / 0.00306816 | header 0.00191192 / 0.0139276 |
| screenshot / advanced/diff-prefs-fonts | 0.0000917354 / 0.00422666 | header 0.00194938 / 0.019484 |
| screenshot / advanced/diff-prefs-orange-compact-square | 0.000090203 / 0.00323826 | header 0.00191681 / 0.0149276 |
| screenshot / matched/diff-about-dark | 0.000597788 / 0.00887363 | canvas 0.000749168 / 0.00993384 |
| screenshot / matched/diff-guide-dark | 0.00000000178112 / 0.00000737325 | canvas 0.00000000223216 / 0.0000082542 |
| screenshot / matched/overlay-diff-about-dark | 0.000597788 / 0.00887363 | overlay 0.00461685 / 0.0246604 |
| backdrop / round3-final-recipe/pair/compare-native-2d | 0.0000195026 / 0.000567021 | history 0.0000697988 / 0.0010727; legend 0.00145399 / 0.00489165 |
| backdrop / round3-final-recipe/pair/compare-native-3d | 0.0000195026 / 0.000567021 | history 0.0000697988 / 0.0010727; legend 0.00145399 / 0.00489165 |
| backdrop / round3-final-recipe/pair/compare-pipeline-alert | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| backdrop / round3-final-recipe/pair/compare-pipeline-alerts | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| backdrop / round3-final-recipe/pair/compare-pipeline-clear | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| backdrop / round3-final-recipe/pair/compare-populated-2d | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| backdrop / round3-final-recipe/pair/compare-populated-3d | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| backdrop / round3-final-recipe/pair/compare-train-2d | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| backdrop / round3-final-recipe/pair/compare-train-3d | 0.0000000935554 / 0.0000528959 | history 0.00000033483 / 0.000100069; legend 0.00000700873 / 0.000457833 |
| workshop / editor-probe/macro-open | 0.0000495769 / 0.00316474 | details 0.0000896649 / 0.00425608; details_static 0.0000918062 / 0.0043066 |
| workshop / editor-probe/macro-saved | 0.0000495769 / 0.00316474 | details 0.0000896649 / 0.00425608; details_static 0.0000918062 / 0.0043066 |
| workshop / editor-probe/macro-typed | 0.0000496052 / 0.00316531 | details 0.0000897161 / 0.00425684; details_static 0.0000918586 / 0.00430737 |
| workshop / editor-probe/new-empty | 0.0000837101 / 0.00481709 | dialog 0.00104317 / 0.0170049; title_field 0.0106866 / 0.0548226; tree 0.000293388 / 0.00901815 |
| workshop / editor-probe/size-1 | 0.0174765 / 0.0711087 | details 0.0212281 / 0.0786139; details_static 0.0216648 / 0.0795049; traffic 0.075104 / 0.144028; tree 0.00385684 / 0.0354282 |
| workshop / persistence/before-save | 0.0341297 / 0.0802563 | details 0.00667264 / 0.0436463; details_static 0.00633739 / 0.0430855; traffic 0.0197539 / 0.0395314; tree 0.102031 / 0.136179 |
| workshop / persistence/size-1 | 0.0175598 / 0.0715794 | details 0.0220881 / 0.0802362; details_static 0.0226156 / 0.0811885; traffic 0.0735259 / 0.143826; tree 0.00282372 / 0.0316821 |
| workshop / release-help/help-2 | 0.0000588537 / 0.00459047 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929 |
| workshop / release-help/help-3 | 0.0000588537 / 0.00459047 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929 |
| workshop / release-help/size-1 | 0.0174822 / 0.0701902 | details 0.0221982 / 0.0803928; details_static 0.0226581 / 0.0813059; traffic 0.0754021 / 0.136777; tree 0.00193249 / 0.0263237 |
| workshop / release-native/menu-help | 0.0000588537 / 0.00459047 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929 |
| workshop / release-native/size-1 | 0.0176446 / 0.0715851 | details 0.0221958 / 0.0803222; details_static 0.0226211 / 0.0811952; traffic 0.0746579 / 0.14501; tree 0.00266731 / 0.030096 |
| workshop / release-native/traffic-bottom | 0.0188228 / 0.0716827 | details 0.0230019 / 0.0781022; details_static 0.0235512 / 0.0790292; traffic 0.073335 / 0.142326; tree 0.00551601 / 0.0424655 |
| workshop / release-native/traffic-narrow | 0.00649606 / 0.0399949 | details 0.0116226 / 0.0535865; details_static 0.0117451 / 0.0541694; traffic 0.000823663 / 0.0138807 |
| workshop / release-native/traffic-right-empty | 0.0000770184 / 0.00467142 | details 0.000139296 / 0.00628233; details_static 0.000142622 / 0.0063569 |
| workshop / release-native/traffic-right | 0.00640926 / 0.0393891 | details 0.0114657 / 0.0527686; details_static 0.0116998 / 0.0533851; traffic 0.000823663 / 0.0138807 |
| workshop / release-pair/mesh | 0.000157695 / 0.00505986 | details 0.000285207 / 0.00680472 |
| workshop / release-pair/size-1 | 0.0176647 / 0.0704096 | details 0.0221335 / 0.0801119; details_static 0.0225918 / 0.0810215; traffic 0.075655 / 0.136549; tree 0.0026426 / 0.0300313 |
| workshop / release-panels/macro | 0.0000495769 / 0.00316474 | details 0.0000896649 / 0.00425608; details_static 0.0000918062 / 0.0043066 |
| workshop / release-panels/size-1 | 0.0176659 / 0.0718207 | details 0.0219117 / 0.0799675; details_static 0.022435 / 0.0809167; traffic 0.0763704 / 0.146948; tree 0.00288973 / 0.0318101 |
| workshop / release-prefs/prefs-fonts-scale | 0.000120609 / 0.00381725 | details 0.0000456998 / 0.000657549; details_static 0.0000467911 / 0.000665354; header 0.0017061 / 0.0160148 |
| workshop / release-prefs/prefs-light-compact | 0.0000900625 / 0.00446253 | header 0.00161165 / 0.0188775 |
| workshop / release-prefs/size-1 | 0.0172214 / 0.0706352 | details 0.0210968 / 0.0782235; details_static 0.0215303 / 0.0791096; traffic 0.0761673 / 0.145939; tree 0.00298712 / 0.0319162 |
| workshop / repair-native-final/size-1 | 0.0177558 / 0.0709656 | details 0.0221896 / 0.080446; details_static 0.0226492 / 0.0813597; traffic 0.0760694 / 0.137139; tree 0.00276372 / 0.0322253 |
| workshop / repair-native-final/traffic-bottom | 0.0187066 / 0.0714922 | details 0.0230481 / 0.078323; details_static 0.0235985 / 0.0792527; traffic 0.0730904 / 0.142283; tree 0.00507232 / 0.0405392 |
| workshop / repair-native-final/traffic-narrow | 0.00615238 / 0.0385136 | details 0.0110799 / 0.0517947; details_static 0.0111894 / 0.0523546 |
| workshop / repair-native-final/traffic-right | 0.00606559 / 0.0378831 | details 0.0109229 / 0.0509468; details_static 0.0111441 / 0.0515413 |


## Admission repair revalidation — 2026-11-08: ACCEPTED

Exact runtime `e2c3541a498011f549f7c4d60e3d2811851d50c2`. The shared host now retains original native requests/notices in bounded queues until evaluator capacity is available. It does not resend refused requests or enlarge the mailbox. Full detached CBC `gold-six-admission-full-r1-a1` passes fmt, both Clippy profiles, shell/workspace tests (611 result batches), app policies, furniture, Track A (`SMOKE_FAILS=0`), four release builds and empty diff. Complete saved log: 1,120,752 bytes / 19,532 lines, SHA256 `89927054bbcaff20468a5ebac35c260b3b170fbc2b692cf8febc8acf516ed4f8`.

All 16 release-native groups pass with unchanged assertions/deadlines. Ced large passes. Workshop editor runs concurrently with the other fourteen groups and passes New/title/Tab, Unicode editing, undo/redo, Save, integrated Ced Bus/text/window and native Dopus directory/window. No refused request is retried; all 13 owned editor processes exit. CBC `gold-six-native-admission-audit-r1` confirms counts, exact identity and closure. Earlier BUSY/Full and diagnostic failures remain retained.

| App | Pairs | Raw exact | Functional revalidation |
| --- | ---: | ---: | --- |
| monitor | 10 | 10 | 46/46 Bus |
| ced | 35 | 31 | 11 suites |
| prefs | 91 | 88 | 69 checks including 40 functional and 42 pages |
| screenshot | 47 | 42 | 55/55 Bus; 11 suites |
| backdrop | 44 | 35 | 4 native groups; scale 1/1.5; settled states |
| workshop | 66 | 37 | 99/99 protocol; 25/25 macros; editor, panels, live Prefs and persistence |

All 293 paired sets, 40 contact sheets and the 50 non-zero difference images were inspected. No threshold waiver or new deviation. Monitor whole and every region are 0. Screenshot Open-dialog is 0. Workshop Tabbed New, restored/raw70, authority loss and both empty Traffic docks are 0. Help phase 5 is exact; the other blink phases remain diagnostics. Populated Traffic bottom whole/data SSIM is 0.0187951/0.0840832; right 0.00597892/0.02519; header/static content/controls/status are 0. Backdrop settled pipeline-clear/train pairs close the temporal diagnostic.

Every raw non-zero set follows; region figures are normalised SSIM/RMSE distortion, zero identical. Nonce directory components are removed from public labels.

| App / state | Non-zero regions: SSIM / RMSE | Disposition |
| --- | --- | --- |
| ced / accepted-integrated/prefs-light-large-compact | actions 0.0147586 / 0.0436787; status 0.00355067 / 0.00564538; full 0.000301469 / 0.00405315 | Existing separator/font-glyph raster edges |
| ced / accepted-integrated/search-replace | content 0.0120445 / 0.0356656; gutter 0.027558 / 0.0410017; tabs_search 0.0138192 / 0.0083526; full 0.0111743 / 0.0324682 | Existing Fable C1 Replace repair; canonical whole 0.0111743 |
| ced / accepted-integrated/view-output | content 0.00000257402 / 0.00060235; gutter 0.00000343203 / 0.000695534; full 0.00000215932 / 0.000551697 | Existing separator/font-glyph raster edges |
| ced / accepted-integrated/view-problems | content 0.00000257402 / 0.00060235; gutter 0.00000343203 / 0.000695534; full 0.00000215932 / 0.000551697 | Existing separator/font-glyph raster edges |
| prefs / release-diff-authority-recovered | header 0.00166035 / 0.012979; full 0.0000927844 / 0.00306816 | Existing Macros glyph raster edges in header |
| prefs / release-diff-search-edited | header 0.00191192 / 0.0139276; full 0.0000927844 / 0.00306816 | Existing Macros glyph raster edges in header |
| prefs / release-diff-variant-compact | header 0.00191192 / 0.0139276; full 0.0000927844 / 0.00306816 | Existing Macros glyph raster edges in header |
| screenshot / diff-prefs-fonts | header 0.00194938 / 0.019484; full 0.0000917354 / 0.00422666 | Header glyph rasterisation |
| screenshot / diff-prefs-orange-compact-square | header 0.00191681 / 0.0149276; full 0.000090203 / 0.00323826 | Header glyph rasterisation |
| screenshot / diff-about-dark | canvas 0.000857397 / 0.0102486; full 0.000684147 / 0.0091548 | S2 own source stamp only |
| screenshot / diff-guide-dark | canvas 0.00000000223216 / 0.0000082542; full 0.00000000178112 / 0.00000737325 | Guide-edge rasterisation |
| screenshot / overlay-diff-about-dark | overlay 0.00528382 / 0.0254418; full 0.000684147 / 0.0091548 | S2 own source stamp only |
| backdrop / round3-final-recipe/compare-native-2d | history 0.000125605 / 0.00427167; full 0.0000350956 / 0.00225798 | S2 native history readings; scene geometry matches |
| backdrop / round3-final-recipe/compare-native-3d | history 0.000125605 / 0.00427167; full 0.0000350956 / 0.00225798 | S2 native history readings; scene geometry matches |
| backdrop / round3-final-recipe/compare-pipeline-alert | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; full 0.000000497689 / 0.000072643 | History/legend glyph and trace rasterisation |
| backdrop / round3-final-recipe/compare-pipeline-alerts | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; full 0.000000497689 / 0.000072643 | History/legend glyph and trace rasterisation |
| backdrop / round3-final-recipe/compare-pipeline-clear | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; scene 0.00589802 / 0.0235063; full 0.00197807 / 0.0136114 | Temporal diagnostic; settled pipeline-clear is whole/all-regions 0 |
| backdrop / round3-final-recipe/compare-populated-2d | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; full 0.000000497689 / 0.000072643 | History/legend glyph and trace rasterisation |
| backdrop / round3-final-recipe/compare-populated-3d | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; full 0.000000497689 / 0.000072643 | History/legend glyph and trace rasterisation |
| backdrop / round3-final-recipe/compare-train-2d | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; full 0.000000497689 / 0.000072643 | History/legend glyph and trace rasterisation |
| backdrop / round3-final-recipe/compare-train-3d | history 0.0000017812 / 0.000137427; legend 0.0000372845 / 0.000628751; full 0.000000497689 / 0.000072643 | History/legend glyph and trace rasterisation |
| workshop / integration-figures-macro-open | details 0.0000896649 / 0.00425608; details_static 0.0000918062 / 0.0043066; full 0.0000495769 / 0.00316474 | Destination-combo chevron rasterisation |
| workshop / integration-figures-macro-saved | details 0.0000896649 / 0.00425608; details_static 0.0000918062 / 0.0043066; full 0.0000495769 / 0.00316474 | Destination-combo chevron rasterisation |
| workshop / integration-figures-macro-typed | details 0.0000897161 / 0.00425684; details_static 0.0000918586 / 0.00430737; full 0.0000496052 / 0.00316531 | Destination-combo chevron rasterisation |
| workshop / integration-figures-new-empty | dialog 0.00104317 / 0.0170049; title_field 0.0106866 / 0.0548226; tree 0.000293388 / 0.00901815; full 0.0000837101 / 0.00481709 | Focused caret phase diagnostic; matched Tabbed New is whole/all-regions 0 |
| workshop / integration-figures-size-1 | details 0.0221106 / 0.0801355; details_static 0.0226053 / 0.0810818; traffic 0.0756468 / 0.137377; tree 0.00254801 / 0.0315713; full 0.0176261 / 0.0707155 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-before-save | details 0.00646709 / 0.0432936; details_static 0.00610397 / 0.0426676; traffic 0.0194121 / 0.0410391; tree 0.108665 / 0.143029; full 0.0360088 / 0.0835302 | Smooth-scroll diagnostic; matched restored and raw70 are whole/all-regions 0 |
| workshop / integration-figures-size-1 | details 0.0221034 / 0.0802376; details_static 0.0226312 / 0.08119; traffic 0.073943 / 0.135505; tree 0.00419018 / 0.0372061; full 0.0179839 / 0.0713355 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-help-1 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929; full 0.0000588537 / 0.00459047 | Search caret phase diagnostic; matched Help phase 5 is whole/all-regions 0 |
| workshop / integration-figures-help-2 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929; full 0.0000588537 / 0.00459047 | Search caret phase diagnostic; matched Help phase 5 is whole/all-regions 0 |
| workshop / integration-figures-help-3 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929; full 0.0000588537 / 0.00459047 | Search caret phase diagnostic; matched Help phase 5 is whole/all-regions 0 |
| workshop / integration-figures-help-4 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929; full 0.0000588537 / 0.00459047 | Search caret phase diagnostic; matched Help phase 5 is whole/all-regions 0 |
| workshop / integration-figures-help-6 | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929; full 0.0000588537 / 0.00459047 | Search caret phase diagnostic; matched Help phase 5 is whole/all-regions 0 |
| workshop / integration-figures-size-1 | details 0.0211521 / 0.0782967; details_static 0.021587 / 0.0791838; traffic 0.074973 / 0.144238; tree 0.00301119 / 0.0335459; full 0.0171851 / 0.0706798 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-menu-help | header 0.000354783 / 0.00478142; tree 0.000166452 / 0.00832929; full 0.0000588537 / 0.00459047 | Search caret phase diagnostic; matched Help phase 5 is whole/all-regions 0 |
| workshop / integration-figures-size-1 | details 0.0219753 / 0.0802021; details_static 0.0224667 / 0.0811493; traffic 0.0750169 / 0.136313; tree 0.00254733 / 0.0297663; full 0.0175122 / 0.0704061 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-traffic-bottom | details 0.0228549 / 0.0779829; details_static 0.0234007 / 0.0789085; traffic 0.0721861 / 0.140722; tree 0.00517967 / 0.0409366; full 0.0185746 / 0.0711596 | S2 truthful traffic rows only; expanded header/static/controls/status 0 and empty docks whole/all-regions 0 |
| workshop / integration-figures-traffic-narrow | details 0.010781 / 0.050223; details_static 0.0108833 / 0.0507625; full 0.00598378 / 0.0373449 | S2 truthful traffic rows only; expanded header/static/controls/status 0 and empty docks whole/all-regions 0 |
| workshop / integration-figures-traffic-right | details 0.0106243 / 0.0493504; details_static 0.0108384 / 0.0499256; full 0.00589715 / 0.036696 | S2 truthful traffic rows only; expanded header/static/controls/status 0 and empty docks whole/all-regions 0 |
| workshop / integration-figures-mesh | details 0.000285207 / 0.00680472; full 0.000157695 / 0.00505986 | S2 mesh scrollbar; static details 0 |
| workshop / integration-figures-size-1 | details 0.0221501 / 0.0802173; details_static 0.0226088 / 0.0811282; traffic 0.0744015 / 0.135324; tree 0.00067092 / 0.0148776; full 0.0170339 / 0.0689356 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-macro | details 0.0000896649 / 0.00425608; details_static 0.0000918062 / 0.0043066; full 0.0000495769 / 0.00316474 | Destination-combo chevron rasterisation |
| workshop / integration-figures-size-1 | details 0.0219183 / 0.0801029; details_static 0.0224417 / 0.0810537; traffic 0.0760812 / 0.137633; tree 0.00267335 / 0.0316663; full 0.0175824 / 0.070738 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-prefs-fonts-scale | details 0.0000456998 / 0.000657549; details_static 0.0000467911 / 0.000665354; header 0.0017061 / 0.0160148; full 0.000120609 / 0.00381725 | Header glyph and field-edge rasterisation |
| workshop / integration-figures-prefs-light-compact | header 0.00161165 / 0.0188775; full 0.0000900625 / 0.00446253 | Header glyph and field-edge rasterisation |
| workshop / integration-figures-size-1 | details 0.0219687 / 0.0800553; details_static 0.0224599 / 0.0810007; traffic 0.075264 / 0.13684; tree 0.00272144 / 0.0308195; full 0.0175735 / 0.070506 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-size-1 | details 0.0211711 / 0.0784068; details_static 0.0216064 / 0.0792952; traffic 0.0743491 / 0.143705; tree 0.00283758 / 0.0308991; full 0.0171076 / 0.070335 | Startup/data diagnostic; not a matched acceptance state |
| workshop / integration-figures-traffic-bottom | details 0.0230452 / 0.0784526; details_static 0.0235955 / 0.0793838; traffic 0.072602 / 0.141623; tree 0.00549359 / 0.0423478; full 0.0187951 / 0.0717884 | S2 truthful traffic rows only; expanded header/static/controls/status 0 and empty docks whole/all-regions 0 |
| workshop / integration-figures-traffic-narrow | details 0.0109406 / 0.0509248; details_static 0.0110468 / 0.0514735; full 0.00606555 / 0.0378667 | S2 truthful traffic rows only; expanded header/static/controls/status 0 and empty docks whole/all-regions 0 |
| workshop / integration-figures-traffic-right | details 0.010784 / 0.0500637; details_static 0.0110019 / 0.0506475; full 0.00597892 / 0.0372264 | S2 truthful traffic rows only; expanded header/static/controls/status 0 and empty docks whole/all-regions 0 |

Same-worker `gold-six-perf-e2c3541a4980-a1` passes 510 samples against 4c1d715. ACK median before→after (ms): sort-size 115.6507→114.7186, sort-name 109.1179→108.1466, refresh 98.7981→103.6125, first 62.0501→65.3504, last 64.4634→65.1937, Email 113.3090→117.6564; Email p95 133.7858→141.8773. These are observed shifts, not identical timings or a new tolerance. Final process audit confirms all seven performance processes exited.

Four-engine snapshot under `/build/es-artifacts/e2c3541a498011f549f7c4d60e3d2811851d50c2/` on the CBC artifact worker:

- mix: `90591e8aa9aae722374e353c610656cd9c10154e5f02cca6d0d98862debbe9b2`
- mixd: `dd3ba0f81f73ffb51c00239fe26782bb2128b8c163b6504b8bf78b0379c8cad8`
- viud: `ab4ee7beeb1741c9d153ce84d2fea0feccbe662f3027899c8dd2eba1763167ab`
- viu: `b0a8dc502713de4ebc55fe13983d861cbfb974b052b7496a8b0d53cb4019ac40`
- SHA256SUMS: `7245799c3031343c07c6dac6dadf7474df2a076eba4e950e2df2647f1093d2da`

The [22 inherited import-hash mismatches](import-hash-debt.md) remain open before production derivation. This closes the six-app admission prerequisite; the subsequent Email/Dopus merge and eight-app final validation are separate and still pending. No install or cut-over bundle was made.
