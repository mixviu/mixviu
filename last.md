# Gold parity discovery checkpoint — 2026-11-07

**Incomplete. No app is at visual parity.** Work is on `lab/gold-parity`;
`lab/app-ports` 3e6883a is merged onto the four-engine bf8019e base.
The final tested code is 0a7ae37937e8b022b38fb5b0adc176a6b83ce97b.

Implemented a lab shared Mix frame scene/model helper with opt-in generic
bundle composition, existing Viu design/font/icon/menu/toolbar primitives,
attached native header slots, bottom status and Mix-owned caption close.
No application selects it yet. Independent survey caught root identity
relocation and native close bypassing Mix; intent/control roots now refuse
explicitly, and close requires `viu.frame.close` → `window_close: true`.
Identity-preserving mounting still needs implementation before Dopus adopts
the frame. Public contract and limits: `docs/ports/gold-frame.md`.

Exact scoped CBC `20261107-004421-72b146fc-a1`: BUILD_RC=0; fmt, locked
check, Viu/viu-engine tests, clippy and build; empty worker diff. Focused
frame gate `20261107-004254-ea9ed470-a1` also passed. Restored the app-port
minimal-wgpu Offscreen cfg after isolated Viu testing exposed the integrated
baseline mismatch. The full four-engine release/process gates were not rerun.
Native binary hash matched the final CBC artifact exactly.

Gold-first Monitor baseline captured through a private headless compositor
and deterministic native Mix Bus fixture: same 1040×680 size and paused
six-process state. Side-by-side and SSIM diff figures inspected; normalised
SSIM 0.137345 and RMSE 0.183086. This is a failed baseline. Missing regions,
controls, command column, table proportions, striping, units, typography and
status details are recorded privately; no deviation accepted. A separate
generic framed-bundle pilot drew successfully and its native caption click
ran Mix close policy before exiting. All owned processes stopped. Private
captures, fixtures, hashes, closure and detailed receipt are retained under
the gold-parity-20261107 hub evidence directory. No VT/install/deploy used.

Outstanding work remains the full brief: finish the shared frame/widgets and
identity-preserving composition; match Monitor's pages/states first, then
Prefs, Screenshot, Backdrop, Ced, Workshop, Email; run each visual and behaviour
gate; complete generic Quoin/Dopus acceptance and retire the Dopus app host.
Term remains excluded. Interpreter and Dopus/Email hot paths were not edited
(the merge brings one pre-existing interpreter test).

Earlier app-port checkpoint history follows.

# Round 6 Email core completed

Validated implementation: 6429a07f62a7b7d9dc43f2ee9f05f212bce71dc8 on lab/app-ports.
Email now uses actual authenticated maild JMAP for accounts/mailboxes, 200-row
paged list and full bounded plain-text read. Native compose/Send uses the
existing maild.submit Bus wire, with accepted receipts, held draft/operation
ID on uncertain outcomes and explicit same-ID retry. No automatic replay.
Mix owns all application policy; per-app Rust added = 0. Shared production
Rust +165/-1, shared tests +226/-3; reusable HTTP text transport and read-only
editor viewport only. Existing reqwest reused, lockfile adds only its viu edge.
Embedded Mix stays 0.117, Dopus and Term unchanged.

Final CBC 20261106-232952-4d47287e-a1 BUILD_RC=0: 117 tests, fmt-check,
locked all-target checks, clippy and generic-host debug build. Full saved
58558-character log read, empty worker diff. Existing warnings remain.
Final fetched artifact is byte-identical to the inspected native witness:
e93dec80db970db54990c61de9f8389a13155be8bf0772d944c72189b52d086d.

2000-message final-source callback median/p95/p99 in ms: Mix loop
388.56/392.58/392.92, jq bulk 454.74/462.44/508.83. Runtime ACK
392.68/396.57/396.80 vs 458.62/466.52/512.64. Method: debug embedded
Mix 0.117, 3 warmups + 21 retained samples per path, nearest-rank tails,
identical projected rows. Initial rerun 20261106-225448-9bf578e7-a1 also
recorded: 387.95 vs 450.90 ms callback medians. Both cohorts and adapter/draw
tails sit beside Dopus reference medians 14.99/34.50/237.40 in email.md and
the private Dopus report. Bulk remains slower. Email confirms substantial
row-scaled cost beyond Dopus; differing operations/builds prevent a direct
ratio or attribution to the exact same cause. No performance-parity pass.

