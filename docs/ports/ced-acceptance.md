# Ced parity acceptance — 2026-11-07

Verdict: **PASS**, with Fable C1's Replace-row repair. This record supersedes
the open Ced round-three checkpoints. The private evidence collection is
`editor-parity-r2-20261107`; final regional receipts are in `accepted-5ad8ad4`.
Captures, scripts, daemon logs and failed attempts remain outside this public
repository. No production install, VT change or main merge occurred.

## Exact source and checks

Branch: `lab/viu-editor`. Tested code:
`5ad8ad4aac6cba0363c28dc90103493842a42c2e`. The commit adding this record is a
receipt-only documentation commit; its resolved SHA is in the private
`acceptance-receipt.json`. Shared frame merged through
`5ab4207ab7a0c74aaf4b40592370a69b875e5eed` before Ced's final gate.

Gold is the executable built from
`8779f33284b7d3f1178c3d57a699efb9b0867b22`, version
`ced 0.1.2 (8779f33284b7, built 2026-11-04T19:10:03Z)`. Source inspection is
pinned to that revision; `gold-reference-8779.tar` preserves the relevant
source independently of the maintainer's moving checkout.

| Binary | SHA256 |
| --- | --- |
| Gold Ced | `d83f730e3d69b10f7bd1ab0e4cec5cd7bcf299b4705f864393c8cea96e4aa328` |
| Candidate Viu | `a4cbb133f7de1d4d74509a8ebc6e9c894bcba1f014c6f94c6751e5042ac02952` |

CBC `20261107-063528-808d3be7` passed at the tested source with no worker
diff. The complete saved log was read and is indexed by that job ID in the
private `acceptance-receipt.json`.
Commands were `cargo fmt --check -p viu -p viu-engine -p toolkit`, normal
Clippy `--locked -p viu --all-targets`, 29 tests across `documents`, `ced_bus`,
`ced_editor`, `gold_frame`, `ports_round2`, `app_call`, `preview`, `calls` and
`monitor_signal`, then `cargo build --locked -p viu-engine`. Clippy has
existing style/dead-code warnings; this is not a warnings-as-errors result.
The scoped native app integration below ran on that exact artifact and the
committed Mix bundle. No whole-workspace or release-gold gate is claimed.

## Behaviour, Bus and live Prefs

Mix owns tabs, menus, actions, dialogs, recovery choices, sessions, editd
requests and mail policy. The reusable editor and Mirror retain drafts in
process. Root and Jobs share document handles; application-local traffic
does not travel over the Bus.

| Contract | Expected and observed result |
| --- | --- |
| Actions | All **61/61** IDs dispatch. The text suite succeeds on 59, and correctly refuses the two mail-only actions; the mail suite succeeds on those two. Menus, keyboard input and Bus use the same actions. |
| Bus catalogue | All 53 pinned gold HELP descriptors, INFO shape, 16 native drive operations, three debug operations, `ced.dialog` and all 12 macro operations are driven. Structured argument/refusal cases and action, macro and edit-selection topics are checked. |
| Deferred replies | Open/Save As/Goto and scratch Save remain pending while the native form is open; native input or cancellation completes them once. Scratch Save acknowledges disk completion. Tests cover cancellation, stale/invalid choices and bounded held receipts. |
| Close/save prompts | Bus dirty-close and disk-modified refusals return rc 10 without a modal. Native Close/Save prompts match gold. Cancel preserves text; Don't Save discards; Save waits through Save As; Save Anyway overwrites only after the native choice. Another editd holder permits dirty-tab close. |
| Recovery | File/scratch sessions restore views and drafts after a crash; an editd restart reattaches them. Recovered-buffer Open/Discard/Close preserve or discard the selected buffers exactly. |
| Conflicts | A real editd is briefly paused so an overlapping remote replacement precedes a queued local edit. Native Re-insert at Caret restores retained text; Dismiss clears the conflict without inserting it. A fresh editd without the draft produces a retained disconnected copy; native Keep Mine and Take the Service's choose the corresponding text. Both gold and candidate are driven. |
| Editor | Multiline editing, Unicode/grapheme selection, native clipboard, IME preedit/commit, undo/redo including other origins, line operations, find/regex/replacement, syntax, diagnostics, Problems/Output and Markdown modes are exercised. Preedit does not change document text. |
| Mail | Optional header rows, thread metadata, native attachment chooser/chips, header undo/redo, Save/Save As and crash recovery pass. Explicit Send reaches an isolated Email fixture once; an uncertain outcome is never retried. No external message is sent. |
| Large files | Real editd-backed 16/64 MiB opening, paging, editing, undo and save pass. Scene models contain metadata, not full text. Gold's 64 MiB/two-million-line refusals, 4 MiB inline-read bound and transactional replacement limits remain. Timings are retained in the native receipt rather than represented as a benchmark guarantee. |
| Live Prefs | One candidate PID follows nine paired states: dark; Ocean light/compact with 18 px, weight 700 and 1.5× text; independent light editor at small size/spacious density; Pro, Inter/JetBrains Mono, blue accent, icon stroke, high contrast/app overrides and `tune.style.title_control_spacing`; auto night; client frame; left captions; authority loss; authority recovery with a new accent. Text survives, and the non-monospace editor-family setting is refused as gold requires. |
| Shutdown | The native shadow-plugin exit owner releases its retained window before shutdown. Native Quit/Exit completes within the five-second witness and leaves no process behind. Gold Quit also exits cleanly. This is **gold-behaviour restoration**, not a design deviation or a claim that gold Quit is broken. |

