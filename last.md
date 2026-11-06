Round 4 shared-frame checkpoint: SSD and CSD matched 1040×680 captures are now byte-identical to gold, whole/header/options/content/status SSIM and RMSE distortion 0. SSD evidence JzNlZz; expanded native Bus/macros/topics WTwtHX and single-instance/minimise/restore xFXPHq passed. Real monitord disposable-child fence run PRUyUC passed. Full scoped preparation CBC 20261107-043323-de95ebb8-a1 at 9e9f054 passed every test/check/clippy/build, rc97 formatting applied; entire saved log read. Acceptance remains OPEN pending current hosted titlebar, live keymap and matched Prefs tests. No proposed deviation.

Round 4 visual-region closure at d358237 renderer: native matched 1040×680 Processes PNGs are byte-identical, SHA256 800c28506c24d9bc9c755c740d4cbca72486ce67aee48597e548461d12da19fc on both sides; whole/header/options/content/status SSIM and RMSE distortions ZERO. Seven Performance pairs (CPU, Memory, TCP, three disks, Network) also have whole-window and all-region ZERO distortion and identical per-pair PNG hashes. Figures inspected; durable zero-processes-d358237 and zero-performance-d358237 receipts under round4. Live Prefs on one PID: five variants plus authority loss/recovery and caption close passed, captures retained under live-prefs-d358237 (980×680 responsiveness case, not final 1040×680 matched variant acceptance). Native 51-command/45-action harness passed on 84423fe including actual UI click settlement, menus, debug, atomic theme changes and refusals. Full scoped CBC preparation 20261107-035522-be012539-a1 passed ALL Viu/viu-engine tests/check/clippy/build at d358237, rc97 formatting applied; final exact-source acceptance remains open. Async per-kind sampling and stale-ticket checks now implemented in Mix using reusable Jobs; unlimited gold process query restored. Populated TCP and full macro/topic/real-daemon tests remain in progress. No visual deviation proposed or approved; P1 remains the sole approved non-visual fence.

Round 4 TCP/options/status checkpoint: at renderer 7537b6b (CBC artifact), matched 1040×680 captures now have ZERO content distortion on all seven Performance sections including TCP; options and status also match pixel-exactly. Processes full distortion 0.00202308, Performance full 0.00202375; remaining header distortion 0.0374929 is native title pixel rounding, mark tint and separator. Figures inspected; corrections at 40d48ab on CBC now. TCP paused navigation no longer fetches; disposable-child regression also asserts no requests while navigating paused TCP and back. Native Bus HELP/describe 51-command identity and 45-action registry, dialog admission/copy availability, native dialog capture, sort toggling, native window/menu reads, macro status/output passed up to debug initialisation failure. Gold debug control was not initialised by generic host; fixed, rerun pending. No acceptance yet: input-settlement, full surface, async sampling/tickets/peers, topics/macros mutations, populated TCP connections, live Prefs and final exact-source full gate remain. Durable evidence round4/native-bus-and-processes-7537b6b and performance-7537b6b; all owned processes stopped. No approved visual deviations.

Round 3 Performance region checkpoint at 0807e73: native canonical gold/Mix paired 1040×680 captures with the same constant 60-sample histories now match CPU, Memory, all three disk pages and Network CONTENT pixel-for-pixel (SSIM distortion 0 / RMSE 0). CPU, Memory, disk and TCP side-by-sides inspected. TCP content still fails at 0.00417085: connection heading typography and paused fetch policy differ; populated TCP table still needs native evidence. Processes content remains exact, whole 0.0064456 (header 0.0928214, options 0.0203370, status 0.0016791). These are bounded fixture witnesses, not full app acceptance. Native reusable menu and option-row controls are now used; application composition/data stay Mix-owned. Narrow-row scrolling is implemented but unaccepted. Shared native drive/macros/debug facilities compile and focused tests pass; full compatibility checks remain. CBC 20261107-031548-86f79d4c-a1 passed checks/six focused targets/clippy/build, rc97 formatting applied; 20261107-031044-7b2134a3-a1 earlier focused gate rc0. Native receipt roots round3/processes-0807e73 and performance-0807e73; no install/VT, owned children stopped.

Round 3 regions checkpoint c18677b: real native sequential canonical monitor service, identical fixture/theme, 1040×680. Processes content region matches pixel-for-pixel (SSIM distortion 0, RMSE 0); whole window remains FAIL at 0.0139849 (header 0.0943344, options 0.134687, status 0.0016791). Inspected figure confirms all table typography/stripes/proportions/command-line cells now match for six-row tree fixture. No general Processes acceptance: interactive/large-list states remain. Performance seven sections render through reusable data-authored dashboard/time-series widgets; matched captures reveal chooser ordering and a blank chart header adding 20 px, being repaired. Native evidence retained privately under round3/processes-c18677b and performance-c18677b; owned PIDs stopped. CBC preparation 20261107-024928-e337ff81-a1 passed check, six focused integration targets, clippy and build; rc97 formatting returned/applied, not a full acceptance gate. Earlier schema/fixture/numeric repairs failed honestly; no test weakened. P1 disposable-child native witness previously passed at bacd117.

# Round 3: Fable P1 signal implementation checkpoint

P1 is decided: exact gold admission/no confirmation plus the invisible start-time fence. docs/ports/monitor.md records the approved non-visual deviation and corrects the UID/confirmation claim. API and native primitive are implemented. CBC 20261107-013833-4b57ccb6-a1 ran all scoped checks successfully on ddfea5f, including both disposable-child signals and mismatch survival; rc97 records returned formatting/lock changes, not exact-source acceptance. Full log read; libc lock edge applied. ESRCH/integer-token refinements at 3551c1d await the next exact-source gate. No production process was signalled.

Processes region work is in progress and unaccepted. Existing round-two SSIM distortion 0.0957189 remains a failed baseline, not evidence for current edits. P1 no longer blocks work.

# Gold parity round 2 — frame and method checkpoint

The standing parity mandate has been read. **Monitor is not at full parity.**
No deviation has been accepted, and no production install or VT work occurred.

## Fable decision P1: process-signal contract

Opus: put this source/inventory conflict to Fable before implementing Monitor's
effectful process actions. The mandate requires gold behaviour and the full
inventory, but the inventory currently names safeguards that gold does not have.

**Question:** which signal contract should the Mix Monitor implement?

- **A — exact pinned-gold behaviour (recommended for parity):** enable signals
  for a selected local process with no dialog open; accept a positive PID;
  issue SIGTERM or SIGKILL as the app's user, with kernel permission checks.
  Correct the inventory's inaccurate UID/start-time/confirmation claim.
- **B — stronger admission as an approved deviation:** add UID/start-time
  identity checks and confirmation before signalling. Approve the resulting
  behaviour, UI and refusal-contract differences explicitly and specify the
  confirmation contract for Bus callers.

Evidence from immutable gold `8779f33284b7d3f1178c3d57a699efb9b0867b22`:
`apps/monitor/crates/vitals/src/engine.rs:623` tests only local node, no dialog
and selection. `apps/monitor/src/shell.rs:172–187` validates positive PID and
calls `kill(2)` with SIGTERM/SIGKILL. There is no identity fence or confirmation.
`docs/ports/monitor.md` instead says the original UID/start-time admission and
confirmations must be ported. The discrepancy is now annotated in that document.
Neither signal policy is implemented in Mix, and no process-signalling test or
production signal was performed. No question is being sent directly to Mark.

## Frame implementation and validation

The frame now mounts independently resolved content. Original app root/control
IDs, bindings, native input, focus targets and dock intents are preserved;
there is no root rename. Reserved frame-node collisions refuse explicitly.
Content uses the dock-allocated region, clips to it, and draw receipts cover
both scenes. Model updates validate both scenes before committing app state.
Monitor now opts into the shared frame; application state and callbacks remain
Mix-owned. Caption close returns to Mix before viewport closure.

The generic host follows settingsd through an embedded Mix session worker and
the application's existing native connection. Intra-app delivery is in process.
The retained gold `session_theme` resolver/font installer supplies the complete
Look contract: app overrides, automatic mode, schemes/styles, accent/tints,
density, roundness, fonts, text scale, icon stroke, tunables and file-theme/
lost-authority fallback. Frame bar lengths, title/status type and app-mark sizes
use native design metrics. No application policy was moved into per-app Rust.

