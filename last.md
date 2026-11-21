# PR #220 round 3c — a+ implementation stopped on host startup failure

Tested implementation head: `a698636482f56c0531f7cf5f3e60fc2e948e6da1`.
This receipt commit also applies the final worker rustfmt diff; it makes no
semantic repair after the failure. #220 remains draft and is **not ready**.

Fable's correction is implemented at the loader: startup still logs a bad
requested context and falls back to embedded Studio/Dark. Fatal versus formal
fallback is recorded separately in `~/.viu/_doc/TODO-viu.md`, citing CBC
`20261121-184422-5bc4c8cf-a1`. The generic editor save policy remains out of
scope: `~/.viu/_doc/TODO-editd.md`, “Generic saves of theme and scene sources
skip design validation”. Neither earlier ownership stop is reopened.

## Red result and stop

CBC **`20261121-192227-2f615926-a1`**, source `a6986364`, BUILD_RC=101,
runs the new real-host controls. Both fail during startup, before their
surfacing assertions, with **`invalid type: null, expected a map`**:

- `startup_fallback_surfaces_in_app_info_and_painted_notice`
- `generic_save_gets_late_report_and_bad_context_switch_is_refused`

The source trace points to the new pre-use validation at
`libraries/viu/src/app_host.rs:1327`: when the initial model supplies a theme
choice, it calls `appearance::session_base` as soon as a session object exists.
The session's initial snapshot is `{}` (`libraries/viu/src/session.rs:136`);
`session_base` tries to decode its absent values as a map. The new validation
therefore runs before the session snapshot is usable. This is a product-path
failure in this implementation, not a reason to weaken the controls or change
the startup fallback policy. Per the stop rule, no semantic fix was made.

Saved log: `20261121-192227-cbc2-a698636482f5-20261121-192227-2f615926-a1-f13befb6.log`.
The whole log was read. Rust tests finish red in 0.28 s; the 750 ms latency test
is not reached. No after latency or successful app startup is claimed.

## Implemented, but not yet accepted

| Area / write path | Implementation and current verification |
|---|---|
| Synchronous requested context | `libraries/design/src/compiler.rs:27` adds `compile_design_context`; the public `compile_design` keeps whole-source semantics. Runtime theme/style projection uses the requested entry point. The new compiler control passes. |
| Startup fallback | `libraries/theme/src/theme.rs:310` retains logged fallback plus source-addressed diagnostics; the loader control passes. The new host info/painted-notice control is red during startup as above. |
| Context switch | `Theme::with_choice` validates a new context before returning an artifact; file-backed choices read the current source snapshot. The host pre-use check is the failing path above. Context refusal, return code and painted notice are not yet verified. |
| Background report and hand edits | `libraries/viu/src/design_validation.rs` starts one native worker after a ready frame, checks source snapshots and file-byte changes, and reports without restyling. The generic-save control uses the actual editd router/writer; it is blocked by host startup before reaching its save assertions. |
| Info and notice | `app_host.rs:1548` amends the app info reply; the host owns a dismissible rendered notice. These remain unverified end to end. |
| `appearance.source` | The existing registry whole-source check is retained. `services/settingsd/tests/transactions.rs:44` adds a refusal-before-profile-write control and valid-source control; not run yet. |
| Scene install / upgrade / Reset | The existing stage → `shell.scene.validate` → placement sequence remains. `engines/viu/src/store.rs:497` adds whole-source desk-theme admission to that validator. New writer refusal checks at `engines/viud/tests/scenes_test.mix:518` cover all three operations; not run yet. Keep performs no write. |
| Bundle | `tools/apps_bundle.mix:39` calls the headless `viu --validate-design --embedded` path before admission and checks its source SHA. Bundle/env gate modes build the validator. No bundle admission run or negative control is claimed. |
| Cache identities | Bounded whole-source admission cache uses authored-content digests. Background reports identify parsed content or file bytes, not paths/context names alone. First-use artifacts are synchronously validated. |

The new timing contract is documented in `docs/design-validation.md`, linked
from the scene guide. These are implementation changes on a draft branch,
not a claim of completed or accepted behaviour. In particular, negative-control
mutation runs and final resource/backlog review remain outstanding.

## Checks and measurements

| CBC job | Result |
|---|---|
| `20261121-190812-4795805d-a1`, `6f73c199` | `cargo check -p lib-viu --features test-support --all-targets` passes; worker fmt/lock diff applied locally. |
| `20261121-191937-0e1886b8-a1`, `f635410f` | Requested-context compiler control passes (0.72 s); loader fallback control passes (0.23 s). New host fixture does not compile because the toolkit wrapper lacked `load_from`. That adapter mistake is corrected in `a6986364`; not a product runtime result. |
| `20261121-192227-2f615926-a1`, `a6986364` | Host controls compile and both fail during startup as above. Worker fmt succeeds; its remaining three-line diff is applied with this receipt. |

All changed Mix files (`tools/apps_bundle.mix`, `tools/cutover_gate.mix`,
`engines/viud/tests/scenes_test.mix`) lint with zero diagnostics. No new `MIX_*`
variable. The host fixture uses the explicit-path loader; the earlier probe
used registered `MIX_ETC`. Generic writes run through editd without a new
transport. `git diff --check` passes. No full CI run,
all-target clippy, complete changed-crate test suites, width-eight pool, env
registry or negative-control run is claimed after this stop.

Before remains CBC `20261121-175445-2c24d7dc-a1`: two one-second timeouts.
Round-2 lazy prototype worst calls remain 616.096–627.692 ms (five runs, CBC
`20261121-175911-93071cbf-a1`). **Round-3c after is unavailable** because the
host controls are red; the 750 ms bound is unchanged. The separate residual
roughly 620 ms follow-up and callback profile remain in TODO-workshop.

Desk n=0. The implementation never reached the all-green prerequisite, so no
desk receipt eligibility check, quiet run or probe was started in this round.
No deployment or merge occurred.

# PR #220 round 3b — product failure in the startup theme loader

Source head: `7b2ea1f6edf4396fdb460d22998d9fe30015306d`. This round's
public change is this receipt only; #220 stays draft. Generic Ced/editd saves
are the authorised hand-edit bypass, covered by the planned background report.
No Ced/editd save policy or filename heuristic is needed in this PR. The
separate ownership follow-up is `~/.viu/_doc/TODO-editd.md`, **“Generic saves
of theme and scene sources skip design validation”**. This supersedes the
previous round's ownership stop.

Before changing runtime semantics, the mandatory bad-requested-startup control
found an existing product failure. CBC **`20261121-184422-5bc4c8cf-a1`**,
source above, BUILD_RC=1, ran the real native `theme` loader with an isolated
configuration root, using the existing registered `MIX_ETC` variable:

| Case / run timestamp (2026-11-21, UTC+10) | Result |
|---|---|
| Valid source control, 19:00:04.719357795 | Parses, requested Ocean/Light compiles, `Theme::read` succeeds, `Theme::load` returns Ocean/Light. This proves the fixture root is read and the default is not returned unconditionally. |
| Bad requested context, 19:00:06.574789891 | Source still parses. The existing Ocean/Light block references `missing.round3b.requested` in its base foreground. Requested-context compilation is fatal and `Theme::read` rejects it. **`Theme::load` logs the rejection and returns embedded Studio/Dark instead of failing startup.** |

The fallback is explicit at `libraries/theme/src/theme.rs:297`–`:308`.
`libraries/viu/src/app_host.rs:2431` calls it during host construction;
`:1609` and `:2016` also use it for later reloads. The probe exercises that
loader directly, not an entire rendered app or a desk. The startup caller
trace establishes its relevance to the fatal-startup requirement.

Per the unchanged stop rule, work stops on this product failure. No fallback
repair, lazy compiler, background worker, bundle gate or new runtime contract
was landed. The 750 ms bound and product deadlines remain unchanged. No new
startup latency after measurement is claimed; the round-2 prototype numbers
below remain the only lazy results. No scoped green suite is claimed.

Private reproducible probe: evidence directory
`2026-11-21-workshop-startup/round3b/startup-control.mix`. Saved CBC logs were
read. Two own-fixture attempts precede the product result: job
`20261121-184141-79d550ac-a1` failed before Rust ran because the temporary
examples directory was absent; job `20261121-184303-8e9b9873-a1` reached its
valid control, then rejected a duplicate `semantics` field in our bad fixture
(BUILD_RC=101). Both mistakes were corrected; neither is product evidence.
A queued cancellation of the second attempt acknowledged `state=queued` but
the attempt later ran; the existing buildd cancellation audit remains open.

No temporary Rust source was retained in the public tree. The private Mix
driver lints cleanly, and `git diff --check` passes. Full fmt/clippy/test pools
and env-registry are not rerun after this stop: no production code or env
registry changed. Desk n=0; the everything-else-green condition was not reached,
so no desk receipt poll or quiet/probe run was started in round 3b.

# PR #220 round 3 — stopped at the desk write-path survey