Final private native.soFAsx: 13 records, 12 inspected captures including
pinned original reading the same seeded message/server. Verified real wire,
native compose/send/readback, lost accepted reply and same-ID retry without
duplicate, 201-message paging (200 + 1), a 23690-byte/700-line Unicode body
scrolled first-to-last, unchanged reader after typing, auth failure/recovery.
All seven owned processes confirmed stopped. No installs or desktop changes.
App archive SHA256 047f6fbb0a36b57bb1b1e1d330e2dada98d68f67b321d77e2f17c6e04746b44f.

Inventories/README describe bounds and explicit deferrals: settings registry/
account migration, tokens/non-primary JMAP selection, durable drafts/outbox/
restart reconciliation/quit prompts, richer addresses/compose, attachments,
HTML/PIM/mutations and full compatibility. Generic native API documentation
updated. All eight-bundle/provenance/import checks pass. Baseline legacy
verify_runtime missing jwt fixture remains as previously recorded. Term needs
inventory stands; no implementation begun.

Earlier checkpoint trail follows.

Round 6 first native mail connect exposed Tokio I/O disabled in shared Mix worker (HTTP primitive unit socket test alone had not covered embedded runtime). Shared worker now enables I/O; added real embedded-JMAP socket regression and origin/method/truncated-body refusals. No test weakened. Full read/compose harness is ready against isolated actual maild and pinned original.

Round 6 maild fixture verified: temporary account created via native CLI, message seeded via maild.submit, authenticated JMAP session returned 200. Mix account/mailbox/page/read jobs and UI controller implemented; generic HTTP Basic transport added. CBC transport adaptation still queued; headless real-mail client verification follows once artifact returns.

Round 6 step 1 rerun complete: CBC 20261106-225448-9bf578e7-a1 green on 4988d29; full log read, no worker diff. Runtime ACK median/p95/p99: loop 391.93/396.76/404.86 ms, jq 454.88/458.86/459.13 ms; 2000 messages, 21 samples after 3 warmups, debug. Report in docs/ports/email.md. Reusable bounded HTTP primitive implementation now goes to CBC; native maild read path remains in progress.

Round 6 active: finish measured Email read core. Saved round-5 measurements verified as existing; raw native samples available, tails being added and embedded measurement rerun. Native maild/JMAP integration next; no self-imposed early cutoff.

# Round 5 Email list budget checkpoint

Validated source `7c55df5d2de07b87e0ff6032fb9fade36b6a55b1`, lab/app-ports.
Email core and viability step 2 remain OPEN. No new app port is complete.
This checkpoint contains an explicit fixture-only 2000-row virtual message
table, equivalent Mix-loop/jq projections, selection and an in-memory native
draft editor. Real accounts/mailboxes/read/send remain required work, not
optional feature deferrals. No mail service was contacted and no send attempted.

CBC `20261106-224820-566c0878-a1`: rc 0, 112 tests, fmt/check/clippy/build
passed; full saved log read, worker diff empty. Adaptation job
`20261106-224144-1545a147-a1` ran earlier source and caught missing table flags;
fixed without weakening assertions, formatting returned/applied locally.
Zero per-app Rust and zero shared production Rust added. One shared Rust
measurement test; no embedded interpreter, dependency or lockfile changes.

2000-row medians (loop / jq bulk): CBC callback 387.34 / 451.79 ms;
runtime ACK 391.30 / 455.80 ms; fresh adapter 53.92 / 54.68 ms;
cold-context draw 65.71 / 65.46 ms. Separate native full-snapshot Bus ACK
562.13 / 636.96 ms. Each cohort uses 3 warmups + 21 samples per path.
Bulk is slower, not an optimisation win. The Dopus private report now places
this separate cohort beside its 14.99/34.50/237.40 ms retained-row sort-size
results, with operation/build/transport differences stated. No parity ratio.

Native final-source fixture run passed, two captures inspected; four owned
processes confirmed stopped. Pinned original Email saved snapshot inspected
as a different-data layout reference. No original live-mail comparison or
Email acceptance. Shared host binary hash matches final CBC artifact:
f48c8fff1cd637700833d7b584c9fc553ebb763ae54b07176de8f8b7f675594a.
No installs or desktop changes. `docs/ports/email.md` records scope/deferrals
and remaining native HTTP/JMAP wiring; `docs/ports/term-needs.md` now surveys
pinned PTY/parser dependencies, native session/descriptor lifecycle and IME
ownership. Gold Term remains; no Term implementation started.

Previous checkpoint trail follows.

Round 5 measured native fixture ACK (21 samples after 3 warmups, 2000-row full snapshot over private Bus): Mix loop 562.13 ms, jq bulk 636.96 ms. Bulk is slower on this end-to-end path; no faster-builtin claim. Four owned processes stopped. CBC adaptation job 20261106-224144-1545a147-a1 ran stale pre-fix source and caught missing table flags; full log read, worker formatting applied. Current source fixes flags; exact-source full gate next.