Independent surveys identified and prompted fixes for root relocation, close
ownership, content clipping, empty placeholder geometry, lost-authority theme
fallback and premature frame draw receipts. Two frame regressions and the
strengthened app-bundle gate cover actual mounting instead of an unframed bypass.

Final native private settingsd test used the exact final CBC binary: five live
variants on one Monitor PID (light, Studio/orange, compact/square, fonts/scale/
stroke, dark/Pro), then authority disappearance/restart and Mix-owned caption
close, all passed. Seven captures and revision/PID receipts are retained;
light, enlarged-font, dark and lost-authority captures were inspected. The
full-region background and large-font status repairs were confirmed. This
frame responsiveness test was 980×680, not a Monitor visual acceptance pair.
Automatic-mode clock transitions, all app overrides and all tune keys still
need representative native acceptance cases; installing the complete gold
resolver alone does not certify every input combination.

Exact scoped CBC `20261107-012348-cc3a95cf-a1`, worker cbc3, source
`bb950a59f82ef5c400fe89eaf1175a42bef6f93c`: BUILD_RC=0. Fmt-check, locked
all-target check, all Viu/viu-engine tests, clippy and debug host build passed;
the complete 1803-line saved log was read and worker diff was empty. Existing
warnings remain. This was not the full four-engine integration/process gate.
Artifact SHA256 is
`2044c741983b90bdc48ae2bc6ce7c07f461f779341a13dd8df9044ef6659ce51`,
identical to the binary used in the final native runs.

## Method and remaining Monitor work

`docs/ports/parity-method.md` specifies pinned oracle, isolated native services,
matched data/state/theme/scale, 1040×680 capture, region inspection, four parity
gates and acceptance records. `tools/parity_compare.mix` refuses rescaling and
existing evidence directories; it writes crops, figures, raw metrics, hashes
and an `inspection-required` verdict. Identical-image and old-baseline probes
were run. ImageMagick 7.1.2-35 SSIM output is normalised **distortion**, where
identical images return zero. The historical baseline value 0.137345 must not
be called mathematical SSIM similarity; its RMSE distortion is 0.183086.

Monitor still needs its gold menu/options/status composition, seven-column
styled table, complete tree/selection behaviour, node discovery, all Performance
resources/charts and navigation, actions/dialogs/macros, and its 50-command ABP
surface with gold replies/topics. Original refresh is asynchronous; original
rows/info/ping/refusal shapes differ from the current specimen. Use pinned git
objects for shell/main: the maintainer's source checkout has unrelated edits.
A fresh gold-first 1040×680 pair used matched Studio Dark settings, client
decorations, Inter 400 at 14.667, scale 1, paused state and the same six-row
native fixture. The inspected pair **fails**: normalised SSIM distortion
0.0957189, RMSE 0.118710. Region SSIM distortions: header 0.119769, options
0.258504, content 0.0859221, status 0.125844. The method document records the
rectangles and remaining differences. These scores are not directly comparable
with the historical mismatched-theme diagnostic. Fable has approved nothing.

Private durable evidence is in the hub's `gold-parity-20261107/round2`:
`final-live-prefs` and `matched-monitor-pair`, including scripts, PNGs,
side-by-side/diff figures, raw metrics and closure receipts. Every owned PID
was confirmed stopped. The next implementation checkpoint needs Fable P1's
signal contract; Opus can relay the question above. Monitor remains unfinished.

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


branch lab/gold-parity; head bb950a59f82ef5c400fe89eaf1175a42bef6f93c (tested code; receipt-only commit follows); frame: identity-preserving mount adopted by Monitor, five live Prefs variants/recovery/close passed, remaining input combinations unaccepted; Monitor: NOT at parity, matched 1040x680 SSIM distortion 0.0957189, menus/options/table/status/Performance/ABP/actions remain, no accepted deviations, Fable P1 pending; CBC job ids: 20261107-012348-cc3a95cf-a1 (final scoped green), 20261107-012031-152ba7e4-a1 (preparation passed with formatting diff), 20261107-011138-9b732aa9-a1 (earlier scoped green).

## Prefs parity: Fable P2 — inventory contradicts gold command contracts

Stopped before frame adoption under the per-app brief's explicit instruction
to ask Fable and stop whenever gold and the inventory disagree. Opus should
relay P2; no Fable answer or approved deviation is recorded here.

Inspected candidate source: `64e3b7b49707f4e14ac49f6b342315536786db05`.
Gold oracle: `8779f33284b7d3f1178c3d57a699efb9b0867b22`, read through
`git show`/`git grep` at that revision, not the oracle's working tree.

Evidence:

- `docs/ports/prefs.md:14` labels search/section/key selection and Bus
  commands as ported. These names conceal incompatible gold contracts.
- Gold `apps/prefs/crates/preferences/src/engine.rs:1138` returns
  `search_view(query)` for `prefs.search`; `model.rs:131` declares it
  read-only and describes page/row landing hits. Candidate
  `apps/prefs/app.mix:41` changes `state.search`, republishes the window
  model and returns `{}`. A valid query therefore changes the candidate
  UI and supplies none of the gold search result data.
- Gold `engine.rs:1178` requires `prefs.select {app}`, checks the software
  app catalogue and busy/dialog state, selects the app and returns
  `apps_view()`. `model.rs:133` describes opening its Software group and
  loading release notes. Candidate `apps/prefs/app.mix:45` instead requires
  `{key}` through `core.choose` and returns `{}`. A gold-shaped request
  cannot select a software app in the candidate.
- Gold `prefs.section` also returns section/category/UI data and validates
  optional software group; the candidate only validates its reduced
  section list and returns `{}`. The inventory does not enumerate these
  missing reply/error contracts.

Question P2: should the inventory's claimed ported navigation/Bus surface
be replaced with the pinned gold contracts, and how should the existing
registry-browser commands survive?

Options:

1. Recommended: gold owns the canonical `prefs.search`, `prefs.select` and
   `prefs.section` names, arguments, replies and refusals. Correct the
   inventory, port the full gold search/navigation/software behaviour and
   move any retained registry-only operations to distinct command names.
   No acceptance credit carries over from the current handlers.
2. Keep the current registry command contracts and explicitly approve them
   as incompatible deviations, with affected callers and missing gold
   behaviour recorded. This sacrifices the requested gold Bus compatibility.

Acceptance status: **BLOCKED ON P2, NOT AT PARITY**. No implementation was
changed, no binary built or tested, no native harness launched, no service
or desktop state changed. Feature, Bus and live Prefs gates remain
unaccepted. No matched captures, binary hashes, SSIM/region metrics or
cleanup receipt exist for this round; no CBC jobs were submitted. Source
inspection is evidence for the question, not runtime acceptance. This is a
receipt-only checkpoint; the source SHA below is the inspected code, and
the containing Git commit identifies the hand-off receipt.

branch lab/parity-prefs; head SHA 64e3b7b49707f4e14ac49f6b342315536786db05 (inspected source; receipt-only commit follows); frame not adopted; parity evidence: features/Bus/Prefs unaccepted, P2 contract evidence above, SSIM/regions not measured; CBC job ids: none.

## Prefs frame checkpoint (round 2)

P2 closed by the updated standing brief: pinned gold wins; inventory
corrections proceed without stopping. The canonical pinned search handler
returns page/row hits, superseding the older section helper's matches.

Tested source: `48b5b430f56fd7c2101f199ef15cd2cd89cabdde`. Prefs now
declares the shared gold frame and publishes its frame model; caption close
is an in-process Mix owner event. CBC `20261107-014533-4e174e67-a1`
passed locked checks, Viu/viu-engine tests, clippy and build with no worker
diff. Preparation `20261107-014020-4b3285ad-a1` passed commands but
returned rc 97 for formatting; its patch was applied.
Artifact SHA256: `2044c741983b90bdc48ae2bc6ce7c07f461f779341a13dd8df9044ef6659ce51`.
New embedded test checks real scene mounting and Mix-owned close; existing
round-two draw test now mounts framed bundles correctly. Mix core tests
and host-aware lint pass. No complete functional/Bus/live-Prefs/visual
acceptance is claimed. Feature work continues after this checkpoint.