The native witnesses use private Bus, headless compositor, editd, settingsd,
home/config/data/cache/runtime and Email fixture instances. The provenance
manifest hashes every service binary, harness, script, pinned source archive
and fixture. The final native receipt checks the candidate hash/source and
verifies every owned PID and broker has exited. Earlier failed runs remain
available, including the synthetic-input race fixed by waiting for the
rendered Save As field and asserting its echoed path, and a heavily
concurrent large-file timeout rerun with the same deadline at lower load.

## Visual method and inspected differences

All 35 gold/candidate pairs are **1000×720**, scale 1, with identical Rust or
mail text, selection, theme and framing in each pair. Live settings use the
same private settings authority and effective titlebar-form readbacks.
The mail apps run sequentially so shutdown of one cannot detach a shared
origin's other draft. Captures set native window focus and pointer position;
path forms use the same selected text. Recovered-dialog captures include the
same native title drag, with before/after AccessKit bounds retained; its
title bounds also matched before the drag in the final run.

`tools/parity_compare.mix` produces original-size crops, full and regional
side-by-side/diff figures, hashes and raw/normalised metrics. ImageMagick
7.1.2-35 reports SSIM **distortion**, zero identical and lower better.
No arbitrary score threshold is used. Originals and figures were inspected.
Each state has a `comparison.json`; `metrics.json` indexes them. Baseline
regions are header `[0,0,860,38]`, actions `[860,0,140,38]`, tabs/search
`[0,38,1000,40]`, gutter `[8,78,47,604]`, content `[55,78,937,604]` and status
`[0,682,1000,38]`. Prefs crops cover the entire enlarged header and status;
modal states also retain the overlay `[250,130,500,500]`. Rectangles and
decoded dimensions are retained per comparison, including the overlaps.

The region disposition is:

- Header/menu/actions/captions: exact in ordinary and framed states. At the
  enlarged compact setting, only the Search icon's raster edge differs in
  `[913,6,14,13]`; the other glyphs and text are exact.
- Tabs/search: exact except C1's deliberately taller Replace rail.
- Editor/gutter/content: exact except the vertical displacement caused by
  C1 and a single antialiased panel-separator row in Problems/Output. That
  row has the same location and geometry; its channels differ by at most 5.
- Status: exact except fractional edge coverage on one row in the enlarged
  compact setting; text and controls match. The viewport readback's bottom
  rounding differs by 1.5 logical points in that setting, without a changed
  visible row count; the retained native readbacks expose this distinction.
- Overlays and recovery notices: Open, Save As, Goto, Keys, About, dirty close,
  overwrite confirmation, recovered buffers and all four conflict/copy
  states match. Empty window and mail/header/macro states match.

## Fable C1: gold bug and approved port behaviour

Question: retain gold's fixed-height options rail, clipping the Replace row,
or measure both rows so the replacement field, Replace and All controls are
fully visible? Gold's `bars::options` fixed height clips the second row; this
hides function. Options and the original question are retained in `last.md`.

