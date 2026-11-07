# Email gold parity

## Round 6 acceptance record, 2026-11-07: ACCEPTED

Tested application source: `d80c067ca5f602f90a63828e7cb0406606869c7f`.
Gold: `8779f33284b7d3f1178c3d57a699efb9b0867b22`, executable SHA256
`79f18fb67c44a32e3f1aff7672c409bfec7d458b6f1b35f081f1d0bd0ff328fe`.
Verified release host from CBC `email-r5-final-64a9e95dee75-a1`:
`be31ad4b8dea2788d0d6000885a57bca7058ee3a8a27d9f89089c9e40cf018cb`.
This host was tested with an exact archive of the application source above.
Exact-source gate `email-r6-d80c067ca5f6-a1` passed on cbc2. Its saved
3,274-line log was read in full: fmt, locked all-target check, all Viu/engine
tests, clippy, debug and release builds passed with no worker diff. Existing
warnings and declared GPU-only ignored tests remain visible in that log.
Command: `mix tools/gold_parity_gate.mix`. Saved log basename:
`20261107-152057-cbc2-d80c067ca5f6-email-r6-d80c067ca5f6-a1-015f9fcb.log`.
The fetched `/build/es-artifacts/d80c067ca5f602f90a63828e7cb0406606869c7f/viu`
and `viu.json` verify source, release profile and the SHA256 above. These
bytes are identical to the executable used by every release suite below;
`final-release-receipt.json` records the equality proof.
The intervening gate `email-r5-final-ab0d62feb7e7` predates the Prefs-launch repair.
No installation, desktop restart or production change is part of this record.

All evidence names below are relative to the private
`email-parity-20261107/round2` archive. Each run retains its executed requests,
source archive, binary hashes, PNGs, settings revisions and process closure.
The canonical service is `email`, gold first, with independent application
homes and an owned native Bus, settingsd, headless compositor and real mixd
maild personality. Captures use 1040×680, scale 1, Studio Dark, Inter 400 at
14.667, client decorations, bar menus and row actions; composers are 900×700.
Narrow captures use 760×680 and compact captures 600×680. No image is resized.

| Gold inventory | Evidence on release bytes |
| --- | --- |
| Accounts, default/selection, passwords, connection/reconnect and live settings | `native.IPmqHI`, `native.m1XnN0`, `native.kGQ983`; poison-pill/cache/migration and token refresh covered by CBC frame/wire tests |
| Folders, list/query/search, selection and triage, partial writes | `native.IPmqHI`; `native.sStjhQ` fetches and verifies 1,200 IDs before a folder-wide write and tests cancellation |
| Reader, headers, plain/HTML-source/Markdown policy, attachments and previews | `pair.CLi31d`, `pair.WW9U1i`, `native.hakzPd`; byte-exact saves, collisions, preview/open-all, save-only and transfer failures |
| Compose, MIME send/delivery, reply/forward, quote position, identities, stale sender and send errors | `native.IPmqHI` (117 requests), `native.m1XnN0` (52); independent same-PID windows use local intents |
| Drafts, restore, fixed-root ownership, disabled accounts, documents and close/Quit gates | `native.kGQ983` (57), `native.YrvZRA` (48); Open/Save As/overwrite/Cancel/Replace and parked-root revival |
| Calendar/contact CRUD, native forms, filtering/completion and details | `native.IPmqHI`, `pair.CLi31d`, `pair.3YbLAI`; Refresh reloads children and clears details with late-read fences |
| Menus, shortcuts, command search, print, macros, desktop activation and Prefs launch | `pair.3YbLAI`, `pair.kmDqEB`; identical print HTML, async Prefs absence and positive section routing, macro run/output/stop/move/remove, Show and Quit |

Gold inventory corrections remain binding: there is no conversation-thread
view, mailbox search field or desktop notification implementation. Threading
means Message-ID/References and reply policy; search is the Bus query surface;
notices use status. HTML without plaintext is escaped source as in gold.

ImageMagick 7.1.2-35 reports normalised **distortion**, zero identical.
`pair.CLi31d`: inbox, dark/light reader, settled calendar, contacts and basic
composer have full-image and every-region SSIM/RMSE **0/0**. Main regions are
header `[0,0,1040,38]`, places `[0,38,226,612]`, list `[226,38,436,612]`,
reader `[662,38,378,612]`, status `[0,650,1040,30]`; composer header/form/body
are `[0,0,900,38]`, `[0,38,900,138]`, `[0,176,900,524]`.
`pair.3YbLAI`: About, shortcuts, event/contact forms, Move, Delete and command
search are exact full-image and in header/content/status regions. Print HTML
is byte-identical. `pair.X4LwyJ`: all five narrow/compact/tree-cursor captures
are exact full-image and in header/content/status. Originals were inspected.

Live Prefs in `pair.CLi31d` keep both PIDs and draft text: ten variants cover
mode, scheme, style/accent, density/roundness, fonts/scale, mono/icon/tune,
app overrides, automatic day/night and reader/composer settings. Every main
region is exact. All composer content/bottom regions and eight full images
are exact. Two scaled headers differ only in rounded-corner antialiasing:
header SSIM `1.11203e-9`, RMSE `.0000113206`; full SSIM `1.58862e-10`,
RMSE `.00000427878`. Invalid input, authority loss/recovery and unchanged
PIDs are retained. Paperclip alignment is repaired. Earlier failed captures
are historical evidence, not substitute acceptance figures.

**Standing Fable C1:** gold's expanded attachment panel paints over part of
the disclosure/count/Save all header. The port reserves the measured header
and spacing from the tile scroll budget. `pair.WW9U1i` keeps both originals:
collapsed dark/light are exact; expanded and no-inline differ only in the
repaired reader region (SSIM `.00368646`, RMSE `.0150809`; full `.00126013`
and `.00881719`). Other regions are exact. Reader region is
`[645,38,395,612]`, list `[226,38,419,612]`. The earlier vertical-clipping-only
repair did **not** close this defect. `native.hakzPd` verifies the controls work.