Round 5 native list/draft prototype now maps 2000 explicit fixture rows through the generic host. Initial missing tree-table flags fixed; harness argv-size and JSON framing corrected without changing application admission. Term native session/IME survey committed. CBC adaptation job 20261106-224144-1545a147 is still queued behind occupied heavy workers; no new CBC acceptance yet. Email core and viability step 2 remain open.

Round 5 list checkpoint in progress: explicit Email fixture-only bundle on the generic host; per-row Mix and existing lib-mix jq bulk projection prepared for an equivalent 2000-message measurement. No maild connection or core acceptance yet. No new native primitive or interpreter update.

# Round 4 Workshop checkpoint

Validated source: `e330c14a5c7725804ef62614badd7c8b08dd1b69` on
`lab/app-ports`. A report-only commit records this receipt after the gate.

Workshop now runs as `viu run apps/workshop`: local service discovery,
on-demand HELP/app.describe, advertised command selection/metadata, service
filtering, JSON body editing/format/clear, explicit native Bus call, peer rc
and bounded reply/history. Mix owns all policy. Per-app Rust added = 0.
The only shared native primitive change is editor visible row count (1..100,
default 24), +7/-2 production Rust lines, +121/-1 shared test lines.
The embedded interpreter remains Mix 0.117; no dependency/lockfile change.

Workshop is a bounded local browser. Remote discovery/tree identities, live
broker traffic, macro manager, mesh/agent pages, clipboard, settings/theme,
saved layout and full old API compatibility are deferred with reasons in
`docs/ports/workshop.md`. Email is inventoried in `docs/ports/email.md` but
not implemented: the core list/read/compose path and viability step 2 remain
open. It needs a reusable native upstream for the existing JMAP seam; the
original mail data path uses JMAP rather than GUI automation over the Bus.

CBC final job `20261106-223216-699f3e94-a1`: rc 0, 111 tests, fmt-check,
all-target check, clippy and generic host debug build passed. Full saved log
read, worker diff empty; existing clippy warnings remain. Earlier green jobs
were `20261106-221954-40c3a368-a1` and `20261106-222833-c4cabdf8-a1`.
Preparation `20261106-221807-830d4ca7-a1` found a row integer conversion;
`20261106-222735-1552df20-a1` and `20261106-223118-23ebbff1-a1` stopped at
fmt-check. Full logs read and exact worker changes applied; no tests weakened.

Final private native witness `native.nZEvnf`: eight records passed, five
captures inspected including pinned-original Workshop executing the same
Unicode echo. Native typing/Call dispatched once; malformed JSON, unknown
commands and oversized UTF-8 refused; peer rc 17 retained; reply/history
bounded; busy requests held and ping responsive. A delayed call found Mix
timeout rc -2 incorrectly labelled a peer completion; fixed in Mix and the
unchanged native uncertainty/no-retry assertion now passes. All seven owned
processes confirmed stopped. No install or desktop change.

Final CBC binary SHA-256 matches the native witness:
`f48c8fff1cd637700833d7b584c9fc553ebb763ae54b07176de8f8b7f675594a`.
Private app archive SHA-256:
`7da043f1a3e742aa2a9e077dff6a5b683564645ce1da93e9284d10fb05c71b13`.
Captures show the simpler layout and a missing-emoji-glyph font fallback;
no visual/full-feature parity or performance claim.

Static app-port verifier passes all seven bundles. Extra legacy
`tools/verify_runtime.mix` remains red on the baseline missing
`libraries/mix/tests/fixtures/jwt-rsa-test.pem` (absent at the round-3 base
too); verifier and embedded Mix are unchanged.

Earlier checkpoint trail follows.

Workshop timeout fix passes native delayed/busy/no-retry assertions (8 records, all 7 owned processes stopped). CBC 20261106-222833-c4cabdf8-a1 green on pre-fix source 4ffb375; follow-up 20261106-223118-23ebbff1-a1 found formatting only in new timeout regression probe, exact saved diff applied. Final gate pending.

Workshop adversarial native check found send timeout rc -2 labelled as a completed peer reply. Fixed in Mix: negative codes now raise TRANSPORT and retain an explicit uncertain-delivery/no-retry status; native timeout assertion retained. CBC/unit and exact-source native reruns required. Workshop checkpoint remains open; Email inventory only.