Native gold exploration retained privately under prefs-parity-20261107:
42 pages read and captured using private Bus/settingsd/headless compd; all
four owned PIDs confirmed stopped. Gold Appearance image inspected. This
is discovery evidence, not a matched candidate acceptance pair.

branch lab/parity-prefs; head SHA 48b5b430f56fd7c2101f199ef15cd2cd89cabdde (tested frame source; receipt-only commit follows); frame adopted; features/Bus/Prefs unaccepted; SSIM/regions not measured; CBC job ids 20261107-014533-4e174e67-a1 green, 20261107-014020-4b3285ad-a1 formatting preparation.

## Prefs round 2 progress: features and visual gate still open

Code head before this receipt: `2aed4f80f0dabf31d0583eb0aeab11bffc7a9620`.
Gold remains `8779f33284b7d3f1178c3d57a699efb9b0867b22`; the installed
oracle's version explicitly attests that revision. Oracle binary SHA256:
`18590ca486b4b417b9de5a762f2dcf31b6b13d2351bb3eddd46d66e783fe3c55`.
No inventory disagreement requires a question; no gold deviation is proposed.

Implemented since the frame checkpoint: 42-page navigation and typed wrappers,
topic/reconnect settings reads and queued writes, appearance conflict/receipt
handling, specialised Terminal/Ced/Email/Software pages, Email identities and
deferred actual Email replies, release recovery/notes, bounded wallpaper scan
and native chooser/media controls, installed font/XKB catalogues, native macro
session/editor integration and canonical drive/debug command registration.
These are implementation progress, not a completed feature checkpoint.
The rewritten inventory in `docs/ports/prefs.md` names remaining acceptance.

CBC evidence, complete saved logs read:

- `20261107-024331-ac8cf226-a1`, source `f6370e6ce2260b87ec244177048823115b2f056d`:
  scoped fmt/check/test/clippy/build passed; rc 97 returned formatting, applied.
- `20261107-030733-1696f3b7-a1`, source `e081ecd78247`:
  six focused Prefs/deferred-request tests passed, including all-page mounting,
  settings fences, appearance receipt recovery and Email admission/redaction.
- `20261107-031534-7d749887-a1`, source `f2d7a9cb12ef19e4001036ffa0e4f6b96927d308`:
  scoped fmt/check/test/clippy/build passed; rc 97 formatting patch applied.
  Candidate artifact SHA256:
  `e875d774114f169d3215762d97971055b03f936c2e693c99af5cfb162fbfa1af`.
  Saved log basename:
  `20261107-031534-cbc2-f2d7a9cb12ef-20261107-031534-7d749887-a1-bead5316.log`
  under the private CBC log root.
- `20261107-032557-0d6fad36-a1`: native-menu compile repair needed public
  toolkit APIs; fixed at `c85aaa588953574345cc9eeb53d742ac0013d757`.
- `20261107-032646-bc64fe9b-a1`: invalid repair check; the caller supplied HEAD,
  resolving the primary checkout instead of this worktree. No Prefs evidence.
- `20261107-032704-f4d3462d-a1`, exact source `c85aaa588953574345cc9eeb53d742ac0013d757`:
  nine file-widget tests passed, including owned native menu ids, keyboard
  activation and public browser action admission.
- Queue job `20261107-033410-20638506`: full Prefs gate at the code head above,
  including retained toolkit menu tests; result pending at this receipt.

Private native evidence: `prefs-parity-20261107/pair.WrTxPr` captured gold
first, stopped it, then the exact CBC candidate from `f6370e6`, using the
same private native Bus/settingsd/headless compd. Both client images are
1040×680, scale 1, Studio Dark, comfortable density, CSD, right captions,
bar menus and row actions. Candidate binary SHA256:
`45eaefa8089de70e276ed89b8eaba43199bcda29675c49b4421a51540bb7ec55`.
Placement corrected native client dimensions without resizing either PNG.
Default app description, actions, pages, Appearance and missing-releases
Software replies compare structurally equal. Search remains read-only.
HELP exposed extra host commands and native menu queries returned empty;
subsequent code routes explicit HELP to Mix and data-authored menus through
the retained toolkit path. Native recomparison is outstanding.

Appearance original images, montage, full diff, header montage and sidebar
diff were inspected; figures and raw metrics are in `appearance-diff`.
ImageMagick 7.1.2-35 SSIM values are **distortion**, zero identical.

| Region | x,y,w,h | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| Full | 0,0,1040,680 | 0.113842 | 0.211563 |
| Header | 0,0,1040,38 | 0.120927 | 0.120185 |
| Sidebar | 0,38,296,642 | 0.0670151 | 0.121422 |
| Content | 296,38,744,642 | 0.131337 | 0.243328 |

Unresolved inspected differences: frame content gap, displaced title/actions,
caption glyph sizing; heading faces/spacing, mode preview geometry, disabled
row geometry, scheme chips, accent spacing/custom control and missing disabled
extras. Generic frame repairs and page/control repairs are committed; further
matched captures must verify them. This pair is **FAIL**, with no approved
deviation. Gold image SHA256:
`7285279ecdec3c3ddf4c1aebee1236353042d7ae89caf70809bfafd45f2fbbab`;
candidate image SHA256:
`c570b6f91d38e43196eb8ce2a24ba81f91a946fa2c91221354e4ca30cba335c3`.

A later all-page run `pair.izlSrW` uses the `f2d7a9c` artifact and canonical
service identity for both processes. It captured gold pages then candidate
pages through Applications, but Terminal capture timed out. It is failed
diagnostic evidence. Numeric controls now reuse gold's value-field/slider
commit rules, with a passive redraw regression test; no acceptance yet.
Both runs' cleanup receipts confirm all five owned PIDs stopped.

Remaining required work includes native service fixtures and real daemon
cases, complete action/refusal/deferred delivery coverage, launch/deep-link
compatibility, full live-Prefs axes/tunes/overrides and authority recovery,
all visual states and final exact-source checks. Frame adoption is the only
accepted checkpoint so far. Work continues.

branch lab/parity-prefs; head SHA 2aed4f80f0dabf31d0583eb0aeab11bffc7a9620 (code head before receipt-only commit); features/Bus/Prefs unaccepted; SSIM distortion 0.113842, header 0.120927/sidebar 0.0670151/content 0.131337 FAIL; CBC job ids 20261107-031534-7d749887-a1 checks passed with applied formatting, 20261107-032704-f4d3462d-a1 focused green, 20261107-033410-20638506 pending.
branch lab/gold-parity; head 3551c1d (signal implementation; receipt commit follows); Monitor parity: NOT accepted, previous matched SSIM distortion 0.0957189, region/Performance/full ABP work pending; CBC job ids: 20261107-013833-4b57ccb6-a1 (preparation, checks passed, rc97 formatting/lock patch).

Round 3 Processes prototype: native striped/resizable seven-column grid, command-line projection, gold binary units, options row, frame menus and Mix-owned End task/Kill share P1 admission. Embedded-Mix regression passed at a424cf3: mismatch survives, refreshes both inputs, no dialog; omitted start_time uses snapshot and signals only the disposable child. Mounted table exposes command-line geometry. CBC 20261107-014455-cb1b2c26-a1 all scoped checks passed, rc97 returned formatting. Native paired visual acceptance is still pending. Status-line reusable widget and native bar details now in progress.

branch lab/gold-parity; head a424cf33a694614a07e6d86eba753e9981c3f41e (tested preparation source; next source includes returned formatting and status work); Monitor NOT at parity, latest accepted visual evidence none, previous failed SSIM distortion 0.0957189; CBC job ids: 20261107-014455-cb1b2c26-a1, 20261107-013833-4b57ccb6-a1 (both preparation, tests passed, formatting returned).