Bus `pair.kmDqEB` retains 255 same-request probes per implementation and 33
additional success records: 86-command HELP/describe, validation order,
dialog fields, actions/menu replies, macros, topics and Quit. No refusal
rc/code/message differs. Refresh/PIM common labelled accessibility semantics
match; the candidate additionally exposes the window title. Native IDs,
containers, PIDs, compositor identities, paths, debug targets/events/cursors
are truthful runtime data. The places fraction measures 225.5 versus 226
logical points before raster snapping; the painted divider and matched
images are identical. Both hosted titlebar forms work; gold emitted no
titlebar-change notice in this transition, retained in `pair.b6JowQ`.
The delivered action topic has matching v2 fields, IDs, origin and sequence.
All owned PIDs in the completed release suites were confirmed stopped.

The required historical 2,000-row release median is **13.752336 ms**:
source `27738f785b1e1ac5dfe3997b32c1d4fbb3017d27`, CBC
`20261107-111355-394fcb63-a1`, hash
`4a655372a221a7aad903e5218c2f372bc694cf1c61615cbeb9c5ee747f0473d9`,
`native.A7CLGn`, three warm-ups/21 retained samples, p95 19.176699 ms.
`native.CT53c9` repeats on the verified newer release bytes: median
13.075858 ms, p95 16.384905 ms, p99 19.675764 ms. Both use the bulk path and
fixture SHA256 `1ca642d313c13d9a6eb58c086f51336650076ed2897d2a18f0e94b43e36cd780`.
Acknowledgement timings are recorded separately from projection timings.
The final exact-source receipt repeat `native.nDqPNY` records median
13.056324 ms, p95 16.967374 ms and p99 17.903102 ms on the same verified
release hash, with the same warm-ups/sample count and fixture hash.

Private `acceptance-index.json` indexes 13 completed runs and all 44 matched
comparison sets, hashes the receipts and complete gate log, and verifies
every owned PID stopped. Index SHA256:
`b1aede7ec80e60b89aa4618fe300f3a7f9a9eed3cf84d2e6aa615384961a57e0`.
The source and executable equality proof makes the release captures evidence
for the final exact-source artifact, rather than debug or earlier bytes.

Overall **ACCEPTED**: feature inventory, compatible Bus surface, inspected
visual gate and live Prefs response pass, with the standing C1 repair and
AA/runtime-data classifications above. No open item or Fable question remains.
Following commits which only update this record are receipt-only commits;
the tested code SHA remains the one stated above.

## Round 5 acceptance work, 2026-11-07: OPEN

The heavy release gate email-r5-e2b83bff89c8-a1 passed at e2b83bff89c82ac58c3d75786ce9c98b4a4cb266, release SHA256 cbc5cbaa58a6ad4de15efd625132d0f1aa04f72f8126f8dfe188729ea080b95d. Its release native repeat native.BGJDqA passes the wider mail/PIM/composer suite, stale address adoption in both directions, reply-position toggles, visible transport failure and draft recovery. This cohort does not attest later root, keyboard or clipping changes.

Native native.agzK2d at 0a21fb52a6a8de3eb00568d1a47298d4f2ca66bb passes 32 requests / 34 records: deferred mailbox-owner close, Keep Editing cancellation, exit after the final child, compose-only root launch, one initial native surface, fixed-root ownership with a sibling, deferred root confirmation and final exit. All six PIDs stopped. Debug SHA256 110fb5885949d24fc31cfc61645faacbca781801c1a39c1adea1260aa66064fa, CBC 20261107-141101-0a90cb2d PASS. The full saved log was read; its formatter patch is applied. Nine wire tests include bearer refresh once, explicit refusal ambiguity and a refused second 401.

**Gold defect, Fable C1:** in pair.JoSejA the expanded attachment panel matches gold geometry, but gold paints the reader body across the upper part of the disclosure, count and Save all controls. Both gold and candidate originals are retained, including gold-attachments-expanded.png and mix-attachments-expanded.png; the inspected regional comparison isolates the one-pixel header clip elsewhere. The port now clips message painting vertically to the remaining body rectangle, preserving the full attachment controls. This fixes hidden function under the standing C1 ruling; it needs its own native capture. Cosmetic layout stays as gold.

The paperclip repair is closed in diagnostic pair.pRs6SZ: every baseline region and composer content region is exact. Two scaled composer headers retain only inspected rounded-edge antialiasing, full SSIM distortion 1.58862e-10 / RMSE .00000427878. Release pair.pPCV5C completed all main states and ten live Prefs variants, but exposed one-pixel reader clipping and a short status-border extension after attachment changes. Repairs and the final exact-source release repeat remain required.

The 2,000-row release median remains 13.752336 ms at the previously documented source and receipt. No later release projection is claimed yet. Complete Bus success/stale/topic/Quit coverage and the remaining inventory witnesses are still open. No proposed deviation requires a Fable decision.

## Round 4 checkpoint, 2026-11-07: OPEN

Latest formatted code: 9077408a8d7687143bc832faf6b45ded1e8931a8. CBC 20261107-131332-aa236b2f passed the attachment-row repair. Its final visual repeat remains open. The queued old-source release job was superseded by detached email-r4-9077408a8d76 at this code SHA. No release acceptance is claimed.

Settled-state capture now waits for Calendar details in the rendered widget tree. Private round2/pair.g0x2Wa at fa565275e521870f258daec8703bee8a45584e7e has zero SSIM/RMSE distortion for all six baseline states and every region: inbox, dark/light reader, calendar, contacts and composer. Calendar originals, montage and difference were inspected. The previous premature calendar capture is superseded, not discarded. All eight owned processes stopped.