Fable C1's answer, relayed in the round-four brief: **show both rows**, log
the deviation with both captures and a gold bug note. The port retains gold
controls, styling and insets and measures their combined height. The editor
moves down by that added height. This is the sole approved design deviation.
Gold and port PNGs, `search-replace/full-side-by-side.png`,
`search-replace/full-diff.png` and its regional receipt preserve the bug and
repair; `gold-replace-clipping-bug.md` in the private collection records the
source cause and reproduction. Cosmetic oddities elsewhere are copied.


Final result: **31/35 pairs are pixel-identical**; three pairs differ only at
inspected raster edges, and one is the approved C1 repair. No capability is
deferred and no Fable question remains.

## Final native witness index

| Witness | Native run |
| --- | --- |
| 61 actions, complete Bus, macros, clipboard/IME, native scratch Save, Exit | native.aTi10A |
| Mail fields, attachment, Save/Send, uncertain result, crash restore | native.PmZSyG |
| Baseline, panels, menus, dialogs and empty window | native.TY9hi2 |
| Goto with matched selected 2:2 input | native.o5BuKY |
| Nine live Prefs variants, authority loss/recovery, same PID | native.daN9kD |
| Open/Goto/deferred saves, prompt choices, recovered Open/Discard/Close | native.upHvZp |
| Recovered Open/Discard/Close through native mouse clicks; gold Close | native.4f6HDT |
| Paired close/overwrite prompts and another editd holder | native.zvhcmi |
| 16/64 MiB, edit/undo/save, virtualisation and limit refusals | native.QXDskw |
| Sessions, draft/view restore, editd epoch recovery | native.4OdiU4 |
| Paired mail/header/macro appearance | native.TLDsIE |
| Gold/Mix real conflicts, native choices, disconnected copies | native.eJZ5HB |

All twelve receipts have the same tested source and candidate hash; every
owned PID and private broker is confirmed stopped. The private
`final-native-receipts.json`, `provenance-final.json`, `final-pairs.json`,
`accepted-5ad8ad4/metrics.json`, inspection figures and
`acceptance-receipt.json` form the acceptance evidence index.

## Capture matrix

Each row names its private comparison directory and native run. Its receipt
contains both capture filenames/hashes, dimensions, regional crops and raw
metrics. The selected Goto receipt supersedes the earlier empty-field capture
whose caret blink phase differed.

| State / comparison directory | Native run | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| baseline | native.TY9hi2 | 0 | 0 |
| view-problems | native.TY9hi2 | 0.00000215932 | 0.000551697 |
| view-output | native.TY9hi2 | 0.00000215932 | 0.000551697 |
| search-replace | native.TY9hi2 | 0.0112466 | 0.0328173 |
| help-keys | native.TY9hi2 | 0 | 0 |
| help-about | native.TY9hi2 | 0 | 0 |
| menu-File | native.TY9hi2 | 0 | 0 |
| menu-Edit | native.TY9hi2 | 0 | 0 |
| menu-Search | native.TY9hi2 | 0 | 0 |
| menu-View | native.TY9hi2 | 0 | 0 |
| menu-Tabs | native.TY9hi2 | 0 | 0 |
| menu-Help | native.TY9hi2 | 0 | 0 |
| macros | native.TY9hi2 | 0 | 0 |
| file-open | native.TY9hi2 | 0 | 0 |
| file-save_as | native.TY9hi2 | 0 | 0 |
| search-goto_line-selected | native.o5BuKY | 0 | 0 |
| empty | native.TY9hi2 | 0 | 0 |
| prefs-dark | native.daN9kD | 0 | 0 |
| prefs-light-large-compact | native.daN9kD | 0.000301469 | 0.00405315 |
| prefs-light-editor | native.daN9kD | 0 | 0 |
| prefs-overrides-auto | native.daN9kD | 0 | 0 |
| prefs-auto-night | native.daN9kD | 0 | 0 |
| prefs-client-frame | native.daN9kD | 0 | 0 |
| prefs-left-captions | native.daN9kD | 0 | 0 |
| prefs-authority-absent | native.daN9kD | 0 | 0 |
| prefs-authority-recovered | native.daN9kD | 0 | 0 |
| close-dirty | native.zvhcmi | 0 | 0 |
| disk-modified | native.zvhcmi | 0 | 0 |
| recovered | native.upHvZp | 0 | 0 |
| mail | native.TLDsIE | 0 | 0 |
| mail-macros | native.TLDsIE | 0 | 0 |
| notice-reinsert | native.eJZ5HB | 0 | 0 |
| notice-dismiss | native.eJZ5HB | 0 | 0 |
| notice-reinsert-detached | native.eJZ5HB | 0 | 0 |
| notice-dismiss-detached | native.eJZ5HB | 0 | 0 |