Round 3 native checkpoint at bacd117: matched gold/Mix pair at 1040x680 with identical settings/fixture now has SSIM distortion 0.0678706 (content 0.0558222, header 0.136426, options 0.208595, status 0.0016791). Full/status side-by-sides and individual originals inspected: still FAIL, header/options and table offsets/divider extent need repair. Private pair frame-live-prefs-round2.qcWsF1, gold PNG hash unchanged 800c2850..., candidate a785c896.... Native Bus test monitor-signals-round3.Cm2qTi passed both disposable-child signals, explicit mismatch survival/refresh/no prompt and omitted-token snapshot admission; all six owned PIDs stopped. Exact scoped CBC 20261107-015349-51192852-a1 BUILD_RC=0, empty diff, binary 6c7064907fecc95a749d68d2f28b584e712f87064c70544c606b98c6aa7f9aee matches native runs. Header full-band/no-gap/native caption and table-fill repairs follow; no new visual acceptance yet.

branch lab/gold-parity; head bacd11720d20f22a472c7f67cbfa4bdd579e7a7c (tested code; next checkpoint includes visual repairs); Monitor NOT at parity, matched SSIM distortion 0.0678706, header/options/table/Performance/full ABP pending; CBC job ids: 20261107-015349-51192852-a1 (scoped green), 20261107-014915-bbf51126-a1 and 20261107-014455-cb1b2c26-a1 (preparation, formatting returned).

branch lab/gold-parity; head c18677b4207789f4e3ad4b5f4f4a3d3b76c0b6b9 (capture source; receipt/format commit follows); Monitor NOT at full parity, Processes content SSIM distortion 0, full 0.0139849, header/options/status/Performance/full ABP remain; CBC job ids 20261107-024928-e337ff81-a1 (preparation passed, rc97 fmt), 20261107-015349-51192852-a1 (earlier full scoped green).

branch lab/gold-parity; head 0807e73 (native region witness; receipt/format commit follows); Monitor NOT at full parity: Processes and six Performance content regions SSIM distortion 0, Processes full 0.0064456, TCP/header/options/status/ABP/action states remain; CBC job ids 20261107-031548-86f79d4c-a1 (preparation passed, rc97 formatting), 20261107-031044-7b2134a3-a1 (focused green), 20261107-024928-e337ff81-a1 (preparation).

## Prefs round 3: shared-frame merge and repair checkpoint

Merged `lab/gold-parity` at `0841770` before further parity work, merge
commit `0047ba72342cc7b3737950dd205f1e59acef97a7`. Conflicts preserved
Prefs deferred requests, bounded jobs and application theme choices alongside
the shared frame's native captions, structured status, overlays, local UI
operations and generic native facilities. Monitor source changes here come
from that authorised merge; this job does not change Monitor policy.

Data-authored frame menus use the retained native toolkit renderer with
owned command identities, preserving separators, shortcuts, checks and gold's
File/View/Help hierarchy. The explicit application HELP handler bypasses the
host's automatic list, so private host commands do not replace the canonical
surface. Embedded native model/keyboard/admission tests pass; exact native
Bus recomparison is still required.

Terminal numeric controls now use gold's value field and slider commit rules.
Passive precision rounding never emits a settings write; that regression test
passes. Appearance headers/cards/previews/chips/swatches, pending notices and
disabled extras have implementation repairs. New paired evidence must verify
the native Terminal timeout and every visual change. Pages accepted: **0/42**.

CBC (saved full logs read): `20261107-033410-20638506-a1` found a scheme-chip
colour conversion error; fixed. `20261107-033807-34f3d949-a1` found a merge
omission in session theme readback; fixed, formatting patch applied.
`20261107-034027-0a438d39-a1` and `20261107-034248-534fb98a-a1` passed
checks, native widget/geometry tests and the passive redraw test, then found
the optional hidden status model incompatible with the new status-line widget.
Fixed in `72c70c6`. Focused repair `20261107-034401-76fce065-a1` caught a
temporary empty-theme lifetime; fixed in `d0b95f7`. Current full gate queue
`20261107-034553-0321d97f` tests that source; result pending here.

The owned capture driver now covers all 42 pages, native menu paths and 12
live Prefs variants, with settings revision/PID readbacks and gold-first
ordering on identical private services. It has not yet run with the repaired
artifact. Previous figures remain failed diagnostic evidence:
`prefs-parity-20261107/pair.WrTxPr/appearance-diff`; no new visual acceptance
or Fable-approved deviation is claimed. No owned native capture processes
are running at this checkpoint.

branch lab/parity-prefs; head SHA 86c003c1b64c0df4be2fe191b0d723baf358111d (code head before receipt-only commit); pages at acceptance 0/42, figures pair.WrTxPr/appearance-diff FAIL; Bus/menu native implementation repaired, exact native HELP/menu comparison pending; CBC job ids 20261107-033807-34f3d949-a1 merge repair, 20261107-034027-0a438d39-a1 and 20261107-034248-534fb98a-a1 status repair, 20261107-034401-76fce065-a1 lifetime repair, 20261107-034553-0321d97f pending.

## Prefs round 3 native capture checkpoint: Bus menus and HELP matched

Tested source `d0b95f7063d32bf49c2577d549d90215871d20bf`; candidate artifact
SHA256 `53117836f345049dd31857fc31f5e24669d929157369cc925e5d2102f9b1bc12`.
CBC `20261107-034553-0321d97f-a1` passed the scoped Viu/viu-engine check,
test, clippy and build sequence, plus 32 toolkit menu unit tests and 32 menu
integration tests. It returned rc 97 for one formatting patch, now applied.
The complete saved log was read:
`20261107-034554-cbc2-d0b95f7063d3-20261107-034553-0321d97f-a1-838e35aa.log`
under the private CBC log root. A final exact-source gate is still required.

Private run `prefs-parity-20261107/pair.TD1EvM` completed all **42 gold
and 42 candidate page captures**, then 11 accepted live variants per process.
Both clients are 1040×680 at scale 1 on identical isolated native services,
gold first then candidate. Terminal now captures successfully. All five
owned processes are confirmed stopped in `closure.json`. The receipt records
source/binary hashes; earlier gold pin/hash remain unchanged.

Exact structured comparisons passed: HELP, app.describe, prefs.actions,
prefs.pages, the full native menu model, and open-path replies for File, View,
View → Theme and Help. HELP has exactly gold's 55 commands; host extensions
are absent. Menus preserve original command IDs, shortcuts, separators,
checks and enablement. This closes the reported HELP/menu readback gaps;
complete Bus mutation/refusal/service acceptance remains outstanding.

Live settings tests retained the same PID through mode, scheme/style, accent,
compact/spacious density, square roundness, interface/mono families, weight
and size, text scale, icon stroke, app overrides and auto mode. Each accepted
change waited for its settings receipt revision before capture. The invalid
`tune.chrome.title_size` request was refused by both authorities; this is
refusal evidence, not a valid tunable variant or full Prefs acceptance.

All 42 regional comparisons and figures are indexed in
`pair.TD1EvM/comparison-index.json`; each `diff-<page>` retains originals,
montages, differences and header/sidebar/content crops. ImageMagick 7.1.2-35
SSIM numbers are distortion, zero identical. Appearance full is `0.0734909`,
header `0.0883552`, sidebar `0.00161027`, content `0.100063`. Terminal full
is `0.0577293`; pending-page full values are about 0.011–0.015. These remain
**FAIL**, not arbitrary score-based passes. Originals, Appearance montage/full
diff and header montage were inspected. All-page inspection/closure continues.

Gold live visual quirk confirmed by pinned shell.rs: follow_theme increments
look_key.0, but follow_look only rebuilds COSMIC page colours after density or
roundness differs. Mode changes update header/global typography and control
swatches while page/sidebar colours remain retained until that refresh.
The light pair visibly demonstrates this. Gold wins: Mix now authors the same
palette refresh key; a reusable browser cache implements that key. No deviation
is proposed and no Fable question is needed.

Subsequent source `03db134d0a6027fcd622cd6ba7e961c8fc66ff7f` replaces guessed
header widths with native measured menu/icon/action widths, native action
spacing and the native automatic-title offset. It also repairs the sidebar
divider and page back-link spacing and preserves the gold palette quirk.
Generic native facility polling now stays with the opted-in request owner.
CBC queue `20261107-040306-60d6ff3c` tests those changes; result pending.
No page is visually accepted yet; further paired captures are required.