The live Prefs run keeps both PIDs and draft contents. Every main-window variant is exact. Composer light, scheme, style/accent, compact density/roundness, app overrides, auto-night/day and own-surface settings are exact. Font/scale and mono/icon/tune variants retain a paperclip alignment difference (full SSIM distortion 0.000153700, RMSE 0.00393402; header 0.00000000111203 / 0.0000113206). The inspected diff isolates the attachment control. Native stacked-row layout is now ported; its capture repeat is pending. No unexplained difference is accepted.

Private round2/pair.uiG2dG at dee1267051a9a9897ce507e04265aef202bdbc4f has no refusal mismatches across 255 same-request probes, and exact HELP/app.describe including all 86 descriptors. Dynamic actions match too. Repairs cover validation order, complete dialog reply fields and exclusivity, debug parser routing, descriptor deduplication, compose success fields and the debug baseline. Truthful PID, compositor window identities, capture paths and AccessKit IDs differ; these need explicit field-level classification. This is not yet a full success/stale/topic/Quit acceptance matrix.

Real-maild focused round2/native.QgLIhJ passed 24 requests: native stale-From adoption in both directions, reconnect, stopping the owned maild, sending failure, preserved draft text, visible error footer, discard and owned daemon recovery. Stale and error screenshots were inspected. All six PIDs stopped. Native reply-position toggles both directions also passed before the separate native.QCDj3q run stopped at its stale-control drive case. Gold-named UI clicking of that stale footer still needs closure; the successful focused witness uses the generic native input command.

CBC 20261107-130258-ee55799d, 20261107-130623-938614e9 and 20261107-130812-0193389a passed focused checks. Debug SHA256 at dee1267: c831b8531a951ad0a0c2c88df189d5a30f7cbc6c562f1cdc3e07553be113fc84. Formatter patches applied and pushed. The full gate plus release artifact recipe was submitted with --detach as email-r4-b25ec324aa51 at b25ec324aa51edf235278e65227364c872763fc7; it is queued on heavy CBC workers and does not attest subsequent attachment-row changes.

Release-byte 2,000-row evidence remains the verified earlier cohort: median 13.752336 ms, p95 19.176699 ms, p99 21.413170 ms, CBC 20261107-111355-394fcb63-a1. Final release bytes/captures, remaining inventory and full Bus coverage remain open. No deviation is proposed.

## Round 3 native, Prefs and Bus checkpoint: OPEN

Code checkpoint `d0c003814fbf0bf2433ba5386ba19901c0fd85f5` remains incomplete. The latest real-maild witness, private `round2/native.4L2vTD`, passed **89 Bus requests and 99 total evidence records** at `69ac705d8dd5f2d22cee3f6b435391c636dc1a40`, debug SHA256 `e1ab8a2500eaa83639e7d8ee3d3107a6579a29ea693380bb146b416aa964588d`. The earlier `native.3wUH77` has the same counts; 99 denotes records, not requests. These cover account, mail and PIM CRUD, partial failures, MIME send/delivery/body readback, attachment byte/collision saves, native PIM form submission, contact completion, two same-PID composers with native edits and independent Keep/Discard, reply/forward, Show and restart/restore. All six owned PIDs stopped. A failed intervening restore-close probe was a missing draw barrier in the harness; the retained failure and corrected run are separate receipts.

Private `round2/pair.mumAqf` at `3140dbebeb93cb62aee1c7dbad10eb243c7cb557` records exact inbox, dark/light reader, calendar, contacts and basic composer captures, including the former status border and composer spacer. Private `pair.0xNmjO` repeats with independent `MIXOS_APP_HOME` roots: inbox, dark/light reader, contacts and composer are exact in every region. Calendar was captured before the selected detail model had drawn; its nonzero comparison remains a failed capture, requiring a settled-state repeat. This correction prevents gold's cached app state from seeding the candidate. These are debug diagnostics, not final release acceptance.

Live Prefs diagnostics `pair.wgbFhf` and the isolated-root repeat `pair.Kw7ofC` retain unchanged gold/candidate PIDs and draft contents through mode, scheme, style/accent, density/roundness, fonts/scale, mono/icon/tune inputs, app overrides, automatic day/night, reader/composer settings, invalid input and settings-authority loss/recovery. Light, scheme and style/accent captures are exact for both windows. Remaining density/scaling differences were traced through AccessKit rectangles to fractional toolbar advances and composer scroll-origin snapping. Repairs are committed; their rerun is pending. Authority cleanup receipts confirm every owned process stopped. No visual threshold or unexplained difference is accepted.

The same-request Bus diagnostic `pair.Xgi9mM` runs **255 probes per implementation**, including empty objects, arrays and scalars across 85 commands (Quit excluded from this first matrix). It exposes refusal-code, validation-order and dialog-payload differences. Repairs now configure the generic host's argument error contract, preserve gold's window-routing errors, return the full dialog description, and validate requests in gold's observable order in Mix. This matrix is diagnostic and is not a claim that all replies match; successful payloads, asynchronous/stale/topic cases and Quit still require matched verification.

CBC repair jobs: `20261107-120319-fc22e5da`, `20261107-121748-ba884176`, `20261107-123600-7aa2d730` and `20261107-123826-317f25b6` passed focused tests/debug builds. Cache poison-pill and pinned legacy identity migration tests pass. `20261107-123307-9e1fc74c` failed because the existing Markdown dependency's HTML feature was disabled; the feature and inert-export test passed in the next job. `20261107-124250-2f35f0b3` caught a nil access in the new composer footer, since repaired. The queued geometry/footer run does not attest subsequent source changes. Final heavy exact-source release checks remain required.