Workshop final-source gate 20261106-222735-1552df20-a1 stopped at fmt-check on the new metadata refusal case; full saved log read and exact formatting applied. Native source 50746d4 is being checked with the previously gated identical production Rust. Static app-port verifier passes. Extra legacy verify_runtime check exposes a baseline missing libraries/mix/tests/fixtures/jwt-rsa-test.pem (also absent at 6edaf28); verifier and embedded interpreter preserved.

Workshop native checkpoint: source 087b891 passed CBC 20261106-221954-40c3a368-a1 (111 tests, full log read, no worker diff), private native run with 6 records and 5 inspected images including original same-request comparison; all 7 owned processes stopped. Inspection found missing field labels, now fixed in Mix scene. Exact-source final rerun next. Email inventoried only; its JMAP seam needs shared native upstream wiring, embedded interpreter unchanged.

Workshop implementation checkpoint: Mix bundle committed; CBC preparation 20261106-221807-830d4ca7-a1 found a u64/usize editor row conversion. Full log read; worker formatting applied and conversion fixed. Exact-source gate next. No native acceptance yet.

Round 4 started: Workshop inventory reviewed against pinned 8779f3328 source. Local discovery / JSON request specimen next; Email follows its checkpoint. No new acceptance yet.

# MixViu app ports — round 3 Ced checkpoint

Ced is a bounded Mix specimen over the unchanged editd wire. Workshop is
inventoried but not implemented at this Ced-first budget checkpoint.
No installs, real desktop/VT changes, interpreter changes or main merge.

Validated source: `6edaf28e63a2192a67ee52feb3398d6349e8179b`.
The native run uses this exact immutable bundle archive and CBC-built generic
host. Four captures were inspected beside original Ced 8779f33284b7.
This is not complete feature, visual, collaboration or performance parity.

Implemented: semantic open/create, tabs, native text input/selection, explicit
revision-fenced Apply, Save/Save As, editd Undo/Redo/Reload, Refresh/Discard and
Close. Mix owns all file/tab/session/command policy. Failed CAS retains the
local draft; no automatic write replay. Native checks passed open, selection,
Unicode typing, save bytes, undo/redo, two-tab selection/close, concurrent edit
refusal and retained draft, refresh and clean close. Oversized Unicode input
was refused without changing the draft.

Per-app Rust: **0**. Shared production Rust: **+12/-5 lines**; shared test diff
**+10/-1 lines**. The generic editor kind uses the existing native egui text
buffer, code-editor rendering and bounded UTF-8 admission; it changes no
ordinary field limits. No new dependency or imported Rust source. Embedded
Mix remains 0.117, with libraries/mix, engines and Cargo.lock unchanged.

The editor-model/edit mirror and storage closure were surveyed, not imported.
Optimistic typing/OT, event/reply-loss reconciliation, large paged buffers,
highlighting/diagnostics, rich commands/search/markdown/mail, OS clipboard,
session settings/keymaps and unsaved-quit prompts remain deferred. The native
control caps interactive insertion at 16384 scalars and admission at 65536
UTF-8 bytes; partial snapshots are refused. Generic host quit has no original
draft-loss prompt. Details and rationale: docs/ports/ced.md. Workshop inventory:
docs/ports/workshop.md. No Workshop acceptance is claimed.

Final native evidence: `native.0JyRoF`, six records, all six owned processes
stopped. App archive SHA256:
`de6a766387b76fdfc0921b58b132bb6e36497994c61bb3191215c640367f78dc`.
Binary SHA256:
`a2ce609315a551b9a2847ccb658c451da8b362473e5818b00a4eab11e3968e6f`.
Private scripts, captures and receipts remain in app-ports-r3-20261106.
Mix lint and the 101 native-import checks pass. No test was weakened.

Earlier round-3 CBC: 20261106-215555-cd9f02ee-a1 (three scoped tests/build,
full saved log read, fmt diff applied locally); 20261106-220141-a624f945-a1
(109 tests, fmt/check/clippy/build green, full saved log read, worker diff
empty). The final corrected-source gate is recorded at the end of this file.
Clippy has existing style warnings; no warning-free claim. Commits after the
validated source change documentation/receipts only.

## Earlier round-2 and round-3 checkpoint history

# MixViu app ports — round 2 checkpoint

Screenshot and Prefs are implemented and verified as bounded Mix lab specimens.
Round-1 results stand. No install, real desktop/VT change, main merge or
production acceptance. Inventories were committed before implementation.

- Screenshot: output, fenced window, explicit/interactive region capture;
  confirmed minimise/restore, delay/cursor, UI Cancel, bounded image preview,
  exclusive PNG export/Save and native blobd/clipd image copy. Clipboard bytes
  were read back and compared. Stale window refusal restored its own window.