branch lab/parity-prefs; head SHA 03db134d0a6027fcd622cd6ba7e961c8fc66ff7f (code head before receipt-only commit); pages at acceptance 0/42, all 42 pairs/figures in pair.TD1EvM/diff-<page> and comparison-index.json FAIL; Bus HELP/full menu/four open paths exactly gold, full mutation matrix open; CBC job ids 20261107-034553-0321d97f-a1 checks and 64 menu tests passed with applied formatting, 20261107-040306-60d6ff3c pending.
branch lab/gold-parity; head 40d48ab (code; following receipt-only checkpoint); Monitor acceptance OPEN: Processes whole SSIM distortion 0.00202308, Performance 0.00202375, all content/options/status 0, header 0.0374929; CBC job ids 20261107-034243-bf3be532-a1 (preparation passed, rc97 fmt applied), 20261107-033915-8cc83126-a1 (focused green), latest 40d48ab gate pending.

branch lab/gold-parity; head af5654a (latest code; receipt/format commit follows); Monitor acceptance OPEN, matched Processes and seven Performance whole/per-region SSIM distortion 0 and RMSE 0, remaining functional/topic/populated TCP/live matched Prefs/final-source gate evidence pending; CBC job ids 20261107-035522-be012539-a1 (full scoped preparation passed, rc97), 20261107-040200-eac42299-a1 (focused preparation passed, rc97), 20261107-035232-0923c6e8-a1 (focused preparation passed, rc97).

## Prefs round three: shared frame merge and page repairs

Merged the newly advanced `lab/gold-parity` through `15956a6` at `3a3178a`.
Shared native title rounding, chrome spacing, transparent window and header
separator fixes replace overlapping local frame work. Prefs retains owned
menu IDs, explicit HELP, optional status, its own drive request owner and
application ID. No Monitor policy was edited outside the authorised merge.

CBC `20261107-040306-60d6ff3c-a1` and
`20261107-041056-1850f5c9-a1` passed the complete scoped checks and 64 native
menu tests, returning formatting now applied. Job
`20261107-041404-834c2726-a1` likewise passed at `83d9d12` with formatting
subsequently covered by the newer patch. Complete saved logs were read.
Job `20261107-041930-a1c36911-a1` caught lost settings-browser routing after
the shared merge; `20261107-042649-ef41c654-a1` confirmed the route repair
and caught its missing decode registration. Both regressions are repaired;
tests were retained. CBC `20261107-042818-cf12b0c2` checks `00e0e45` through
the new scoped capture repair build. Final acceptance still uses the full
`apps/prefs/gate.mix`, with no worker diff.

Repairs cover vertical radio rows, native dropdown widths, segmented/image
choices, sliders and spin controls, custom form heading/button spacing,
Email's empty account divider, and removal of duplicate child cards on
section pages. Native page readbacks in `pair.TD1EvM` match on 34/42 pages
after normalising the sequentially accepted settings revision. Corrections
for the other eight cover backend messages, gallery/ordered/readout metadata
and the hidden battery section; new native verification is required.
No page is visually accepted. No deviation from gold is proposed.

branch lab/parity-prefs; head SHA 00e0e454ae0787978d9828e39f870930c856e9af (tested code before receipt-only commit); pages at acceptance 0/42, figures pair.TD1EvM/diff-<page>, Appearance full/header/sidebar/content SSIM distortion 0.0734909/0.0883552/0.00161027/0.100063; Bus HELP 55/full menu/four open paths exact gold, 34/42 normalised page replies matched before latest repairs, full mutation matrix open; CBC job ids 20261107-040306-60d6ff3c-a1, 20261107-041056-1850f5c9-a1, 20261107-041404-834c2726-a1 checks passed with formatting, 20261107-041930-a1c36911-a1 and 20261107-042649-ef41c654-a1 caught repaired merge regressions, 20261107-042818-cf12b0c2 pending.

## Prefs round three: complete native page pair and first visual acceptance

Tested source `8297a5c474679b4e4049dde11d2d4e4ed9a6f388`, candidate
SHA256 `dd34591e59f1c6b72bbe57b434020db9ae82c764618324f78c4cfe81a79f9827`.
Pinned gold is `8779f33284b7d3f1178c3d57a699efb9b0867b22`, installed
Prefs 0.1.2, binary SHA256
`18590ca486b4b417b9de5a762f2dcf31b6b13d2351bb3eddd46d66e783fe3c55`.
Private evidence `prefs-parity-20261107/pair.QUy9dU` contains all 42 paired
1040×680 client captures, 17 accepted live variants on each unchanged PID,
the invalid tune refusal and verified cleanup of all five owned processes.
Gold ran first against the same private settingsd/Bus/headless compositor;
scale 1, Studio Dark, comfortable, CSD/right captions/bar/row actions.

Native HELP (55 commands), app.describe, actions, pages, full menu and all
four tested open paths are structurally identical. The preceding successful
`pair.PC3WBd` also confirms all 42 page replies after normalising only the
sequential accepted store revision. Full mutation/topic/native-input and
authority restart acceptance are still open.

ImageMagick 7.1.2-35 uses SSIM distortion (zero identical). Every header and
sidebar region is pixel-identical. Original pairs, side-by-side figures and
the zero-difference contact sheet were inspected for the 20 accepted pages.
Ced's remaining 16×14 slider-edge residue is retained as open, with full
distortion 0.00000000184564; it is not silently passed by a threshold.
All remaining page figures are retained under `diff-<page>`.
| Page | Full SSIM distortion | Content SSIM distortion | Verdict |
| --- | ---: | ---: | --- |
| network | 0.000507405 | 0.000500839 | open |
| vpn | 0 | 0 | accepted |
| wifi | 0.00538869 | 0.00797843 | open |
| wired | 0 | 0 | accepted |
| bluetooth | 0 | 0 | accepted |
| accessibility | 0.000317218 | 0.000469669 | open |
| magnifier | 0.00523187 | 0.00774624 | open |
| osk | 0 | 0 | accepted |
| desktop | 0.00101478 | 0.00100165 | open |
| wallpaper | 0.0380345 | 0.0563134 | open |
| appearance | 0.000260795 | 0.00038613 | open |
| panel | 0.000124565 | 0.00018443 | open |
| panel_applets | 0 | 0 | accepted |
| dock | 0.000136277 | 0.00020177 | open |
| dock_applets | 0 | 0 | accepted |
| window_management | 0.000353627 | 0.000523576 | open |
| workspaces | 0.00880702 | 0.0130396 | open |
| displays | 0 | 0 | accepted |
| sound | 0 | 0 | accepted |
| sound_applications | 0 | 0 | accepted |
| sound_profiles | 0 | 0 | accepted |
| power | 0.0488153 | 0.0722754 | open |
| input | 0.000676554 | 0.000667799 | open |
| keyboard | 0.000498594 | 0.000738212 | open |
| keyboard_shortcuts | 0 | 0 | accepted |
| mouse | 0 | 0 | accepted |
| tablet | 0 | 0 | accepted |
| touchpad | 0 | 0 | accepted |
| applications | 0.00101478 | 0.00100165 | open |
| default_apps | 0 | 0 | accepted |
| startup_apps | 0 | 0 | accepted |
| x11 | 0 | 0 | accepted |
| term | 0.000197217 | 0.000291998 | open |
| email | 0.0155522 | 0.0230263 | open |
| ced | 0.00000000184564 | 0.00000000273264 | open |
| time | 0.000338261 | 0.000333884 | open |
| date_time | 0 | 0 | accepted |
| region | 0.00634753 | 0.00939807 | open |
| system | 0.000507405 | 0.000500839 | open |
| users | 0 | 0 | accepted |
| about | 0 | 0 | accepted |
| software | 0.031198 | 0.0461913 | open |