Still open: native identity-editor and stale-From witnesses; complete attachments/inline previews/open/save-all; deferred bulk-ID selection and cancellation; owner-close/launch/document gates; saved layouts/narrow captures; full matched Bus success/refusal/stale/topic coverage; remaining live Prefs differences; every-main-state release captures and acceptance. Print, account cache/migration, reply-position controls and footer rendering have code and focused checks in progress; they are not accepted by implementation alone. Gold has no mailbox search field: mail search is its Bus surface, while the visible search action searches commands. Do not invent a mailbox field to satisfy the old inventory wording.

Overall **OPEN / INCOMPLETE**. No proposed deviation or Fable question.

## Round 3 release projection and native checkpoint, 2026-11-07: OPEN

Code head `3140dbebeb93cb62aee1c7dbad10eb243c7cb557`; this is a repair checkpoint, not acceptance. Gold remains pinned to `8779f33284b7d3f1178c3d57a699efb9b0867b22`. No install or desktop change.

Release CBC `20261107-111355-394fcb63-a1` on cbc2 passed release frame tests and the release host build at `27738f785b1e1ac5dfe3997b32c1d4fbb3017d27`. Actual executable SHA256 `4a655372a221a7aad903e5218c2f372bc694cf1c61615cbeb9c5ee747f0473d9`; its release/source/hash receipt was verified before the native run. Private `round2/native.A7CLGn` measures the actual modern bulk projection with 2,000 distinct dates: three warm-ups, 21 retained samples, median **13.7523 ms**, p95 **19.1767 ms**, p99 **21.4132 ms**. The fixture hash is `1ca642d313c13d9a6eb58c086f51336650076ed2897d2a18f0e94b43e36cd780`. Bus acknowledgement timings are retained separately and include process/transport overhead. All owned processes stopped. This attests those release bytes, not later feature changes. The first attempt failed in the harness because the fixture exceeded the process-argument limit; it was corrected to use a Mix request file without changing application bytes.

CBC `20261107-114349-8a1c199d` passed focused checks/debug at `a38cd5af0eaf177c52242fe78bdb95e3da771aec`, SHA256 `29b2e67b9c78e15f52d38970dafc62b569a333dfa624fabcde1dc5e999a75d76`. Private `round2/native.3vSdbG` passes 75 Bus requests (85 total evidence records) against real isolated mixd maild: previous CRUD/send/transfer cases plus reply/forward creation, composer attachment add/remove and pristine close. Two composer surfaces remain in one PID with local intents. Every owned PID stopped. Later native PIM form input/submission and contact completions passed before the next run exposed an embedded-interpreter UUID call error in Show; the code now uses the existing identity module, awaiting rerun.

Matched diagnostic `round2/pair.js7Buv` at that source has calendar and contacts header/places/list/reader SSIM and RMSE distortion 0/0. A short right-pane border remains in status (SSIM 0.000348970 / RMSE 0.00121496); whole image 0.0000151023 / 0.000255193. Composer full distortion is 0.0101222 / 0.025756. Originals, montages and diffs inspected: remaining composer status strip, body spacing, From alignment and theme action were repaired next. Main mail captures also exposed initial table-width instability; earlier exact captures do not close that later observation.

CBC `20261107-115343-0e2890be` passed focused checks/debug at `2a4a7e59ccbbab9d134c06a114410c73789dfab3`; formatting returned and applied. Matched `round2/pair.OB5gwe` closes composer **header and form regions exactly (0/0)**. Body/full remain 0.00456722 / 0.0261336 and 0.00439362 / 0.0226108: an extra six-pixel spacer was identified and repaired. Originals inspected; figures retained, further figure inspection and final rerun remain required. This repair adds Mix-owned PIM/triage forms, draft restore/debounce/quit gates, contact-first completions, document chooser/save/replace flows and a reusable native command-search renderer. These newer flows remain under test.

Still required: identity editing/stale identity reconciliation; account cache/migration/authority recovery; complete attachment previews/open/save-all and print; bulk selection/search/narrow/saved layouts; all Bus replies/refusals/stale/topic cases; live Prefs matrix; all main-state release comparisons and the final exact-source gate. No omission is approved. Overall **OPEN / INCOMPLETE**, with no proposed deviation or Fable question.

## Round 3 native composer/PIM checkpoint, 2026-11-07: IN PROGRESS

Exact tested code `1a14f1c499c83df1398b58a2629d4b962faf8b82`, CBC
`20261107-111024-53ca9556` (permitted light-worker override), passed byte
primitives, frame/native input, connected PIM and composer mounting, five wire
tests, the legacy list measurement and debug build, with no worker diff. Full
saved log read. Debug SHA256:
`3d0e4da2391b6e80cbdc696ae9df3ce51e08ea23d755d72e69fd77502c494ef2`.

Private `email-parity-20261107/round2/native.k2W4lA` passes 72 real native
requests against an owned mixd maild personality. Event/contact create, get,
update, filtered list and delete pass; native pointer selection opens both
PIM readers. Direct MIME upload, draft creation, submission, Sent filing and
delivery/body readback pass. Attachment save twice preserves exact SHA256
bytes and chooses distinct names. Two composers create separate native
surfaces under the same PID; native typing reaches only its draft; keeping
one and discarding the other preserves ownership. All five owned PIDs stopped.
Calendar, contacts and composer captures inspected. The server reports the
ASCII filename fallback for the Unicode attachment; the pinned gold MIME
writer supplies the same fallback and extended filename. This remains an
inference from source until the matched gold send witness.

The generic host now accepts scene-authored child viewports. Models are
validated, callbacks queue local intents, and one interpreter owns application
state and jobs. There is no intra-app Bus transport. Composer headers, body,
attachments, private schema-1 draft writes, send attempts and close questions
are authored in Mix. Full restore/debounce, identity editing, reply/forward,
document workflows and owner/quit gates remain required. Current composer
menus, header rows, body surface and status differ from gold.