- Prefs: real settings.describe/get/status, typed registry editor, fenced
  settings.apply/reset, category/search/selection and authoritative refresh.
  Real checkbox Apply passed. A six-second draft retained its fence; a
  concurrent authority edit caused conflict without replay. Authority loss
  made the snapshot stale, inhibited writes and recovered on restart/refresh.
- Per-app Rust added: **0 for each app**. Shared production Rust: **+90/-10
  lines**, plus 91 integration-test lines. Count and rationale are in
  docs/ports/round2-native.md. No new dependencies or libraries/mix changes;
  embedded Mix remains 0.117. Term needs are inventoried, not implemented.

## Exact-source evidence

Validated runtime/bundle/test source:
`df85b9fe84bf07ed3634015037814765e58fabcc`.
CBC `20261106-214217-d2c1545d-a1`, BUILD_RC=0: fmt check, locked scoped
checks, **108 tests**, clippy and debug build. Full 44185-character saved log
read, worker diff empty. Clippy succeeds with existing style warnings,
including the generic host; this is not a warning-free claim.

Binary SHA256:
`05017aa8e5e5fed5b56a4b1ab13422c314f5817116c1cd2ef842529b893ecb27`.
Final artifact fetched and verified byte-identical to the native witness.
Immutable-archive native run: **native.5I3U3L**, **13 result records**.
App archive SHA256:
`d59e04f1aa5ecc0ca3e2c52c6917319081db19fa8c33ec997b688a74dcea1ce5`.
All eight application/comparator captures inspected beside the originals;
all eleven owned processes confirmed stopped. Evidence remains private in
app-ports-r2-20261106/report.md and the native run directory.

Native compd ran nested under the existing private headless Weston harness.
Output PNG dimensions exactly matched the compositor registry (1280x800,
scale 1); interactive selection produced 300x200. Native headless pilots also
passed capture/settings/copy cases, but have no human input source, so pointer
selection acceptance used the nested harness. No real display was used.

Pure Mix capture-shape/refusal, revision/type/range/owner/stale/operation-ID
checks pass. Mix lint passes with the actual embedder functions declared.
Seed provenance, fourteen external pins, 101 native imports and five-bundle
zero-per-app-Rust checks pass. No test/verifier was weakened. The inherited
runtime-import verifier limitation (jwt-rsa-test.pem absent at the base) remains.

## Limits and deferrals

Complete source-derived inventories: docs/ports/screenshot.md and prefs.md.
These are functional specimens with simpler composition, not complete visual,
feature or performance parity, and not installation acceptance.

Screenshot annotations/crop/history, dialogs, original raw pruning/on-disk
recent index and session theme/keymaps remain deferred. Save folder is explicit
or HOME/Pictures/Screenshots; XDG user-dirs compatibility remains deferred.
Track B7 needs an owned selection handle with cancellation/status/removal
acknowledgement. Stopping the Mix worker cannot certify overlay removal;
the app exposes the Escape/55-second timeout limit. No overlays moved to Rust.

Prefs specialised backend pages, appearance packages, library/chooser controls,
email secrets/migrations, software installs and complete topic/receipt/wire
compatibility remain deferred. Lists and Email-owned workflows are read-only.
Prefs itself does not yet follow the edited theme/keymaps. Polling pauses
during draft edits; explicit refresh discards a draft and recovers uncertainty.
settingsd was tested through its unchanged wire, not a new mixd personality.

## Checkpoint history

Round 2 starts at reporting head 11945c0. CBC 20261106-210513-b604d880-a1
hit worker compiler tmpfs exhaustion before tests; full log read and fmt patch
applied. Later jobs use an owned build-volume TMPDIR and no compiler wrapper,
without touching shared caches. Scoped jobs 20261106-210914-2b236015-a1 and
20261106-211636-ceb74f7b-a1 passed; full logs read and adaptations empty.
Full gate 20261106-212655-f9a8eff8-a1 passed 108 tests at c8132dd. The
superseded queue job 20261106-213101-2218a4fc was confirmed cancelled with
zero attempts; it is not a failed build. Reporting-only commits after the
validated source change no runtime, bundle, manifest or test source.

Round 1 validated source: 14943caf69394c93e66acdd7af878b7634707b7f;
CBC 20261106-195953-c07e7f58-a1, 106 tests. Generic host/Gallery, Backdrop
and Monitor remain as accepted bounded specimens. All six wallpaper mode/
colour pairs had AE=0 at 1280x900; Monitor captures and native interactions
passed. Its native.CvXIFK run had 16 records, all nine owned processes stopped.
Round-1 inventories retain multi-output/hotplug, slideshow, detailed monitor
views/signals/actions and full compatibility deferrals. Dopus is unchanged.