The next repair source `e476df87b84fc90c3a5233e84dc2c1af2c98b39a`
matches gold's authored theme-install trigger, live/empty Wi-Fi controls,
radio alignment, empty ordered-list separators, searchable append drawers,
child-link alignment and native Software/Email composition. CBC jobs
`20261107-044205-93245db2-a1`, `20261107-045001-4c23a9e7-a1` and
`20261107-045438-a29b5928-a1` passed all 15 scoped widget/Prefs tests and
the native build, returning formatting now applied. Complete saved logs
were read. The typo checkout queue job `20261107-044157-09b52b69` did not
run (rc75); it was requeued correctly, not counted as a failed test.
The full exact-source app gate remains required. No deviation is proposed.
branch lab/parity-prefs; head SHA e476df87b84fc90c3a5233e84dc2c1af2c98b39a (code before formatting/receipt); pages at acceptance 20/42 at 8297a5c, full/header/sidebar/content SSIM and RMSE 0 on accepted pages, figures pair.QUy9dU/diff-<page> and comparison-index.json; Bus HELP 55/full menus/four open paths exact gold, mutation/topic matrix open; CBC job ids 20261107-044205-93245db2-a1, 20261107-045001-4c23a9e7-a1, 20261107-045438-a29b5928-a1 scoped tests/build passed, formatting applied, final full gate open.

## Prefs round three: thirty accepted pages, shared frame through 9a6cfd8

Merged the advanced shared frame at `8c52cd1`, including logic-only native
Bus polling, hosted chrome, single-instance activation and native key bindings.
Prefs retains owned menu IDs, its request queue, explicit HELP and optional
status. The merge's private toolkit-string call was corrected at `4d736a8`.
CBC `20261107-050018-d95872b5-a1` caught that compile issue;
`20261107-050310-4e784986-a1` then passed the complete scoped checks,
clippy, tests, native build and 64 menu tests. Its sole formatting patch is
applied. Complete saved logs were read. This is preparation, not the final
exact-source gate after the subsequent repairs.

The complete pair `prefs-parity-20261107/pair.nnxBpc` tests
`39c9a762f6dc5658958494dc5fa0ffdee1b202e7`, candidate binary SHA256
`dacad78fd9b9001f14175a5057521ff17132218ef4de70a340f7118963690460`.
Gold revision/binary, native 1040×680 dimensions, scale and services follow
the preceding record. All 42 baseline pairs and 17 live variants per app
completed; both PIDs stayed unchanged. All seven owned process IDs, including
the restarted private settingsd processes, are confirmed stopped.

Thirty baseline pages now have full/header/sidebar/content SSIM **and RMSE
distortion zero**. The newly accepted pages are Network & wireless, Wi-Fi,
Magnifier, Desktop, Input devices, Applications, Time & language, Region &
language, System & accounts and Software. Their originals, side-by-side
figures and zero-difference contact sheet were inspected; the earlier twenty
remain pixel-identical. `acceptance.json` names every accepted and open page.
All 42 header and sidebar regions are pixel-identical. HELP (55), describe,
actions, page tree, full menus and the four menu paths still match exactly.

The twelve remaining baseline pages are Accessibility, Wallpaper, Appearance,
Panel, Dock, Window management, Workspaces, Power, Keyboard, Term, Email and
Ced. Inspection locates most differences in the bottom scroll fade, caused
by the candidate's transparent UI stack background; the native panel frame
is now supplied. Wallpaper layout matches apart from thumbnail rendering:
the candidate now uses the native thumbnail pool, cache sizes and rounded
texture mesh. Ced retains the tiny slider-edge residue. These fixes require
new captures; none is silently accepted by a threshold.

The paired native authority probe reveals gold's precise behaviour: local
settingsd absence retains the accepted role/revision/stale state and look,
reports the missing service, and can queue a valid control write. A transport
failure is distinct from a mirror reporting authority_unreachable. The
candidate's stale/error state and validation of effective defaults differed;
repairs now check only proposed values through a reusable native registry
binding. A regression test includes effective defaults and nulls. Recovery
and queued-write parity still need native verification.

The independent native functional pair `pair.BI8UOc` matched 28/40 direct
requests exactly, including macro queries/refusals and debug. Twelve typed
refusal/code/wording differences were recorded and corrected from gold;
About and shortcut dialogs now use the existing native toolkit dialog and
gold strings. Full mutation, topic, native-input, populated Software/Email,
drawer and variant visual acceptance remain open. No deviation is proposed.

CBC `20261107-045438-a29b5928-a1`, `20261107-051345-3457782c-a1` and
`20261107-052000-ee775a89-a1` passed all 15 scoped widget/Prefs tests and
build, with formatting applied. `20261107-050905-9b06833f-a1` caught a
gallery popup call error; `20261107-051013-204dc482-a1` passed at the tested
capture source with no worker diff. New exact-source repair job
`20261107-052621-6dd8b9e9` is pending. Overall acceptance remains **OPEN**.
branch lab/parity-prefs; head SHA 9b13b5b2fb29c76136f4b5314caad058d0562230 (code before receipt); pages at acceptance 30/42 at 39c9a76, all full/header/sidebar/content SSIM and RMSE distortions 0 on accepted pages, figures pair.nnxBpc/diff-<page>, comparison-index.json and acceptance.json; Bus HELP 55/full menus/four open paths exact gold, functional 28/40 before latest gold corrections, full mutation/topic acceptance open; CBC job ids 20261107-050310-4e784986-a1 full preparation passed with applied formatting, 20261107-051013-204dc482-a1 repair gate green, 20261107-051345-3457782c-a1 and 20261107-052000-ee775a89-a1 passed with applied formatting, 20261107-052621-6dd8b9e9 pending.

## Prefs round four checkpoint: admission and exact refusal replies

Effective settings readback is now reduced to its authored representation
before native registry transaction admission. Defaults, nulls and live
observations are excluded; domain/account and cross-setting validation remain
in registry::apply. The Email rejection regression and effective-default
regression both pass without weakening tests.

Native paired functional requests now match **40/40 exactly**, including
all twelve previous refusal/code/wording differences. Evidence:
`prefs-parity-20261107/pair.1jvzHj/functional-differences.json` (empty),
gold/candidate-functional.json and closure.json (all five processes stopped).
Tested code `6e17e2af265ad6bf22248150a49fbc22d129214f`; candidate binary
SHA256 `acc0ce1ec168624c55a508fd247efaf7c660e7c070eae8ac3f63c6ec995562f0`.

The same source completed all 42 baseline pairs and 17 live variants in
`pair.ZQshvb`. Thirty-six baseline pages have full/content/header/sidebar
SSIM and RMSE distortion zero; originals and side-by-side figures inspected.
All 42 headers and sidebars match. HELP 55, describe/actions/pages, full
menus and four menu paths are structurally identical. Service/theme/size
recipe remains the pinned gold, private native Bus/settingsd/compositor,
1040×680, scale 1, Studio Dark/comfortable/CSD. ImageMagick 7.1.2-35
reports distortion: zero identical. Comparison-index.json retains all metrics.

Three further pages, Wallpaper, Email and Ced, are pixel-identical at
`8f163b122acbdddfa251e65bff91f8cf3cb72350`, artifact SHA256
`0f90aaa13f409948267a3326d6e98f51dc99bd0d69c36a34b78570060893a40d`.
`pair.2ABWvc/diff-<page>` retains original/region/difference figures and
acceptance.json records inspection. Wallpaper uses the native thumbnail
pool/cache/rounded mesh; an extra background beneath ready textures caused
the last corner residue. Email/Ced now use gold's 14.667 text default.
These three PNG pairs have identical hashes. Baseline acceptance is **39/42**.

Appearance, Panel and Dock have only scrollbar differences at their initial
positions; bottom captures exposed a reset button incorrectly placed in a
labelled card. The standalone reset composition is corrected at `6d4b6c7`;
new complete captures are in progress. Live light-mode icon and fractional
density frame rounding fixes are also awaiting paired acceptance.

Authority loss preserves gold's accepted role/revision/look and a valid set
returns gold's queued-write reply. The first probe caught a lost write on a
missing-service reply; retry handling was repaired. The next probe correctly
kept the write active, exposing a harness assumption that Refresh is always
enabled. The harness now tolerates gold's DISABLED while writing. Recovery
must still be verified. Both failed authority runs retained all captures and
confirmed all seven processes stopped. No deviation is proposed or approved.