Matched diagnostic `round2/pair.l8Wdn3` uses the same canonical identity,
1040×680, scale 1 and settings recipe as round 2. The inbox, dark reader and
light reader all have **zero SSIM/RMSE distortion** in header, places, list and
reader regions. Whole-image SSIM/RMSE distortions are respectively
`0.0000151023/0.000255193`, `0.0000307483/0.000360898`, and
`0.0000302703/0.000355387`. The remaining difference is two short pane-border
extensions in the status region. Dark/light originals, montage and diff
figures inspected. Matched PIM and composer originals now exist in
`round2/pair.7N9x3b`; regional comparisons and inspection are in progress.
All owned processes in these completed runs stopped.

CBC `20261107-111753-9a7b5c65` at `0433f98f9999dd66976d406ab8870ccab2907962`
passes the focused checks/debug build for pane clipping and view actions;
full saved log read, formatting patch applied. Binary SHA256
`7d32f9f85be4997d9012272849e826bb3ba021e6cf82b38151fbda5218122678`.
The subsequent `round2/pair.Fndyun` inbox, dark reader and light reader are pixel-exact across whole image and every region, including status (SSIM/RMSE 0/0). Originals, montages and diffs inspected; all owned processes stopped. Later header-form changes are unverified.
Release diagnostic CBC `20261107-111355-394fcb63` at
`27738f785b1e1ac5dfe3997b32c1d4fbb3017d27` is running on cbc2. The explicit
fixture-only benchmark now calls the modern projection on actual host bytes;
no release timing or release acceptance is claimed yet. Debug observations
range about 95–106 ms and are not substituted for release evidence.

Overall **INCOMPLETE**. Full inventory, Bus/refusal/stale cases, live Prefs
matrix/authority recovery, narrow/overflow/forms/attachments/composer states,
release-byte measurement and final exact-source gate/acceptance remain open.
No deviation from gold is proposed and no Fable question is pending.


## Round 2 checkpoint, 2026-11-07: IN PROGRESS