Branch: lab/app-ports; head/source SHA: df85b9fe84bf07ed3634015037814765e58fabcc (validated source; reporting-only commit follows); apps done: screenshot/prefs bounded lab specimens with listed deferrals; CBC job ids: 20261106-214217-d2c1545d-a1 (final green), 20261106-212655-f9a8eff8-a1 (earlier full green), 20261106-211636-ceb74f7b-a1 and 20261106-210914-2b236015-a1 (scoped green), 20261106-210513-b604d880-a1 (infrastructure failure), 20261106-213101-2218a4fc (cancelled unstarted).

Round 3: Ced and Workshop source inventories committed before implementation. Ced first; shared editor control and explicit editd CAS draft planned, richer editor-model mirror integration deferred with reasons. No installs or interpreter changes.
Branch: lab/app-ports; head/source SHA: 76bfc148c84c1fa3bfcdbe8c11775155290db73d (checkpoint parent); apps done: rounds 1-2, round-3 inventory only; CBC job ids: 20261106-214217-d2c1545d-a1 (round 2 green).

Round 3 implementation checkpoint: Ced Mix bundle and reusable editor control. Explicit Apply draft uses native editd CAS; app policy stays in Mix. Native/CBC acceptance pending. Workshop inventory only.
Branch: lab/app-ports; head/source SHA: e018f24345b79b1524f636e0548e676bbdc0b799 (checkpoint parent); apps done: Ced prototype, Workshop inventory; CBC job ids: round-3 pending.

Ced checkpoint: CBC 20261106-215555-cd9f02ee-a1 passed three scoped embedded-model/editor-bound tests and build; full saved log read and fmt diff applied. Native pilots passed real editd open, Unicode typing, Apply/save bytes, Undo/Redo, two-tab selection/close, stale-revision draft retention and original Ced mapping/captures. Inspected captures show a simpler explicit-draft editor, not visual/feature parity. Final exact-source gate and native binary match pending. Workshop remains inventory only under the Ced-first budget checkpoint.
Branch: lab/app-ports; head/source SHA: 3afe5229942cd596744555e3a699268e1d3e2110 (source; documentation checkpoint follows); apps done: Ced bounded specimen pending final gate, Workshop not started; CBC job ids: 20261106-215555-cd9f02ee-a1 (scoped green), 20261106-220141-a624f945 (final queued/running).

Ced final boundary correction: use byte_length rather than scalar-count len for the 65536-byte UTF-8 fence. Added native oversized-Unicode refusal with unchanged draft assertion. Full CBC 20261106-220141-a624f945-a1 passed 109 tests with empty worker diff and full saved log read; corrected Mix source has a new exact-source gate. Rust unchanged after that gate.
Branch: lab/app-ports; head/source SHA: 6edaf28e63a2192a67ee52feb3398d6349e8179b (source; reporting-only commit follows); apps done: Ced bounded specimen, Workshop inventory only; CBC job ids: 20261106-220647-449d25c2 (queued), 20261106-220141-a624f945-a1 (full green), 20261106-215555-cd9f02ee-a1 (scoped green).

Round-3 final CBC 20261106-220647-449d25c2-a1 BUILD_RC=0: 109 tests, fmt/check/clippy/build; full 43894-character log read, empty worker diff. Final artifact fetched and byte-identical to the inspected native witness. Ced is checkpointed with listed deferrals; Workshop remains inventory only.
Branch: lab/app-ports; head/source SHA: 6edaf28e63a2192a67ee52feb3398d6349e8179b (validated source; reporting-only commit follows); apps done: Ced bounded specimen, Workshop inventory only; CBC job ids: 20261106-220647-449d25c2-a1 (final green), 20261106-220141-a624f945-a1 (earlier full green), 20261106-215555-cd9f02ee-a1 (scoped green).

branch: lab/app-ports; head SHA: e330c14a5c7725804ef62614badd7c8b08dd1b69 (validated source; report-only commit follows); apps done: Workshop bounded local Bus specimen; Email inventory only; CBC jobs: 20261106-223216-699f3e94-a1 (final green), 20261106-222833-c4cabdf8-a1, 20261106-221954-40c3a368-a1, 20261106-221807-830d4ca7-a1, 20261106-222735-1552df20-a1, 20261106-223118-23ebbff1-a1.

branch: lab/app-ports; head SHA: 7c55df5d2de07b87e0ff6032fb9fade36b6a55b1 (validated source; report-only commit follows); apps done: no new complete app, Email fixture/list budget checkpoint only, Term needs updated; CBC job ids: 20261106-224820-566c0878-a1 (green), 20261106-224144-1545a147-a1 (adaptation failure fixed).
# Round 6 real-mail checkpoint