Code head remains `b098cb8cef32f5978b9a2b2868fac570599a723c`; this round
changes this receipt only. [PR #220](https://github.com/mixviu/mixviu/pull/220)
remains draft. Fable's a+ ruling is accepted, but not implemented: the required
write-path survey found a boundary needing an additional ownership decision.
No runtime validation was weakened before that boundary was resolved.

## Write paths inspected

| Path | Current owner and validation | Required work / disposition |
|---|---|---|
| Prefs appearance save | `apps/prefs/look.mix:121` sends `settings.apply`; `services/settingsd/src/authority.rs:180` uses `registry::apply` before the commit at line 336. `libraries/settings/src/registry/kind.rs:133` checks a source through `libraries/settings/src/resolve.rs:62`, which calls the all-context compiler at line 65. | Retain this whole-source check when adding a separate runtime per-context entry point. |
| Scene install, upgrade and Reset | `engines/viud/scripts/lib/scenes.mix:1433` / `:1444` stage a copy; `:1457` calls `shell.scene.validate`; upgrade invokes Reset at `:2838`; Reset stages at `:2884`, before the swap at `:2926`. | The host validator at `engines/viu/src/store.rs:496` parses/resolves the scene and validates templates. It does not call the design compiler. Staging writes a private temporary copy before this check; installation placement follows validation. |
| Keep | `docs/scene-documents.md:56` describes the edited-copy policy; line 59 explicitly says keeping requires no action. | There is no Keep write transaction to insert a check into. Do not invent one. |
| Scene move | `engines/viud/scripts/lib/scenes.mix:2596` writes the rewritten scene before reconciliation at line 2597, with rollback at line 2607. | Also needs a check before the write under the proposed write-time rule. |
| Scene Editor's Edit action, including edits to a kept copy | `share/scenes/editor/lib.mix:487` delegates to `plan_edit`; `:539` routes the file to Ced. `apps/ced/actions.mix:77` sends the generic document save (`:65` for Save All); `libraries/viu/src/documents.rs:560` emits `ServerOp::Save`, which becomes `edit.save` in `libraries/editor-model/src/mirror.rs:1635`. | **Stop boundary:** the actual writer is editd, not the scene loader or the design compiler. |
| Generic editor write / Save As | `services/editd/src/actor.rs:1301` owns save. At `:1327` it serialises the buffer, then calls `files::save` at `:1333` or `:1380`. `services/editd/src/bus.rs:19` declares the save arguments; there is no source-validation policy or scene/design authority in that request. | Refusing invalid themes/scenes here needs a defined protected-file classification and validation owner. A Ced-only check would leave direct `edit.save` and Save As uncovered. |
| Build/install bundle | `tools/apps_bundle.mix:88` copies shipped scenes into the bundle; its existing checks cover assets, environment and roots. | Add an explicit whole-source design check before admitting the bundle when this work resumes. No such check was added in this stopped round. |

These are inspected paths, not a completed claim that every desk writer is
covered. The survey stopped before implementation, as required by the brief.

## Decision needed

The a+ ruling specifies validation timing and context refusal. It does not
specify how the generic text service identifies a protected live theme/scene,
or which source validates a scene's inherited design references. Scene files
and design-source documents have different grammars; calling the design
compiler on every saved scene is not a valid implementation. Choosing a
filename heuristic, adding a new save policy to editd, or enforcing only in
Ced would each make a new ownership/scope decision. Resolve that boundary
before changing runtime acceptance. This is a source-survey finding, not a
newly executed product-failure test.

## Evidence and checks

No new CBC jobs, performance claims, compiler changes, tests or environment
variables in round 3. Round-2 before remains red: CBC
`20261121-175445-2c24d7dc-a1`, two one-second timeouts. The private lazy
prototype remains the only after evidence: CBC `20261121-175911-93071cbf-a1`,
five fresh runs, worst calls 616.096–627.692 ms. It is not landed a+ evidence.
The 750 ms regression bound and all product deadlines are unchanged.

The residual roughly 620 ms startup call is recorded as its own private
Workshop follow-up, with the existing callback profile and the remaining
critical-path attribution gap. Rust and Mix checks are not rerun for this
documentation-only stop; `git diff --check` is the scoped check. Desk n=0:
the round-3 condition to attempt baseline samples (everything else green)
was not reached, so the receipt gate was not re-polled and no desk probe ran.

# PR #220 round 2 — port-time regression and appearance policy prototypes

Draft [PR #220](https://github.com/mixviu/mixviu/pull/220).
Measured test checkpoint `6f181ca6`; final fixture `8b59d53a`.
Production compiler policy remains unchanged.
Only the regression test and this receipt land in round 2. The three compiler
variants are private, temporary CBC prototypes, awaiting a ruling.

## A regression that includes startup

`libraries/viu/tests/workshop_startup.rs:17` registers 200 peers and starts an
independent Bus client before constructing the native host. It detects the
Workshop port with the real service-list API (line 63), then alternates ping
and info calls whose starts fall within the next 1.5 s (line 69). It does not
wait for `host.ready()`. The native host pumps and draws its first ready frame;
the first pump's CPU and wall are recorded separately (line 91).

The bound is 750,000 us (line 14), leaving 250 ms below the unchanged one-second
call deadline. No retry or widened call deadline is used; failed calls fail
the regression too. The prototypes' slowest successful call is 630,427 us,
leaving 119,573 us (19%) below the selected bound. The bound was set before
those measurements.

**Expected before red:** CBC `20261121-175445-2c24d7dc-a1`, test source
`1bd4f587`, worst observed call 1,001,520 us, two calls timed out, test wall
3.40 s. The first pump used 2,501,943 us CPU / 2,502,676 us wall. Failed call
durations are censored at the unchanged timeout; they are not completed reply
latencies. This is the actual product path, and serves as the unchanged-source
negative control against the fast prototypes.

Correction to round 1: its 200-peer fixture followed a four-peer case in the
same process. Its 1.30–1.31 s first pump was not fully cold. Round 2 uses one
200-peer case per fresh test process. Each prototype sample launches a new
test executable; compilation time is outside the reported test measurement.
The renderer uses the real native headless frame path, not physical presentation.

## Five fresh runs of each option

All samples: CBC `20261121-175911-93071cbf-a1`, same worker and checkpoint.
Times below are milliseconds. Calls are initiated during the first 1.5 s
from port appearance; first-pump time is a different interval and is not
added to call latency. The last column includes timeouts for (c), marked †.

| Option / sample | First pump CPU | First pump wall | Worst call |
|---|---:|---:|---:|
| a / 1 | 336.804 | 336.947 | 619.661 |
| a / 2 | 341.398 | 342.041 | 621.754 |
| a / 3 | 337.977 | 339.070 | 627.692 |
| a / 4 | 336.803 | 336.938 | 618.903 |
| a / 5 | 335.289 | 335.367 | 616.096 |
| b / 1 | 339.076 | 339.592 | 622.505 |
| b / 2 | 342.590 | 344.567 | 630.427 |
| b / 3 | 339.346 | 339.414 | 621.980 |
| b / 4 | 340.605 | 340.652 | 622.490 |
| b / 5 | 339.788 | 339.819 | 622.736 |
| c / 1 | 2531.990 | 2532.812 | 1001.450† |
| c / 2 | 2520.937 | 2521.848 | 1001.436† |
| c / 3 | 2521.550 | 2522.322 | 1001.550† |
| c / 4 | 2591.881 | 2594.974 | 1001.567† |
| c / 5 | 2527.401 | 2528.304 | 1001.436† |

(a) and (b) have no failed calls. Every (c) sample has two timeouts; † is a
lower-bound observation, not a successful completed-call maximum. There is
no responsiveness pass for (c). Whole fixture work is 1.775–1.857 s for (a),
4.508–4.570 s for (b), and 4.170–4.280 s for (c); process/test teardown adds
about 0.05 s. No new or widened app `*-test.mix` file is introduced.

### a — lazy context compilation

Anchor: `libraries/design/src/compiler.rs:63`. Private snapshot
`compiler-lazy.rs:63` substitutes the requested context for the enumeration
of all reachable contexts. Structural validation, v0 equivalence and the full
requested-context compile remain synchronous. Other contexts compile and
validate on their first use. This is a policy change, not a production fix.

An invalid requested context returns Fatal before its artifact can be used.
A resolved error confined to another context is postponed until that context
is requested. Current-context values are validated before rendering, but the
source has not passed today's all-context acceptance contract when the first
frame is prepared.

### b — full validation after the first native frame, on a worker

Anchors: `libraries/design/src/compiler.rs:17`,
`libraries/viu/tests/workshop_startup.rs:98`. Private snapshot
`compiler-split.rs:3910` compiles the requested context synchronously;
`:3980` starts one native worker only after `host.draw()` returns. Full
validation jobs then run sequentially on that worker. The fixture joins the
worker after collecting startup calls (`:3995`), so late failures cannot be
lost when the process exits. All six full-validation jobs in each sample
succeeded; each job took roughly 0.60–0.64 s wall.

The initial resolve was already on the native host thread, not the Mix
evaluator thread. This prototype removes the host's blocking full compile
from the first frame's critical path. It still validates each requested
context synchronously before use. Cross-context source errors can be found
after that frame. The lab reports them as `SPLIT_LATE_ERROR`; it has no
rollback, quarantine or live-owner refusal protocol. Thus a frame from a
source later rejected globally can already exist. Choosing late-error
behaviour needs a ruling and a design; none is silently supplied here.

### c — share colour-primitive conversion, keep all-context validation

Anchor: `libraries/design/src/colour.rs:539`; private snapshot
`colour-shared.rs:1290`. `compile_primitives` reads only the authored colour
map. The prototype reuses the converted map and its identical diagnostics
for exact-equal input maps across contexts, in a bounded 128-entry thread
cache. It does not reuse whole context results, omit any context, or delay
validation. All current full-source checks still precede artifact return.

All 337 design tests passed, zero failed, eight existing ignores, in the
same CBC job. This bounded sharing is not enough: startup remains red.
It does not establish that every possible semantics-preserving compiler
optimisation is exhausted. A broader factoring of context compilation is
further design work, not implemented in this round.

## Error timing control, checks and desk gate

CBC `20261121-181540-5864df8a-a1` injects a missing foreground reference
only into the existing Studio/light modifier, leaving active Studio/dark
and the Ocean/light v0-equivalence context valid. Baseline and (c) reject
the source synchronously even when dark is requested. (a) and (b) accept
dark, and reject light when light is requested. The after-frame worker in (b)
then reports full_ok=false / SPLIT_LATE_ERROR (578,224 us wall in this control).
This pins the changed error timing; it does not supply a late-error recovery
protocol. All four controls pass their intended assertions.

Two own control-fixture failures are retained: `20261121-180429-2f5325c9-a1`
used a mode-only fault that also broke v0 equivalence;
`20261121-181333-2938d58f-a1` then created a duplicate Studio/light block.
The final fixture edits that existing block. A queued cancellation was
requested for the first job, but its attempt still ran; its red result is
retained. No product repair was made in response to either fixture failure.

Scoped CBC `20261121-180106-e09c5cb2-a1` covers fmt, all-target clippy and all 55
lib-viu test binaries plus doctests in both configurations. Default is green.
Both width-eight app_mix runs pass. Test-support is red only in the new
startup test: my five-second whole-fixture assertion fired under pooled
contention before printing its latency result (5.84 s). This was my fixture
error, corrected in `8b59d53a`; the standalone budget was not a universal
loaded-run deadline. No product deadline or numeric latency bound changed.

Final CBC `20261121-181507-9c287e33-a1`: fmt and default/support all-target
clippy pass; all test-support targets and doctests are rerun, with app_mix
green in the strict width-eight pool. Only workshop_startup remains red, now
at the intended 750 ms call assertion. Its final standalone test wall is
4.21 s (reported whole work 4.149 s); first pump 2.488 s wall / 2.487 s CPU,
worst observed call 1.002 s, two timeouts. Its pooled work is 6.425 s, reported
without the inappropriate whole-fixture guard. The full test-support suite
is deliberately **not green** while the startup defect remains. The receipt
driver recognises the expected before-red; it does not ignore or waive the
test. Workshop serial policy is unchanged.

Env registry: 277 names, 0 problems. No new `MIX_*` variable. Changed and private
Mix tools lint cleanly; literal worker-source heredocs have a documented
non-interpolation lint pragma. No tools/ci.mix all or local Rust execution.


The private desk runner now accepts `desk7.check_rc` 0 or 2 and rejects any
`desk7.after` line starting with `FAIL`. It retains the take-window exclusion,
quiet check, receipt reread and SHA/quiet/probe evidence per run.

At **2026-11-21 18:24:31 AEST**, after the measurements and error controls,
the required `7ad2570fc71e3d7035fa642623578ac0570791c2` receipt still did not
exist. The corrected gate was closed. Desk samples: **n=0, not run**. No
quiet check or remote probe was started against an unverified build. Per
the stop rule, no waiting/polling for gold follows this check; desk numbers
and quiet JSON do not exist for this round.

No compiler policy has landed, no PR merge or deployment occurred, and #220
stays draft. A ruling is needed before deriving (a), defining (b)'s late-error
behaviour, or pursuing a broader (c) redesign.

---

# PR #220 — Workshop startup investigation (draft)

[PR #220](https://github.com/mixviu/mixviu/pull/220), branch
`fix/workshop-startup-spike`, starts from #216's merged `7ad2570f`.
Implementation checkpoint: `1b42391d`; fixture follow-up: `d2150068`.
This is incomplete: no startup repair or desk improvement is claimed.

## Measurements before product changes

Untouched main, CBC `20261121-172151-2c445536-a1`, passes the real 200-peer
fixture. Across discovery, worst ping / info are 289,758 / 351,010 us;
load callback CPU is 80,697 us, initial pump CPU / wall 1,265,019 /
1,265,768 us. These cover different intervals, not a summed critical path.
The fixture starts its first Bus call only after `host.ready()`, so it excludes
the initial native hold. The requested regression must start at port appearance
and measure the first 1.5 s; the existing passing calls cannot prove that bound.

CBC `20261121-172329-1a2e0482-a1` runs five fresh fixture processes with
section-dirty and five with full projection, on the same worker and source.
Temporary native tracing records callback sequence, command, CPU/wall,
full/patch bytes and in-flight sequence ids. Publication tracing records
presentation/frame CPU, full versus patch and bytes. Discovery completion
traces record operation, queued/active jobs and service count.

The first full model is **24,182 bytes** in every 200-peer sample. Its ACK
(sequence 1) precedes the first notice (sequence 2). That notice takes
653,826–661,497 us wall across the dirty samples, with 11,116–18,021 us CPU.
The theme callback (sequence 3) publishes a 969-byte patch; the first ping
(sequence 7) follows those callbacks. This fixture already has a small,
acknowledged first model; a smaller-first-model fix is not supported here.

The table is each sample's **whole-discovery** worst call, not a first-1.5 s
measurement. Instrumentation adds work and affects scheduling; the populations
and callback counts differ. It does not establish a speedup or regression.

| Sample timestamp (2026-11-21, UTC+10) | Projection | Ping max us | Info max us |
|---|---|---:|---:|
| 17:23:31 | dirty | 304190 | 368136 |
| 17:24:08 | dirty | 357205 | 264461 |
| 17:24:29 | dirty | 299501 | 123959 |
| 17:24:51 | dirty | 327311 | 401178 |
| 17:25:13 | dirty | 292291 | 320937 |
| 17:25:34 | full | 443370 | 392593 |
| 17:25:59 | full | 389066 | 292550 |
| 17:26:19 | full | 312756 | 265406 |
| 17:26:42 | full | 299597 | 269342 |
| 17:27:04 | full | 225140 | 334908 |

Mount probe `20261121-172947-4f6d01e1-a1`: at 200 peers the application
scene parse/resolve/templates/controls/bindings take 3365/5171/368/673/300 us
CPU; the frame takes 287/1700/110/573/89 us. Native scene construction is
small compared with initial host retirement.

Host-phase probe `20261121-173133-0678f994-a1`: at 200 peers cumulative CPU
is 615801 us after appearance resolution, 632650 us after mounting,
632720 us after style installation and 632724 us after window sync.
The dominant initial native phase is appearance resolution. The compiler
validates all reachable design contexts; cache reuse is being prototyped
outside this branch. No compiler policy or scene contract was changed.
Cache prototype `20261121-173525-12520251-a1` (private lab only) passes five
fresh 200-peer processes, but initial pump wall remains 1,302,588–1,310,278 us.
Identical inputs hit the cache; other startup contexts each still compile for
about 0.6 s. It does not solve cold startup and is not derived into this PR.


## Accepted hardening implemented

- `tools/workshop-section-timing.mix:16`: all instrumentation rewrites refuse
  source drift. Line 30 uses a unique sibling temporary file, so a killed run
  cannot overwrite the tracked test.
- `apps/workshop/traffic.mix:38`: a five-second active deadline releases lost
  batches as dropped, preserves waiting rows and rotates the generation before
  another batch starts. Traffic Jobs themselves have a four-second timeout
  (`work.mix:20`). Paused active completions count every admitted row as dropped.
- `apps/workshop/controller.mix:192`: every native task completion is fenced
  before retirement. Unproven non-Traffic completion marks are conservative.
- `apps/workshop/app.mix:43`: simple Bus, viewport, theme, keymap, layout and
  connection events declare their writes. Complex execution/UI/restore paths
  remain conservative; the all-event narrowing requested by the brief is open.
- `apps/workshop/section-test.mix:37`: event steps mark only their writes and
  assert retained-section reuse. Additional input cases cover connected, busy,
  jobs, queue, dropped and pause. A source-mutated missing-filter dependency
  must disagree with the same full oracle.
- `redaction-budget-test.mix` covers pause during an active batch, live/expired
  deadlines, queue progress, late completions, nil queues and all-owner fencing.

## Validation and remaining work

| CBC job | Result |
|---|---|
| `20261121-172406-9af27c7b-a1` | Hardened timing tool passes, 3.173 s standalone. |
| `20261121-172631-ebf2365c-a1` | **Red**: my first expiry implementation read a nil queue. Fmt/clippy/app_mix passed; frame and Bus tests exposed the defect. Fixed by `1b42391d`, with a nil-queue regression. The original result is retained. |
| `20261121-173311-a62ec201-a1` | Corrected checkpoint passes six Bus tests, seven frame tests, real-fixture full-projection oracle and env registry (277 names, zero problems). Section 3.545 s and redaction 0.920 s standalone, below 5 s. |
| `20261121-173704-d78b968d-a1` | Final d2150068: fmt and all-target clippy in default/support, all54 lib-viu test binaries plus doctests in both configurations, strict width-eight pool with app_mix green; env registry277/0. Pool213/220s; no retry. |
| `20261121-174916-5f98f09d-a1` | Final standalone section3.566s / redaction0.926s, both below5s. |

No new `MIX_*` name. All changed Mix sources lint cleanly. No Rust builds ran
locally. No full workspace CI, merge, gold shipment or desk probe from this PR.
Product call/stall deadlines stay at 1 s / 2 s; serial enrolment is unchanged.

Historical desk before values remain 482/340 ms ping/info on `a3b1e286` and
537/422 ms on `6b4278c7` (one run each). Desk after: **not measured**.
The required `7ad2570f` gold receipt is absent at the latest check. Do not
probe a different build or roll the desk back. The startup repair, absolute
first-1.5-second trace and first-paint measurements, standalone latency bound
with negative control, revalidation of any later repair and five quiet desk samples
remain required before review. After a carrying gold ships, five more quiet desk
samples and the TODO-workshop before/after update remain required.

---

# PR #216 — round 3 app-test cost

PR: https://github.com/mixviu/mixviu/pull/216, ready as `app/mixviu-agent`.
Test source/checkpoint `29b85934`; product source remains `3674d978`.

The strict merge gate `20261121-155510-c630ee17-a1` correctly failed
`lib-viu/app_mix`: section-test exceeded its unchanged 20 s embedded deadline
under the eight-wide workspace pool. This receipt preserves that red result.
No gate retry, deadline increase, serial-list change or ignore was introduced.

## Measure, then fix

`tools/workshop-section-timing.mix:3` temporarily instruments the actual
section script, runs it through the real app_mix harness with the opt-in
native thread CPU probe, and restores it in finally. Wall time includes
runtime setup through completion; CPU/phase counters cover evaluated source.
The timing instrumentation itself adds a small cost; these are single samples.

| Metric | Before, `4c6b11d5` | After, `2dae960b` |
|---|---:|---:|
| CBC job | `20261121-161610-502de042-a1` | `20261121-161718-42bc8936-a1` |
| Real harness wall | 5.192 s | 3.208 s |
| Evaluated source wall / thread CPU | 5.186 / 5.184 s | 3.203 / 3.202 s |
| Full oracle calls / CPU | 155 / 1.994 s | 95 / 1.258 s |
| Dirty projection calls / CPU | 155 / 2.103 s | 95 / 1.292 s |
| Stored-model wraps / CPU | 299 / 0.844 s | 166 / 0.489 s |
| Seed rotations / transition steps | 8 / 80 | 2 / 20 |
| Nonempty fixture construction, wall / CPU | 1.124 / 1.122 ms | 1.096 / 1.094 ms |

`apps/workshop/section-test.mix:23` keeps two distinct seed rotations; each
still covers all ten event classes. The existing 35 nonempty input cases and
their reversions remain byte-for-byte unchanged, including every row-key form.
Pending-publication coalescing, clock advancement and forgotten-mark controls
remain. Every transition still compares the whole presentation to the full
oracle; no sampling or partial-section substitute was needed.
`section-test.mix:15` wraps a candidate once and reuses that exact stored model
after checking it against the wrapped oracle. All original assertions remain.

`libraries/viu/tests/app_mix.rs:61` records each real script's wall time, so
loaded results identify the slowest script rather than only the parent binary.
The optional CPU probe is enabled only in test-support and only for the named
timing environment variable; ordinary app tests retain the normal bindings.

## Three strict width-eight runs

All use the native `tools/ci/test_pool.mix --jobs 8`, selecting lib-viu with
test-support and all targets. App_mix runs in the pool; the unchanged serial
list leaves only workshop_bus alone. Retry controls are unset. Each run has
55 binaries, 658 passed, zero failed and 12 existing ignores; all 57 app
scripts and six Bus tests pass.

| CBC job | Section-test wall | Slowest app script |
|---|---:|---|
| `20261121-161808-4792e530-a1` | 7.343 s | monitor/core-test.mix, 7.658 s |
| `20261121-161823-feb766a1-a1` | 8.239 s | workshop/section-test.mix, 8.239 s |
| `20261121-161823-e8b3fae0-a1` | 9.273 s | workshop/section-test.mix, 9.273 s |

These script times include real runtime setup and completion. They exclude
child process spawn/teardown; the parent app_mix binaries took 45.1 / 41.0 /
44.6 s respectively. The original failure was a single script at 20.01 s,
not the parent binary total.

## Validation

| Check | CBC job / result |
|---|---|
| Worker fmt, diff applied locally | `20261121-161727-5432d71d-a1`; applied in `29b85934` |
| fmt check, all-target clippy, all 55 non-doc targets plus doctests, default | `20261121-162147-26fb7f5a-a1`: 648 passed, 0 failed, 12 existing ignored |
| Same scoped checks with test-support | `20261121-162147-54adf583-a1`: 658 passed, 0 failed, 12 existing ignored |
| All 35 external negative controls | `20261121-162147-18f14e5c-a1`: every intended assertion hit, outer rc 0 |
| Inline eager-redaction and forgotten-mark controls, all 57 app scripts | Pass in both feature configurations and all three loaded runs |
| Changed Mix lint | Both section-test.mix and workshop-section-timing.mix: zero errors, warnings or notes |
| Diff and base | git diff --check clean; main still `6b4278c7`, already an ancestor; no merge needed |

Clippy retains warnings in unchanged Rust files; none point at the changed
Rust harness. Every job above ran against `29b85934` except the worker fmt
and timing captures explicitly identified earlier. The final change after
that checkpoint is this receipt only.

All saved logs were read and audited. Initial timing-fixture mistakes were
repaired without touching product code: `20261121-161453-8fc12c80-a1` lacked
a separating source newline; `20261121-161532-8637ef11-a1` had not opted the
test-support runtime into its CPU probe. The two simultaneous identical pool
submissions `20261121-161808-52b7057a` and `20261121-161808-af5ec43e` returned
75 (not run), because buildd refused joining different required slots. They
were replaced with distinct Cargo argument orderings for independent runs.

No `tools/ci.mix all`, workspace gate, normal-pool Workshop soak, PR merge or
deployment in this round. The existing serial list is unchanged. These are
scoped lib-viu pool checks, not a replacement for the eight-wide workspace
merge gate; `pr_gate cbc` is the next full run.

---

# PR #216 — round 2 accepted review fixes

PR: https://github.com/mixviu/mixviu/pull/216 (`app/mixviu-agent`), branch
`feat/workshop-section-dirty`. Round-2 implementation: `3674d978`; complete
validation checkpoint: `503122da`; final fixture checkpoint: `e1717e5f`.
Only test orchestration/fixtures and documentation follow the implementation.
Green: fmt, all-target clippy, all 55 non-doc targets plus doctests in both
configurations, all 57 app scripts, six Workshop Bus tests and every negative
control. Main is still `6b4278c7`, already an ancestor; no merge was needed.

## Review response

| Finding / adjudication | Resolution and checks |
|---|---|
| 1 — Gemini3, oversized admission | `apps/workshop/traffic.mix:16`: active bytes plus incoming encoded row are checked before eviction. Impossible admission drops only the incoming row. Real 8 MiB, active 4 MiB plus incoming 5 MiB, and combined row/byte caps are tested; admission and active-count controls fail as intended. |
| 2 — Gemini4, upstream drops | `traffic.mix:9`: account upstream drops before pause/quit/rejection. Redacted `observed()` skips that accounting. Paused, evicted, rejected, failed and successful rows pin the exact count; upstream/double-count/rc/drop controls detect regressions. |
| 3 — Gemini1, reset correctness | `model.mix:14` clears raw work and creates a Traffic generation. `work.mix:4` carries it in the native owner tag, including admission failures. `controller.mix:195` checks the returned tag before deleting a task, protecting reused job ids. Nil/reset/new-active/stale-result and reused-id cases pass; generation/native-tag/task-tag controls fail. Pause and quit prevent launches; quit prevents ingestion. |
| 4 — Gemini2, malformed result | `traffic.mix:38`: guard a nil queue; reject non-map bodies, wrong batch lengths and non-map rows as counted batch drops. No partial malformed batch is retained. Malformed-body and isolated nil-queue controls pin both guards. |
| 5 — honest section cost | `tools/workshop-section-check.mix:59` starts the timer before `inputs()`/`marks()`. Baseline preparation consumed up to 23% of a clean-tree sample's net saving, so mutations now advance revisions. The publisher compares only the reset epoch and three counters. Dead agent input and full catalogue comparisons are gone. Mutation marking is measured separately below. |
| 6 — Gemini5/6, fixture gaps | `redaction-budget-test.mix` covers real8MiB admission, combined caps, batch bounds/FIFO, failure, pause/quit, reset and native tags. Only this script's embedded fixture receives a16MiB string allowance. `section-test.mix` wraps with frame, saved and app-mode projections (headless suppresses mode-file IO only), varies every declared cached input and command/macro/agent row keys, and coalesces multiple mutations before acknowledging the captured pending model. |
| 7 — Gemini8, format restore | Format restores only `libraries/viu`, the package it formats, rather than the whole tree. Worker formatting at the final implementation produced no diff. |
| 8 — GLM3, input closure | `presentation.mix:29` documents the transitive read closure and `dependencies()` names its fields. Mutation seams use that declaration to advance section revisions. `description()` reads command/snapshot/selection/tree, not `call`; call affects macro busy state. Height affects macro and freshly rebuilt placements, not retained Traffic widgets. Tests vary these inputs; removing `filter` from the closure fails the oracle. |
| 9 — GLM2, real Job contract | `workshop_frame.rs:11` runs the real script through native Jobs and controller completion. It verifies a returned native generation tag, decoded map rows, exact redaction and FIFO. Malformed-reply and reversed-batch controls fail. JSON string replies remain correct: runtime parsing is retained, not replaced by an unencoded map. DeepSeek's BLOCKING interpretation is rejected as adjudicated. |
| 10 — GLM3, throughput | One native Job takes up to64 rows /256KiB in order; one larger admissible row runs alone. Caps include the active batch and upstream accounting remains immediate. Before/after drop counts, backlog and notice-to-visible measurements are below. |
| Rejected — Gemini quit blocking | `model.busy()` does not count Traffic jobs. No quit-blocking or MEDIUM-security claim is made; only the accepted reset/admission/launch correctness changes were implemented. |
| Rejected — DeepSeek no-op/pending | Unreachable: `publish_workshop()` returns while a publication is pending, before computing a diff. Matching acceptance installs its captured model/inputs and flushes dirty state against that acknowledged model. The no-op branch was not changed. |

## Honest 200-peer measurements

Before: `d32f7d96`, CBC `20261121-145401-096d77e6-a1`.
After: `3674d978`, CBC `20261121-151832-376d4fdc-a1`.
Both run the real discovery/retained-body fixture with full-projection equality
and unchanged response, owner-stall, wire and native apply guards. No soak.

| Metric | Before | After |
|---|---:|---:|
| Inputs + marks, median / max | 0.865 /3.315ms | 0.087 /0.252ms |
| Complete preparation + section projection, median / max (14 samples each) | 55.377 /122.704ms | 37.544 /176.717ms |
| Full oracle projection, median / max | 61.492 /75.507ms | 63.303 /68.242ms |
| Observed notice-to-owner-accepted-visible-model latency, median /p95 /max | 6,292.6 /7,419.9 /7,459.0ms | 1,135.3 /1,987.3 /2,174.8ms |
| Visible observations /ingress at final sampled acceptance | 71 /149 | 122 /137 |
| Not yet observed visible at that cutoff | 78 | 15 |
| Raw waiting /active rows at that cutoff | 73 /1 | 0 /0 |
| Reported drops at that cutoff | 0 | 0 |

The after timer includes input preparation and counter comparison. Mutation-time
marking is outside that publication timer and is not free:828 calls in the
200-peer phase, median0.148ms, p950.414ms, max1.329ms, sum182.6ms. No aggregate
CPU-saving claim follows from moving that work to mutation time.

Four clean-tree after samples pair complete section cost28.833 /29.512 /30.818 /
32.277ms with full projection63.303 /63.592 /65.122 /65.111ms at the same state
and Mesh clock. The full oracle runs second and benefits from formatted Traffic
row cache warming. Cold rebuilds can be slower than the warm oracle, as the
176.717ms maximum shows. This is not a uniform or whole-tick speedup claim.

Latency measures ingestion to an accepted publication containing that row/body,
including correlated response bodies, not a frame physically drawn. Only the
last100 filtered Traffic rows are displayed. Unseen observations are censored;
they may remain queued or fall outside that window/the retained-model fixture
replacement. The populations differ because observations arise from live test
calls. The table therefore reports their counts and does not claim a matched
payload latency experiment, a zero-drop stress budget or normal-pool enrolment.
The after diagnostic's native patch CPU maximum is11,275us; wire max7,467 bytes.

## Validation receipts

| Check | Commit / CBC job | Result |
|---|---|---|
| Worker fmt, restored only the formatted package | `3674d978` / `20261121-151832-900e6eba-a1` | Pass, no diff |
| Real native Jobs redaction | `3674d978` / `20261121-151832-f66a2775-a1` | Pass |
| fmt check, all-target clippy, every test target and doctests, default | `503122da` / `20261121-152237-0590287a-a1` | 648 passed, 0 failed, 12 existing ignored; 55 non-doc targets plus doctests |
| Same checks with test-support | `503122da` / `20261121-152237-d7307e0f-a1` | 658 passed, 0 failed, 12 existing ignored; all six Workshop Bus tests pass |
| Original and round-2 controls | `503122da` / `20261121-152237-c0f95895-a1` | All 34 source mutations fail at their intended assertions; outer job rc 0 |
| Final queue controls, including isolated nil-queue | `e1717e5f` / `20261121-153138-a74e76dd-a1` | All 15 queue mutations hit intended assertions, outer rc 0; 35 distinct external controls across these two runs |
| All 57 app Mix scripts, final fixtures, default | `e1717e5f` / `20261121-153138-ee99fcf9-a1` | Pass |
| All 57 app Mix scripts, final fixtures, test-support | `e1717e5f` / `20261121-153138-0d3df86e-a1` | Pass |
| All 15 changed Mix files, including prior-round files | `e1717e5f` / local lint | Zero errors, warnings or notes; git diff --check clean |

Clippy passes with warnings in unchanged Rust files; no diagnostics point at
the changed Rust tests. Inline forgotten-mark and eager-redaction controls also
pass within the app scripts. The final fixture-only changes isolate the nil
guard from generation mismatch and check upstream accounting while quitting;
they received both native app-script configurations and all queue controls.
No Rust or product source changed after the complete validation checkpoint.

Saved logs were read and audited. The original baseline probe
`20261121-145159-424166e8-a1` counted only outer row ids; its latency population
was corrected to include correlated responses and censoring before the accepted
before/after capture. The first real-Job fixture used a nonexistent program path
and failed to resolve rpc-job.mix in `20261121-150637-b9dffae2-a1`; using the
existing application path fixed that fixture, and the real test passes in
`20261121-150840-e615a2c4-a1` and `20261121-151832-f66a2775-a1`.
The first control run `20261121-151156-ec5c318e-a1` stopped on its own missing
source needle after its earlier controls had hit their assertions; the needle
was corrected. Superseded checks at9928480f (`20261121-151357-0d4f6b01-a1` and
`20261121-151357-33c97351-a1`) were cancelled with143 when the native-tag/reused-id
fixture was added; they are not complete validation receipts.

The serial entry is unchanged. No `tools/ci.mix all`, soak, merge or deployment
was run in this round; `pr_gate cbc` is the next full run. Main freshness is
checked before hand-off. Desk7's after probe remains conditional on a gold
carrying this PR; the existing before and pending follow-up remain in the hub.

---

# PR #216 — Workshop section projection and notice redaction

PR: https://github.com/mixviu/mixviu/pull/216, opened by `app/mixviu-agent`.
Branch: `feat/workshop-section-dirty`, from merged #214 main `6b4278c7`.
Current source checkpoint: `f0f6c0cf`; production behaviour is unchanged since
`80e4ae38`, with later test/documentation changes. This receipt is a subsequent
documentation commit. **Scoped checks pass; Workshop stays serial after the
strict normal-pool failure recorded below. Ready for PR review.**

## Behaviour and locations

| File:line | Change |
|---|---|
| `apps/workshop/presentation.mix:33` | Section input signatures and dirty marks. Tree, macro and Traffic reuse unchanged acknowledged sections; Mesh and agent time-dependent values project each turn. Full `project()` remains the oracle. |
| `apps/workshop/app.mix:17` | Candidate inputs are captured with the publication and commit on the matching applied receipt. Pending events coalesce against the acknowledged inputs/model; refusal re-projects from scratch. |
| `apps/workshop/traffic.mix:8` | Notice rows enter private bounded pending state. One ordered native Job runs the unchanged recursive redactor; only redacted completion enters display/export retention. Pending work includes active bytes/rows in its 1,024-row / 8 MiB cap. |
| `apps/workshop/controller.mix:192` | Redaction completion, correlation and existing visible drop accounting; admission at `:260` through the existing native Jobs path. No new daemon or transport. |
| `libraries/viu/mix/model-publication.mix:13` | Direct differ calls reject changed non-map roots or long root keys instead of producing empty-path sets. |
| `libraries/viu/src/app_host_model_tests.rs:107` | Real host at mailbox width eight retains eight receipts and pauses completion retirement. Restoring literal 32 fails the test. |
| `docs/scene-model-publication.md:106` | Implementation/checks appendix heading and the asynchronous Traffic visibility/drop contract. |

Traffic observation visibility is asynchronous. Overflow drops the oldest waiting
row; failed redaction work drops its row. Both increment the existing visible
drop count. Pending raw values are absent from Traffic display/filter/export.
Direct `workshop.call` / `workshop.reply` semantics are unchanged; their Traffic
copy still uses the original synchronous redactor. Closing the app can abandon
pending diagnostic observations.

## Measurements before changes

The measurement-only `lab/workshop-section-measure` branch is not merged.
CBC `20261121-140224-5181f254-a1` at lab `5055105a`, based on #214, passes the
real 200-peer fixture. There are 17 projections in its 200-peer phase:

| Section | Median CPU | Changed projections |
|---|---:|---:|
| Tree | 36.55 ms | 11/17 |
| Macro | 0.299 ms | 1/17 |
| Agent | 0.389 ms | 1/17 |
| Mesh | 18.62 ms | 17/17 |
| Traffic | 11.30 ms | 14/17 |

These are callback projections, not uniform tick samples. Mesh clock output
changes every projection. Redaction by encoded row size: <1 KiB, n82,
median/max 1.712/3.068 ms; 1–16 KiB, n27, 3.574/39.347 ms; 16–64 KiB, n68,
0.399/178.643 ms. An earlier diagnostic `20261121-140014-8a6b8b06-a1` reached
228.964 ms for a 53,404-byte row. Shape matters; a byte threshold alone would
not establish a time budget. Notice callbacks now perform zero recursive
redaction; the same function runs in the native Job evaluator.

## After measurements and limits

Standalone six-test pass `20261121-140851-7c5640fd-a1` measured steady tick
median 75.425 ms and notice CPU max 6.512 ms in the retained-body phase. Native
apply reached 47.403 ms while delayed Traffic was still arriving. These are
different retained states and cannot establish a paired whole-tick speedup.

The paired diagnostic `20261121-141251-fdb97a25-a1` compares the same state and
Mesh clock. With a clean tree, section projection took 40.10 / 51.55 / 53.47 /
65.41 ms against full projection 69.95 / 76.66 / 78.65 / 80.98 ms. Full oracle
comparisons succeeded until the run hit the unchanged native apply guard:
118,166 us observed against 100,000 us. This red diagnostic remains reported;
it is not a responsiveness pass. Instrumentation adds a full projection and
changes how many delayed rows arrive before the next publication.

## Validation receipts

| CBC job | Result |
|---|---|
| `20261121-135818-65efb9b7-a1` | Baseline instrumentation caller error: diagnostic module/global cache scope; fixed before usable measurements. |
| `20261121-135907-2851e274-a1` | Baseline fixture passed, but callback diagnostics were not forwarded; no section measurement claim. |
| `20261121-140851-6444b063-a1` | Worker fmt diff applied locally; worker restored its own files. |
| `20261121-140455-4792ad62-a1` | Initial host-width fixture failed to settle because its bootstrap published no model. Corrected, not hidden. |
| `20261121-140851-7c5640fd-a1` | Corrected host width test, section/redaction/Traffic/publication Mix policies, and all six Workshop Bus tests pass. |
| `20261121-141057-277a0c45-a1` | Literal-32 negative control triggers the host saturation assertion. Source restored. |
| `20261121-141057-f7a67ce3-a1` | Full oracle after real discovery, steady and notice events passes at 200 peers. |
| `20261121-141251-fc7cd5f6-a1` | Forgotten tree dirty mark triggers the real-fixture oracle assertion. Source restored. |
| `20261121-141251-fdb97a25-a1` | Paired diagnostic red on unchanged native apply bound, detailed above. |
| `20261121-141917-42b10066-a1` | Corrected generated property fixture passes through the real native app harness: all 57 app Mix scripts pass. |
| `20261121-141057-6e4cb2de-a1` | Source787c32bd: fmt check, all-target clippy, all55 non-doc targets and doctests in default and test-support configurations pass. Parent-only totals:647/657 passed, zero failed,12 existing ignored per configuration. Child app-harness summaries excluded. |
| `20261121-143113-d9c3858e-a1` | Final sourcef0f6c0cf: all57 embedded app Mix scripts, including wrapped-model/clock checks, and all six Workshop Bus tests pass. Only documentation changes follow. |
| `20261121-142253-d58056ae-a1` | Redundant full repeat cancelled after the full scoped pass; final fixture changes received the focused native run above. Cancellation receipt143; not a test failure or pass. |

All-target clippy exits0 with existing warnings in unchanged Rust files: library
114 and unit target126 (110 duplicates), plus integration/dependency warnings.
There are no diagnostics in the changed Rust test file. This is not a
warnings-as-errors result. Saved logs were read and audited. All13 changed Mix
files lint with zero errors, warnings or notes; git diff --check passes.

The final standalone retained-body phase reaches4,052,371 model bytes, tick
median86,569us, notice CPU max5,743us and native apply max68,520us. Delayed
Traffic is still changing the model; this is not the baseline's retained state
and no paired whole-tick speedup is claimed.

The generated property test now uses eight seeds, one complete ten-event
permutation each, plus a forgotten mark, wrapped acknowledged models and a
clock-advance check. The original repeated 320-step version passes under the
CLI but exceeded the embedded 20-second callback deadline in the loaded pool.
Only duplicate permutation repeats were removed. Product deadlines, 200 peers,
silent/departing peers and all existing numeric guards are unchanged.

Redaction tests pin the existing substring-key semantics, deferred request/
response correlation, FIFO, pending row bound and absence of raw Traffic output.
A test double raises on recursive notice redaction; restoring eager work hits
that assertion. Every changed Mix file has been linted; final lint has zero
errors, warnings and notes.

The independent GLM whole-path survey is recorded in the private hub with
accepted checks and explicit scope corrections. Earlier GLM/DeepSeek attempts
exhausted their reasoning budgets without a report and are not approvals.

## Pool and desk disposition

Initial strict normal-pool job `20261121-141302-0dbe7a6d-a1` stopped in round one
on the new generated test's embedded deadline. Workshop Bus passed. A fresh
strict twelve-round width-four no-serial/no-retry run after the fixture repair,
`20261121-142018-d010b2ad-a1`, passed round one and failed round two on native
apply: 105,920 us observed against the unchanged 100,000 us guard. Rounds 3–12
did not run. All 54 binaries were queued at width four, with zero serial entries
and no retry. Failure-phase notice CPU max was 5,840 us; tick median 80,530 us;
wire max 157,462 bytes; retained model max 3,852,294 bytes. The one-second calls
did not fail. **Workshop stays serial.** The conditional full CI is not triggered.

Desk7 after measurement is pending the gold carrying this PR. Its recorded
before is worst ping 482 ms in the first 1.3 seconds on gold `a3b1e286`.
The private TODO-workshop records the exact probe and this pending follow-up.
No PR merge, gold shipment or desk deployment has occurred in this task.

---

# PR #214 — rounds 2, 3 and 3b complete

PR: https://github.com/mixviu/mixviu/pull/214 (ready). Validated source head: `3fa0291585fe731e545ca656916df967b8e12629`; the following receipt commit changes only this report. Base remains `a3b1e286` (#213); final fetch found no newer main commit. Workshop stays serial. No full CI or soak was run; `pr_gate cbc` remains the next full run.

The differ now collapses at eight path components, below Mix's call-depth limit of 16 including publisher frames. Changed long root keys use full publication; changed long nested keys replace their parent subtree. Keys inside replacement values remain ordinary data. The recursive long-key walker is gone. The fixture's shared data is initialised outside callback scopes; nested add/change/remove and matched-ID list cases now execute successfully.

## Validation receipts

| CBC job | Result |
|---|---|
| `20261121-121747-7465b4f7-a1` | Worker formatting at d31be7e9: clean, no diff. |
| `20261121-121928-832f1c97` | Corrected native deep/long-key fixture at ae5bef1c: 1 passed, no refused sends or recursion errors. |
| `20261121-122002-113007cc-a1` | Final source 3fa02915: BUILD_RC=0. Fmt check; all-target clippy for scene and lib-viu in both configurations; scene 79 passed; all 55 lib-viu targets in each configuration: default 640 passed / 0 failed / 12 ignored, test-support 650 / 0 / 12. Child-process summaries excluded from totals. Doctests pass (zero cases). All nine changed .mix files lint with zero errors, warnings or notes. Production host vocabulary and exact counter/wire equivalence checks pass. |
| `20261121-122003-79412edd-a1` | Final source 3fa02915: BUILD_RC=0. All 15 negative controls hit their intended assertion: depth 1, flush bytes 1, mutation guards 2, numeric bounds 3, binding work 1, discovery work/expiry 3, incremental wire/apply/equivalence/receipt 4. Positive native full/incremental equivalence passes. Every mutation is restored; drift checks pass. |
| `20261121-122113-026613aa-a1` | Final source 3fa02915: BUILD_RC=0. All six Workshop Bus tests pass standalone, one test thread; original deadlines unchanged. |

Saved CBC logs were read and audited. The depth control restores 32 and fails with recursion depth exceeded (limit 16). Flush control observes 101 bytes against 1. Numeric controls observe pump 731 us against 0, wire 14,372 bytes against 1, and apply 2,750 us against 0. Forced full publication observes 790,234 bytes against 786,432; forced full application observes 105,771 us against 100,000. Skipping bindings fails rendered equivalence; dropping receipts fails mailbox-retention assertion. Both wire/compare source-drift controls refuse their unchanged mutation.

## Before / after measurements

#213's reported baseline at 200 peers was 2.1–2.4 MB published per tick, Mix rebuild 213–278 ms and native apply 226–323 ms. The new standalone retained-body fixture is a different, larger retained state: 3,552,515 bytes maximum, steady publication 8,855 bytes maximum and native patch apply 10,825 us CPU maximum; tick CPU median 82,361 us. Discovery publication maximum is 406,088 bytes and patch apply maximum 44,261 us CPU. These are scoped standalone measurements, not evidence that Workshop can leave the serial pool. The earlier nine-green/round-ten-miss normal-pool result below remains the disposition.

Round 2 red (`20261121-121016-808e91e7-a1`) exposed the recursive walker defect and led to the bounded design ruling. Round 3 red (`20261121-121804-71f4ed93-a1`) was our callback-local fixture variable, corrected in ae5bef1c under round 3b's authority. Neither red run substitutes for the final green receipts above.

## Review response (round-1 findings, accepted in rounds 2–3)

| Finding | Resolution and evidence |
|---|---|
| 1. Fable contract status, receipts and narrative | `docs/scene-model-publication.md:1`: v1, provisional; exactly one applied/refused receipt or generation ends; configured-width retention is non-normative; discovery moved to appendix. |
| 2. Gemini differ path limits | `libraries/viu/mix/model-publication.mix:3`: changed root long keys require full publication; nested long path keys collapse at their parent; named depth bound 8 leaves caller frames below Mix's limit 16. Native 40-deep maps and matched-ID lists, root/nested 300-byte keys, and nested add/change/remove pass without refused sends. Restoring depth 32 fails the native test with the intended recursion error. |
| 3. Gemini direct decode/edit limit tests | `libraries/viu/src/model_patch.rs:288`: direct tests cover >1 MiB, >512 operations, >32 components, >256 UTF-8 bytes, zero/nonconsecutive/oversized revisions, invalid splice removal, and exact rollback on middle-operation aggregate overflow. |
| 4. Gemini probe base | `libraries/viu/tests/workshop-probe.mix:250`: validate derives its base with `git merge-base HEAD origin/main`. |
| 5. Gemini dead index | `apps/workshop/traffic-view.mix:9`: removed the unused index initialisation. |
| 6. GLM splice wording | `docs/scene-model-publication.md:34`: append at index == len with remove == 0 is explicitly allowed. |
| 7. GLM terminal revision | `docs/scene-model-publication.md:37`: MAX revision ends revisioned publication for that generation; recovery uses a new generation or legacy full, which does not reopen the sequence. Native fencing test covers terminal full refusal and legacy recovery. |
| 8. GLM receipt bound | `libraries/viu/src/app_host.rs:1096`: derives retention from `Runtime::capacity()`; width-eight test passes. Contract states that saturation pauses replies and all other completion retirement. |
| 9. GLM flush byte guard | `libraries/viu/src/app_host.rs:1108`: counts publications from applied handlers as well as ticks. Positive flush test passes; lowering its bound to one byte fails the byte assertion. |
| 10. GLM template early exit | `libraries/viu/src/templates.rs:49`: chose the structural predicate: no declared template roots and no reads_item bindings. Documented this choice. |
| 11. DeepSeek dotted root | `libraries/viu/src/view_binding_tests.rs:12`: patching literal key a.b read via model index syntax produces the same tree as full publication and updates the text. Binding logic unchanged. |
| 12. DeepSeek raw operation count | `libraries/viu/src/model_patch.rs:58`: raw operations length is checked before deserialisation; malformed 513-element input verifies ordering. |
| 13. DeepSeek legacy resynchronisation | `libraries/viu/src/app_host.rs:1258`: both successful legacy full paths clear the refusal fence; native fencing test verifies it. |

Fable's four deferred items are recorded with the ruling's wording in private TODO-viu.md / TODO-workshop.md: host-supplied full-publication keys, integer capability version, section-dirty projection, and a Traffic redaction budget. Mix's recursion-limit decision is recorded separately in TODO-mix.md. None is implemented here.


---

# PR #214 — incremental hosted model publication

PR: https://github.com/mixviu/mixviu/pull/214 (`mixviu-agent`). Branch `feat/viu-incremental-model`, from #213's merged main `a3b1e286c561386719f429b13f1fb77ed5303ed4`. Tested native code `45a1618eddf4`; later changes document retained receipts and add their test-probe control. PR code/documentation head: `771fb8ecd1f9d27fc27f6a95129f28c86cf7a511`.. This receipt is a following documentation commit.

Ready for review: incremental publication, Workshop conversion, equivalence, negative controls and all scoped gates pass. The wider normal-pool condition is NOT met: nine rounds passed, round ten missed the original one-second ping deadline. Workshop stays serial; no wider responsiveness claim is made.

Workshop publishes ordered scene-data edits through the existing local native endpoint. Unchanged Traffic rows and action/menu projections are retained. Other apps keep whole-model publication. There is no new broker, transport, daemon, dependency or toolkit field in the scene contract.

## Contract and implementation

| Location | Change |
| --- | --- |
| `docs/scene-model-publication.md:1` | Toolkit-neutral contract, limits, recovery and fixture provenance. Proposed in the draft PR and private design note before bulk production code. |
| `libraries/viu/src/model_patch.rs:58` | Strict generation/base/revision envelope; typed-path set/remove/splice; 1 MiB, 512 operations, depth 32, 256-byte keys, exact positive revisions. |
| `libraries/viu/src/model_patch.rs:132` | Exact encoded-byte deltas, 8 MiB aggregate budget and moved-value rollback; no unchanged aggregate clone/serialisation. |
| `libraries/scene/src/bindings.rs:353` | Cached dependency set prepares only affected root bindings and retains unaffected ports. |
| `libraries/viu/src/view.rs:188` | Transactional root/frame/model/ports/template/control application and rollback. Hosted overlay invariant shared with full publication. |
| `libraries/viu/src/runtime.rs:826` | Native local endpoint, opt-in owner capability, failed/conflicting-callback discard and queued publication metadata. |
| `libraries/viu/src/app_host.rs:159` | Applied/refused receipts, base fence, refusal fence and full recovery; revision high-water prevents legacy opt-out/re-entry base reuse. |
| `libraries/viu/src/app_host.rs:196` | Patch at the existing owner application point; host/style/window roots require full publication. Independently changed host metrics refuse the patch before mutation. |
| `apps/workshop/app.mix:17` | One outstanding publication, retained acknowledged model, full fallback, no-op skip and refusal recovery. Acceptance immediately flushes coalesced input (`:47`). |
| `libraries/viu/mix/model-publication.mix:1` | Generic pure Mix map/list differencing and host-input policy. |
| `apps/workshop/traffic-view.mix:5` | Retained formatted rows and stable collapse ids; both placements keep complete redacted bodies. Actions/menus cache their complete policy inputs. |
| `libraries/viu/tests/workshop_bus.rs:16` | 100 ms native apply guard and 768 KiB tick-publication guard; original 1 s call/2 s owner-stall checks retained. |
| `libraries/viu/tests/workshop-probe.mix:12` | Wire/compare mutation guards and source-drift controls. Private TODO-workshop work entry removed, resolution cites #214. |

Discovery stayed on the unmerged `lab/viu-incremental-model` branch. An independent different-family whole-specimen survey preceded derivation. The first capture mixed peer-count phases and is excluded from acceptance. Fresh diagnostic capture/replay `20261121-091529-dcb5cbe3-a1` reached 2.64 MB and compared a mounted frame; its later refetch assertion failed, so it establishes captured-state replay only. No relaxed lab deadline is acceptance evidence.

## Before and after at 200 peers

These are named fixture phases, not a paired same-payload speedup benchmark. The steady phase deliberately retains representative escaped JSON through the real Traffic retention code; it is additional to the original discovery/silent/departing-peer checks.

| Measurement | Whole-model baseline | Derived incremental path |
| --- | --- | --- |
| Retained model | #213 ruling: 2.1–2.4 MB | Latest discovery 1.49 MB; deterministic steady phase 3.56 MB (earlier discovery reached 2.81 MB) |
| Published per dirty tick | Whole 2.1–2.4 MB | Latest discovery max 380,127 bytes; steady max 8,857 bytes (earlier discovery max 411 KB) |
| Mix build / diff | #213: 213–278 ms | Final phase probe: discovery 97–141 ms; steady 76–81 ms |
| Native apply | #213: 226–323 ms | Latest discovery max 41.0 ms CPU; steady max 11.3 ms CPU |
| Actual changed data shape | Changed top-level Traffic key retained most bytes | Final phase probe: discovery 54–168 edits / 40.6–392.8 KB; steady 7–15 edits / 1.5–6.8 KB, under 0.2% of a 3.55 MB model |

Baseline receipts: `20261121-032412-ad76b5d7-a1` and `20261121-033200-ed59f7bd-a1` from #213. Reading the latter's full saved log gives a 2,399,127-byte native full apply: validation 59.345 ms, binding/diff/copy 132.207 ms, controls/templates 6.257 ms, frame 124.960 ms; total 322.769 ms CPU. New pre-derivation diagnostic job `20261121-090200-7896b668-a1` at 1.89 MB: Mix build 237.7 ms and native full apply 264.6 ms (47.6 validate, 109.0 binding/copy, 6.9 controls, 101.1 frame). Its instrumented one-second run failed and is diagnostic only.

Earlier uninstrumented focused receipt `20261121-103204-1ec0d59f-a1`: retained 3,552,511 bytes, tick max 8,855 bytes, native max 10,289 us CPU; Mix tick median 81,615 / max 82,905 us CPU. All twelve Workshop tests passed. Final native code adds the small host-input comparison verified by `20261121-103841-d4396d3e-a1`.

Latest uninstrumented native-code receipt `20261121-111901-f7f6f217-a1`, source 771fb8ec (same Rust as 45a1618e), passes all six Workshop Bus tests: discovery retained max 1,488,815 bytes, wire max 380,127 bytes, native apply max 40,987 us CPU, tick median 133,649 us CPU. The retained-body phase reaches 3,561,324 bytes, publishes max 8,857 bytes per tick, native apply max 11,298 us CPU, tick median 89,660 / max 92,031 us CPU, and eight one-second info calls max 114,866 us. This is the final-source uninstrumented result.

Final phase probe `20261121-104543-0a10feb7-a1` on documentation head `a9dde3ab25bc` (same native code as 8aabdda7): steady Mix build/diff 76.1–80.9 ms, presentation 63.3–66.4 ms, frame projection 2.26–2.34 ms. Native root+frame 9.24–9.78 ms: edit/budget 0.052–0.205 ms, bindings/copy 3.44–3.73 ms, controls/templates 5.03–5.10 ms, frame 0.686–0.852 ms. Outer native host apply max 10.583 ms. Counting full model bytes in the diagnostic callback adds serialisation overhead; its ~189 ms total callback is not the uninstrumented ~82 ms callback.

The historical 743.324 ms CPU notice in #213 has no retained payload/branch trace; its exact breakdown cannot be recovered. Fresh diagnostic job `20261121-091206-9e110876-a1` attributes 106 accepted 200-peer observations: counter extraction 323.3 ms total / 6.45 ms max; mesh add-event 156.5 ms / 2.15 ms; Traffic processing 2,746.2 ms / 175.7 ms. Traffic redaction is nested inside that last span: 2,473.2 ms / 166.6 ms, so those totals must not be added. This identifies the measured hot branch, not the historical event. No redaction semantics were changed.

A three-green-round normal-pool attempt then exposed dropped acceptance under a full evaluator mailbox in round 4 (`20261121-103914-d277db38-a1`). The original Bus deadlines passed, but Workshop remained pending and the added retained-data seed never published. The owner now retains at most 32 acceptance/refusal receipts, flushes them before ordinary producers, and pauses completion retirement at that bound (`app_host.rs:183`). The deterministic mailbox-saturation test (`app_host_model_tests.rs:5`) passes with revision/recovery tests in `20261121-105826-ef5fb3a2-a1`. Restoring the old lossy dispatch fails its explicit retention assertion in `20261121-110041-11def735-a1`; all four source controls and the changed probe lint pass in that job. Native code under the final gates includes this fix.

## Verification

Scoped CBC `20261121-105828-6b4b9900-a1`, BUILD_RC=0, native source 45a1618e: scene all-targets 79 passed / 0 failed; lib-viu default 635 / 0 / 12 existing ignored and test-support 644 / 0 / 12, all 55 Cargo targets per configuration including the example. Doctests pass (zero cases). Fmt and all-target clippy pass for both changed crates and both lib-viu feature configurations. All nine changed Mix files lint cleanly; the later probe-control addition also lints cleanly in 20261121-110041-11def735-a1. Exact production vocabulary and counter precision/equivalence checks pass. The full saved log has been parsed, excluding child-process summaries from target totals.

Native full/incremental equivalence compares resolved content, mounted frame and prepared template lists after actual hosted patches. Unit/owner tests cover typed list paths, exact byte accounting, reverse rollback, invalid controls/frame rollback, wrong generation/base, refusal fencing, full recovery, failed/conflicting callbacks, legacy publication and retired-base reuse, independent host metrics, coalesced input and idle acceptance.

Equivalence and source controls before the receipt fix: `20261121-103942-750dab98-a1`, BUILD_RC=0, source 8aabdda7. Positive native oracle passed at 3.55 MB (max wire 53,231 bytes, native apply 19,485 us during continuing refetch). Forced full publication failed at 1,125,040 bytes against 786,432. Forced full application failed at 407,079 us against 100,000. Skipped binding updates failed rendered-state equivalence. Each control requires the intended assertion and restores source.

After the receipt fix, `20261121-110005-caf9cb59-a1` passes fresh full/incremental rendered equivalence and the three source controls. Forced full publication fails at 1,021,121 bytes; full apply at 133,300 us; skipped bindings fails equivalence. `20261121-110041-11def735-a1` additionally proves the mailbox regression detects lost acceptance (and repeats all three controls).

Numeric and probe-drift controls: `20261121-103314-6a801c89-a1`, BUILD_RC=0. Pump bound zero failed at 744 us; byte bound one failed at 376,969 bytes; native apply bound zero failed at 2,286 us. Both wire and compare source-drift controls refused their unchanged mutation. Later native guard changes do not alter these probes.

Strict normal-pool attempt `20261121-105827-6852a295-a1`, source 45a1618e, BUILD_RC=1: nine complete green rounds; round ten failed `workshop.ping` at 1,004,407 us against the unchanged one-second deadline (`workshop_bus.rs:510`). Rounds eleven and twelve were not run. All ten rounds used 54 binaries, width four, zero serial targets and strict retry disabled. The nine completed steady phases retained 3.55 MB, published at most 8,857 bytes per tick and applied in at most 20,645 us CPU. In the failing discovery run, native apply max 86,828 us, wire max 375,795 bytes and pump max 112,838 us remained inside their 100,000 us / 786,432 bytes / 800,000 us guards. Its notice CPU max was 638,652 us and tick CPU max 481,610 us; these are per-command maxima, not proof of adjacent events. `tools/ci/serial-tests.conf.mix` is unchanged. The exact twelve-round condition was not satisfied and no retry result is substituted.

Full `tools/ci.mix all` was not run: its requested trigger was removing Workshop from the serial list after 12/12 normal-pool rounds. That condition failed, so pool enrolment is unchanged. All changed-crate targets and the explicit positive/negative controls above passed.

Earlier attempts are not combined into the final soak: `20261121-094901-2e1a4354-a1` failed an added natural-history minimum-size guess (all Bus deadlines met), which led to the deterministic fixture. Jobs `20261121-101255-c36ceafa-a1`, `20261121-103214-0babcb3e-a1` and `20261121-103215-281cc2d5-a1` were explicitly cancelled with rc143 when code changed; their partial rounds are not acceptance. No retry, ignore, reduced peer count, relaxed deadline or loosened numeric guard was introduced.

No merge, production install or deployment. Earlier receipts below describe #213 and are historical.

---

# PR #213 round 8 — review fixes validated

PR: https://github.com/mixviu/mixviu/pull/213; branch fix/workshop-discovery-flake.
Final tested code: `d5de50233afc2508c83c24e04fec11f15d8a7769`; the following commit records this receipt only. Main fetched/merged first, already current at 6033d061 (#196/#210).

All four requested fixes are complete. No binding/encoder policy, deadlines, peers, serial list or numeric limits changed. No tools/ci.mix all in this round. PR remains ready; pr_gate cbc is the next full run.

## Review r1 response

| Finding | Response and evidence |
| --- | --- |
| Gemini BLOCKING / GLM: expiry loses accepted services and may admit describes after deadline | Fixed in `apps/workshop/discovery.mix:151`: flush remaining local/remote entries into node services, original identities and deadline-error placeholders, without enqueueing work; clear lists before finalising. `discovery-expiry-test.mix` covers partially expanded lists, routing, original versus synthetic identities, completion, empty queue and zero backend admissions. Removing the flush fails with `expiry must retain every accepted service`. |
| GLM / Gemini: live test-support host enables CPU probe | Fixed: `app_host.rs:1893` calls the constructor with false; only `test_support::Harness` opts in (`:2671`). Registration remains outside runtime.rs. Exact registry comparison, generator and lint_context unchanged and passing in both feature configurations. |
| Gemini: controls depend on unreachable branch commits | Fixed: `workshop-probe.mix:62,98,114` uses current-source mutations/copies for jq, eager expansion, unbounded scan and expiry-flush removal. No commit SHAs or `git show`; exact assertion checks and source restoration/diff checks retained. Jq helper smoke test is byte-identical in all three samples. |
| GLM: slow-only equivalence cases look like fast-path coverage | Fixed: `counter-test.mix:22` requires an expected `needs_thaw` value for every equivalence call. Separate false/fast and true/slow corpus groups; snapshot explicitly false, digit-string counters and witnesses true. Equality remains parsed JSON inside jq to preserve u64 low bits. |
| DeepSeek: `model.sessions[0]` dependency is skipped | Rejected after code check: `expr::reads()` stops at the first index (`analysis.rs:71`, existing semantics test at `:661`), yielding `model.sessions`, not `model.sessions[0]`. Comparing that containing value detects element changes. Binding code unchanged. |
| GLM: dynamic-read dependency coverage | Existing syntactic contract covers `$model.m[$model.k].x` as `model.m` and `model.k`, and indexed `item.cells[0]` as `item.cells` (`expr/tests/semantics.rs:661`). The binding predicate compares the containing top-level subtree conservatively. No change to dependency recording or comparison. |
| DeepSeek: double encode on slow path | Retained intentionally: the fallback is rare by design and keeps the exact old thaw semantics. No encoder behaviour change in this review round. |

Validation: scoped CBC `20261121-071033-20bf890d-a1` (fmt, all-target clippy, every lib-viu target with default features and test-support, exact vocabulary checks, counter path/equivalence cases and six source-mutation negative controls); jq helper `20261121-071255-4ba36e94-a1`. Strict twelve-round width-four crate pool with workshop_bus alone: `20261121-073011-73434aeb-a1`, 12/12 green. No full CI was run in this round; the serial list is unchanged and `pr_gate cbc` is the next full run.

Tested code `d5de50233afc2508c83c24e04fec11f15d8a7769`; receipt head the following receipt commit. All original deadlines, peers, discovery bounds and numeric guards are unchanged. Full history and job receipts: `last.md`.


Scoped checks: 55 all-target binaries per configuration (includes terminal_ime_driver example); default 627 passed / 0 failed / 12 existing ignored; test-support 636 / 0 / 12; doctests zero. All eight branch-changed Mix files lint cleanly. Six mutation controls require expected cargo-101 assertion failures, then restore source. Pump negative observed 702 us against 0; model negative 24,206 bytes against 1. Worker fmt applied from 20261121-070923-d33e8115-a1.

Twelve rounds each run all 54 test binaries, width four and workshop_bus alone, retry disabled. Pump maximum 498960 us; model maximum 2668231 bytes, below 800,000 us / 4 MiB. Every expiry regression and 200-peer discovery test passed in every round. Full logs parsed and retained privately.

Earlier receipts below are historical.

---

# PR #213 — test probe registration and final isolation validation

PR: https://github.com/mixviu/mixviu/pull/213 (`mixviu-agent`).
Branch: `fix/workshop-discovery-flake`.
Final tested code head: `f8eb1c27f24c8d3b7d2dad144c7a3dd4ae835e1a`.
The following commit records this receipt only.

**Ready for review.** The registration mismatch is fixed; scoped checks, one full CI and the strict twelve-round isolated soak are green. The flake is resolved by the authorised isolation; incremental model publication remains structural follow-up work.

Current main was fetched and merged first: already up to date at `6033d061`,
including #196 and #210. All earlier measured gains, the workshop_bus serial
entry and five negative controls remain. The 1 s call deadline, 2 s owner
stall assertion, 200 peers, silent peer and disconnecting peer are unchanged.
No retries, ignores, reduced peer count or pool budget were added.

## Registration repair

`libraries/viu/src/app_host.rs:2570` now registers `viu_test_cpu` from the
existing test-support module. `runtime.rs:284` calls that helper only when
the test-support-only `Services.test_cpu_probe` flag is enabled; its default
is false. The feature-enabled host enables it at `app_host.rs:2075`.
The registration literal is outside runtime.rs, the only file scanned by
both vocabulary generators. The default runtime does not gain the hook.

The exact live/generated registry test, lib-mix build generator and
`tools/ci/lint_context.mix` are unchanged. Both source scanners produce the
same **41 production names**, excluding the probe. The lint-context host,
unknown-function, shell and remote-body controls pass. The unchanged exact
registry test passes with default features and with test-support.

## Scoped verification

CBC `20261121-051153-e9b95699-a1`, BUILD_RC=0, tested code above:

- fmt and all-target clippy pass in both feature configurations.
- Default features: every lib-viu test target, **55 binaries, 627 passed,
  zero failed, 12 existing ignored**; doctests pass (zero cases).
- With test-support: every target, **55 binaries, 635 passed, zero failed,
  12 existing ignored**; doctests pass (zero cases).
- Counter precision / native-versus-jq equivalence and slow-path cases pass.
- All five negative controls fail at their intended assertion (cargo 101),
  with source restored. Numeric controls observed a **730 us** pump against
  **0 us**, and **24,206 bytes** against **1 byte** (pump 10,803 us).
  Expansion, queue scan and unrelated-binding controls also fail as intended.
- All seven changed Mix files lint with zero errors, warnings or notes.
  Worker source diff is empty after verification.

Worker rustfmt job `20261121-050948-529f401c-a1` supplied the applied diff.
An initial scoped attempt `20261121-051031-ea07b6e3-a1` was deliberately
cancelled (143) before full CI to make the test-support flag public: external
integration fixtures construct Services with struct-update syntax. The
corrected source above was then checked completely. No normal test failure
was retried, and no earlier full CI was repeated within this round.

## Full CI and soak

Exactly one authorised full `mix tools/ci.mix all`: CBC `20261121-053118-35178bee-a1`, **BUILD_RC=0**. The strict Mix and workspace pools passed, including the unchanged default-feature registry equality check; all later gates completed. Full CI's default-feature workshop_bus has zero tests, so it does not substitute for the feature-enabled soak.

Strict twelve-round soak: CBC `20261121-060439-18c6e862-a1`, **BUILD_RC=0**, all **12/12 rounds green**. Each round ran all **54 lib-viu test binaries** (the scoped all-targets run also checks one example target), with the remaining binaries at **width four** and **workshop_bus alone** through the default serial list. The actual discovery test passed twelve times at 200 peers. Retained numeric guards observed a worst post-startup pump of **476986 us** and a largest tick model of **2670825 bytes** across those twelve 200-peer cases. Every round observed a tick model. No failed test was retried.

Full-CI pool receipts:

- `test_pool: 75 binaries; build+record 6 s, pool 24 s (8 at a time), serial 84 s (2 alone), doc-tests 0 s`
- `test_pool: 526 binaries; build+record 204 s, pool 381 s (8 at a time), serial 201 s (2 alone), doc-tests 13 s`
- `test_pool: 6 binaries; build+record 20 s, pool 35 s (8 at a time), serial 0 s (0 alone), doc-tests 1 s`


## Bounds and structural follow-up

The unchanged post-startup pump limit is **800,000 us**, just over twice the
standalone maximum 387,676 us. The tick-model limit is **4,194,304 bytes**,
about 75% above the standalone 2,398,986 bytes. Bases: CBC
`20261121-033200-ed59f7bd-a1`. Each prints measured values on failure; the
200-peer fixture must observe a tick model. Guards are installed at
`libraries/viu/tests/workshop_bus.rs:554`, constants at `:19`, assertions at
`libraries/viu/src/app_host.rs:2689`. All five controls and the validation
commands are retained in `libraries/viu/tests/workshop-probe.mix:133`;
the strict twelve-round pool starts at `:159`.

Fable ruling D narrows the responsiveness claim to an uncontended core.
Serial enrolment and its measured reason are at
`tools/ci/serial-tests.conf.mix:11`. Structural work remains in the private
incremental-model-publication brief. Round-5 figures: **2.1–2.4 MB** model,
**213–278 ms Mix rebuild** and **226–323 ms scene apply** per tick standalone.
Start that follow-up by attributing the **743.324 ms CPU / 743.674 ms wall
workshop.notice** callback from `20261121-034520-3cacd411-a1`; its branch,
payload and critical path remain unmeasured. Remove isolation only after the
structural fix passes twelve normal width-four pool rounds.

The ledger and TODO-workshop / TODO-viu record the current disposition.
Earlier receipts below retain the before/after measurements and original
red results; their readiness statements are historical. No merge or
deployment was performed.

---

# PR #213 — Fable ruling D: discovery isolation

PR: https://github.com/mixviu/mixviu/pull/213 (`mixviu-agent`).
Branch: `fix/workshop-discovery-flake`.
Final tested code: `167bcbe5e5040a38885ed6f4c762f522bd445ba8`.
The following commit records this receipt only.

**Not ready: full CI is red.** Ruling-D isolation and numeric guards are
implemented, but the required isolated soak was not started. Stop with the
reported failure; do not retry full CI or mark the PR ready.

Current main was fetched first and was already merged: `6033d061` includes
#196 and #210 (merge `594aa21f` from round 5). All measured gains remain:
eight-service expansion, sixteen-entry admission scan, exact counter-name
lookup, native JSON wire fast path, and dependency-aware scene binding apply.

## Ruling and claim

Fable ruling D (2026-11-21) authorises enrolling the **lib-viu/workshop_bus
binary** in the existing serial list. Its reason names
`workshop_answers_during_discovery`, PR #213, the measured structural cause
and the follow-up. At 200 peers, the 2.1-2.4 MB model is rebuilt in Mix
about 213-278 ms and applied about 226-323 ms per tick, standalone. This is
substantial whole-model work, not an established pure CPU-starvation limit.

The tested responsiveness claim is now **within 1 s at 200 peers on an
uncontended core**. The 1 s call deadline, 2 s owner-stall assertion, both
peer counts, silent peer, disconnecting peer, streamed-results/BUSY checks
and every existing negative control remain. No retry, ignore, reduced peer
count, test-thread override or pool-budget constant is introduced.

The serial entry is removed by the structural follow-up **incremental model
publication**, brief `_plan/briefs/2026-11-21-workshop-incremental-model.md`,
after its twelve-round normal width-four pool validation. That follow-up
starts with the round-5 743.324 ms CPU / 743.674 ms wall workshop.notice
callback, whose branch/payload and exact timeout critical path remain
unmeasured (CBC `20261121-034520-3cacd411-a1`).

## Numeric standalone guards

| Property | Standalone measurement | Bound | Margin |
| --- | --- | --- | --- |
| Longest post-startup pump hold | 387,676 us | 800,000 us | Just over 2x |
| Largest published tick model | 2,398,986 bytes | 4,194,304 bytes (4 MiB) | About 75% |

Both bases come from CBC `20261121-033200-ed59f7bd-a1`. The pump window starts
after readiness, matching that measurement; the separate existing 5 s
startup assertion remains. Each pump checks its numeric hold and the maximum
serialized size of completed tick models. The model guard must observe at
least one tick in the 200-peer case. Collection and assertions are confined
to the existing test-support harness. Model-size measurement overhead is
included in the guarded pump; production behaviour and budgets are unchanged.

Both failures print measured values and limits. Worker-only negative
controls lower one bound at a time and require cargo 101 at that assertion:

- Pump: observed **746 us**, bound **0 us**.
- Model: observed **24,206 bytes**, bound **1 byte** (pump 11,135 us).

These fail numerically, before any RPC timeout. The worker restores each
source file. Existing expansion, queue-scan and binding controls also remain,
making **five** negative controls in the scoped gate.

## Checks

CBC `20261121-042324-fee24a9e-a1`, code `167bcbe5`, BUILD_RC=0:

- All five negative controls fail at the intended numeric/work assertion
  (cargo 101), and worker sources are restored.
- fmt, all-target clippy, counter precision and fast/slow equivalence pass.
- Every lib-viu target passes with test-support: 55 binaries, 635 passed,
  zero failed, 12 existing ignored; doctests pass (zero cases).
- The worker diff is empty.

Exactly one authorised `tools/ci.mix all` invocation: CBC
`20261121-043424-f9640729-a1`, same code, **BUILD_RC=1**.

- Workspace fmt, environment checks, all-target clippy, the engine boundary
  and foreign gate pass before testing.
- Strict Mix pool: 75 binaries, pool width eight, two serial; passes.
- Strict workspace pool: 526 binaries, pool width eight, two serial; one
  failed binary, lib-viu/viu. Its unit result is 400 passed / 1 failed /
  5 existing ignored.
- Failure: `registry_tests::generated_lint_vocabulary_matches_the_live_complete_registry`
  at `libraries/viu/src/registry_tests.rs:61`. Generated vocabulary contains
  `viu_test_cpu`, but the default-feature live registry does not. The hook is
  feature-gated in runtime.rs while lib-mix build-time vocabulary scans its
  registration source. This is a PR integration regression, not a timing
  failure. The test-support scoped run does not expose this feature mismatch.
- The serial entry matches, but default-feature workshop_bus contains zero
  tests in this workspace run. This does not substitute for the required
  feature-enabled isolated soak. The actual 200-peer fixture passes in the
  scoped test-support run above.

Full CI stops at that workspace-pool failure; later stages do not run.
The exact registry contract is retained, and no fix or rerun follows this
red result under the brief’s stop condition.

The strict twelve-round feature-enabled width-four pool with the serial
entry honoured was **not started**, because the preceding full CI failed.
There is no isolated-soak success claim. PR #213 remains draft; isolation
validation and registry integration repair remain outstanding.

All seven changed Mix files lint with zero errors, warnings or notes.
Worker rustfmt diff `20261121-041732-cb61d1cd-a1` was applied locally.
The initial scoped job `20261121-042204-44da0458-a1` found a test-support
field visibility error (E0616); it was corrected before the successful
scoped run, any full CI invocation or the soak. No production deadline
changed. Saved full logs and parsed evidence are retained privately.

## File references

- Readiness blocker: `libraries/viu/src/registry_tests.rs:61`, conditional
  hook `libraries/viu/src/runtime.rs:281`, static vocabulary scan
  `libraries/mix/build.rs:20`.

- `tools/ci/serial-tests.conf.mix:11`: lib-viu/workshop_bus enrolment;
  `:14` contains ruling, measurements, PR and structural follow-up.
- `libraries/viu/tests/workshop_bus.rs:15`: standalone bounds and rationale;
  `:554` installs them for the 200-peer post-startup window;
  `:610` refuses a vacuous model-size guard.
- `libraries/viu/src/app_host.rs:969`: published tick byte measurement;
  `:2665` checks pump/model maxima and prints observed values and bounds;
  `:2701` reports model-byte distributions.
- `libraries/viu/tests/workshop-probe.mix:16`: both numeric negative controls;
  `:133` retains all five controls and scoped gates; `:145` runs the strict
  twelve-round pool using the default serial configuration.
- Earlier retained gains: `apps/workshop/discovery.mix:125`,
  `apps/workshop/work.mix:7`, `apps/workshop/counter.mix:26`,
  `libraries/viu/src/view.rs:313`.

The ledger is resolved by isolation only when the checks above are green;
structural performance remains tracked in TODO-workshop / TODO-viu and the
incremental-publication brief. No deployment or PR merge was performed.
Earlier receipts below retain the diagnosis, observer limits and original
red results; their acceptance status is historical.

---

# PR #213 — tick model application

PR: https://github.com/mixviu/mixviu/pull/213, branch
`fix/workshop-discovery-flake`, author `mixviu-agent`.
Final tested code: `d7b5da26643dc5821f6b19a4428faa9da7311f07`.
The following commit records this receipt only.

**Not fixed; PR remains draft.** The strict pool failed in round 6.
Stop with measurements, with no retry or deadline relaxation.

Current main `6033d061` (#196 and #210 included) was merged as `594aa21f`
before measurement and final gates. All earlier discovery bounds and wire
optimisations remain. The 1 s call and 2 s owner-stall deadlines remain
unchanged; no serial entry, retry, skip, full workspace CI or deployment.

## Diagnosis before repair

The premise needs a distinction: Mix produces a **model**, not a new scene
document. Workshop's scene source stays 14,759 bytes with 100 resolved
content nodes; its independent frame has 11 nodes. Model data grows from
24,206 to 2,141,151 bytes during the standalone discovery sample. Host-added
metrics add 141 bytes. The source is not reparsed on these ticks.

CBC `20261121-032412-ad76b5d7-a1`, code `cf82608c`, CBC worker, measures the
200-peer case standalone and with eight CPU loaders on that same worker.
Standalone passed; the loaded sample stopped on info at 1002.437 ms.
The loaded quantiles therefore describe a shorter, earlier part of discovery.
All per-tick records are retained in private `round5-scene.json`.

Triples below are median / p99 / max, milliseconds. The sample counts are
11 standalone and seven loaded. With these small samples, p99 equals max.

| Model construction | Standalone CPU / wall | Loaded CPU / wall |
| --- | --- | --- |
| Mix saved reconciliation + presentation/frame/persistence projection | 212.994 / 263.306 / 263.306; 213.124 / 263.415 / 263.415 | 500.504 / 552.508 / 552.508; 500.551 / 552.583 / 552.583 |
| Model bytes, min / median / max | 24,206 / 1,497,205 / 2,141,151 | 24,206 / 323,273 / 661,096 |

The construction timer ends before diagnostic equality checks, JSON-size
encoding and the native model send; those probes add overhead to the
instrumented callback and are disabled in normal acceptance runs. The
outer apply timer also includes the nested frame probe’s metadata encoding
and logging after its own timer stops; before/after use the same observer.
These profiled totals are not substituted for uninstrumented pool latency.

| Content apply phase | Standalone CPU triple | Standalone wall triple | Loaded CPU triple | Loaded wall triple |
| --- | --- | --- | --- | --- |
| Model type/byte-budget validation | 36.493 / 52.470 / 52.470 | 36.501 / 52.474 / 52.474 | 18.248 / 36.299 / 36.299 | 18.248 / 36.301 / 36.301 |
| Binding preparation, reevaluation and port diff | 87.448 / 119.572 / 119.572 | 87.468 / 119.597 / 119.597 | 55.697 / 91.212 / 91.212 | 55.707 / 91.217 / 91.217 |
| Template/control preparation and validation | 5.643 / 6.374 / 6.374 | 5.643 / 6.375 / 6.375 | 9.756 / 10.165 / 10.165 | 9.770 / 10.166 / 10.166 |
| Frame application and commit | 155.469 / 222.982 / 222.982 | 155.502 / 223.031 / 223.031 | 83.105 / 157.216 / 157.216 | 83.111 / 157.228 / 157.228 |
| Total content apply | 285.054 / 400.901 / 400.901 | 285.117 / 400.981 / 400.981 | 166.807 / 294.876 / 294.876 | 166.838 / 294.897 / 294.897 |

Frame apply is nested inside the content total and must not be added again.
Its standalone CPU was 117.172 / 169.296 / 169.296 ms, loaded
63.316 / 118.674 / 118.674 ms. Each apply evaluated **68 content and seven
frame bindings**, although typically five content ports and **zero frame
ports** changed. The existing port diff runs after reevaluation; it did not
prevent the unchanged binding work. Copying and serialising the full model
also contributes to the binding/validation phases.

Scene-text parse/resolve, widget drawing/rebuild and layout are **not entered**
by this model-only, headless pump fixture. The control phase above decodes
definitions and validates them; persistent widget state is not rebuilt there.
No layout or GPU timing is claimed. The measurements therefore distinguish
actual ingress work from phases that this test never invokes.

Does every tick resend an unchanged scene? No: app.mix gates publication on
dirty state or an animated pane. All observed published models differed;
the first changed only the hidden mesh_canvas, and subsequent publications
changed values/tree/mesh summaries/traffic_view with no selected detail pane.
The frame's bound ports never changed. A whole-model equality skip would
miss the measured redundant frame work. No Workshop publication policy is
changed in this round.

## One contained viu repair, measured after

`SceneView::apply_model` now selects bindings whose recorded top-level model
dependencies changed. Whole-model and dynamic reads remain conservative.
With no affected bindings, it installs the new model without reevaluation.
It still validates the full model budget, templates, controls and overlays,
and updates frame models and revision state. There is no scene/widget schema
change and no toolkit-specific name added to a contract.

Same-worker after job `20261121-033200-ed59f7bd-a1`, code `81499703`, CBC worker.
The three new regression tests pass; standalone passes; the eight-loader
diagnostic still fails on ping at 1004.530 ms. Instrumented diagnostics are
not substituted for the strict acceptance pool.

| 200-peer before -> after | Standalone | Eight CPU loaders |
| --- | --- | --- |
| Content bindings evaluated per tick | 68 -> median 5, max 5 | 68 -> median 5, max 5 |
| Frame bindings evaluated per tick | 7 -> 0 | 7 -> 0 |
| Content apply CPU median / max ms | 285.054 / 400.901 -> 226.307 / 322.769 | 166.807 / 294.876 -> 154.708 / 216.128 |
| Frame apply CPU median / max ms | 117.172 / 169.296 -> 45.287 / 64.373 | 63.316 / 118.674 -> 30.105 / 42.389 |
| Mix build CPU median / max ms | 212.994 / 263.306 -> 226.601 / 278.018 | 500.504 / 552.508 -> 461.859 / 556.115 |

P99 equals max in these small samples. After samples: 11 standalone ticks,
six loaded ticks; model max
2,398,986 / 651,314 bytes respectively. Progress and traffic volume differ,
so aggregate medians are not an identical-document microbenchmark. A closer
standalone size match is 2,003,945 bytes before versus 2,002,484 after:
content apply CPU **374.126 -> 273.446 ms**, wall 374.173 -> 273.534 ms.
At roughly 0.65 MB under load: CPU 294.876 -> 216.128 ms, wall
294.897 -> 216.159 ms. The repair is retained for these measured gains;
full-model validation/copying remains material.

The initial probe jobs `20261121-032030-4bbf0c7a-a1` and
`20261121-032228-85f8c381-a1` failed startup because of diagnostic nil-index
and map-equality mistakes. They provide no product timing evidence; both
were corrected before the before/after jobs above.

## Gates and strict soak

CBC `20261121-033454-ce98550b-a1`, code `d7b5da26`, BUILD_RC=0:

- All three negative controls fail at the intended assertion (cargo 101):
  old list expansion, old blocked-queue scan, and unconditional binding
  reevaluation. Worker files are restored.
- fmt, all-target clippy, exact-counter/equivalence contracts and every
  lib-viu target pass: 55 binaries, 635 passed, zero failed, 12 existing
  ignored. Doctests pass (zero cases); worker diff is empty.
- The three new tests cover the work bound and full-resolve equivalence
  after changes/removal, conservative dependencies and atomic rejection of
  invalid unbound control values.

Strict full-crate width-four soak CBC `20261121-034520-3cacd411-a1`, source `d7b5da26`, BUILD_RC=1: stopped at the first failed round, 6.
Each round ran 54 binaries, serial=0, retry strict; no round was retried.
Detailed model profiling was disabled. The 200-peer triples below are
median / p99 / max.

| Round | Info call wall ms | Ping call wall ms | Pump wall ms |
| --- | --- | --- | --- |
| 1 | 72.487 / 493.445 / 493.445 | 55.2 / 676.706 / 676.706 | 2.068 / 10.035 / 217.113 |
| 2 | 70.004 / 274.005 / 274.005 | 71.824 / 342.06 / 342.06 | 2.306 / 5.792 / 91.481 |
| 3 | 63.807 / 648.51 / 648.51 | 55.677 / 476.108 / 476.108 | 2.158 / 10.467 / 205.968 |
| 4 | 60.391 / 836.86 / 836.86 | 41.84 / 676.299 / 676.299 | 2.247 / 6.561 / 198.478 |
| 5 | 75.923 / 798.993 / 798.993 | 68.285 / 529.783 / 529.783 | 2.159 / 7.354 / 257.363 |
| 6 | 212.434 / 1003.122 / 1003.122 | 94.236 / 632.594 / 632.594 | 2.058 / 11.384 / 220.794 |

Failure markers: ::error::test failed: lib-viu/workshop_bus (rc 101). This is not twelve green rounds.


Round 6 failed on `workshop.info` at **1003.122 ms** (`workshop_bus.rs:461`,
send timed out). Its 200-peer completed info CPU was
8.967 / 17.018 / 17.018 ms (n=14; the failed call is not in CPU samples).
Tick CPU was 198.574 / 240.176 / 240.176 ms (n=6). Host scene application
reached 215.818 ms CPU / 215.862 ms wall; pump CPU was
2.041 / 10.555 / 220.756 ms and wall 2.058 / 11.384 / 220.794 (n=1387).
Discovery completions per pump were 0 / 1 / 1.

The same case recorded a workshop.notice callback at **743.324 ms CPU /
743.674 ms wall** (median CPU 0.445 ms, p99 197.612 ms, n=317). Its exact
notice branch/payload and the timed-out call’s critical path were not logged.
The evidence supports remaining synchronous notice/queued work as the next
lead; it does not establish CPU starvation or justify a larger deadline.
The measured scene gain is retained, but this PR is not ready and the flake
is not fixed. No further measurement run, product change or retry follows.


CBC rustfmt was run and its diff applied from
`20261121-033413-b81b1acb-a1`; all six changed Mix files lint cleanly.
Detailed earlier wire/discovery evidence and failed experiments remain below.

## File references

- `libraries/viu/src/view.rs:16`: conservative model dependency comparison;
  `:313`: dependency-aware model application with full validation retained.
- `libraries/viu/src/view_binding_tests.rs:5`: unrelated replacement work
  bound, changed/removal oracle comparisons, conservative dependency and
  atomic invalid-control tests.
- `libraries/viu/tests/workshop_bus.rs:16`: opt-in Mix projection timing;
  `libraries/viu/src/view.rs:24`: test-support apply-stage timing.
- `libraries/viu/tests/workshop-probe.mix:16`: after-measurement recipe;
  `:19`: binding negative control; `:116`: scoped gates; `:127`: strict soak.
- `apps/workshop/app.mix:189`: existing tick publication gate, unchanged.

The following receipts are historical; the round-5 status above supersedes
their acceptance status.

---

# PR #213 — Workshop wire fast path

PR: https://github.com/mixviu/mixviu/pull/213 (`mixviu-agent`).
Branch: `fix/workshop-discovery-flake`.
Final tested code head: `8e8d6735bcc8c06cbd16f35d554991d8d7219e9f`.
The following commit records this receipt only.

**Not fixed; PR remains draft.** The fast path passes equivalence and
reduces info CPU, but the strict soak failed in round 2. Stop with the
measurements; do not mark ready or retry.

Main `0820aded` (including Deck #204/#210) was already merged as `b899402d`;
current main was checked before the final runs. No deployment or PR merge.
The 1 s call timeout and 2 s owner-stall assertion remain unchanged, with
no serial-list entry, retry or skip. Scene application cost is outside this
repair and remains separate Viu work.

## Change and equivalence

`counter.wire` first uses native `json_encode`. It takes the unchanged jq
thaw path only when one regex finds a listed counter key followed by a
quoted ASCII digit value, or a `_workshop_` witness prefix. Witnesses cover
tool/raw and permission restoration as well as private-field removal.
Matches inside opaque data can conservatively take the slow path; they
cannot change its semantics. Small counters are already numeric, so the
ordinary discovery reply avoids walking every object in jq.

Equivalence is **parsed JSON equality inside jq**, preserving exact u64
integers before crossing Mix's numeric boundary. Native JSON may order keys
differently; byte equality is not the asserted contract for the fast path.
The corpus includes the existing exact-counter cases, a 200-service
snapshot, tool and permission witness rows, protected raw/payload/headers,
all 26 counter keys with `0`, `001`, 2^53, 2^53+1 and u64 max. Assertions
require ordinary snapshots to use the fast path and digit-string counters
and witnesses to use the slow path. Existing precision/opaque-data checks
remain in place.

## Same-worker before/after

CBC `20261121-025638-a5a71b1a-a1`, code `8e8d6735`, CBC worker, BUILD_RC=0.
The same compiled fixture ran in fresh processes; only counter.mix changed
between the retained jq encoder from `f66d2324` and the new candidate.
The full counter/equivalence corpus passed in this job.

200-peer triples are median / p99 / max, milliseconds unless counts:

| Measurement | Retained jq encoder | Native fast path |
| --- | --- | --- |
| Info evaluator thread CPU | 298.190 / 386.986 / 386.986 (n=21) | 8.949 / 15.460 / 15.460 (n=25) |
| Info evaluator callback wall | 300.964 / 577.196 / 577.196 (n=21) | 11.292 / 394.414 / 394.414 (n=25) |
| Info call wall | 388.123 / 838.514 / 838.514 (n=21) | 200.619 / 855.635 / 855.635 (n=25) |
| Ping call wall | 210.271 / 478.408 / 478.408 (n=20) | 137.945 / 496.080 / 496.080 (n=24) |
| Refresh call wall | 197.868 / 197.868 / 197.868 (n=1) | 188.110 / 188.110 / 188.110 (n=1) |
| Pump holds | 1.810 / 6.948 / 227.328 | 1.875 / 14.136 / 392.870 |
| Discovery completions per pump | 0 / 1 / 4 | 0 / 1 / 4 |

Info median CPU fell 97.0%; wall median fell 48.3%. The wall maximum did
not improve in this sample. Host scene work remains material; this comparison
alone is not pool acceptance. Pump data excludes initial mount, as before.

## Validation

CBC `20261121-025714-d18a25b5-a1`, code `8e8d6735`, BUILD_RC=0:

- Both discovery-bound negative controls fail at their intended assertion
  (cargo 101), then restore the worker files.
- Counter precision, opaque data, fast/slow equivalence and branch checks pass.
- cargo fmt check and all-target clippy pass (existing warnings).
- Every lib-viu test target passes: 55 binaries, 632 passed, zero failed,
  12 existing ignored. Doctests pass (zero cases). Worker diff is empty.

Strict soak CBC `20261121-030748-af73c5db-a1`, same source and worker,
BUILD_RC=1: round 1 passed; round 2 failed on `workshop.ping` at 1005.603 ms
(`workshop_bus.rs:442`, send timed out). Both rounds ran 54 full-crate
binaries at width four, serial=0, retries disabled. No further round or
retry ran. This is **not twelve green rounds**.

Round 1 200-peer info: CPU 11.037 / 21.086 / 21.086 ms (n=24), call wall
85.415 / 957.461 / 957.461 ms. Round 2 200-peer distributions:

| Measurement | Median / p99 / max | Samples |
| --- | --- | --- |
| Info CPU ms (completed) | 8.819 / 17.518 / 17.518 | 11 |
| Info call wall ms | 211.109 / 384.669 / 384.669 | 11 |
| Ping call wall ms | 210.350 / 1005.603 / 1005.603 | 12 |
| Refresh call wall ms | 261.181 / 261.181 / 261.181 | 1 |
| Pump wall ms | 1.878 / 9.980 / 386.977 | 1119 |
| Pump CPU ms | 1.860 / 9.474 / 386.899 | 1119 |
| Discovery completions per pump | 0 / 1 / 1 | 1119 |

Completed ping callbacks used at most 2.800 ms CPU. The same round measured
viu.tick callback CPU max 662.663 ms, scene application 380.859 ms CPU /
380.946 ms wall inside 383.315 / 383.401 ms host retirement. This points
to remaining tick/rendering and queued work rather than info encoding as
the residual latency source; the aggregates do not prove the exact critical
path of the timed-out ping. That work is outside this wire-only repair.
The measured, equivalent fast path is retained; no rendering change follows.

All six changed Mix files lint with zero errors, warnings or notes. No
`tools/ci.mix all` or local Rust build was run. Saved logs were read and
full measurements retained in the private evidence directory.

## File references

- `apps/workshop/counter.mix:26`: native encode and guarded slow path;
  `:31` documents the exact conservative condition; `:38` retains jq thaw.
- `apps/workshop/counter-test.mix:22`: parsed-JSON equivalence helper;
  `:31` constructs the 200-service case; `:39` exercises every counter key.
- `libraries/viu/tests/workshop-probe.mix:16`: counter corpus and same-worker
  comparison; `:90` runs both bound negative controls and scoped validation;
  `:100` runs the strict twelve-round width-four full-crate soak.
- `apps/workshop/discovery.mix:125`: eight-service expansion;
  `apps/workshop/work.mix:7`: sixteen-entry rotating queue scan;
  `apps/workshop/controller.mix:260`: advance on follow-up.
- `libraries/viu/tests/workshop_bus.rs:153`: structural bounds regression;
  `:418` retains the 1 s call timeout and `:429` the 2 s owner budget.

The earlier receipts below retain the failed candidates, distributions,
negative-control evidence and the scene-cost attribution limit. Their
acceptance status is historical; the round-4 status above supersedes it.

---

## Earlier rounds (historical)

# PR #213 — Workshop discovery diagnosis (not fixed)

PR: https://github.com/mixviu/mixviu/pull/213, author `mixviu-agent`.
Branch: `fix/workshop-discovery-flake`.
Final tested code head: `f4c9dc081304aad741cdba3de92bdf3b71384f9d`.
The receipt-only commit follows this source.

Merged main `0820aded` (Deck #204 and #210 included) in `b899402d`
before the final checks. No deployment or merge of this PR.

Decision: the product-stall branch of the brief. Admission was limited to
two jobs, but list expansion and blocked-queue inspection still scaled with
the service count in one callback. Discovery now expands eight services per
turn, reserves the global budget before expansion, and keeps the node's list
unfinished until the last batch. Each launch inspects sixteen queued tasks
and rotates the rest. Replies can run between turns. The regression checks
all 201 services and a ready task behind 100 blocked entries.

Batching alone did not cure the info timeout. Counter encoding also matched
a large field-name regex against every object key. Exact-name membership
cuts that cost while keeping the same wire fields and value predicate. The
state-round-trip experiment still failed and was reverted; it is absent
from the final change. The 1 s call timeout and 2 s owner-stall assertion
remain unchanged. There is no new serial entry, retry or skip.

## Measurements

Each triple is median / p99 / max. Times are milliseconds; discovery
completions are counts. Pump measurements below exclude the separately
recorded initial scene mount. The evaluator runs on another thread: the
completion count measures discovery job callbacks retired by a pump, not
the number of service entries expanded inside a callback. The structural
regression separately proves the eight-entry expansion bound.

Before the bounds, CBC `20261121-004418-0160ba49`, source `68903edf`:

| 200-peer measurement | Standalone | Eight CPU loaders |
| --- | --- | --- |
| Pump holds | 1.687 / 3.469 / 269.862 | 0.746 / 2.246 / 70.547 |
| Discovery completions per pump | 0 / 1 / 6 | 0 / 0 / 1 |
| Evaluator callbacks | 5.482 / 473.060 / 664.589 | 5.510 / 1177.098 / 1177.098 |
| workshop.ping | 436.375 / 717.830 / 717.830 (n=15) | 395.358 / 395.358 / 395.358 (n=1) |
| workshop.info | 547.920 / 655.039 / 655.039 (n=15) | 825.986 / 825.986 / 825.986 (n=1) |
| workshop.refresh | 335.150 / 335.150 / 335.150 (n=1) | 1004.405 / 1004.405 / 1004.405 (n=1, timeout) |

The loaded sample stops at the failure; its single-call quantiles are not
tail estimates. The initial 200-peer scene mount was 1267.516 ms standalone
and 2027.538 ms loaded, with zero discovery completions. Earlier 32-loader
CBC `20261121-002809-c0913dec-a1` failed startup before calls; it does not
diagnose the reported RPC timeout.

Thread-CPU follow-up `20261121-005558-7a65eb02-a1`, source `be270734`,
measured job callback CPU max 49.665 ms standalone and 172.483 ms loaded.
After batching, the same-worker standalone target run
`20261121-010021-44bf3ce7-a1` measured max 9.985 ms. The eight-loader sample
`20261121-010022-8d6fe210` passed but was insufficient acceptance: subsequent
pool candidates `20261121-010911-60bf0ca7-a1` and
`20261121-011557-1c30419a-a1` still failed on info. Those reds are retained.

The encoder comparison changed only counter.mix between two fresh runs on
the same worker and compiled source, after merging main. CBC
`20261121-012915-63fbae2d-a1`, source `df45072b`:

| 200-peer measurement | Regex encoder | Exact-name encoder |
| --- | --- | --- |
| Info thread CPU | 512.400 / 561.034 / 561.034 | 337.778 / 379.094 / 379.094 |
| Pump holds | 1.743 / 4.782 / 189.067 | 1.740 / 5.953 / 110.858 |
| Discovery completions per pump | 0 / 1 / 5 | 0 / 1 / 4 |
| workshop.ping | 359.888 / 428.263 / 428.263 | 140.704 / 318.472 / 318.472 |
| workshop.info | 586.361 / 934.801 / 934.801 (n=18) | 383.655 / 648.171 / 648.171 (n=22) |
| workshop.refresh | 216.835 / 216.835 / 216.835 | 202.758 / 202.758 / 202.758 |

CBC `20261121-012646-b39dacdd-a1` compared both encoders on the same
20,025-byte, 200-service fixture, including all 26 counter names, maximum
u64 values, ordinary digit strings and opaque headers/payloads. Output was
byte-identical 3/3. Median encoding time was 35.811 ms before and 25.144 ms
after (three samples; maxima 44.109 and 25.154 ms).

### Round 3: helper CPU and rejected table experiment

Status: **not fixed; PR remains draft**. Final tested code head
`f4c9dc081304aad741cdba3de92bdf3b71384f9d`; the following commit updates
this receipt only. Current main remains `0820aded`, already merged.

Test-only per-helper probes run on the evaluator thread using
`CLOCK_THREAD_CPUTIME_ID`; wall time uses monotonic time. CBC CBC worker job
`20261121-022644-fbc42309-a1`, source `2fa53b47`, full-crate width-four
pool, measured the retained array lookup. The 200-peer info call timed out
at 1003.254 ms. Completed successful info callbacks (n=14):

| Helper | CPU median / p99 / max ms | Wall median / p99 / max ms |
| --- | --- | --- |
| saved.project | 0.034 / 0.108 / 0.108 | 0.035 / 0.116 / 0.116 |
| native.saved | 0.168 / 0.205 / 0.205 | 3.281 / 6.562 / 6.562 |
| model.info | 1.581 / 1.852 / 1.852 | 1.586 / 1.857 / 1.857 |
| handle.inclusive | 2.970 / 5.129 / 5.129 | 5.932 / 9.353 / 9.353 |
| state.assign | 0.049 / 0.057 / 0.057 | 0.051 / 0.059 / 0.059 |
| counter.wire | 296.927 / 397.609 / 397.609 | 297.115 / 397.693 / 397.693 |

The handler includes the first three helpers; these are not additive rows.
Whole info callback CPU was 300.729 / 403.639 / 403.639 ms. The incomplete
timed-out callback is absent from these helper distributions. Pump wall was
1.849 / 6.091 / 278.628 ms; CPU max 278.581 ms; discovery completions per
pump 0 / 1 / 1. Ping wall was 219.544 / 483.736 / 483.736 ms (n=15);
info 505.006 / 1003.254 / 1003.254 (n=15); refresh 227.259 ms (n=1).

This points at encoding, not the state round trip. One candidate built a
constant key table once per jq invocation and checked string type before
membership. Same-worker before/after CBC `20261121-023255-74c991df-a1`,
source `c8c24638`, changed only counter.mix between fresh owner processes.
All three 20,025-byte fixture outputs were byte-identical; fixture medians
24.984 -> 21.767 ms. Real 200-peer info CPU median **295.477 -> 304.088 ms**
(n=21 each), p99/max 373.195 -> 340.087 ms; wall median 384.577 -> 364.586 ms,
max 821.312 -> 741.685 ms. Median CPU did not improve, so the change was
reverted in `644a5612`. Its subsequent diagnostic pool also failed:
`20261121-023744-82e4b1d6-a1`, source `5cd181b4`, info 1005.597 ms.
No unchanged acceptance retry followed.

### What the long host holds mean

Round 6 of strict soak `20261121-014417-85cb4bbf-a1` recorded a 288.008 ms
pump (287.998 ms thread CPU) with zero discovery submissions. That log did
not record the callback being retired, so exact retrospective attribution
is unavailable. It must not be described as measured discovery expansion.

The next CBC worker diagnostic added per-completion retirement timing:
`20261121-022906-2cd20225-a1`, source `155ee9b6`, still using the retained
array encoder. A 314.582 ms 200-peer pump spent 310.753 ms CPU / 310.844 ms
wall retiring **viu.tick**. The finer scene probe in
`20261121-023744-82e4b1d6-a1` (the subsequently rejected table candidate)
measured scene application as 114.187 ms CPU / 114.197 ms wall inside a
115.341 / 115.350 ms tick retirement (4-peer case); its early-failing
200-peer case measured 58.614 / 58.620 ms inside 60.110 / 60.167 ms.

`Host::pump` applies the model returned by the tick callback before servicing
later work. `SceneView::apply_model` validates the model budget, reconstructs
the binding set, reevaluates bindings, validates templates and controls,
and applies the frame model. This is host-thread scene work, separate from
the evaluator's callback CPU. The repeated tick-retirement holds and fine
scene timings make scene application the best explanation for the earlier
288 ms sample, **an inference rather than an exact attribution**.

The remaining failure is consistent with substantial synchronous work on
both threads: roughly 300 ms encoding, plus tick generation, scene
application and queued notices. It is not established as pure CPU starvation
and does not justify increasing the 1 s deadline. A repair needs a measured
scene/binding breakdown and encoder traversal profile, then a correctness
control for any reduced or deferred work. No speculative rendering change
is retained here.

The first helper submission `20261121-021915-82c7c21d-a1` did not reach tests:
lib-mix's build-time host vocabulary assertion rejected `_viu_test_cpu`.
Renaming the feature-gated hook to `viu_test_cpu` corrected the diagnostic.

Final source checks: CBC `20261121-023857-83be4ef1-a1`, source
`f4c9dc08`, BUILD_RC=0. Both negative controls fail at the required assertion
(cargo 101); fmt, all-target clippy, all 55 lib-viu targets (632 passed,
0 failed, 12 existing ignored), doctests (0 cases), counter contracts and
empty worker diff pass. All five changed Mix files lint cleanly.

Final-source standalone host attribution, CBC `20261121-024317-1c8dcd3b-a1`,
BUILD_RC=0, is a diagnostic, not pool acceptance. In the 200-peer case,
scene.apply viu.tick reached 222.952 ms CPU / 223.017 ms wall inside
224.437 / 224.502 ms retirement; pump max 226.884 ms, CPU max 226.833 ms.
Info CPU 294.449 / 370.460 / 370.460 ms; info wall
388.903 / 809.436 / 809.436 ms (n=21). Scene application therefore
accounts for almost all of a comparable long hold on the retained encoder.
Relevant code: `libraries/viu/src/app_host.rs:1058`,
`libraries/viu/src/view.rs:294`; test-only attribution at
`libraries/viu/src/app_host.rs:961`, helper probe fixture at
`libraries/viu/tests/workshop_bus.rs:15`.




## Files and checks

- `apps/workshop/discovery.mix:125`: resumable eight-service expansion.
- `apps/workshop/controller.mix:260`: advance on owner follow-up.
- `apps/workshop/work.mix:11`: sixteen-entry rotating admission scan.
- `apps/workshop/counter.mix:26`: equivalent exact-name wire predicate.
- `libraries/viu/src/app_host.rs:2535` and `libraries/viu/src/runtime.rs:131`:
  opt-in test-support probes, including thread CPU time; off in normal builds.
- `libraries/viu/tests/workshop_bus.rs:153`: bounded progress/fairness
  regression; `:418` retains the 1 s call and 2 s owner checks; `:450` runs
  the real 4/200-peer fixture, including malformed query refusals.
- `libraries/viu/tests/workshop-probe.mix`: reproducible CBC wire, owner
  comparison, negative controls, scoped checks and strict pool soak.

Earlier scoped validation, before the helper probes: CBC
`20261121-013259-c00b82a9-a1`, source `b303db87`, BUILD_RC=0:

- Both negative controls fail with cargo 101 at the intended assertion:
  old expansion gives "list expanded in its receive turn"; old scan gives
  "scanned the entire blocked queue". Worker files are restored.
- Existing exact-counter contracts pass, including opaque tool/permission
  JSON and u64 precision.
- cargo fmt and all-target clippy pass (existing warnings).
- Every lib-viu test target passes: 55 binaries including the example,
  632 passed, zero failed, 12 existing ignored; doctests pass (zero cases).
- Worker diff is empty. Later changes add diagnostic probes; production
  behaviour remains the measured bounds and array encoder.

Strict twelve-round full-crate width-four soak `20261121-014417-85cb4bbf-a1`
failed in round 6 after five green rounds (54 binaries each round, serial=0,
retries disabled). Info reached 1001.341 ms; pump 1.855 / 5.904 / 288.008 ms;
discovery completions 0 / 1 / 1. Completed info CPU was 295.967 / 314.156 /
314.156 ms, discovery job callback CPU max 11.782 ms. This is not twelve
green rounds. The later table experiment also failed and was reverted.
No further unchanged soak was run; PR #213 remains draft, not ready.

All five changed Mix files lint with zero errors, warnings or notes.
No tools/ci.mix all, install, desktop restart or production action was run.
Saved logs were read and the full evidence parsed, including expected
negative-control failures. Quantiles and per-target receipts are retained
in the private Workshop evidence directory.

---


> Current naming: Mix `term`, Rust reference `term-rs`. See [runtime and migration](docs/ports/term-runtime.md). The receipts below retain historical branch, job and installed-file names.

# PR #24 refreshed settlement — 2026-11-10

PR: https://github.com/mixviu/mixviu/pull/24
Branch: `feat/term-mix`. Validated source head:
`309f2e1e114d8815ef026620c97193b29666399b`.
This receipt-only commit follows that source; the PR exposes the final head.

Merged `origin/main` at `dcb6713431057d78029a3cbd03c49b6a8ddc4f25`
with merge commit `966cafabf894c16c50e7ff23ee5e1f0f4f2efba6`.
Conflicts: Cargo.lock took main's version, then CBC re-resolved the workspace;
last.md retained this PR's previous receipts and appended main's receipt.
All other files merged automatically, preserving both sides' intent.
SQLite remains rusqlite 0.41.1 / libsqlite3-sys 0.39.1.
No Rust Term code was changed during this refresh.

The existing bundle fix in this PR remains correct: apps.tar includes
apps/term runtime files under `share/mixviu/apps/term/`, the executable
`bin/term-mix` and `share/applications/mixviu.term-mix.desktop`.
The archive manifest declares prefix `/opt/mix`. The desktop Exec selects
`/opt/mix/bin/term-mix`; the launcher selects the bundled app at
`/opt/mix/share/mixviu/apps/term` with service `term-mix` and app id
`mixviu.term-mix`. Shared library relative imports remain in the same layout.
No further bundle edit or PR body update was needed in this refresh.

CBC jobs:

- `20261110-141653-56d15876-a1`: BUILD_RC=0. Workspace lock regeneration
  and rustfmt both rc 0. Applied printed lock patch locally; fmt patch empty.
- `20261110-141817-ddee3ef0-a1`: BUILD_RC=0 at the validated source.
  `tools/ci.mix all` passed, followed by `tools/cutover_gate.mix bundle`
  (both bundle steps rc 0), archive listing and archived content inspection.
  624 Rust result groups: 10,944 passed, 0 failed, 106 ignored.
  All 13 Term Mix suites ran. Environment registry: 199 names, 0 problems.
  Foreign gate clean; Mix lint: 0 regressions (existing diagnostics remain).

Saved logs read and checked, including expected negative-test diagnostics
(release test-hooks refusal, rejected attachment and invalid screencopy):

- `20261110-141653-cbc2-966cafabf894-20261110-141653-56d15876-a1-44a728ef.log`
- `20261110-141830-cbc2-309f2e1e114d-20261110-141817-ddee3ef0-a1-d32989a8.log`
  (1,233,912 bytes; SHA256
  `c145b1c5d0703b976f185ad03db1ed2adf2e85c6bb44bfea4da2b76ff318fe6a`).

CBC apps.tar: 46,807,040 bytes, SHA256
`98a9e865e3431f05b655ffdaaa8867b7374264e7da820b4af5fb61ebccbc5e86`.
Manifest source matches the validated source. The actual Term entries below
are relative to `/opt/mix`; Rust source and test scripts are excluded.

```text
-rwxr-xr-x 0/0             414 2026-11-10 14:18 bin/term-mix
-rw-r--r-- 0/0             214 2026-11-10 14:18 share/applications/mixviu.term-mix.desktop
drwxr-xr-x 0/0               0 2026-11-10 14:18 share/mixviu/apps/term/
-rw-r--r-- 0/0           13197 2026-11-10 14:18 share/mixviu/apps/term/actions.json
-rw-r--r-- 0/0            9183 2026-11-10 14:18 share/mixviu/apps/term/actions.mix
-rw-r--r-- 0/0           19172 2026-11-10 14:18 share/mixviu/apps/term/app.mix
-rw-r--r-- 0/0           45586 2026-11-10 14:18 share/mixviu/apps/term/bus-contract.mix
-rw-r--r-- 0/0            8718 2026-11-10 14:18 share/mixviu/apps/term/bus.mix
-rw-r--r-- 0/0            1473 2026-11-10 14:18 share/mixviu/apps/term/clipboard.mix
-rw-r--r-- 0/0           17221 2026-11-10 14:18 share/mixviu/apps/term/contract.json
-rw-r--r-- 0/0            2542 2026-11-10 14:18 share/mixviu/apps/term/controls.mix
-rw-r--r-- 0/0            3458 2026-11-10 14:18 share/mixviu/apps/term/core.mix
-rw-r--r-- 0/0             999 2026-11-10 14:18 share/mixviu/apps/term/discovery.mix
-rw-r--r-- 0/0             711 2026-11-10 14:18 share/mixviu/apps/term/event-job.mix
-rw-r--r-- 0/0            4427 2026-11-10 14:18 share/mixviu/apps/term/find.mix
-rw-r--r-- 0/0             477 2026-11-10 14:18 share/mixviu/apps/term/font-job.mix
-rw-r--r-- 0/0            3611 2026-11-10 14:18 share/mixviu/apps/term/font-persistence.mix
-rw-r--r-- 0/0            1499 2026-11-10 14:18 share/mixviu/apps/term/help.txt
-rw-r--r-- 0/0            2008 2026-11-10 14:18 share/mixviu/apps/term/lifecycle.mix
-rw-r--r-- 0/0           11035 2026-11-10 14:18 share/mixviu/apps/term/menus.json
-rw-r--r-- 0/0             214 2026-11-10 14:18 share/mixviu/apps/term/mixviu.term-mix.desktop
-rw-r--r-- 0/0           11628 2026-11-10 14:18 share/mixviu/apps/term/model.mix
-rw-r--r-- 0/0            1627 2026-11-10 14:18 share/mixviu/apps/term/notifications.mix
-rw-r--r-- 0/0             660 2026-11-10 14:18 share/mixviu/apps/term/notify-job.mix
-rw-r--r-- 0/0            2865 2026-11-10 14:18 share/mixviu/apps/term/protected-layout.mix
-rw-r--r-- 0/0            1534 2026-11-10 14:18 share/mixviu/apps/term/scene.mix
-rw-r--r-- 0/0             431 2026-11-10 14:18 share/mixviu/apps/term/show-job.mix
-rw-r--r-- 0/0            1375 2026-11-10 14:18 share/mixviu/apps/term/startup-policy.mix
-rw-r--r-- 0/0            1245 2026-11-10 14:18 share/mixviu/apps/term/startup.mix
-rw-r--r-- 0/0             414 2026-11-10 14:18 share/mixviu/apps/term/term-mix.mix
-rw-r--r-- 0/0            8334 2026-11-10 14:18 share/mixviu/apps/term/wire.mix

```

No deployment, installed-prefix writes, desktop/VT access or PR merge.
The coordinating session owns review, merge and the normal gold install on desk4.

---

Previous receipts follow.

# PR #24 settlement — 2026-11-10

PR: https://github.com/mixviu/mixviu/pull/24
Branch: feat/term-mix. Tested source head: `0743ce4d7214cd3612f380338ee28c1ce74df1aa`.
The final receipt commit follows this source revision; the PR exposes its head SHA.

Merged `origin/main` at `d17f7caab647954da3d2b05a7aa552e4e02fc461` without rebasing.
Only conflict: `last.md`; retained both the Term and main receipts below.
Cargo.toml and source merged automatically, preserving both sides. Per brief,
replaced Cargo.lock with main then regenerated it on CBC; rusqlite 0.41.1 and
libsqlite3-sys 0.39.1 remain. No Rust Term edits in this settlement.

Bundle fix: `tools/apps_bundle.mix` already packed apps/term runtime files,
but omitted the installed launcher and desktop entry. It now packs executable
`bin/term-mix` (0755), `share/applications/mixviu.term-mix.desktop` (0644),
and the existing runtime under `share/mixviu/apps/term/`. The launcher now
selects that actual bundle path. The shared library remains at
`share/mixviu/libraries/viu/mix/`, preserving relative imports. Documentation
and PR install paths follow the same layout. No new environment variables.

CBC lock/fmt job `20261110-065011-46868179-a1`: BUILD_RC=0; lock step rc 0,
fmt step rc 0 with empty formatting patch. Generated lock patch applied locally.
Saved log read in full: `20261110-065021-cbc2-67b5d010bf56-20261110-065011-46868179-a1-3b96b124.log`.

CBC job `20261110-065218-ac87e1de-a1`: BUILD_RC=0; tools/ci.mix all
rc 0 and cutover_gate.mix bundle rc 0 at the tested source. Saved log read
in full (1,113,444 bytes), SHA256
`08bf156955f05e654ae0f3a93097c08684c49e8e7fd22e014125da856e5f90e2`.
Log: `20261110-071956-cbc2-0743ce4d7214-20261110-065218-ac87e1de-a1-b8070d82.log`.
610 Rust result groups: 10,894 passed, 0 failed, 106 ignored. All 13 Term
Mix suites ran; env registry has 198 names and 0 problems; lint has 0
regressions. Existing warnings and explicit ignored cases remain.

CBC exact-source apps.tar: 46,796,800 bytes; SHA256
`1e50eb6d9805030ad1748f2a0b28dc49861dc3d378164e905ff716c39bdb2f57`.
The listing below is from this CBC archive; archived launcher and desktop
contents were printed and checked too.

Local native Mix bundle verification at the tested source returned rc 0.
apps.tar: 46,796,800 bytes; SHA256
`50faa103b637499ee3c8d33849448b725103b9eb02646c663c27003bc0a94f9a`.
Manifest source matches the tested source and prefix is `/opt/mix`. Archived
launcher and desktop contents inspected; test and Rust files excluded.
No local Rust build, /opt writes, desktop access, deployment or PR merge.
Opus owns the final review, merge and gold installation on desk4.

Actual Term apps.tar entries (relative to /opt/mix; runtime files 0644):

```text
-rwxr-xr-x 0/0             414 2026-11-10 06:52 bin/term-mix
-rw-r--r-- 0/0             214 2026-11-10 06:52 share/applications/mixviu.term-mix.desktop
drwxr-xr-x 0/0               0 2026-11-10 06:52 share/mixviu/apps/term/
-rw-r--r-- 0/0           13197 2026-11-10 06:52 share/mixviu/apps/term/actions.json
-rw-r--r-- 0/0            9183 2026-11-10 06:52 share/mixviu/apps/term/actions.mix
-rw-r--r-- 0/0           19172 2026-11-10 06:52 share/mixviu/apps/term/app.mix
-rw-r--r-- 0/0           45586 2026-11-10 06:52 share/mixviu/apps/term/bus-contract.mix
-rw-r--r-- 0/0            8718 2026-11-10 06:52 share/mixviu/apps/term/bus.mix
-rw-r--r-- 0/0            1473 2026-11-10 06:52 share/mixviu/apps/term/clipboard.mix
-rw-r--r-- 0/0           17221 2026-11-10 06:52 share/mixviu/apps/term/contract.json
-rw-r--r-- 0/0            2542 2026-11-10 06:52 share/mixviu/apps/term/controls.mix
-rw-r--r-- 0/0            3458 2026-11-10 06:52 share/mixviu/apps/term/core.mix
-rw-r--r-- 0/0             999 2026-11-10 06:52 share/mixviu/apps/term/discovery.mix
-rw-r--r-- 0/0             711 2026-11-10 06:52 share/mixviu/apps/term/event-job.mix
-rw-r--r-- 0/0            4427 2026-11-10 06:52 share/mixviu/apps/term/find.mix
-rw-r--r-- 0/0             477 2026-11-10 06:52 share/mixviu/apps/term/font-job.mix
-rw-r--r-- 0/0            3611 2026-11-10 06:52 share/mixviu/apps/term/font-persistence.mix
-rw-r--r-- 0/0            1499 2026-11-10 06:52 share/mixviu/apps/term/help.txt
-rw-r--r-- 0/0            2008 2026-11-10 06:52 share/mixviu/apps/term/lifecycle.mix
-rw-r--r-- 0/0           11035 2026-11-10 06:52 share/mixviu/apps/term/menus.json
-rw-r--r-- 0/0             214 2026-11-10 06:52 share/mixviu/apps/term/mixviu.term-mix.desktop
-rw-r--r-- 0/0           11628 2026-11-10 06:52 share/mixviu/apps/term/model.mix
-rw-r--r-- 0/0            1627 2026-11-10 06:52 share/mixviu/apps/term/notifications.mix
-rw-r--r-- 0/0             660 2026-11-10 06:52 share/mixviu/apps/term/notify-job.mix
-rw-r--r-- 0/0            2865 2026-11-10 06:52 share/mixviu/apps/term/protected-layout.mix
-rw-r--r-- 0/0            1534 2026-11-10 06:52 share/mixviu/apps/term/scene.mix
-rw-r--r-- 0/0             431 2026-11-10 06:52 share/mixviu/apps/term/show-job.mix
-rw-r--r-- 0/0            1375 2026-11-10 06:52 share/mixviu/apps/term/startup-policy.mix
-rw-r--r-- 0/0            1245 2026-11-10 06:52 share/mixviu/apps/term/startup.mix
-rw-r--r-- 0/0             414 2026-11-10 06:52 share/mixviu/apps/term/term-mix.mix
-rw-r--r-- 0/0            8334 2026-11-10 06:52 share/mixviu/apps/term/wire.mix
```

---

# Mix Term integration — 2026-11-08

PR: https://github.com/mixviu/mixviu/pull/24

Accepted lab/parity-term (tested 9ed90beb; receipt f5d7e1fa) is integrated with merge commits. Current main security migration 2ec0dca9 is included. Main child-window, raster, retained-reply and clipboard behaviour is preserved; authored no-window admission and native selection are opt-in. apps/term remains the shared bundle. The comparison launcher selects service term-mix, app id mixviu.term-mix and title Term (Mix). Rust Term is untouched.

Tested code checkpoint: 5f006b1aa99af248b58758c390c93df30ac78423. This status and CI-fixture receipt follows that checkpoint; no runtime source changes follow it.

- CBC term-mix-ci-5-a1 rc 0: tools/ci.mix all, including fmt, env, clippy, viud egui-free, all tests and Mix lint (0 regressions). All 13 Term Mix suites ran, as did Term bundle and the eight apps bundle/contract tests. Complete saved log loaded and checked: 602 Rust result groups, 10820 passed executions, 0 failures, 96 ignored. Warnings and ignored-test limits remain; the explicit smoke below runs separately. Log SHA256 1869237433624f42826fb84be41122413e3ec5178344b01c66b2a23580394061.
- CBC term-mix-smoke-6-a1 rc 0 at the same code: simultaneous term-mix and default term, HELP descriptors, native shell attachment and real output, last-tab reply delivery and both normal process exits. Full smoke log reviewed against its preceding complete log; SHA256 823a3352f20c7f41e69e648a4e83f52cb6c38dacd4e990cf5359c44a588fc6a1.
- CBC term-mix-foreign-5-a1 rc 0: 16 foreign crates listed, strict lib-mix/mix-shell/mix/mixd clean, shell ELF checked. Existing allocator debt follows the main package rename; retired debt was removed.
- CBC regenerated Cargo.lock and supplied formatting patches. After the latest main merge, term-mix-prepare-6-a1 rc 0 returned an empty lock/fmt patch.

The default CI interpreter is now rebuilt incrementally on reused CBC targets, rather than trusting an executable left by another commit. Earlier superseded full gates were cancelled and are not claimed as passes. GitHub run 42478516774 found three raster failures because its image lacked the required Adwaita Mono and Noto Color Emoji fonts. The workflow now declares adwaita-fonts and noto-fonts-emoji; raster assertions and application behaviour are unchanged. Final receipt-head checks are recorded in the PR.

Install/undo recipe (files, prefix layout, launcher and desktop entry): docs/ports/term-runtime.md and PR body. No installation, desk4/VT access or desktop restart performed. The receipt proves headless integration; the desk4 visual comparison remains an operator step.

branch feat/term-mix; head 5f006b1aa99af248b58758c390c93df30ac78423; PR https://github.com/mixviu/mixviu/pull/24; CBC term-mix-ci-5-a1 rc 0; smoke term-mix-smoke-6-a1 rc 0

---

# CI flakes 2 — 2026-11-10

Branch: `fix/ci-flakes-2`, based on `origin/main` at
`aaedc99d03a3c6c286fda4c3e292988c688c52f6`.
Tested code head: `e055f84615df0bf63c17f24980d4714440768db0`.
The receipt commit changes only last.md; the PR exposes its current head SHA.

## Audio: queued change mistaken for terminal exit

The source publishes per read, not per printf. The fixture repeats sink #5.
If the reader publishes one sink change, the test consumes it, and another
read publishes the duplicate, the queue is ready after Drop for an ordinary
change. The old assertion required an empty queue and did not inspect
`closed`. Drop sets cancelled before killing the group, then joins the reader;
it is not an early check for worker completion.

The replacement drains the now-stable queue after Drop, checks each remaining
batch contains the expected sink change and rejects any closed marker.
The original filtering, coalescing and child-reaping assertions remain.

Controlled proof: a FIFO-fed real cat child emits one sink line, the test
consumes its batch, then emits the duplicate and waits until that second read
has queued a batch before dropping the source. Restoring the old empty-queue
assertion fails (cargo rc 101); the corrected assertion passes (rc 0).

A natural EOF can pass the worker's cancelled check before a concurrent Drop
records cancellation; that is an already-observed stream exit. This is not
the cause with the still-live fixture. The public source_unwatch path removes
the pending slot BEFORE dropping the source, so even such a late terminal
callback is discarded. No product change is required.

## Lifecycle: boot trace corrupted by formatted fixture writes

The failing wait was for one boot record, before RELOAD, not a heartbeat or
commit. CI run [42581336097](https://github.com/mixviu/mixviu/actions/runs/42581336097)
contains:

```text
legacy-ready|{"pid":boot|{"legacy":11230,"managed":11231}|t=407.542488794
11230}|t=0
```

The helper's writeln!(File, format, pid) performs several writes. The citizen's
append landed between them, so parse_trace could never find a valid boot
record. Waiting longer cannot repair those bytes. The helper now formats the
whole readiness record before write_all, giving the regular append file one
complete short record per write.

Controlled proof: a Write adapter inserts a valid boot record after each
write. Restoring the old writeln! makes the parser regression fail (rc 101);
the complete-record helper passes (rc 0). This forces the CI interleaving.

The grace assertion still requires at least 1.5 seconds between candidate-done
and commit and verifies the old child is reaped. The heartbeat lives until the
citizen is stopped, rather than expiring after five wall-clock seconds.
At least one stamp must lie strictly inside the grace window: that proves
reactor progress; the previous minimum of five also imposed scheduler
throughput. A runner starved for the entire window can still fail this
wall-clock responsiveness test; deadlines were not lengthened.

## CBC evidence

All Rust commands ran on CBC. Saved logs were read. Two yes processes and
each cargo test process (including descendants) were pinned to the same
allowed CPU with taskset. Each original named test was invoked separately
50 times before and 50 times after; counters include every invocation.
Load processes were stopped in finally.

| Run | Audio pass/fail | Lifecycle pass/fail |
| --- | --- | --- |
| Before | 50 / 0 | 50 / 0 |
| After | 50 / 0 | 50 / 0 |

Both baseline loops passed: repetition alone did not expose the races.
Each controlled regression failed once with the old code and passed once
with the fix. The final module/binary run repeats both passing regressions.

| CBC job | Revision | BUILD_RC | Evidence |
| --- | --- | --- | --- |
| 20261110-002554-b937890f-a1 | aaedc99d | 1 | Load harness setup refused a stdout path string; no test ran. Corrected to native stdout null and requeued. |
| 20261110-002703-51923577-a1 | aaedc99d | 0 | Before: 50/50 each under load. |
| 20261110-003109-73e78118 | 78417d67 | 0 | CBC rustfmt diff, fixed regressions rc 0 each, old-code mutations rc 101 each. Own worker edits restored in finally. |
| 20261110-003329-79ce459c | e055f846 | 0 | Final checks and after loops: 50/50 each. |
| 20261110-002809-00858da7 | 3d39ac37 | 143 | Superseded queued probe, cancelled before running. |
| 20261110-003024-e567df98 | 78417d67 | 143 | Superseded queued probe, cancelled before running. |

Final checks: cargo fmt --all -- --check; cargo clippy --locked -p lib-mix
--all-targets; separately cargo clippy --locked -p mix --all-targets;
cargo test --locked -p lib-mix --lib desktop_events:: (17 passed);
cargo test --locked -p mix --test serve_reload_lifecycle (8 passed,
1 ignored subprocess helper); tools/ci.mix env (196 names, zero problems).
Existing unrelated deprecation/dead-code/clippy warnings remain.

Saved logs (under the maintainer's CBC log directory):

- `20261110-002703-cbc3-aaedc99d03a3-20261110-002703-51923577-a1-59246340.log`
- `20261110-003109-cbc3-78417d6752f8-20261110-003109-73e78118-aa20931d.log`
- `20261110-003329-cbc3-e055f84615df-20261110-003329-79ce459c-6326d611.log`

PR: https://github.com/mixviu/mixviu/pull/50
Pushed and opened through gh_agent as mixviu-agent.
No product changes, timeout increases, new environment variables, deployments,
desktop/VT interaction or installed-prefix writes.

---

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

# viud output identity — 2026-11-09

---

# SQLite security fix — 2026-11-09

PR: https://github.com/mixviu/mixviu/pull/49 (`fix/security-sqlite` -> `main`),
opened and pushed through mixviu-agent. Validated head SHA:
`23809a9d26fa97553ef9460dd614417baf4d4cc4`; this receipt-only commit follows it.
The PR exposes the current head SHA. Based on origin/main `95382ad1`.

- rusqlite: 0.38.0 -> 0.41.1; crates.io confirms its libsqlite3-sys ^0.39.0 requirement.
- libsqlite3-sys: 0.36.0 -> 0.39.1; linked bundled SQLite: **3.54.1**.
  The new files/sqlite_version test prints rusqlite::version(), asserts at least
  3.54.0, and checks SELECT sqlite_version() agrees.
- Targeted cargo update reused hashlink 0.12.2 and removed 0.11.1; no other
  registry versions changed. SQLite remains bundled; no new C crate or deny ignore.
- The old spamlite 0.9.6 pin also required rusqlite ^0.38. New pin
  `e4cda94713f696414550f414feaea4e8d6c4355e` is exactly one manifest change
  from `98f91a9d8142f92ae89ba4f070a165ce86fa5e8d`, on spamlite's retained
  fix/security-sqlite branch. No unrelated spamlite changes were imported.
- API adapters: consume column_names() directly as an iterator in shell stats;
  webd AuthGuard borrows the locked connection mutably for authorizer install
  and Drop cleanup. Queries use that guard's connection; policy, binding and
  schema isolation remain unchanged. Existing webd authorizer tests pass.
- Open PR overlap checked: #8 and #25 both touch Cargo.toml/Cargo.lock but
  neither upgrades SQLite. #25 references old hashlink 0.11.1; lockfiles may
  need regeneration after merge. Neither PR was edited.

CBC jobs (all worker any, CARGO_BUILD_JOBS=2; saved logs inspected):

- `20261109-203931-86119cbe-a1`: rc=101; found the pinned spamlite constraint.
- `20261109-210051-72e1ea03-a1`: rc=0; targeted dependency resolution;
  exact generated lock diff applied locally.
- `20261109-210123-3cce10fa-a1`: rc=1; fmt output applied, version test and
  cargo-deny passed; temporary foreign-gate binary-name adaptation corrected.
- `20261109-210242-a4c52dd8-a1`: rc=1; found webd's mutable authorizer API;
  version test, cargo-deny and adapted foreign gate passed.
- `20261109-210420-33f8ce8f-a1`: **BUILD_RC=0**, validated source above.
  Full `mix tools/ci.mix all`, explicit SQLite version test, `cargo deny check`
  and foreign gate each returned 0. Across 605 Rust test summaries: 10,614
  passed, zero failed, 100 ignored (includes the explicit version test).
  Mix policy checks and lint ratchet passed, zero regressions.

MixViu main lacks tools/foreign_gate.mix and tools/foreign.conf.mix. CBC used
temporary copies of MixOS's existing gate/policy with repository root adapted,
strict crates mix/mix-shell/lib-mix, and the inherited system allocator exception
mapped to the renamed mix binary. Result: 15 foreign crates, all listed;
strict closure and ELF checks clean. No committed policy change or new exception.

GitHub security-ok passed on the validated source. Code-owner approval and
cross-family review remain merge requirements. No deployment, VT interaction
or /opt writes.

---

# Quoin P0 — 2026-11-09

Production fix derived from origin/main on `fix/viud-output-identity`.
PR: https://github.com/mixviu/mixviu/pull/47 (opened and pushed as mixviu-agent).
Implementation head: `c6d7aba223d4bb3ee4951533cb2d15ef1e49f684`.
The final report commit changes only this receipt; the PR records its head SHA.

## Identity contract

`world::state::output_key()` now returns the backend connector/protocol name.
EDID fields are never runtime equality. `output_preference_key()` and
`MonitorIdentity::preference_key()` return `make model serial @connector:NAME`.
The connector qualifier is always present, avoiding identity changes when the
connected set changes. Two identical serial-42 panels remain separate, and
unplugging/replugging one cannot rename or consume the survivor's view state.
This deliberately binds new preferences to ports; physically indistinguishable
units cannot follow swapped cables. `MonitorIdentity::key()` retains the legacy
EDID format for compatibility and physical properties retain the actual EDID.

## Consumer audit

All paths below are relative to `engines/viud/crates`. Direct consumers and
transitive `current_output_key()` / `active_output_key()` consumers were checked.

| Consumers | Identity and reason |
| --- | --- |
| `world/src/state.rs`, `state/wire.rs`, `camera/pin.rs`, `canvas/draw/scene.rs`, `surface/interface/hit.rs`, `surface/interface/core/hit.rs` | Runtime: current/render/cursor output, independent view trees, camera, geometry and hit testing. |
| `world/src/comp/corners.rs`, `comp/session_lock.rs`, `comp/region.rs`, `comp/presentation.rs` | Runtime: select the exact output for corners, lock, region and frame source. |
| `frames/src/draw/present/{parked,callbacks,cursor}.rs`, `frames/src/scene.rs`, `scene/{hooks,layershell}.rs`, `hook/window/interface.rs` | Runtime: callbacks, cursor, fractional scale, render pass, layers and per-output surfaces. |
| `native/src/render/execute.rs`, `native/src/wire/{frame,session}.rs`, `native/src/context/display/apply.rs` | Runtime: redraw registration/release, render scheduling, capture, frame liveness and teardown. |
| `native/src/context/display/reconcile.rs` | Runtime for redraw, retire, live-set and cursor routing; preference for preferred/default/active/mode selection. Compare full preference identity when deciding whether immutable Output properties must be rebuilt on fail-over. |
| `native/src/context/display/mode.rs` | Preference for picker request/watchdog pipe resolution and snapshot updates; runtime for redraw release. |
| `native/src/wire/input/map.rs` | Preference for saved touch claims, converted to runtime before routing. Automatic matching and returned keys stay runtime. |
| `nested/src/scene/compose.rs`, `headless/src/frame.rs` | Runtime: render/capture/redraw routes. |
| `seat/src/pointer/{restore.rs,input/motion.rs}` | Runtime: pointer target, saved live cursor output and teleport destination. |
| `policy-host/src/{project,control,region,input,capture,chrome}.rs` | World identity calls stay runtime: select output for props, geometry, input and capture. |
| `recorder/src/interface.rs`, `graphics/src/capture/registry/source.rs`, `effects/src/host.rs`, `model/src/stats/registry/counter.rs` | Runtime: capture sources, effect state and per-output presentation counters. |
| `comp-model/src/{snapshot,diff,snapshot_tests}.rs`, `policy-host/src/{truth,edges}.rs`, `world/src/comp.rs`, `world/src/comp/geometry.rs` | Separate public `comp_model::snapshot::output_key(name)` slug, not the EDID function. Retained; public comp keys already derive from connector names. Mixed policy-host files retain these public-slug calls too. |
| `kms/src/connector/select.rs`, `native/src/assemble/display.rs`, `native/src/context/display/enumerate.rs`, `native/src/wire/entry.rs` | Preference: startup selection, per-panel modes, settings picker and snapshots. |
| `drivers/src/output/base.rs::build_teleport` | Preference for placement lookup/active flag; converts to connector runtime keys in the live layout. |

Current main has no persisted per-output scale, workspace assignment or scene
profile in OutputProfile (mode/active/touch only); KMS scale is explicitly global.
Viewport trees/cameras and scene/render worker buffers are live runtime state.
No new persistence subsystem was added for those future settings.

## Compatible migration

The preference identity helper tries exact qualified keys first, then the legacy
EDID-only key, independent of profile order. Existing mode/active/touch fields
are copied in full before a qualified profile is edited. The legacy entry stays
available for other ports and old versions; reads do not rewrite the document.
Mode/default/active startup and reconcile paths use compatible lookup.

Legacy teleport placements resolve to one deterministic matching connector,
preserving zone ids rather than copying a zone onto two runtime outputs.
Explicit qualified placements supersede legacy placements for that connector.
Persisted layouts keep preference identities; live placements use runtime keys.

Main currently creates one headless output. Its constructor now accepts an
index and assigns matching name/serial `headless-N`, rather than `Unknown`.
The real constructor is tested with indices 0 and 1; no multi-output CLI or
runtime topology feature is introduced.

## Tests and CBC receipts

- `world/tests/output_identity.rs`: duplicate nonzero serial 42, distinct runtime
  and preference keys, independent view state, real Smithay Space unmap/remap,
  survivor identity and reconnect stability; serialless/virtual identity cases.
- `kms/tests/module_paths.rs`: actual EDID serial-42 parse, unchanged stable
  EDID identity and distinct connector-qualified preference keys; retains zero
  serial and unreadable EDID coverage.
- `model/tests/output_identity.rs`: legacy JSON profile read, whole-profile
  inheritance, exact-over-legacy priority, mode/touch preservation and JSON
  round-trip with independent active flags for the pair.
- `drivers/tests/output_identity.rs`: preference-to-runtime layout conversion,
  unplug survivor, reconnect restoration, legacy layout and explicit override.
- `headless/src/wire.rs`: real synthetic constructor yields stable distinct
  `headless-0` / `headless-1` serials and both identity kinds.

CBC jobs (saved logs read; no local Rust execution):

1. `20261109-174328-ab93993b-a1`, cbc2, RC 0: initial formatting capture at
   `0993cbb0`. Superseded by the next complete specimen.
2. `20261109-174530-e7aba0b6-a1`, cbc2, RC 0 at `bf3825c4`:
   `cargo fmt --all`, print diff, restore this job's changes; then
   `cargo test --locked -p model -p world -p drivers -p kms -p headless --lib --tests`
   and `cargo check --locked -p viud --all-targets`. All ran successfully.
   Formatting diff applied locally; final source is c6d7aba2 (format/comments).
3. `20261109-174718-0ef39d70-a1`, cbc2, RC 0 at
   `c6d7aba223d4bb3ee4951533cb2d15ef1e49f684`: `cargo fmt --all --check`,
   `cargo test --locked -p world -p drivers -p model --test output_identity`
   (5 passed), and `cargo check --locked -p viud --all-targets`.

All new CPU regressions passed. Existing GPU/headless acceptance tests retained
their explicit ignores. No real-glass/GPU rendering acceptance, VT, desk or
installation changes were performed. This is a PR for production main, not a
lab promotion or deployment.

## PR 47 review round 1 — 2026-11-09

All ten requested items addressed on the same branch. PR remains
https://github.com/mixviu/mixviu/pull/47.
New tested implementation head: `5549936b1585d11e403566a4b19c2ac39b8e9f43`.
The following receipt-only commit appends this section; its SHA is the PR head.

1. **Headless migration:** `identity::find` and `matches` accept legacy
   `MixOS Headless Unknown` only for qualified headless-0. Exact new keys win;
   headless-1 cannot inherit it. Test:
   `model/tests/output_identity.rs::headless_legacy_maps_to_zero_only_and_exact_wins`.
2. **Inactive teleport target:** active filtering now precedes minimum-key
   selection. Test: `drivers/tests/output_identity.rs::legacy_placement_uses_smallest_active_matching_connector`
   disables DP-1 and proves the legacy zone survives on DP-2.
3. **Startup shadowing:** shared `identity::preferred_index` tries exact matches
   across every profile before any legacy match, retaining priority within each
   pass. KMS startup, hotplug target selection and all-inactive recovery share
   this rule. Test: `qualified_selection_beats_earlier_legacy_but_keeps_priority_within_pass`.
4. **Legacy touch claims:** `touch_index` retains an exact qualified claim first;
   for legacy duplicates it prefers the associated libinput output when it is a
   matching driven output. Otherwise the existing first-match order is preserved.
   Test: `legacy_touch_uses_association_only_within_matching_outputs` covers a
   matching association, absent/unmatched association, and explicit priority.
   This fallback is documented in code and the Bus contract.
5. **Port moves:** profile mutation APIs (`upsert_output`, `set_active`,
   `set_touch_device`, `set_default`) now require the complete connected
   preference-key list, including inactive panels, and call `sync_legacy_output`.
   A sole matching panel refreshes/creates the legacy fallback from the complete
   qualified profile; duplicates or unknown topology cannot refresh it. This
   allows a move to a new port to inherit the latest mode/active/touch fields.
   Test: `single_panel_writes_refresh_legacy_for_moves_but_duplicates_do_not`
   exercises all four APIs, checks a move, then duplicate and unknown-topology
   writes. No in-tree production Rust caller currently invokes these legacy
   profile mutation APIs (the repository search found only definitions/tests);
   future callers must supply live topology rather than infer it from saved
   profiles. Raw JSON file edits remain the editor's responsibility.
6. **Public format:** corrected the snapshot comment in `native/src/wire/entry.rs`.
   `docs/docs/spec/bus/commands.conf.mix` now describes the qualified `edid_key`,
   opaque echo-back requirement, legacy/headless migration and touch fallback.
   `rg` audit of `apps/` and `share/` found no `edid_key`, `@connector:` or
   `MixOS Headless` references; no app parser/constructor needs migration.
7. **Last separator:** `identity::stable` explains why it splits on the last
   qualifier. Test: `stable_splits_only_the_last_connector_suffix` uses an EDID
   model containing the separator text.
8. **Teleport caller proof:** `kms::edid::identity::connector_key` is now the
   shared function used by reconcile's `identity_key` and enumeration. Test
   `kms/tests/output_identity.rs::reconcile_connector_key_preserves_edid_placement`
   supplies a serial-42 EDID, constructs the Output, compares preference keys,
   and feeds that connector key through `build_teleport`, retaining the DP-2 zone.
9. **Connector name agreement:** `physical::create` now calls `connector_name`
   directly. A narrow `ConnectorInfo` trait exposes the same three getters on
   real DRM Info and CPU fixtures, allowing the actual constructor to be tested
   without a DRM device or unsafe fabricated struct. Test:
   `physical_output_and_connector_selection_share_names` checks DP, HDMI-A and
   eDP fixtures with `physical::create(...).name() == connector_name(...)`.
10. **EDID-less spelling:** shared `for_connector` deliberately preserves the
    historical Debug spelling in fallback serials (`DisplayPort-1`), while the
    qualifier and runtime name use canonical `DP-1`. Test:
    `enumeration_key_matches_output_even_with_debug_spelled_fallback_serial`
    proves enumeration's saved key equals the real Output preference key.

Round 1 CBC receipts (BUILD_RC and saved logs read):

- `20261109-181155-54b6c08d-a1`, cbc3, **RC 0**, source `e81180f4`:
  `cargo fmt --all`; affected model/drivers/kms/world/headless `--lib --tests`;
  `cargo check --locked -p viud --all-targets`. Captured formatting and the two
  test-only workspace lock dependencies, printed diff and restored this job's
  worker changes in the same job. Applied the captured diff locally.
- `20261109-181328-b0746f10-a1`, cbc2, **RC 0**, exact implementation head
  `5549936b1585d11e403566a4b19c2ac39b8e9f43`: `cargo fmt --all --check`;
  `cargo test --locked -p model -p drivers -p kms -p world --test output_identity`
  (**14 passed**); `cargo check --locked -p viud --all-targets`.

No local Rust commands, deployment, desk, VT or installed-prefix changes.
Existing GPU acceptance tests retain their explicit ignores.

## PR 47 review round 2 — 2026-11-09

All four requested items addressed on `fix/viud-output-identity`.
PR: https://github.com/mixviu/mixviu/pull/47.
New implementation head: `ce38baa18c38fbc5265735f98d8d7ddd497de90c`.
The receipt-only commit appending this section follows that implementation.

1. **Qualified touch override:** replaced the first-claim lookup and identity-only
   `touch_index` with `resolve_touch_claim`, which inspects the effective profile
   for each driven output. Any exact qualified profile decides its own claim,
   including None or a different device. Only outputs without an exact profile
   can inherit the legacy claim; association chooses among those candidates.
   Qualified positive claims take priority over inherited ones. When a saved
   claim exists but all matching live outputs override it, the native caller
   returns unmapped instead of reintroducing it through automatic association,
   size or EDID heuristics. With no saved claim, normal automatic routing remains.
   The Bus spec documents this boundary.

   Tests in `model/tests/output_identity.rs`:
   - `cleared_qualified_touch_does_not_inherit_legacy_on_duplicate_panels` uses
     the real setter to clear DP-2 with both panels connected, asserts legacy L
     still claims touch and DP-2's exact profile is None, then verifies association
     with DP-2 routes to eligible DP-1. It also covers association absent, only
     the overridden panel driven, all candidates overridden and another exact
     device claim.
   - `legacy_touch_single_panel_and_unqualified_duplicates_keep_their_routes`
     covers a single panel, duplicate panels without qualified profiles, associated
     and unassociated selection, an unclaimed device, and clearing the sole panel
     (which refreshes legacy and restores automatic mapping).
2. **Teleport cleanup:** removed the redundant `!output_active(...)` disjunct
   after the earlier active filter. The legacy placement, active-connector and
   unplug/replug tests continue to pass unchanged in behaviour.
3. **Complete topology API:** all four profile mutation APIs and
   `sync_legacy_output` now take the private-field `ConnectedOutputs` wrapper.
   Construction is explicitly named `from_complete_snapshot`; its API requires
   all connected preference keys, including inactive panels, never just the
   driven set or saved profiles. Debug assertions reject unqualified keys and
   repeated runtime connector names. The API explicitly states that shape checks
   cannot discover hardware omitted by the caller. Empty/unknown topology still
   cannot authorise shared legacy writes. Existing duplicate/unknown-topology
   mutation tests use the wrapper; two new debug-build panic tests prove the
   malformed-snapshot guards.
4. **Display flags verified:** `DisplayInfo.active` marks the primary connector,
   so its handle comparison is intentional and independent of saved preference
   identity. The saved active flag is exposed as `DisplayInfo.enabled`, already
   resolved through `profile::active` and `identity::find`. Kept those meanings
   intact and clarified the enumeration comment. Extracted `profile::active_in`
   as the exact lookup called by `active`; the new KMS regression
   `display_enabled_lookup_accepts_migrated_legacy_profile_and_exact_override`
   verifies a migrated qualified key inherits a disabled legacy profile and that
   a later explicit qualified enabled profile wins over it.

CBC receipts (BUILD_RC checked and saved logs read):

- `20261109-183151-fca1937b-a1`, cbc2, **RC 0**, source `8315126e`:
  `cargo fmt --all`, then the same round 1 suites:
  `cargo test --locked -p model -p drivers -p kms -p world -p headless --lib --tests`
  and `cargo check --locked -p viud --all-targets`. Worker formatting diff was
  printed and restored in that same job, then applied locally.
- `20261109-183303-d0250a55-a1`, cbc2, **RC 0**, exact implementation
  `ce38baa18c38fbc5265735f98d8d7ddd497de90c`: `cargo fmt --all --check`;
  `cargo test --locked -p model -p drivers -p kms -p world --test output_identity`
  (**18 passed**); `cargo check --locked -p viud --all-targets`.

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

Branch cutover/four-engines; head is this last.md commit (source 7fddb05); gate: full run held for parity, light checks green apart from cachyos filesystem cases; CBC last full cutover-gate-2d7e1c7, light 20261107-185355-f8c3d8cd, 20261107-185418-c5fe3256, earlier cutover-gate-5d1ec61, cutover-gate-43de632, cutover-removed-4755e9e, 20261107-111516-437fd7b1.


## Mix hygiene pre-PR change (2026-11-07)

The four local Cargo shortcuts now refuse before touching source or installed
binaries. CLI help and the manual point to the CI recipe and CBC. All four
native-session fault variables use the explicit debug-only test-hooks feature;
release builds reject it. The native PTY target requires that feature and CI
enables it. No dependencies or deployment changed.

Source 805db4c61b62812999081e9bb7eb39ab677fd7ab passed fmt, env and
mix-hygiene on CBC job 20261107-204950-4537e2c4: 29 tests passed, expected
release-feature rejection, 176 env names and zero registry problems. Full
worker log was read. An earlier base implementation also passed on cbc3, job
20261107-204601-09d3a4fd-a1. Intermediate rustfmt and help-test findings were
fixed. Full workspace all gate and independent review remain unrun.

Round 2 (source 53357d4): the full test gate failed only because
tests/owned_editor_pty.rs compiles editor/runtime.rs by #[path] and lacked the
new crate::test_hooks module (E0433). The release-profile panic in that log was
the intended negative check passing. Fix: the test includes test_hooks.rs. The
hooks now link a marker; mix-hygiene checks the hook build carries it and the
ordinary build does not, and the new release-mix stage refuses a release mix
that carries it. Full test gate 20261107-210937-ccb133a8-a1 (cbc3) BUILD_RC=0,
10,560 passed / 0 failed / 90 ignored; release-mix 20261107-214025-5234a7b5-a1
(cbc3) BUILD_RC=0, marker absent.

PR.md contains the review draft; no PR opened under the binding brief.

Branch fix/mix-hygiene; head SHA: this evidence commit (tested source 53357d480966881475a04647498bdf8723b724ce); gate: full test GREEN, release-mix GREEN (earlier fmt/env/mix-hygiene GREEN at 805db4c); CBC jobs 20261107-210937-ccb133a8-a1, 20261107-214025-5234a7b5-a1, 20261107-204950-4537e2c4; Mark: wire release-mix into the gold/release recipe, independent review and human approval before merge.

## Main cut-over record
Final lab merge (2026-11-08): origin/lab/gold-parity at 01912877 (eight accepted apps; Email 70ea474, Dopus c66fe3c; runtime e1bbea9). Conflicts took the lab side of libraries/viu (launch, app_host, chooser_host, Cargo.toml) with lib-viu/lib-mix put back, the Dopus Rust crates deleted as the lab retired them, and the lab Cargo.lock with the four-engine renames reapplied. Renames applied to the merged files: package names in the new Dopus/Email gate scripts, MIXOS_INSTANCE -> MIX_INSTANCE, MIXOS_COMP_SERVICE -> MIX_COMP_SERVICE, VIU_SERVICE -> MIX_VIU_SERVICE, /opt/mix, Dopus home ~/.local/state/mix/dopus, MixViu prose in the Dopus and Backdrop contracts. No new registry names. Kept as wire/data: theme value mixos and its MixOS label, the email token label mixos-email. apps.tar at 7bde4c0: 428 entries (share/mixviu 262), 46,138,946 bytes; fonts upstream, wallpapers from the release, none from gold (cachyos 20261108-105441-54343276). Light checks: fmt clean; cargo tree --locked -p viud ok; lib-viu --tests 263 passed, 3 failed (the known cachyos filesystem cases); dopus_contract, dopus_bundle, backdrop, app_ports pass at db16141.

Branch cutover/four-engines; head is this last.md commit (source db16141); full gate: the coordinator runs it on infra/pr-ci; CBC light 20261108-105611-7534e7c2, 20261108-104904-e4317f0f, 20261108-104841-c85e2532, 20261108-105441-54343276.

Branch fix/email-desk4-tree-table; tested commit 1e38766b9a8dfcc260a8f377162aaaa3c37ec19d; Email legacy account-tree startup FIXED; hot-apply apps/email/mail.mix + apps/email/surface.mix to /opt/mix/share/mixviu/apps/email/; CBC email-desk4-scope-r4-a1 rc0 (17/17), email-desk4-native-r5 rc0 (real maild before/after); no desk4/main/infra/pr-ci changes.


## PR #7 main synchronisation (2026-11-08)

Merged main 60cb127 into fix/mix-hygiene without rebasing or force-pushing.
The only conflict was last.md; both evidence records were preserved. Main
cut-over rules and the gated hooks coexist; the env registry has 194 names
and zero problems.

CBC 20261108-153105-ab2990de-a1 resolved Cargo.lock without changes and
regenerated the lint baseline using the merged tree mix. Counts only shrink
against main: 182 to 152 files and 1265 to 1211 diagnostics.

CBC 20261108-153201-d97092aa-a1 at source
7cb24b0b596befc40f05c55893e39124a5b463f7 passed fmt, env, mix-lint, full test
and release-mix (BUILD_RC=0). All 611 test result summaries have zero failures.
Both debug artefact checks and the release artefact check passed; the release
feature guard refused fault injection. The full 904311-byte saved log was read.

The historical scanner false positive and two notes quoting it have exact
fingerprint exceptions. Main scanner rules are unchanged. Pinned gitleaks
8.30.1 passes a full-history redacted scan. Only scanner exceptions and docs
changed after the tested Rust tree. PR.md and PR #7 contain current evidence.
Required GitHub checks are ci-ok/security-ok, verified separately on the final pushed head;
independent review and human approval are separate requirements.

Branch fix/mix-hygiene; head: this evidence commit (tested Rust tree 7cb24b0b596befc40f05c55893e39124a5b463f7); gate: fmt/env/mix-lint/test/release-mix GREEN; CBC jobs 20261108-153105-ab2990de-a1, 20261108-153201-d97092aa-a1; PR #7.


## Final CI fixture repair (2026-11-08)

GitHub run 42453072066 exposed a readiness/boot trace interleaving in the
existing serve_reload_lifecycle grace fixture. Formatting directly into a
File wrote fragments; the child now formats the whole readiness record
before write_all. All assertions and timeouts remain unchanged.

Source 2c80c0bf0ff115ce5d0bfa1985cdf062d3a8424b passed formatting and env
checks. CBC 20261108-161926-f3889961 built a fresh sibling mixd and passed
both lifecycle configurations (7 tests each; existing subprocess helper
ignored), plus ten further runs of the affected grace case. The full saved
log was read. The first focused CBC job 20261108-161651-31de42e8 did not
preserve Cargo failure details; the corrected job retained full output.
Production code is unchanged from the green full gate at 7cb24b0b.

Branch fix/mix-hygiene; head: this evidence commit (full tested tree 7cb24b0b596befc40f05c55893e39124a5b463f7, tested fixture 2c80c0bf0ff115ce5d0bfa1985cdf062d3a8424b); gate: fmt/env/mix-lint/test/release-mix GREEN plus lifecycle repeat GREEN; CBC jobs 20261108-153105-ab2990de-a1, 20261108-153201-d97092aa-a1, 20261108-161926-f3889961; PR #7.
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

## PR #46 review round 1 — 2026-11-09

Addressed GLM-5.3/DeepSeek v4-pro items on the existing branch. Source head tested: `44a81cfbe5a6d1f2f13c8386634b17500a84bce2`. The following receipt/development-staging clarification commit changes only documentation; the bot push's final head is reported in the session hand-off.

1. Preview trust: removed cwd ancestor search and production CARGO_MANIFEST_DIR fallback. Only the explicit VIU_PREVIEW_VALIDATOR override or `<running-binary-prefix>/share/viud/scripts/scene_preview.mix` is selected. Mix comes from that prefix's bin/mix, with its share directory supplied for catalogue discovery. Confirmed packaging in tools/apps_bundle.mix. The planted-ancestor regression executes from a downloaded-scene cwd and proves only the trusted validator runs; the planted marker is absent. Build-machine paths/native test interpreter references remain in cfg(test) fixtures only.
2. In-flight edits: retain the signature captured before preflight, never a new post-commit signature. Compare after commit and repeat on a difference, for filesystem and explicit library/all reloads. Four immediate passes bound continuous writes; a coalesced local task defers further retries. Tests inject a helper write in the batch host call after preflight, proving a second full commit on both entry paths. Singleton reload does not acknowledge a library signature.
3. Mixed batches/overflow: group refusal retains a fence for its dependants while filesystem processing continues for unrelated scenes and overflow rescans. The refusal response survives reconciliation. Post-commit spawn errors also leave the rest of the batch eligible. Tests cover all three; a singleton cannot bypass the standing shared-library refusal. Disable/removal still reaches normal reconciliation.
4. Strip diagnostic: inventory now explicitly publishes load_diagnostic (diagnostic already uses failure(), which prefers that field). errors() prefers that refusal and its nested upstream/helper diagnostic; an unrelated lint problem cannot override it. The integration regression drives the real Mix loader transitions through its notify adapter, reads the refused-lib publication, draws last-good plus the strip with egui and asserts the refusal text is painted without advancing the view revision. Host/process RPCs are fixture adapters, not a live desk.
5. Mount rollback: SceneMount is an immutable input receipt inside acceptance, not a mutable external mount object. Page/dialog/toast effects reside in SceneStore, including in a batch's cloned store. Internal APIs now take shared references, preventing receipt mutation. A two-scene batch with a newly mounted page first and a refused second scene proves pages, caller receipt, removals and notices stay unchanged.
6. Watch lifecycle: templates and custom libraries are watched recursively from an existing parent; custom lib watching is established even when the lib directory does not yet exist. The root parent watch is recursive too. Native tests replace the catalogue, edit its new inode again, create a late lib, then edit it again; each remains observable.
7. Cost/coalescing: path-prefix/ancestor/overflow checks precede library hashing. Pure scene edits do not hash helpers unless retrying a standing library refusal. Native filesystem batching plus unchanged-signature suppression coalesce duplicate saves.
8. Digests: accepted helpers use the same complete bounded SHA-256 read for signatures and check-cache entries. bounded_read already refuses cap+1 bytes; no oversized prefix is accepted. Fingerprints mark oversize for refusal, and the test includes a >256 KiB tail.
9. Symlinks: both passes use no-follow checks, including library roots. Fingerprinting marks a refused link and preflight rejects it; neither silently follows or skips it as valid. Regression covers link refusal.
10. Replies: docs/scene-documents.md and the registry document all three results, including rc10 with new revisions for post-commit behaviour faults. A spawn-failure regression asserts both revisions and the runtime diagnostic.
11. Strip routing/text: edge and toast clicks use the configured SCENES_SERVICE. The passive preview/error parser has no editor-action hint. Strip text and tooltip shorten absolute path tokens to basenames; covered by the surface regression.
12. Preview validation/transport: preview and install staging call the real check_library. Checker/lint output remains captured inside scenes_io.check; only the final scene source is printed by the validator. Preview regression uses a helper containing print (never executed or leaked), then a broken helper and asserts last-good remains.
13. Reload arguments/authority: scene scope requires a string name; lib/all reject name even if nil. Tests cover missing, numeric and inapplicable names; the request error path also guards its name lookup against non-string input. PR and scene docs explicitly defer caller authority to the comp-identity work's later steps.
14. Fan-out scope: scene documents use strict-data fence parsing and cannot call require (libraries/strict/src/lib.rs and libraries/scene/src/envelope.rs). Behaviour-only dependency selection is therefore valid; computed/transitive behaviour imports still use the conservative all-enabled-behaviour set. Documented in the scene spec.

CBC receipts (Rust only on CBC; formatting diff restored in its own worker job and applied locally):

| Job | Source | BUILD_RC | Result |
|---|---|---|---|
| 20261109-180656-91382df9-a1 | e6d913e6 | 101 | New surface test dropped egui texture deltas; test harness cleanup corrected. Other Rust tests passed. |
| 20261109-180846-e9c19d39-a1 | 8003928c | 0 | 419 loader assertions, lint, 104 host tests (5 ignored), binary check; CBC formatting diff applied locally. |
| 20261109-181135-b1281ec5-a1 | 44a81cfb | 0 | 420 loader assertions; scenes.mix/lib/scenes.mix/scene_preview.mix lint: zero errors/warnings/notes; cargo fmt -p viu -- --check; cargo test -p viu --lib: 104 passed, 5 ignored; cargo check -p viu --bin viu --locked. |

Final saved log read in full: `20261109-181135-cbc2-44a81cfbe5a6-20261109-181135-b1281ec5-a1-b546e3ff.log`. Existing dependency warnings remain; no new source warning was reported. git diff --check passes. No desk4, VT, /opt or production changes. No added runtime.rs work beyond the diagnostic service routing/hint hunks; P2 comp subscription/binding work remains untouched. Bot pushes use the existing feat/quoin-p1-dev-loop branch; no new PR.

---

Existing GPU acceptance targets remain explicitly ignored. No local Rust,
deployment, desk/VT or installed-prefix changes. All pushes used the bot.


## PR #7 current-main settlement (2026-11-10)

Validated head/source SHA: `9ad45200302f610533b701fd8ac9762e932cc88f`.
Merged `origin/main` at `d17f7caab647954da3d2b05a7aa552e4e02fc461`
into the shared `fix/mix-hygiene` branch; no rebase or force push.
The final evidence commit changes only PR.md and last.md after that source.

Conflicts and resolutions:

- `PR.md`: retained the hygiene description, then refreshed its current evidence;
  the main version described a different security PR.
- `last.md`: retained both historical evidence records.
- `engines/mix/tests/serve_reload_lifecycle.rs`: retained the main helper and
  regression test. They already implement the complete-record append fix.
- `tools/ci/mix-lint-baseline.json`: retained the main baseline unchanged.
  The merged gate reports zero regressions; no baseline regeneration needed.

Cargo.lock matches main exactly, including rusqlite 0.41.1 and
libsqlite3-sys 0.39.1. CBC `cargo update -w` and `cargo fmt --all` produced
no patch; the worker printed the diff and restored its checkout in the same job.

CBC receipts at the validated source:

- `20261110-065002-54b96c63-a1`, cbc2, **BUILD_RC=0**: lockfile resolution
  and rustfmt. Saved log read; no diff to apply.
- `20261110-065037-808eaae5-a1`, cbc2, **BUILD_RC=0**:
  `/opt/mixos/bin/mix tools/ci.mix all`. The complete 1,099,182-byte saved
  log was read and audited: 619 summaries, 10,681 passed, zero failed,
  100 ignored; 197 environment names, zero registry problems; 153 lint
  files, zero regressions. Clippy and the viud dependency check passed.
  The release-feature build failed at the intended profile guard, as required.
  The hook marker was present in the opt-in debug binary and absent in the
  ordinary debug binary. Screencopy client errors were passing refusal tests.
- `20261110-065038-039c7d15`, **BUILD_RC=75**: release check did not run;
  the queue wait expired. Saved log read. Requeued as a check with a longer wait.

- `20261110-072134-e547727e-a1`, cbc3, **BUILD_RC=1**: the wrapper
  hit the Mix default 30-second process timeout during Cargo compilation.
  No scan ran. Saved log read; corrected the wrapper with an explicit
  14,400-second timeout and inherited compiler output.

Release artefact verification: pending corrected CBC job
`20261110-073834-988e97f8`.

No desktop access, installation, production deployment or PR merge performed.
Opus retains independent review and the protected merge gate.

---
# viud agent seat opt-in — 2026-11-10

PR: https://github.com/mixviu/mixviu/pull/56 (against main, stacked on #54; unmerged).
Branch: feat/viud-agent-seat-optin. Base 496f627b is preserved.
Tested source head: 44dc62b2bf93b4c2f55fca735f874b2e067f50ca.
The final receipt commit changes only last.md; the PR exposes the current head.

## Design choices

- Keys: agent_seat.default=false, agent_seat.allow=["viu"], agent_seat.deny=[], applied by viud, page applications/agent-seat. Deny wins, then allow, then default. Lists use the registry's directory-name grammar for executable basenames (255 bytes, up to 256 entries).
- Current policy lives in comp-model::agent_seat behind a shared RwLock. The existing settingsd consumer sends it through the generation-fenced chrome Update. The shell change comparison includes agent_seat.*; settingsd publishes the new prefix topic and Prefs subscribes to it.
- Per-client decision lives in protocols::seat::state::Seat.agent_clients: Arc<Mutex<HashMap<ClientId,bool>>> shared with the seat filter. The first decision stays fixed for the connection; subsequent filter calls prune dead clients. Input checks and window snapshots read that decision. Credentials identify the executable through /proc/<pid>/exe; app IDs and directories are not policy inputs.
- Headless tests have ONE injection: MIX_TEST_VIUD_AGENT_SEAT_ALLOW, comma-separated executable basenames, read exclusively by the headless backend and registered in etc/env.conf.mix. A settingsd snapshot supersedes it. Test support allows the probe/app binaries and its own executable. In-process testkit clients always receive the seat with no override. Real sessions use settingsd only.
- Windows report agent_seat; named targets and output hit tests refuse seat_unavailable with the seat, window and setting to change. No fallback to seat0. Docs explain that policy changes apply the next time the app starts.

## Validation

- 20261110-144808-b0fca6c2-a1: **BUILD_RC=0**, full tools/ci.mix all at the tested source head. Saved log audited: 620 summaries, 10,688 passed, zero failed, 101 ignored; 198 environment names with zero problems; 153 Mix lint files with zero regressions. Rustfmt, clippy and the viud dependency check passed.
- 20261110-151842-e9270fac-a1: **BUILD_RC=0**, real render-node headless capture (9/9) and input (7/7), without settingsd. This includes the one-seat Firefox regression and seat_unavailable replies for named key/pointer targets and output hit tests. Built mix/mixd and set MIX_TEST_MIX_EXE to the source-built interpreter.
- The renderer-free external-client test passed at the same SHA: denied sees only seat0, allowed sees both. Existing testkit clipboard and interactive suites passed in the full gate.
- Rustfmt was run on the worker and its printed diffs applied locally. Cargo.lock's dispatcher dependency change also came from the worker.

Earlier CBC attempts, all resolved or superseded:
- 20261110-141334-80bc4248-a1: wrapper BUILD_RC=0; nested check=101 (missing map type annotation), fixed.
- 20261110-141541-819a736c-a1: BUILD_RC=101 (missing WaylandFocus import), fixed.
- 20261110-141725-0718304c-a1: BUILD_RC=1 (nested default 30-second timeout during compilation). Corrected subsequent wrappers to use explicit timeouts.
- 20261110-141941-b7a0ec7f-a1 and 20261110-142450-906ee564-a1: BUILD_RC=143, explicitly cancelled as superseded. The latter exposed the missing settings topic route, fixed before the green full gate.
- 20261110-150848-f8642d43-a1: BUILD_RC=101, private broker startup failed with the installed interpreter; corrected invocation passed as above.
- Unstarted headless jobs 20261110-141952-456c8a60, 20261110-142501-3890a14a, 20261110-144745-0b9827a4 and 20261110-150813-5d3fe0d9 were explicitly cancelled as their invocations were superseded.

## Left undone

- Prefs GUI deferred under item 6: the current list controls select fixed catalogue entries; executable allow/deny editing needs a new editor. No partial page was added. All keys remain writable through settings.get/settings.apply; docs include the Bus shape and restart behaviour. The page and its restart note need a second PR.
- Pre-existing app-test fixture is absent: headless/tests/app.rs requires examples/ced-headless-probe and a ced Cargo package, neither present in this checkout. That ignored test could not be run against exact-source artefacts. No stale worker binary or alternate app was substituted.

No installation, deployment, desktop/VT changes or merge performed.

---

Previous hand-backs follow unchanged.

# Dependency bundle — 2026-11-10

PR: https://github.com/mixviu/mixviu/pull/53
Branch: `deps/2026-11-10`.
Base: `a0a5bef3a9d7b6b34aa33cf5a9209f9fe1469587`.
Pushed source head SHA tested by the full gates:
`5948680add0ab1fd53142f9fd3c89fc9a69b33a3`.
The subsequent receipt commit changes only this file; the PR exposes its current head SHA.

Supersedes #18 #19 #20 #25. All four remain open; no merge or closure performed.
PR author verified as `app/mixviu-agent` (`is_bot: true`).

## Versions

The table lists resolved versions; tungstenite and SCTK already had both
versions on main, so their direct dependency requirements changed while the
older transitive lines remain. PR #25's current lockfile resolves arc-swap
1.9.4, beyond the 1.9.3 in its description. SQLite stays on main's versions.

| Package | Before | After |
| --- | --- | --- |
| hyper | 1.11.1 | 1.12.0 |
| unicode-normalization | 0.1.25 | 0.1.26 |
| arc-swap | 1.9.2 | 1.9.4 |
| blake3 | 1.8.6 | 1.8.7 |
| serde_json | 1.0.151 | 1.0.154 |
| thiserror | 1.0.69, 2.0.20 | 1.0.69, 2.0.21 |
| tokio | 1.53.3 | 1.54.1 |
| uuid | 1.24.1 | 1.29.0 |
| bitflags | 1.3.2, 2.13.1 | 1.3.2, 2.13.2 |
| cc | 1.4.3 | 1.11.0 |
| crossbeam-channel | 0.5.16 | 0.5.17 |
| proptest | 1.11.0 | 1.12.0 |
| crc32fast | 1.5.1 | 1.5.4 |
| indexmap | 2.14.1 | 2.14.2 |
| jaq-core | 3.1.1 | 3.1.2 |
| jaq-std | 3.0.3 | 3.0.4 |
| unicode-width | 0.2.2 | 0.2.3 |
| log | 0.4.34 | 0.4.35 |
| zeroize | 1.9.0 | 1.9.1 |
| tokio-util | 0.7.19 | 0.7.20 |
| candle-core | 0.10.2 | 0.11.1 |
| candle-nn | 0.10.2 | 0.11.1 |
| candle-transformers | 0.10.2 | 0.11.1 |
| tokenizers | 0.22.2 | 0.23.2 |
| tungstenite | 0.26.2, 0.29.0 | 0.26.2, 0.29.0 |
| smithay-client-toolkit | 0.19.2, 0.20.0 | 0.19.2, 0.20.0 |
| rusqlite | 0.41.1 | 0.41.1 |
| libsqlite3-sys | 0.39.1 | 0.39.1 |

## Repairs and foreign-code inspection

SCTK 0.20 requires RawModifiers in update_modifiers and a repeat_key callback.
Both toolkit and Viu layer handlers now accept the new argument and forward
compositor repeats as repeated presses, stopping the local repeat timer first.

candle-nn alone failed compilation: Device/DType came from candle-core 0.10.2,
while VarBuilder came from 0.11.1, and candle-transformers expected the old
builder. Align core/nn/transformers to 0.11.1 and tokenizers to 0.23.2.
No update was dropped.

Base has no tools/foreign_gate.mix. Inspected changed registry crates through
locked metadata and their actual build-script sources. No newly introduced
crate compiles C/C++, and no new -sys crate was introduced. Existing foreign
debt remains, including blake3's cc build edge and onig. Tokenizers defaults
stay disabled with esaxx_fast off; its existing onig feature stays enabled.
SCTK 0.20 was already locked on main. Workspace cargo tree -i confirms desktop
consumers; scoped inverse trees for mix and mix-shell report no matching
package. Their full normal/build trees contain no desktop crate.

cargo deny check passes without policy changes. It reports wildcard and
yanked-dependency warnings, unused licence allowances and now-unused zip
advisory exceptions. The new Candle, tokenizers and fancy-regex versions were
included in the final foreign audit.

## CBC receipts

All jobs used --repo mixviu --worker any --lock-wait 7200 at pushed source
commits. No rc75 occurred. Resolution jobs printed their full diff, restored
their own worker tree, and the printed patches were applied locally.

| Job | SHA | BUILD_RC | Result |
| --- | --- | --- | --- |
| 20261110-075257-3d6b46c2-a1 | a0a5bef3 | 1 | Resolver harness used reserved function name; no edits ran. |
| 20261110-075313-b4ead392-a1 | a0a5bef3 | 0 | Resolve requested bundle, print and restore diff. |
| 20261110-075438-d1b9024f-a1 | fa8adaac | 1 | Audit harness treated absent SCTK in Mix as failure; fixed harness. |
| 20261110-075446-52822086-a1 | fa8adaac | 1 | Full gate caught SCTK keyboard API breaks. |
| 20261110-075516-0be8e8c3-a1 | fa8adaac | 0 | Foreign audit and cargo deny check. |
| 20261110-075656-dbfba9a3-a1 | 17c06497 | 1 | Full gate caught mismatched Candle tensor and builder types. |
| 20261110-075745-73d0af8d-a1 | 17c06497 | 0 | Align embedding stack, print and restore diff. |
| 20261110-075833-ad85bf35-a1 | 5948680a | 0 | Full ci.mix all passed: 10,640 Rust tests passed, 100 ignored; app checks and lint ratchet passed. |
| 20261110-075835-7a306a11-a1 | 5948680a | 0 | Final foreign audit and cargo deny check passed. |
| 20261110-080720-35129d8b-a1 | 5948680a | 0 | Full ci.mix all with explicit CARGO_BUILD_JOBS=2 passed: 10640 Rust tests passed, 100 ignored; app checks and lint ratchet passed. |

Earlier full gates used the CBC default, which caps jobs automatically only
for gold builds. A final explicit CARGO_BUILD_JOBS=2 full-gate rerun supplies
the required cap within both heavy workers' release_jobs limits.

Live Wayland/compositor repeat behaviour and real model inference were not
tested. Existing hardware tests remain ignored. No deployment, installed
prefix, desktop or VT changes. Code-owner and independent agent reviews are
pending; no review approval is claimed.

Final saved logs read:
- `20261110-075834-cbc3-5948680add0a-20261110-075833-ad85bf35-a1-fe1707d8.log`
- `20261110-082608-cbc3-5948680add0a-20261110-075835-7a306a11-a1-74ceae22.log`
- `20261110-082616-cbc3-5948680add0a-20261110-080720-35129d8b-a1-dd628ada.log`