## Non-identical regional metrics

All regions not listed below are exact in the final index. Zero is identical.

| State | Region | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| view-problems | content | 0.00000257402 | 0.00060235 |
| view-problems | gutter | 0.00000343203 | 0.000695534 |
| view-output | content | 0.00000257402 | 0.00060235 |
| view-output | gutter | 0.00000343203 | 0.000695534 |
| search-replace | content | 0.0120445 | 0.0356656 |
| search-replace | gutter | 0.0292901 | 0.0471239 |
| search-replace | tabs_search | 0.0142043 | 0.0138868 |
| prefs-light-large-compact | actions | 0.0147586 | 0.0436787 |
| prefs-light-large-compact | status | 0.00355067 | 0.00564538 |

### Release-byte revalidation after integration

Verdict: **PASS** at `4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0` on `lab/gold-parity`, merging accepted editor `d4e19db5259e448190f3e877a0021f735195635a` with the bulk/Monitor integration. Subsequent commits are documentation receipts only. The exact **release** viu SHA256 is `8c2eb07cbef36eed50ddcaacdb8b4664554fdfed999b0b42878aea2e3abe1735`. Gold Ced and native service hashes match the original acceptance provenance above.

CBC `20261107-080318-8bce8633-a1` returned BUILD_RC=0 with no worker diff: full-tree fmt check, normal Viu all-target Clippy, 36 scoped Viu tests, 23 maildoc tests, all four Monitor Mix contract tests, then four separate locked release-engine builds. All 59 Rust tests passed with no ignores. Existing Clippy/style warnings remain; this is not a warnings-as-errors gate. The complete 2,243-line saved log was read and archived (SHA256 `7941819c6ea040ee7aea40de520c9e08a2f6eafc453d3226c65b9d57b8727e33`). The previous whole-workspace gate belongs to the preceding integration source, not this editor merge.

Eleven isolated native release-byte suites pass: 61/61 actions across text/mail, the 53-command surface and macros, native input/clipboard/IME and shutdown, deferred dialogs and saves, native recovery buttons, conflict/copy choices, sessions, mail/attachments/uncertain Send, 16/64 MiB files and refusals, and nine paired live Prefs states including authority recovery on the same app PID. The selected Goto check is incorporated in the visual suite; its overlay was also measured separately at zero. Every native result checks the frozen source and release hash, and all owned PIDs/brokers are stopped.

The fresh 35 matched 1000×720 pairs were inspected with whole/region figures: **31 exact**, three reproduce precisely the original accepted raster metrics, and Replace retains C1. All regions except those already listed in this record are zero. Replace's current figures are:

| Region | SSIM distortion | RMSE distortion |
| --- | ---: | ---: |
| Whole | 0.0111743 | 0.0324682 |
| Content | 0.0120445 | 0.0356656 |
| Gutter | 0.027558 | 0.0410017 |
| Tabs/search | 0.0138192 | 0.0083526 |

The candidate Replace PNG is byte-identical to the prior accepted candidate. Comparing the two gold captures isolates the changed score to 22 pixels of the gold caret blink; both originals and that amplified difference are retained. This introduces no new design deviation. Problems/Output separator and enlarged Prefs Search/status edges remain the same inspected raster differences. C1 is still the sole approved Ced design difference; there are no open items.

Private archive: `gold-parity-20261107/editor-integration/4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0`, containing executed harnesses, native receipts, captures, region metrics, inspection, service/script hashes, exact release bytes and CBC log. Four-engine handoff and the Monitor quick recheck are in [desk4-integration.md](desk4-integration.md).