Private witness native.MLaov3 also passed a lost accepted-reply: held draft,
busy-send/edit refusal, explicit same-ID retry, exactly three total Inbox
messages (seed, first compose, retry specimen), no duplicate. CBC scoped
20261106-232313-c88aa214-a1 green, full saved log read, empty diff.
Shared editor read_only/scroll viewport added so the reader can expose the
entire admitted 64 KiB body instead of a 600-character UI excerpt; tests and
final-source gate pending.

Added query/get identity/order regression caught unsupported list != in Mix;
corrected to deep_eq, preserving assertions. CBC 20261106-231949-9e0c3dae-a1
full saved log read; worker formatting applied. Native lost-receipt fixture
prepared; final gate remains pending.

Real maild private harness passed at source 248958af0d3551dfd3b27c647127e1fff444975e:
configured accounts, authenticated JMAP session, six mailboxes, list/read,
native compose/Send, accepted receipt and exactly one locally delivered message
read back through JMAP. Wrong credentials refused; explicit reconnect recovered.
Seven captures inspected including pinned original reading the same message;
six owned processes confirmed stopped. No desktop/install changes.
CBC scoped 20261106-231333-dc3fd788-a1 passed actual embedded socket regression,
transport tests and build; full saved log read, empty worker diff. Final gate
and uncertainty/retry witness follow. Per-app Rust remains zero.


branch: lab/app-ports; head SHA: 6429a07f62a7b7d9dc43f2ee9f05f212bce71dc8 (validated implementation head; reporting-only commit follows); done: Email core account/mailbox/page/read/compose/send with explicit deferrals, zero per-app Rust; measurement: 2000 rows callback median/p95/p99 ms loop 388.56/392.58/392.92, jq 454.74/462.44/508.83; CBC job ids: 20261106-232952-4d47287e-a1 (final green), 20261106-225448-9bf578e7-a1 (measurement green), 20261106-232802-64e46f05-a1, 20261106-232313-c88aa214-a1, 20261106-231333-dc3fd788-a1, 20261106-231002-3759a9be-a1, 20261106-230423-3183d374-a1, 20261106-225705-550866b6-a1 (earlier green), 20261106-231949-9e0c3dae-a1 and 20261106-231146-1a9b7ff2-a1 (adaptation failures fixed).


## Gold parity integration checkpoint (2026-11-07)

Merged app-ports 3e6883a onto the four-engine bf8019e baseline. Retained the integrated engine manifests, full Track A interpreter, Bus/files/icons/macros APIs and gold lock pins. Reconciled the duplicate imports against round two; imported the later Ced, Workshop and Email Mix bundles plus reusable HTTP and reader controls. No app is visually accepted yet. CBC integration gate pending; no install or desktop use.

branch lab/gold-parity; head: pending merge commit; apps at parity: none; visual evidence: none; CBC jobs: pending.


Gold frame lab prototype: shared Mix scene/model library with generic scene grafting and existing native window slots/design projection; no app policy moved to Rust. CBC integration preparation 20261107-002827-f4f55dd6-a1 returned 0 with one reqwest lock edge, applied exactly. Prototype fmt/check/test pending. Gold Monitor captured headlessly at 1040x680; private capture and harness receipts retained, all owned processes closed. No paired deterministic capture or parity acceptance yet.

branch lab/gold-parity; head: see this checkpoint commit; apps at parity: none; visual gate evidence: original Monitor capture only (private); CBC jobs: 20261107-002827-f4f55dd6-a1 (preparation, lock adaptation).


Independent frame survey found root identity relocation and caption-close ownership issues. Prototype now refuses intent-owning/control roots explicitly (Dopus adoption remains blocked on a real identity-preserving mount) and dispatches viu.frame.close to Mix, requiring model.window_close authorisation. New model publication test now includes its mandatory generation token. CBC preparation 20261107-003617-53f60db7-a1 failed that test; 20261107-004118-523dd8fe-a1 exposed inherited eframe Offscreen feature gating when testing Viu without the engine. Restored app-ports correct wgpu_no_default_features cfg, preserving full-wgpu builds and standalone widget tests. Exact CBC formatting patches applied. No app adopted the prototype.

branch lab/gold-parity; head: see this checkpoint commit; apps at parity: none; visual evidence: deterministic gold Monitor 1040x680 captured privately; CBC jobs: 20261107-002827-f4f55dd6-a1 preparation green, 20261107-003617-53f60db7-a1 and 20261107-004118-523dd8fe-a1 failed, repairs queued.