CBC `20261107-053534-6bd3db64-a1` passed the complete scoped app gate at
`55e404cd2032da1dde26f038c04006b6cf350c24` with no worker diff, including
all Viu/engine tests, check, clippy/build and 64 native menu tests. Complete
saved log read. Repair jobs `20261107-053304-929180a3-a1`,
`20261107-053725-ddcd0572-a1` and `20261107-054123-1ae3f7d6-a1` passed
15 tests/build and returned formatting, now applied. Job
`20261107-054452-bc7007ff-a1` caught an initial nil-look access, corrected
without test changes; `20261107-054605-8c5bf1b0-a1` passed 15 tests/build
with no diff at `6d4b6c7bebaf95255f1c3daeea8bd5d05760eeb4`.
Full gate `20261107-054646-be78c922` is in progress at code head
`2dd337a9719e7bc53a4e7c00b84afb91a14a2ded`. Overall **OPEN**: final
baseline/live variant acceptance, authority recovery and the outstanding
full mutation/topic/native-input/populated-service matrix.
branch lab/parity-prefs; head SHA 2dd337a9719e7bc53a4e7c00b84afb91a14a2ded (tested code before receipt); pages 39/42 with all-region SSIM/RMSE 0; functional 40/40 exact; OPEN: three reset-page captures, live variants, authority recovery and full mutation/topic/native-input/populated-service acceptance; CBC job ids 20261107-053534-6bd3db64-a1 full PASS, 20261107-054605-8c5bf1b0-a1 repair PASS, 20261107-054646-be78c922 pending.

## Prefs round four: complete baseline acceptance, overall still open

All **42/42 baseline pages** now match gold pixel-for-pixel in one complete
native paired run, `prefs-parity-20261107/pair.nIbtod`. Full window, header,
sidebar and content SSIM and RMSE distortions are **zero on every page**.
`comparison-index.json`, `acceptance.json` and `diff-<page>` preserve the
region rectangles, original PNG hashes, raw metrics and difference figures.
Originals/side-by-side images were inspected through the closure runs; the
final reset-page figures were inspected again in `pair.KghwYB`.

Tested source: `9353aaf74812d224eaea8dec5fb0071ba2131f05`.
Candidate binary SHA256:
`237b10a7b326769866426317e2f5f0f9d54257eed63ca663751ea810000e26ef`.
Gold revision remains `8779f33284b7d3f1178c3d57a699efb9b0867b22`, binary
SHA256 `18590ca486b4b417b9de5a762f2dcf31b6b13d2351bb3eddd46d66e783fe3c55`.
Both use the same private native Bus/settingsd/compositor, 1040×680 client,
scale 1, Studio Dark/comfortable/CSD. ImageMagick 7.1.2-35 reports distortion,
zero identical. Every HELP/describe/actions/pages/full-menu/four menu-path
comparison is exact. All seven owned processes, including both restarted
settingsd processes, are confirmed stopped in closure.json.

The functional matrix remains **40/40 exact**, repeated at
`49d9f6ae27bb0b1999c3ecac473d96960ff8faa5` in `pair.bISgS0` after fixing
toolbar accessible names to include native shortcut hints. The native search
click now focuses the same field and rectangle as gold. About and Keyboard
shortcuts PNGs are also exact (`pair.WUYkD6`). The original twelve refusal
differences are closed. Admission accepts effective defaults/nulls while
retaining the native registry's account/domain/cross-setting validation.

Fifteen live variants are exact in every region at the complete-run source;
their gold/candidate contact sheets and regional figures were inspected.
Native panel GUI rounding was then corrected from physical pixels to
egui's 1/32 logical-point rounding. Compact is exact in `pair.FGb10H`.
Spacious differs only at the Macros icon's 16×13-pixel anti-aliased edges
(whole SSIM distortion 0.0000928134); enlarged originals and native widget
bounds are retained. No geometry or other page region differs. Final live
acceptance and the retained-authority chrome capture are being rerun.

Both apps preserve the accepted role, stale flag and revision on local
settingsd loss, return the same valid queued-write reply, and settle it after
restart on the same PID. Pending controls now preview gold's queued values
and show its retained error notice. A shared-host defect reset window settings
on a lost session, hiding the client mark/captions; the host now retains its
accepted window settings. This repair awaits final paired verification.

Native input exposed a further search-result gap: gold presents matching
sections with live controls rather than a button list. Mix now authors grouped
results, rendered through reusable browser cards with page-scoped admission.
The native clear button is reused too. CBC's fifteen focused tests/build pass;
rendered-search and in-place edit verification are in progress.

Full exact-source scoped app gates passed with no worker diff at
`55e404c`, `2dd337a`, `9353aaf` and `49d9f6a` (jobs
`20261107-053534-6bd3db64-a1`, `20261107-054646-be78c922-a1`,
`20261107-055202-a9165bdb-a1`, `20261107-055711-a793ea55-a1`). Complete
saved logs were read. Subsequent repair jobs `20261107-055936-61cb9c95-a1`,
`20261107-060927-c44a0f83-a1`, `20261107-061124-012794a9-a1` and
`20261107-061354-a8769509-a1` passed fifteen tests/build and returned
formatting, now applied at code head
`ff82bf2165570dc896b08629aa1d5650a4bd23a4`.

Overall remains **OPEN**. Beyond final live/authority verification and a fresh
exact-source full gate, remaining acceptance includes native search edits,
drawers/menus/keyboard callbacks, populated Software/Email, macro mutations,
topics/properties and CLI/single-instance behaviour. Baseline page acceptance
does not claim those capabilities. No Fable deviation is proposed.
branch lab/parity-prefs; head SHA ff82bf2165570dc896b08629aa1d5650a4bd23a4 (code before receipt); pages 42/42 at 9353aaf, full/header/sidebar/content SSIM and RMSE 0, figures pair.nIbtod/diff-<page>; functional 40/40 at 49d9f6a; OPEN: final live/authority visuals, search edits and full interaction/mutation/topic/populated-service acceptance, final exact-source gate; CBC job ids 20261107-055202-a9165bdb-a1 and 20261107-055711-a793ea55-a1 full PASS, 20261107-061354-a8769509-a1 repair PASS with formatting applied.

## Prefs round four: live response accepted, native interactions progressing

All **17/17 live Prefs variants** are accepted at tested source
`d23465bbee2436cb538f116f0e146f2601cc8dc3`, candidate SHA256
`fc09a098797c50a5d28c3c68f10419b500a8ee81f5b2b1f2c7d7155d2a88d8ec`.
Pinned gold revision/hash and the isolated 1040×680, scale-1 native recipe
remain as above. `prefs-parity-20261107/pair.UWEu2H/live-acceptance.json`
records both source/binary identities, accepted revisions, unchanged PIDs,
captures, regional figures, raw metrics, tool version and all seven stopped
processes. The invalid tune request returns gold's exact validation refusal.

Sixteen variants are pixel-identical in all regions. Spacious has only the
inspected 16×13 Macros-icon anti-aliasing residue: full SSIM distortion
0.0000928134, RMSE 0.00306816; header (0,0,1040,46) SSIM 0.00137202,
RMSE 0.0117965; sidebar/content both zero. Native bounds differ by 1/32
logical point. The parity method explicitly permits anti-aliasing/hinting;
this is accepted on inspection, without an arbitrary metric threshold.
Compact's correct header crop is (0,0,1040,33); all its metrics are zero.

**Settings authority absence and recovery are pixel-identical**, including
header, sidebar and content. Both preserve the accepted role/stale/revision,
return the same valid queued-write reply, retain the error notice and settle
the write after restart without restarting Prefs. Both original pairs and
the new `acceptance-diff-authority-*` figures were inspected. The lost-session
frame-retention repair is verified. No Fable deviation is proposed.

Native search clicks now focus gold's field and editing the live Compact
result settles in settingsd while the open page stays unchanged
(`pair.zhBDtX`). A candidate-only debug-build “Unaligned” annotation was
identified; normal native clients now suppress egui's development overlay,
with geometry preserved. A fresh native interaction run is pending.