Tested source `05aabb63c7332f84e9651decee790f16a0970c11`, branch
`lab/parity-email`, pinned gold `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Gold executable SHA256 `79f18fb67c44a32e3f1aff7672c409bfec7d458b6f1b35f081f1d0bd0ff328fe`;
CBC debug executable SHA256 `0004999eb22692eac4e10f7bc703ba6422106456ac961a9bae8ef662e3bd41ed`.
The acceptance recipe's final release gate has not run for this checkpoint.

Round 2 implements settings-backed account mutations, private secrets, token
exchange and advertised API routing, mailbox hierarchy and role counts, native
three-column paged/sortable multi-selection tables, asynchronous reads and
triage/folder writes, plaintext/Markdown/escaped HTML reader, attachment
metadata, and calendar/contact JMAP CRUD policy. Held Bus receipts survive
worker callbacks; account generations, removal and newer reader targets fence
late completions. Four workers serve a bounded logical request queue.
Calendar/book discovery runs independently of the mail connection. Live Email
settings now arrive through the shared settings follower as well as appearance.
The `rows_project` primitive remains the list projection path.

Private diagnostic archive `email-parity-20261107/round2/pair.GvbPD8` uses
the exact CBC debug bytes, a real isolated **mixd maild personality**, private
broker/settingsd/headless compd, separate app directories, sequential canonical
`email` identity, 1040×680, scale 1, Studio/CSD/bar/right captions/row actions,
Inter 400/14.667. Seven structured query replies match gold: accounts,
mailboxes, list, search, get, calendars and books. Reader `email.info` agrees
apart from pane proportions. All six owned processes stopped. The earlier
real native mutation witness covered mark/read/flag, folder CRUD, move/readback
and partial failure; it must be repeated on final source with the full matrix.

ImageMagick 7.1.2-35 SSIM/RMSE **distortions**, zero identical:

| State | Full SSIM / RMSE | Header | Places | List | Reader SSIM / RMSE |
|---|---:|---:|---:|---:|---:|
| Inbox | 0.0000151023 / 0.000255193 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| Reader dark | 0.0158136 / 0.0532791 | 0 / 0 | 0 / 0 | 0 / 0 | 0.0473011 / 0.0931529 |
| Reader light | 0.0143698 / 0.0462838 | 0 / 0 | 0 / 0 | 0 / 0 | 0.0434135 / 0.0809219 |

Regions: header `[0,0,1040,38]`, places `[0,38,226,612]`, list
`[226,38,436,612]`, reader `[662,38,378,612]`, status `[0,650,1040,30]`.
Reader originals, full montage and difference inspected: reader width and
header wrapping differ; status contains a small panel-border difference.
Further images remain to be inspected. This is **not** an AA-only acceptance.

CBC `20261107-103429-36f6485a-a1` passed focused HTTP, frame/native input,
JMAP and list tests plus debug build; full saved log read, returned fmt patch
applied. The modern 2,000-distinct-date projection measured 201.99 ms in debug,
so its date stage is being repaired and remeasured. Earlier passing round-2
repair jobs: `20261107-095720-dd05ebd9-a1`,
`20261107-100448-29db146f-a1`, `20261107-101645-1f3e98ef-a1`.

Required work remains: account cache/migration/conflict recovery; attachment
transfers/previews; full native composers, MIME send, identities, quoting,
suggestions, drafts/documents and close gates; PIM views/forms; remaining UI
actions, search, narrow layouts and saved state; complete Bus/refusal/staleness
suite; full live Prefs matrix and authority recovery; all main-state matched
captures, exact-source release gate and acceptance record. No omission has
approval; overall parity is **INCOMPLETE**.

Inventory corrections from pinned gold: HTML is shown as escaped source when
plaintext is absent; gold has no conversation-thread view or desktop
notification implementation. Reply threading is Message-ID/References policy,
and connection/operation notices use the status UI. Direct `email.send` uses
JMAP MIME upload, Email/set, EmailSubmission/set and Sent filing. The historical
bounded `maild.submit` implementation below is not the gold send contract.
Historical “deferred” entries below are prior specimen limits, not approved
parity exclusions.

## Gold parity frame checkpoint, 2026-11-07: IN PROGRESS

Baseline `4f2c0fd`, branch `lab/parity-email`. Gold is pinned to
`8779f33284b7d3f1178c3d57a699efb9b0867b22`; its canonical `email` service
was captured on an isolated native broker, headless compd and private settingsd.
Gold's no-account action readback contains 78 actions; its complete describe
readback contains 86 commands. `actions.json` and `gold-surface.json` preserve
those public reference records. The latter is an **oracle inventory**, not a
claim that all those handlers exist. The host manifest currently exposes the
implemented subset plus native UI, macro and debug facilities.

Mix now mounts the common gold frame, with Email's seven menu roots, Compose
and the standard toolbar tail, live appearance callbacks, Help overlays and
native keymap resolution. The existing account catalogue and bounded mail
core remain; they have not become the gold settings account registry. The
measured `rows_project` path and 2,000-row bounds are unchanged. No per-app Rust
was added. Shared widget changes supply primary buttons and a native section
heading role. The focused frame test exercises native pointer activation,
disabled Compose and the About overlay in the embedded runtime.

Diagnostic pairs use sequential canonical identities at 1040×680, scale 1,
Inter 400/14.667, Studio/CSD/bar menus/right captions/row actions. Images,
readbacks, exact settings revisions, binary hashes, scripts and cleanup are in
the private Email parity archive. First diagnostic pair: header and status
SSIM and RMSE distortion **0**. Whole-image distortion **0.0164125 SSIM /
0.0833201 RMSE**; content **0.0182362 / 0.0878271**. Originals, montage and
difference figure were inspected: heading weight, vertical spacing and the
primary button differ. Subsequent widget repair improves the button and
heading, but content is still unaccepted (whole **0.0162815 / 0.0751937**).
These are ImageMagick 7.1.2-35 normalised distortions, zero meaning identical.
Region rectangles: header `[0,0,1040,38]`, content `[0,38,1040,612]`, status
`[0,650,1040,30]`. Diagnostics do not attest release bytes.

Nine states exercise dark/light, About, shortcuts, accent, density, roundness,
font and text scale on unchanged PIDs. The shortcuts comparison revealed a
missing grid-key projection and a shared renderer that ignored grid rows, subsequently repaired. Compact frame geometry
and no-account content still differ. The complete Prefs matrix (including
authority loss/recovery) remains required. Process cleanup receipts show all
owned applications, compositor, settingsd and private broker stopped.

Remaining work is required, with no approved omissions: settings account
registry/migration/secrets fencing; full JMAP reads and writes including
mailbox/message/PIM CRUD; paged selection and actions; rich reader and
attachments; separate native composers, identities/suggestions/quoting;
durable drafts and mail documents; close gates; menus/search/macros/saved
layouts; complete asynchronous ABP replies, refusals and topics; all matched
visual states and live Prefs cases. The generic host currently owns one app
viewport; separate composer windows require reusable native runtime support.
No replacement process/Bus route or deviation from gold is approved.

CBC preparation `20261107-085153-50a5d098-a1` at `da1ee29f939c` passed
the frame/native input test, all three JMAP-wire tests, the 2,000-row list test
and debug host build. Its formatting diff was applied. Earlier frame gate
`20261107-084505-5c28b11a-a1` at `64fea4e3a423` passed fmt, all-target check,
frame test, clippy, debug and release builds with no worker diff. These are
scoped checks; **overall Email parity remains FAIL / incomplete**. Exact-source
validation of the repaired checkpoint follows in `last.md`.

Final focused CBC `20261107-090221-75f94d9d-a1` at
`376f7f994feba2097552f7d36d97b53bb2b4416d` passes all five focused tests
(including native primary-button input, disabled Compose, and a regression
requiring the shortcuts rows to render), formatting and debug build with no
worker diff. Debug binary SHA256:
`a85315f811a46ad49500ebaa139f4bb7659c60a8d489b72fe5baf3fe6882f1e8`.
In a matched native diagnostic, the shortcuts dialog crop `[294,120,452,458]`
has SSIM and RMSE distortion **0**; whole image **0.01577 / 0.0648568**
(the no-account content behind the dialog still differs). All 78 initial
action records match structurally. These are frame diagnostics, not complete
ABP compatibility or final release acceptance.


## Round 6 measured list rerun

Step 1 is measured, not merely prepared: CBC
`20261106-225448-9bf578e7-a1` at `4988d293933406931fd2bb2904ddc7a8c6f3e2ef`
reran the actual embedded interpreter/scene experiment. Full saved log read;
worker diff empty. Debug build, 2000 identical JMAP-shaped messages, three
warmups then 21 retained samples per path, ordered loop then jq; nearest-rank
p95/p99 (with 21 samples p99 is the maximum). All rows asserted equivalent.

| Milliseconds median / p95 / p99 | Per-row Mix | lib-mix jq bulk |
|---|---:|---:|
| Callback wall time | 387.95 / 393.06 / 400.93 | 450.90 / 454.90 / 455.37 |
| Runtime mailbox ACK | 391.93 / 396.76 / 404.86 | 454.88 / 458.86 / 459.13 |
| Fresh adapter admission | 53.82 / 54.76 / 54.82 | 53.12 / 54.47 / 55.04 |
| Cold context draw | 65.52 / 67.94 / 68.94 | 65.68 / 67.18 / 67.34 |
| Prior native snapshot ACK, raw samples rechecked | 562.13 / 566.52 / 568.02 | 636.96 / 643.48 / 644.36 |

Dopus's recorded 2000-row sort-size ACK medians are original 14.99 ms,
Viu Rust 34.50 ms, Viu Mix 237.40 ms. Email likewise has substantial
row-scaled behaviour/publication cost independent of Dopus's file policy;
existing jq bulk is slower and does not remove it. This supports the shared
runtime/value-copy/publication concern, but does not isolate the same cause:
Email replaces a full input snapshot, Dopus sorts retained rows, and the
Dopus reference cohort is release rather than debug. No direct speed ratio
or performance gate pass is claimed. Adapter/draw stages are separate fresh
objects, not live frame latency; callback includes validation/copy/publication.
Real-mail verification below is separate from these synthetic timings.

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, `apps/email`:
actions/commands/state/shell, account/mail/compose/calendar/contacts views,
reader/documents/drafts/attachments/writer and embedded JMAP client.
Round 6 ports the bounded account/mailbox/list/read/compose core. Run
`viu run apps/email /absolute/path/accounts.json`; the bundle README documents
the explicit account catalogue and native submission route. Mix owns account,
JMAP, paging, reader, draft and receipt policy. Fixture mode remains separate
and cannot contact a server; ordinary launch refuses fixture injection.

| Original capability | Disposition |
|---|---|
| Accounts, enabled/default selection, settings.apply fencing, secrets/CA and reconnect | Ported explicit catalogue (up to 32), account selection, password-file/optional CA and explicit reconnect. Deferred settingsd registry/migration, enable/default preferences, token exchange and non-primary server-account selection: catalogue/primary account is the bounded authority |
| JMAP discovery, same-origin endpoints, token exchange, bounded HTTP/blob upload | Ported Basic-auth discovery and same-origin maild /jmap via shared bounded HTTP. Deferred token flow, arbitrary advertised paths and binary/blob upload until their lifecycle contracts are ported |
| Mailbox roles/counts/hierarchy and selection | Ported Mailbox/get, names/roles/total/unread and selection (up to 256). Deferred hierarchy/tree composition; flat labelled choices retain unique IDs |
| Paged message list, total, sort/query/search and stale-response fencing | Ported 200-row pages, total, receivedAt descending, explicit refresh/previous/next, busy/generation ownership and exact query/get ID matching/order. Deferred custom sort/search, live push and paginated cross-page selection until authoritative update policy |
| Open headers/plain text body | Ported listed-ID Email/get and full bounded text (64 KiB); truncated/invalid parts refused. Native read-only scrollable editor and full text via email.message. Header labels are excerpts; rich reader/layout remains deferred |
| Compose To/subject/body and explicit send/outcome/uncertainty | Ported native in-memory editor, 1..32 bare ASCII recipients, duplicate/header validation, explicit configured Bus route, accepted receipt, same-ID manual retry and held draft. No automatic retry. Deferred full RFC address parser and durable restart reconciliation |
| MIME upload, Email/set, submission, Sent filing and partial-failure receipt | Ported existing maild.submit pipeline/receipt; maild owns MIME/Sent/delivery. Deferred client attachment uploads and direct JMAP mutation recipe; no duplicate submission implementation |
| Separate composers, identities/signatures, Cc/Bcc/Reply-To, suggestions and reply/all/forward/quoting | Deferred richer compose after core |
| Durable drafts/debounce/restore, dirty-close/quit and mail documents with Ced | Deferred; durable atomic ownership and close gates together |
| Attachments/chooser, inspection/limits, inline preview/download/open | Deferred shared byte/image/chooser contracts and lifecycle |
| Markdown/HTML/source/plain reader, links and print | Deferred reusable rendering controls with policy in Mix |
| Read/unread/star/junk, move/delete/archive, labels/bulk selection | Deferred core follow-on; authoritative mutation receipts/paging |
| Mailbox CRUD and account preferences integration | Deferred core follow-on |
| Calendar/event and address-book/contact CRUD | Deferred separate PIM surfaces after mail core |
| Actions/menu/shortcuts/macros, theme/settings-follow, saved layouts, routing/full email.v1 | Deferred shared host and application integration |

The original uses JMAP HTTPS for reads; maild's existing Bus submission command
owns sends. Both wires are preserved. The app neither drives the old GUI as a
backend nor reads maild's database. No embedded interpreter change: Mix 0.117
remains. This is a functional bounded specimen, not full-feature/visual parity
or approval to install it.

## Shared native addition and real-mail witness

The reusable `viu_http` text primitive in `libraries/viu` supplies GET/POST,
Basic headers, optional CA trust, HTTPS (loopback HTTP for tests), 1..30-second
deadlines, 1 MiB request/4 MiB UTF-8 response budgets and explicit HTTP status.
It disables redirects and ambient proxies; failures exclude credentials and
request bodies. Authentication/protocol/retry decisions remain in Mix.
The shared worker now enables Tokio I/O as well as timers. No app-specific
Rust or JMAP handler was added. Existing workspace reqwest 0.12.28 with
json/rustls-tls is reused; only the viu lockfile dependency edge changes,
with no new package/version/C build dependency. Embedded lib-mix is untouched.
The existing reusable editor gains a read_only flag: immutable text buffer,
selection/copy, bounded scroll height and no edit actions. This serves any
plain-text viewer; Email-specific reader policy stays in Mix.

Initial private nested/headless witness at `248958af0d3551dfd3b27c647127e1fff444975e`
passed against actual pinned maild: CLI-created temporary account, seeded local
message via maild.submit, authenticated session/account/mailbox catalogue,
list/read, native compose/Send and JMAP readback of exactly one delivered mail.
Wrong credentials were refused and explicit reconnect recovered. Seven PNGs
inspected including the pinned original reading the same message/server.
Subject/body Unicode agrees; composition/layout is deliberately simpler.
All six owned processes stopped. Final-source receipts follow below.

An initial embedded call exposed disabled Tokio I/O; fixed in the shared
worker with an actual embedded socket regression. Shared tests also cover
off-origin session refusal, wrong method/call identity, incomplete text,
query/get ID mismatch and query-order preservation. No test was weakened.

## Final round-6 receipt

Validated implementation source `6429a07f62a7b7d9dc43f2ee9f05f212bce71dc8`,
CBC `20261106-232952-4d47287e-a1`: 117 tests, fmt-check, locked all-target
checks, clippy and generic-host debug build passed. Full 58558-character saved
log read; worker diff empty. Existing clippy warnings remain. Final artifact
matches the inspected native binary, SHA256
`e93dec80db970db54990c61de9f8389a13155be8bf0772d944c72189b52d086d`.
Subsequent reporting changes alter documentation only.

Final gate repeats the same 3-warmup/21-sample measurement on the final bundle:

| Milliseconds median / p95 / p99 | Per-row Mix | lib-mix jq bulk |
|---|---:|---:|
| Callback wall time | 388.56 / 392.58 / 392.92 | 454.74 / 462.44 / 508.83 |
| Runtime mailbox ACK | 392.68 / 396.57 / 396.80 | 458.62 / 466.52 / 512.64 |
| Fresh adapter admission | 55.53 / 56.60 / 59.12 | 54.62 / 55.72 / 56.25 |
| Cold context draw | 69.35 / 69.94 / 70.11 | 69.29 / 69.76 / 70.06 |

The changed reader composition is included in this cohort. The conclusion
against Dopus's 14.99/34.50/237.40 ms reference remains the same: substantial
cost in a second Mix app, jq slower, exact common cause unproven. Full raw
samples remain in the saved logs. Live lists use the measured faster loop
with 200-row pages, not a claim that all 2000 rows are inexpensive.

Private final native witness has 13 result records and 12 inspected captures:
accounts/mailboxes/list/read; native compose/send and local readback; a dropped
accepted reply with held draft, concurrent-send/edit refusal and explicit
same-operation-ID retry without duplicate mail; real paging 200 + 1 across
201 stored messages; a 23690-byte/700-line Unicode reader scrolled from first
to last line with unchanged text after native typing; wrong-password refusal
and explicit recovery. The pinned original reads the same seeded message on
the same isolated server. It is a comparison oracle, never the new backend.
All seven owned processes confirmed stopped; no installs/desktop changes.
Harness input acknowledgements are distinguished from edit admission:
`viu.ui.text` queues native keystrokes, while immutable-buffer/model assertions
prove no edit. The direct edit-action refusal test remains.

Per-app Rust added = **0**. Compared with the round-5 reporting head, shared
production Rust is **+165/-1 lines**: HTTP text transport 134, editor viewport
28, module registration 1 and worker/binding wiring +2/-1. Shared test diff
is **+226/-3 lines**, including 41 inline HTTP-test lines. Cargo manifest and
lockfile each add one dependency line. No Rust app policy, new package/version
or lib-mix change. Static provenance/import checks and all eight-bundle
zero-per-app-Rust checks pass. Term requirements remain surveyed separately;
no Term implementation started.

## Historical round-5 fixture checkpoint

Validated source `7c55df5d2de07b87e0ff6032fb9fade36b6a55b1`, CBC
`20261106-224820-566c0878-a1`: 112 tests, fmt/check/clippy/build passed; full
saved log read, empty worker diff. The initial adaptation job caught missing
native table flags, fixed in both paths; no test weakened. Existing warnings
remain. Zero per-app or shared production Rust added; one shared measurement
test added. Native binary hash matches the CBC artifact.

| 2000 messages, median ms | Mix loop | Existing jq bulk |
|---|---:|---:|
| CBC embedded callback wall time | 387.34 | 451.79 |
| CBC runtime mailbox acknowledgement | 391.30 | 455.80 |
| Fresh SceneView admission | 53.92 | 54.68 |
| Cold egui context first draw | 65.71 | 65.46 |
| Separate native full-snapshot Bus ACK | 562.13 | 636.96 |

CBC method: debug build, actual embedded Mix 0.117, 2000 synthetic JMAP-shaped
records, 3 warmups then 21 measured iterations per path, loop then bulk.
Every projected row is compared for equality. Timed callback includes input
decoding, validation, projection, copying and publication. Adapter and draw
are separate timings with fresh view/context each iteration, without GPU or
physical presentation; do not sum medians or call these steady-state frames.
Native method uses a separate private headless/nested run with 21 warm samples
per path, full JSON snapshot transfer through generic-host reply. The native
cohort predates only startup-guard/format/report changes. The final-source
fixture run also passed selection/native draft input; four owned processes
stopped and two captures inspected. The pinned original snapshot is a layout
reference, not an equivalent-content or timing baseline.

Bulk jq is slower in both cohorts. These numbers show substantial full-list
cost outside Dopus but do not attribute the same bottleneck. Dopus's retained
2000-row sort-size ACK medians (14.99/34.50/237.40 ms original/Viu Rust/Mix)
use a different operation and release build; there is no valid speed ratio.
The private Dopus results now carry this separate cohort beside those numbers.

At that checkpoint the mail connection and core were still absent. Round 6
implements the core above using generic native HTTP, rather than enabling the
feature-gated lib-mix HTTP/JMAP handlers or changing the embedded interpreter.

## Frame gate receipts

CBC `20261107-085434-f2ceeae1-a1` at `df74a8337fdedecb769810adbc92275ec6b394cd`
passed the complete scoped Viu/viu-engine fmt, check, test, clippy, debug and
release gate with no worker diff. Existing warnings and six environment-specific
ignored tests remain. Release receipt is retained at
`/build/es-artifacts/df74a8337fdedecb769810adbc92275ec6b394cd/viu.json`, SHA256
`ce949216c28bc88c2ba4c8fb96fed6228cc536a9621a3289a9e873c157c862b2`.
This precedes the shortcut-grid renderer repair. The latest code head
`376f7f994feba2097552f7d36d97b53bb2b4416d` has the focused gate above and
debug native evidence; a complete final-source release gate and acceptance
suite remain required. No release-byte acceptance or install is claimed.