branch lab/gold-parity; head 0a7ae37937e8b022b38fb5b0adc176a6b83ce97b (tested code; following receipt commit changes documentation only); apps at parity: none; gate evidence: failing Monitor baseline and framed-host pilot retained privately; CBC jobs: 20261107-004421-72b146fc-a1 full scoped gate green, 20261107-004254-ea9ed470-a1 focused gate green.


Viu editor integration checkpoint 2026-11-07: reusable opt-in native document control over the retained gold editor/model/syntax, in-process undo and Unicode selection, virtualised gold drawing, literal find/single replace and per-frame style resolution. CBC 20261107-005339-52bd9c89-a1 passed fmt and all 5 text-input tests at e3ff143. Full log read. Initial 20261107-005250-80866903-a1 returned wrapper rc 0 but compilation failed; it is NOT green. Exact worker fmt/lock diff applied, egui API fixed. Ced and visual parity remain open; detailed limits in docs/ports/editor-widget.md.
branch: lab/viu-editor; head SHA: e3ff14316f16d867adeb27682519061fcacfc3a3 (tested source; checkpoint commit follows); widget features: retained gold input/rendering with local documents, Unicode selection and undo tested; ced parity evidence: inherited bounded specimen only, no new visual acceptance; CBC job ids: 20261107-005339-52bd9c89-a1 green, 20261107-005250-80866903-a1 failed compilation.


## Editor and Ced integration checkpoint, 2026-11-07

The reusable Viu native-document adapter now uses the retained gold editor/model/syntax. It keeps drafts, selection and undo in-process, supports gold input/rendering, wrap, line numbers, highlighting, literal find/single replace and serial-fenced local commands. Ced adopts it and the existing shared frame with named tabs, modal Open, local toolbar Undo/Redo, draft-safe tab switching/reopening, dirty-caption-close refusal and revision-fenced buffer metadata. App policy and editd traffic stay in Mix; no per-app Rust host.

CBC 20261107-011206-0f7624e2-a1 passed 62 tests/build and supplied exact formatting. Final source 8708a7544ea706f1133cc844ad438cb483f92e12 passed 20261107-011511-17897723-a1: fmt-check, 11 scoped tests, Clippy and host build. Full saved logs read; style warnings remain. Frozen binary SHA256 5813269e8c92bea5e6d8f8090852818b153c22da12f8a15fe8f27b5f24e3ebb2 matches native evidence. Earlier green scoped jobs: 20261107-005339-52bd9c89-a1, 20261107-005727-605323bf-a1, 20261107-005904-7b9d7b44-a1, 20261107-010412-9e33ee36-a1, 20261107-010928-c8dfdc78-a1. Failed checks 20261107-005250-80866903-a1 (wrapper masked compile failure), 20261107-010134-8d6454bf-a1 (Mix numeric admission), 20261107-010821-aa636c87-a1 (incomplete modal initial schema) were corrected without weakening tests. Mix lint, static app-port verifier and git diff --check pass.

Final private native witness native.qgLO4E captures gold and Mix at 1000x720, identical Rust document/revision/saved/cursor state before edits, then passes Unicode typing/Apply/Save and confirms all six owned processes stopped. Gold, Mix and perceptual diff figure inspected. PHASH distance improved from the initial 58.5477 to 22.7572; normalized RMSE 0.0662933. Remaining regions: header menu/action/caption composition; tab shape/language/close; editor inset and vertical offset; status fields/alignment. Private report and figures: editor-parity-20261107/report.md. No install, VT use or main merge.

FULL PARITY IS NOT COMPLETE. No deviation is approved. Ced still has the 64 KiB snapshot/draft limit and specimen Bus reply shapes. Full Bus/topic/prop compatibility, paged/mirrored editd recovery, persisted sessions/drafts, complete find options, diagnostics/previews/mail/macros/dialogs, live Prefs/keymaps, closed-document eviction, native clipboard/IME and representative large-file acceptance remain open. See docs/ports/ced.md and editor-widget.md. This checkpoint must not be treated as full widget or Ced acceptance.

branch: lab/viu-editor; head SHA: 435c3eb1a684c50585ea4961596a9b03a5776904 (checkpoint parent, receipt-only commit follows; validated code 8708a7544ea706f1133cc844ad438cb483f92e12); widget features: gold editor integration, local undo/selection, virtualised drawing, syntax/gutter/wrap, literal find/replace; ced parity evidence: native Unicode Apply/Save and paired inspected failed visual baseline, full parity open; CBC job ids: 20261107-011511-17897723-a1 final green, 20261107-011206-0f7624e2-a1 62-test green, 20261107-005339-52bd9c89-a1 widget scoped green.