All four native menu popups and the new-account form are now pixel-identical
in `pair.eVIWoy`, tested `82a1c159f26b570cfc0db4a234c3f9a3ffbc85fc`, binary
SHA256 `3fc976e5f76cf4d025e8f45993b1c685df6a7385d1e4940924180768aca892f7`.
The original/side-by-side pairs were inspected. Live shortcut bindings use
the native conflict resolver and menu formatter. Populated Software's five
columns/release notes and Email's account pane now render through reusable
widgets; their remaining viewport/separator differences are being closed.
Successful release arrays had exposed a Mix scalar-field assumption; it is
fixed. The private native release fixture is deterministic evidence, not a
claim that the real service was tested (no releases binary is installed).

A Mix launcher now provides page/row deep links, native Bus/service/compositor
options and reuse of the existing window. The generic host keeps the app's
command namespace separate from an overridden service identity. Native first
and second launch checks remain pending. Macro mutation probes follow gold's
per-app/global namespace rules; moving to global removes it from Prefs' own
catalogue and belongs to Workshop thereafter.

CBC `20261107-062527-a1aac21b-a1` passed 15 tests/build with formatting
applied. `20261107-063252-b15af459-a1` caught a missing menu-helper argument;
`20261107-063430-56c4de16-a1` caught an egui API call; both were fixed without
test changes. `20261107-063654-21e3e59d-a1` and
`20261107-064033-96555226-a1` passed all **17** focused tests/build, with
formatting applied. Complete saved logs read. Repair job
`20261107-064358-480a8017` tests code `5329cebc40d9a4eca47427dd4793e604b2ca025f`.

Overall **OPEN**: fresh final baseline/gate, populated-service visuals and
mutations, drawers/pickers, remaining native callbacks/topics and launcher
acceptance. Baseline acceptance stays 42/42; the direct functional/refusal
matrix stays 40/40 (only sequential accepted revisions are normalised after
mutation witnesses). No inventory disagreement or proposed deviation blocks
the work.
branch lab/parity-prefs; head SHA 5329cebc40d9a4eca47427dd4793e604b2ca025f (code before receipt); pages 42/42, full/header/sidebar/content SSIM and RMSE 0 at pair.nIbtod; functional 40/40; live 17/17 accepted (16 exact, Spacious AA), authority absence/recovery exact; OPEN: populated/native interaction/CLI acceptance and final gate; CBC job ids 20261107-063654-21e3e59d-a1 PASS, 20261107-064033-96555226-a1 PASS, 20261107-064358-480a8017 pending.
# Prefs round five: drawers closed, release acceptance pending

Code `876f50a85bcb3f299558425969596dc9f4a63d20` fixes the native drawer
separator inset, scroll fade background and catalogue list scrolling beneath
its fixed search. `pair.7xWNxT` accepts all ten drawers with full/header/
sidebar/content SSIM and RMSE zero; paired originals/contact sheet inspected,
all owned PIDs stopped. `pair.29fCaL` confirms native catalogue font writes
through private settingsd and restores them. Catalogue figures are being
reviewed. `pair.6Uttb8` repeats 42/42 baseline pages at zero distortion and
40/40 functional/refusal cases, plus identical canonical HELP/menu queries.

Heavy CBC `20261107-073545-70c5676f` is queued for the exact code, command
`mix apps/prefs/gate.mix --release`. Scoped CBC `20261107-073405-f2ebf754`
passed 17 tests/build; formatting applied. Overall OPEN until the heavy gate
and release-byte capture/functional/live-Prefs suite complete. No Fable
deviation is proposed. This receipt commit changes documentation only.

branch lab/parity-prefs; head/code 876f50a85bcb3f299558425969596dc9f4a63d20; OPEN: heavy release gate and release-byte evidence; pages 42/42; functional 40/40; CBC 20261107-073405-f2ebf754, 20261107-073545-70c5676f (queued)

## Prefs round five: release witness and native search correction

Exact source `f015520ceca5449a60b35c77284cca750c53993a` passed heavy CBC
`20261107-074849-cf5ada53-a1` on cbc2, command `mix apps/prefs/gate.mix
--release`, with no worker diff. The complete saved log was read. Release
SHA256 is `877737ce2fcaf015206f04df9ea59b311abcc5ea3e5ea6e95db757a0d4e21d73`.
Its nine private release runs and 91 regional comparison sets are retained
under `prefs-parity-20261107/round5-f015520-receipt` and their original run
directories. All 42 baseline pages are pixel-identical, as are ten drawers,
catalogues, the native chooser, populated Software/Email and dialogs.
Sixty-seven structured reply comparisons pass; native settingsd, Email and
the installed Mix Releases citizen are exercised in private roots. The
Releases witness verifies a private package, install, rollback refusal and
removal. This complements the fully logged deterministic service fixture.

That release is not accepted: the search capture exposed a real caret reset,
not merely a blink. Native `den` then `sity` produced gold `density` and
candidate `sityden` in `pair.qqHr5f`. The shared browser now retains pending
text until owner acknowledgement, consumes repeated values in order, and
discards a draft on explicit navigation. The private reset token is absent
from the public Prefs state. Native repaired witness `pair.MTCmDB` produces
`density` on both sides. CBC `20261107-081753-981fcde5` passed 18 focused
checks/build; its formatting patch is applied. The regression includes
delayed/partial replies, edit/undo cycles and numeric reset tokens.

Code `249f9a3483b13a9b9ac0d26b878942d5d78191b1` is queued for heavy
release gate `20261107-082023-f8cb0b05`. Acceptance remains OPEN until that
gate and the ten-group release-byte rerun, including native continued
typing, finish. Spacious retains only the previously inspected Macros icon
anti-aliasing edge. No deviation from gold or Fable decision is proposed.
branch lab/parity-prefs; head/code 249f9a3483b13a9b9ac0d26b878942d5d78191b1; OPEN: final search release gate and release-byte rerun; CBC 20261107-074849-cf5ada53-a1 (pass), 20261107-081753-981fcde5 (18 checks/build pass, formatting applied), 20261107-082023-f8cb0b05 (queued)


## Prefs round five: final acceptance

Exact code `249f9a3483b13a9b9ac0d26b878942d5d78191b1` and release SHA256
`eb6145fe3b07d8c81eb83c7bc445f0b8a0d7e66599732550d29464713e834a16`
are ACCEPTED against pinned gold `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Heavy CBC `20261107-082023-f8cb0b05-a1` passed `mix apps/prefs/gate.mix
--release` with no worker diff; the complete saved log was read.

Final ten-group release suite: 42/42 baseline pages and all ten drawers
are pixel-identical in full/header/sidebar/content; 91 paired capture sets,
90 entirely identical. All 17 live variants are accepted on unchanged PIDs.
Spacious retains only the inspected 16×13 Macros icon anti-aliasing edge
(full SSIM distortion 0.0000928134, header 0.00137202; sidebar/content zero).
Catalogue/chooser, populated services, menus/dialogs, search and authority
loss/recovery match. All 61 owned process records confirm exit.

The 40/40 functional/refusal matrix and 69/69 extended structured comparisons
pass, including exactly 55 HELP commands, canonical menus, rebound actions,
macro lifecycle/topics, actual private Email and Releases operations, CLI
reuse and native continued typing. The final typing witness produces
`density` from `den` then `sity` on both sides; the failed earlier release
is retained. No deviation, Fable question or open item remains.

Full acceptance record: `docs/ports/prefs.md`; private
`prefs-parity-20261107/release-acceptance.json` indexes all final runs,
source/binary and fixture/service hashes, readbacks, figures, raw metrics,
CBC log and cleanup. This checkpoint changes documentation only; the tested
code SHA is separate from the receipt commit.

branch lab/parity-prefs; head/code 249f9a3483b13a9b9ac0d26b878942d5d78191b1; ACCEPTED; pages 42/42; functional 40/40 (extended 69/69); CBC 20261107-082023-f8cb0b05-a1 (pass), 20261107-074849-cf5ada53-a1 (earlier release pass), 20261107-081753-981fcde5 (18 repair checks/build pass)
