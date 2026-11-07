# Backdrop port inventory

Oracle: MixOS `8779f3328`, `apps/backdrop/{src,crates/wallpaper}`, its README
and `docs/backdrop/README.md`. The latest acceptance record is at the end of
this file; earlier checkpoints are retained as historical evidence.

| Original capability | Round 1 status |
|---|---|
| Background layer, all outputs, fractional scale, output hotplug | Ported through shared retained layer backend; native single-output headless capture verified |
| Solid RGB and fallback colour | Ported in Mix document projection |
| PNG/JPEG/WebP images; fill/fit/centre/tile/stretch | Ported through shared bounded image-fill; all five modes captured and inspected against original |
| Per-output document override with default fallback | Ported in Mix; actual headless-output override captured; simultaneous multi-output acceptance deferred (reference headless backend exposes one output) |
| `backdrop.ping`, `.state`, `.set` | Ported, model-ready replies; original wire envelope differs |
| Settings wallpaper image, colour, fit, same_on_all, outputs | Ported Mix follower; native Bus settings witness change captured and inspected |
| CLI config and default overrides | Ported config path as first bundle argument; original flags deferred to avoid a second parser before host contract settles |
| Strict schema, output/image limits | Ported Mix validation; unknown keys and unsupported sources refuse |
| Decode-before-commit and durable store on set | Deferred: generic image widget decodes asynchronously; this specimen reports model acceptance, not original transactional image acceptance |
| Theme colour / gradient first stop | Ported gradient first stop; exact live theme-token colour deferred until shared appearance integration |
| Slideshow folder/list, independent schedules, shuffle, crossfade, next | Deferred: requires Mix scheduling and explicit shared texture-budget/transition primitive; unsupported documents refuse |
| Live train views, queues/history/gate, alerts, 2D/3D, visibility leases | Deferred outside wallpaper specimen; no synthetic replacement |
| Action registry, macros, executed events and original probe CLI | Deferred until generic host exposes common app lifecycle and action contracts |
| Original 256 MiB resident/texture budgets and 3840 downscale | Deferred parity; current per-image limits are 32 MiB input, 8192 sides, 128 MiB decode, 16 MP output; retained decoder mailbox bounded |
| Idle zero polling, failed registration exits | Deferred parity: settings refresh currently one-second timer; Bus disconnect stays observable rather than terminating host |
| Presented revision/frame counts and original error/refusal envelopes | Deferred: model acceptance and native presentation are separate; generic tree/capture endpoints available |

The table above is the historical round-one inventory, not the current verdict.

Per-app Rust added: **0**. Shared native additions: retained layer backend,
generic bundle host, image-fill renderer/decoder, widget admission binding.
No installation or production acceptance follows.

## Gold survey — 2026-11-07

Contract source: `8779f33284b7d3f1178c3d57a699efb9b0867b22`, inspected
`apps/backdrop/src/{main,control,program,views}.rs`, the wallpaper schema,
and the app README and `docs/backdrop/README.md` at that revision.

Backdrop has **no chooser or settings window** at this revision. `main` starts
only `LayerOptions::background`, or the standalone `--probe` layer program.
There is no ordinary app window to decorate. The shared window frame therefore
does not apply to this surface; adding one would diverge from gold. Wallpaper
selection in Prefs belongs to that app. Backdrop's live train overlay is a
separate required state, not a chooser.

Corrections to the round-one inventory (gold wins):

- Resident and texture budgets are **512 MiB each**, not 256 MiB. Inputs are
  32 MiB, headers at most 16384 per side and 32 Mpx, retained sides at most
  3840. PNG, JPEG and WebP are accepted by the pinned decoder; older prose
  saying PNG/JPEG only is stale.
- A present fill requires exactly one source; an omitted default is theme.
  An empty fill, colour with fit/fallback, invalid slideshow options and wrong
  types refuse. Four resident sources count static paths once and each
  slideshow independently; slide lists have at most 4096 entries.
- Gold owns `backdrop.next` and `backdrop.view.{set,list,cycle}` as well as
  ping/state/set, HELP/app.describe and debug controls. The train consumes
  queue/history/gate and retained alert service contracts. These are required.
- Gold does not expose `backdrop.follow`. Round one's extension is not gold
  Bus acceptance. Settings wallpaper projection is a separate existing port
  facility, not evidence of gold's transactional set contract.
- `--config`, `--colour`, `--image`, `--fit`, `--service` and isolated `--probe`
  are the gold launch contract. A positional config is a port extension.

Checkpoint: layer/frame applicability surveyed; **parity remains unaccepted**.
No new capture or CBC result is claimed by this source-only checkpoint.

## Document and retained-raster checkpoint — 2026-11-07

This is an implementation checkpoint, **not parity acceptance**. No Fable
deviation is proposed and no item below is waived.

- `document.mix` implements the pinned strict schema, canonical colours and
  default fit, slideshow defaults/options, 4096-entry lists, 64-output names
  and four-source accounting. Explicit null for required/defaulted non-Option
  fields refuses. A missing default projects without a nil-field access.
- `launch.mix` parses config/colour/image/fit options, resolves relative paths
  without requiring existence, overrides only the default and revalidates the
  combined source limit. `app.mix` reads the native app config location or
  required `--config`; a positional config is no longer the launch contract.
  Probe **rendering is still missing** and explicitly refuses.
- `schedule.mix` owns pure independent traversal/deadlines, deduplication,
  shuffle without cycle-boundary repeats, singleton sleep, and failed-entry
  skipping while retaining the visible selection. It is **not yet wired to
  the running wallpaper**. Tests do not substitute for rendered slideshow
  acceptance, crossfade, native timers or `backdrop.next`.
- Invalid `backdrop.set` values now receive rc 10 / ARGUMENT without changing
  the projected model. Successful replies remain the old model-only envelope;
  durable transactional set and the full gold state envelope remain missing.
- Shared `raster_decode` retains gold's decoder, downscale, folder discovery
  and tests with provenance in `provenance/backdrop-raster.md`. Shared
  `Rasters` exposes in-process retained handles to Mix root/Jobs evaluators.
  CPU and texture leases account separately against 512 MiB, including held
  renderer references and replacement overlap. `image-fill` can draw handles
  without reopening a source and uses original dimensions for native sizing.
  New layer output views receive the same store. **Backdrop does not yet use
  these handles**; it still uses the old asynchronous path projection.

Remaining work is required: transaction/worker cancellation ownership,
supersession and durable store, complete command descriptors/state/replies,
slideshow controller and crossfade, live theme and event-driven settings,
probe, train 2D/3D/queues/history/gate/alerts/visibility, simultaneous outputs
and hotplug, exact release artifacts and all matched captures, whole/region
diff inspection, Bus/Prefs witnesses and owned-process cleanup.

Current visual evidence is historical six-mode AE=0 only; no new SSIM,
regions, output/hotplug or live Prefs acceptance is claimed here.

Exact checked code: `d825c2ae665c1d6fe06d73708242f84bb3682774` on
`lab/parity-backdrop`. CBC `20261107-090116-633bcd47-a1`, cbc3,
`mix tools/backdrop_parity_gate.mix`: **BUILD_RC=0**. Four Mix suites,
eight lint checks, rustfmt check, 13 raster-related tests (including the
renderer source-removal/surface-release witness), three app-port tests and
normal Clippy all pass. No worker diff. Complete saved log read; existing
Clippy warnings were not suppressed. No release artifact was built by this
scoped checkpoint and no native compositor process was launched.

Earlier receipts: `20261107-085433-7d251078-a1` failed on the embedded
interpreter's nil-field access (fixed); `20261107-085523-b9e3c76c-a1`
passed the document checkpoint; `20261107-085822-29188e7c-a1` passed the
raster tests and returned formatting; `20261107-085920-7963d5d5-a1`
passed formatting/tests/Clippy; `20261107-090016-dbccaea6-a1` refused
formatting of the new renderer witness (fixed without changing assertions).

## Round two — 2026-11-07: historical, superseded

This checkpoint was **NOT ACCEPTED**. Its live-Prefs question is now resolved
by Mark's standing mandate, “Live Prefs beats gold”: retain the live follower
as a Mark-mandated improvement and compare pixels at matched settings.
Round three below replaces the open acceptance findings in this section.

Tested source: `d34633db087f1e6d6f3281d5f5e207a65057af99`, branch
`lab/parity-backdrop`. This section and `last.md` are a receipt-only
documentation checkpoint after that source. Gold remains the pinned full
revision above. No per-app Rust, deployment, installation or VT action.

### Current implementation inventory

| Gold capability | Implementation and evidence | Acceptance still required |
|---|---|---|
| Frame / chooser | Surveyed gold: background and standalone probe only; no chooser or ordinary window. Shared window frame is inapplicable. | None invented; Prefs owns wallpaper selection. |
| Strict document and launch | Mix schema/CLI, gold limits, canonicalisation; six Mix suites include launch/document/controller cases. | Native CLI failure/help/exit-code matrix. |
| Transactional set | Mix controller holds native replies, decodes in Jobs before commit, stores durably, preserves applied document/pixels on decode refusal, fences superseded scope ownership. | Native supersession, storage failures and all error envelopes. |
| Retained image fill | Shared bounded gold decoder and fit geometry, retained raster/texture leases, opaque compositing, original dimensions, all five image modes. | Fractional scale and resource-starvation native witnesses. |
| Slideshow | Mix independent deadlines, folder/list selection, shuffle, bad-entry skipping, manual next, half-second crossfade and old-image ownership; native Runtime/Jobs/Calls test drives next and bad entries. | Compositor captures of transition endpoints and timing; complete gold reply audit. |
| Train | Mix-owned 2D/3D scene, bounded history, queue/gate/rejection models, independent native service reads, stale response fence, retained alerts, visibility and animation leases. Shared declarative plot/cuboid widgets. | Populated deterministic fixtures, real train/build services, alerts, native input and history legend repair. |
| Probe | Mix standalone offline launch variant, generic native configure/presentation/input events and forwarded JSON output. | Native probe capture/input/CLI acceptance. |
| Multi-output / removal | Shared store across output views; native removal hooks and renderer-budget ownership tests. | Simultaneous output/hotplug not witnessed. Exact reference limit below. |
| Bus | All 12 HELP descriptors match gold exactly; ping/state/set and train view switches exercised through native Bus. Held transaction receipts work. | Complete arguments/refusals/replies/topics/action-executed and debug matrix. |
| Live Prefs | Generic layer host follows native settings session; same-PID live dark-to-light change captured. | Fable decision: gold does not follow this change. Other appearance keys and authority recovery unaccepted. |

### Exact release and native recipe

CBC release artifact: `/build/es-artifacts/d34633db087f1e6d6f3281d5f5e207a65057af99/viu`,
receipt `viu.json`, SHA256
`5df196a34e6ddf6ccc96ffa3f80ba6fff6eda39d0f9fb0c3749b78fcb292187c`.
Fetched bytes were checked against the source/profile/hash receipt before
the release evidence run. Gold backdrop SHA256:
`3501c2d6d867689d873758b84284e777fa5f68a4db094c5fae36bfba83c01bac`.
Gold reports `backdrop 0.1.2 (8779f33284b7, built 2026-11-04T19:10:04Z)`.

Private archive: `backdrop-parity-20261107/pair.6PjeW0` under the private
hub's `_var/`. It retains `executed-pair.mix`, `executed-compare.mix`, the
immutable source archive/hash, binary receipt, `receipt.json`, `bus.jsonl`,
native state and settings readbacks, HELP, PNGs, ten `compare-*` directories,
`comparison-index.json`, `inspection-sheet.png` and `cleanup.json`.
The 96×64 deterministic gradient/transparency fixture SHA256 is
`0390d6fc9c7f4c2513790f00fb0c67d87450bf1568899fa2dbf10a6e98e6b5de`.
All application Mix files were extracted from the tested commit; source
archive SHA256 is
`531686163177b055cddbab89ab8593bceeed38d67ccf989b02a1019f0b59720e`.

An owned native noded broker, pinned compd and private settingsd used separate
home/config/data/cache/runtime/settings/auth roots. Gold then candidate used
the same canonical `backdrop` identity against those services. Captures are
1040×680, scale 1, Studio dark / comfortable, output `headless-0`; light was
applied live only at the final state. No live desktop identity or Prefs was
used. Native state was polled until pending cleared and the active revision
was presented before capture. Train services were deliberately absent for
the captured unavailable state, not replaced with fabricated data.

Both applications were still alive on their original PIDs at the light
capture (gold 2945883, candidate 2946263). Settings readbacks report resolved
light at revisions 3 and 5 respectively in the same authority incarnation.
Candidate stderr is empty. Gold stderr contains its normal Bus-connected
notice. `cleanup.json` confirms all five owned app/service/broker PIDs exited.

### Whole-image and region inspection

ImageMagick 7.1.2-35 reports **normalised SSIM distortion**, with zero
identical and lower better. Raw metrics, version, image dimensions and
hashes remain in every `comparison.json`. No arbitrary threshold is used.

| State / region | Rectangle x,y,w,h | SSIM distortion | RMSE distortion | Inspected finding |
|---|---|---:|---:|---|
| Colour; fill; fit; centre; tile; stretch; theme dark, each full/wallpaper | 0,0,1040,680 | 0 | 0 | Seven pixel-identical states, including all six wallpaper modes. |
| Train 2D and 3D, header | 0,0,1040,132 | 0 | 0 | Exact unavailable header and error text. |
| Train 2D and 3D, queue labels | 0,132,1040,34 | 0 | 0 | Exact. |
| Train 2D and 3D, scene | 0,166,1040,324 | 0 | 0 | Exact empty 2D plot / 3D rails. |
| Train 2D and 3D, history | 0,490,1040,190 | 0.0000246615 | 0.000493558 | Only the rounded background corners of the “Now” legend differ; unapproved and still open. |
| Train 2D and 3D, full | 0,0,1040,680 | 0.00000689071 | 0.000260892 | Same legend difference. |
| Live light, full/wallpaper | 0,0,1040,680 | 0.418086 | 0.887584 | Gold stays dark; candidate follows light. Fable decision required. |

Original train images, wallpaper pairs, side-by-side/difference figures and
the ten-state sheet were inspected. The legend is not silently classified
as hinting and the live-theme difference is not accepted. Earlier private
`pair.wHauXr` captures used the debug artifact at `5893c425f99f`; those
preliminary metrics agree with the release run but do not attest release
bytes. `pair.jzOvOS` independently records gold-only live Prefs behaviour.

### Behaviour and compatibility findings

The native release suite sent the same set/mode/view requests to both apps.
An unreadable image returned rc 10 / IMAGE and preserved active revision and
applied config. HELP descriptor arrays compare equal. Native controller
tests additionally check same-path replacement rereads pixels, durable source
document retention, failed-slide skipping, manual next and fading ownership.
Pure Mix tests cover independent schedules, supersession and stale gate data.
These checks do not substitute for the remaining full Bus matrix.

One observed state difference remains: the port's train `outputs[].shows`
reports the underlying theme rather than gold's `train`, and its view-frame
counter is not yet gold-compatible. Action executed payloads and invalid-view
error wrapping also need the complete native comparison before acceptance.
Probe/native input, populated service data, large histories and timed
slideshow compositor witnesses remain required after the Fable ruling.

### Exact multi-output limit

Pinned compd help offers `--headless`, `--nested`, `--size` and `--scale`,
but no output-count option. Its headless `Contract` owns one `Output` and
creates only `headless-0` (`services/compd/crates/headless/src/wire.rs`).
The nested `WinitWindow`/`WinitContract` also each own one `Output` and create
only `winit` (`services/compd/crates/nested/src/window/factory.rs` and
`wire/entry.rs`). Neither reference backend supplies the requested
two-output/hotplug capture path. The native witness here is exactly one
1040×680 scale-1 headless output. Multi-output/hotplug is **unverified**, not
passed on the strength of shared renderer tests; no replacement compositor
or transport was introduced.

### CBC receipts

Final exact-source gate `20261107-094413-6a3e7ac6-a1` on cbc3:
**BUILD_RC=0**, 647.2 seconds, no worker diff. Complete saved log read:
`20261107-094413-cbc3-d34633db087f-20261107-094413-6a3e7ac6-a1-c479a8c5.log`
in the CBC client cache. Command: `mix tools/backdrop_accept_gate.mix`:
exact rustfmt check, six Mix
suites and lints, debug/release artifact handoff, raster/fit/controller tests,
normal Clippy, complete `cargo test --locked -p viu -p viu-engine`, and
`git diff --exit-code`. Warnings are preserved, not suppressed. Complete
Viu unit tests report 85 passed; engine library tests report 98 passed and
five existing ignored cases; the separate native GPU process gate is also
ignored without its required binaries. All other executed integration tests
pass. Ignored cases are not claimed as passed and do not grant app parity.

Earlier scoped green jobs: `20261107-093016-40e41e4e-a1` at `10c7f3c848ab`
and `20261107-093600-9a0a97f4-a1` at `5893c425f99f` (prepare plus artifacts).
The latter saved log was read completely; its formatting patch was applied
in `f956b3b`, then the exact-source gate was queued. Earlier repair jobs
`20261107-091234-61e25917-a1`, `20261107-091344-f4f0f262-a1`,
`20261107-091520-25a66611-a1`, `20261107-092022-526852b7-a1` and
`20261107-092516-7fdeaf27-a1` failed on caller syntax, directory return type,
test route, plot borrowing and held-callback NO_REPLY respectively. Those
failures were repaired without weakening assertions; returned formatting
was applied.

## Acceptance — 2026-11-07, round three

**ACCEPTED against the pinned oracle, with its single-output capture limit
recorded below as requested.** No outstanding implementation item or Fable
question remains. Simultaneous physical outputs/hotplug is not claimed as a
native compositor pass.

Mark resolved the round-two Prefs question in the standing brief: **Live Prefs
beats gold**. The follower remains a Mark-mandated improvement. The pinned
app loads `Theme::load()` once; the port accepts live settings revisions.
Visual comparisons therefore use matched resolved settings, including a
fresh-start gold light capture. This is not an outstanding Fable decision.

### Repairs and current feature inventory

| Capability | Implementation and acceptance path |
|---|---|
| Frame and chooser | Gold has background and standalone probe layers only. There is no chooser/settings window or client frame to port; Prefs owns selection. |
| Wallpaper and document | Mix retains the complete source document; shared gold-derived decoder retains PNG/JPEG/WebP, all five image fits, alpha/fallback compositing, 3840 retained-side and 512 MiB CPU/GPU budgets. Strict schema, four-source accounting, folder discovery and 4096 slides remain covered. |
| Transactional set | Jobs decode before commit, held replies settle once, store precedes application, scope closure fences superseded work, same-path replacement rereads bytes. Decode/store refusals preserve active revision and pixels. |
| Slideshow | Independent schedules, folder/list discovery, deterministic traversal policy, shuffle cycles, singleton sleep, failed entries, manual next, half-second outgoing-image ownership and timed advancement are wired into the Mix controller. |
| Train | Mix owns the complete header, queues/rejections, 2D/3D scene, worker data, bounded history, gate stage/failure, zoom/render/service selection and alerts. Cheap queue/worker/history reads continue behind wallpaper; gate reads require a visible train. Stale replies are fenced by the selected service/history key. |
| Presented state | Native output readbacks report train content and consecutive view-frame counts. Train retains the last presented wallpaper revision while hidden slideshow revisions advance. Counts remain truthful native readings. |
| Probe and launch | Offline probe uses native configure/presentation/input; app-authored help, service validation, startup syntax/semantic exit codes and mandatory Bus registration live on the generic host. |
| Bus and topics | All twelve gold descriptors and command names, successful/refused replies, debug controls/paging and v2 action-executed events. View replies survive nested native sends; nullary set/next events carry null params/result. No gold app properties or macro commands were found; the earlier inventory's macro claim was too broad. |
| Live Prefs | Same-process settings follower covers the toolkit inputs and app overrides. Accepted appearance is retained separately on authority loss; active key/input values still clear. Recovery accepts the new authority snapshot. |
| Outputs and resources | Per-output overrides/default fallback, shared retained handles, native removal hooks, fractional scale and budget release/wake tests. The exact compositor limit below constrains simultaneous-output capture. |

The history legend's round-two corner difference was a settings mismatch:
gold's startup theme resolves the large radius to 12. The matched session
sets `tune.style.radius_lg=12`; there is no Backdrop radius constant or pixel
patch. Train rendering continues to use reusable native composition,
plot and cuboid widgets. All app policy remains in-process Mix.

### Exact source, bytes and native evidence

Exact code is `407e4617d0f0dad682e8111b580e7358f7a84e63` on
`lab/parity-backdrop`. The commit containing this record is receipt-only;
it does not change the tested code. Gold is
`8779f33284b7d3f1178c3d57a699efb9b0867b22`, backdrop SHA256
`3501c2d6d867689d873758b84284e777fa5f68a4db094c5fae36bfba83c01bac`.
Candidate release `/build/es-artifacts/407e4617d0f0dad682e8111b580e7358f7a84e63/viu`
has SHA256
`bf8d2387b2d1f43efedb5d0149a2cd714274b2c920491eb0cc321e519c1e712f`.
The fetched source/profile/hash receipt was verified before the native runs.
The extracted Mix source archive SHA256 is
`334fe175a57a98118c9e7c361fe5208c160ce9545ff15e692b22427c234c1df0`.

Private archive paths below are relative to the private hub's
`_var/backdrop-parity-20261107/`. Each run has owned native noded/compd/settingsd,
separate home/config/data/cache/runtime/settings/auth, canonical `backdrop`
used sequentially, source archive, Bus requests/replies, PNGs, dimensions and
hash receipts. No desktop identity, install or VT restart was used.

- `round3-final-recipe/pair.O2JKhK`: full native feature/Bus/Prefs run,
  1040×680 logical/physical, scale 1, Studio dark/comfortable, matched large
  radius 12; 30 comparison sets, five inspection sheets, native probe input,
  real service readbacks, 17 Prefs captures, authority recovery and CLI cases.
- `round3-scale-final/pair.0WiSou`: 1040×680 logical, 1560×1020 physical,
  scale 1.5; eleven matched wallpaper/theme/train/probe comparison sets.
- `round3-settled-recipe/pair.TCi63W`: three further scale-1 populated train
  captures. Both apps ran within the same minute (recorded epoch values,
  minute 29900234), and alert clear settled for one second. No clock or
  application data was overridden. These close the in-flight animation and
  minute-dependent graph differences in the first run.

Every comparison retains `regions-*.json`, raw metrics/tool version,
gold/candidate crops, side-by-side and difference figures. Originals and
figures were inspected; presentation sheets are made only after metrics.
The reusable driver refuses unequal dimensions. ImageMagick 7.1.2-35 reports
**normalised SSIM distortion**: zero is identical, lower is better.

### Whole-image and region findings

| Capture group / region | SSIM distortion | RMSE distortion | Finding |
|---|---:|---:|---|
| Scale-1 colour, five image fits, dark theme, output override/default fallback, JPEG/WebP, five slideshow states, before/after store refusal, probe before/after input, matched light: 21 full pairs | 0 | 0 | Exact pixels. |
| Scale-1 populated train 2D/3D and settled alert clear: three full pairs and every header/queue/scene/history/legend crop | 0 | 0 | Exact, including large-radius legend corners and all 3D geometry. |
| Scale-1.5: all eleven full pairs and every region | 0 | 0 | Exact six wallpaper modes, default theme, unavailable train 2D/3D, probe and click. |
| First run's six populated/retained/stale-alert train pairs, full | 0.0000002125 | 0.0000708241 | Only the minute-dependent zero-history endpoint; header/queues/scene are exact. |
| Those pairs, history / legend crop | 0.000000760527 / 0.0000159195 | 0.000133986 / 0.000613008 | Endpoint enters the crop below the legend. Same-minute repeat is exact; no remaining corner mismatch. |
| First alert-clear scene / full | 0.00588948 / 0.00197492 | 0.0234844 / 0.0135987 | Captured before the 600 ms scene animation settled. One-second settled repeat is exact. Original figures retained. |
| Real service train 2D/3D, full / history | 0.0000398855 / 0.000142748 | 0.00227034 / 0.00429505 | Actual time-relative history readings. Header/queue/scene/legend are exact; timestamps remain truthful under the standing rule. |

Fixture train regions are header `[0,0,1040,185]`, queues
`[0,185,1040,77]`, scene `[0,262,1040,228]`, history
`[0,490,1040,190]`, legend `[880,521,160,59]`. Cleared-alert header is
161 high. Real-service header is 107 high, queues end at 160, scene ends
at 490. Fractional-scale unavailable-state crops and all wallpaper/probe
rectangles are retained in their own region maps. No score threshold or
unexplained visual difference was waived.

### Functional, Bus, topics and Prefs results

`audit-result.json` records **48/48 compatible requests across all twelve
commands**, including invalid view arguments, nullary shapes, document
refusals, debug toggles, invalid limits and recent-event paging. Structured
replies match after explicitly accounting for native frame counts, actual
debug events/cursors and the port's ten additional real logging targets.
These are truthful runtime data, not copied gold values. HELP/app.describe
descriptors match. Successful set/next/view operations and held failure
replies are additionally exercised by the feature sequence.

All seven consumed topics were driven: `train.landed`, `train.rejected`,
`buildd.started`, `buildd.done`, `pipeline.alert`, `pipeline.clear`,
`pipeline.alerts`. Retained replay, stale alert rejection and clear are
visible in captures. The sole gold publication is
`backdrop.action.executed`; 61 distinct observed v2 events cover all four
action IDs in both apps, nullary params/result, view params/result,
per-run ordering, bus origin and zero drops. Gold uses two registry run IDs;
the port uses one. Duplicate observations from two independent subscribers
are deduplicated by run/sequence. There are no gold app properties to port.

The twelve CLI cases include exact help, invalid service/flags/missing
values/fit/probe combinations, semantic failures, missing config and occupied
service registration. Both apps return the required 0/1/2 exit codes without
timeouts. Agent-seat pointer move/press/release changes the offline probe's
native click counter and output at both scales, with exact captured pixels.

Slideshow captures cover first/manual/timed advancement, half-second fading
ownership, bad-middle refusal with unchanged applied state followed by skip,
and bad-first startup skipping. Hidden slideshow advancement preserves
train's last wallpaper presentation revision and increments native view
frames. A forced atomic-store refusal returns STORE and preserves pixels,
source document and active revision. Shared Runtime tests additionally drive
same-path replacement and held decode refusal; Mix suites cover supersession,
independent/shuffled schedules and bounded/stale train history/read deadlines.

Real daemon testing runs unmodified native `train.mix` and `buildd.mix`
against seeded private SQLite state, paused train and zero configured workers.
No production jobs are submitted. Script SHA256s are respectively
`32158ea65c0854b6a95170a6351f15a61df9ef46309ef3ceb8dfa15bd0d2e5c6`
and `b1a4b56ded5aa2e357348cd79d289cd5eed0d617fd131d5b0b26aa077ad26c84`.
Both apps read queue/history/gate/workers and draw queued/landed/rejected
data in both renderers. Deterministic fixtures separately populate busy gate
and worker states; their executed service source/hash is retained.

Candidate PID **3129696** accepts all seventeen live changes without restart:
light/dark, scheme, style, high contrast/app contrast, accent, density,
roundness, font family/size/weight, mono family, accessibility text scale,
icon stroke, app text scale/mode, auto override and tune radius. Every capture
waits for the accepted settings revision. Images were inspected for geometry,
typography and clipping. Invalid density refuses. Lost-authority and recovered
readbacks are observed; all three before/lost/recovered PNG hashes equal
`b0711c35545482312ba416b02b102083d0476518af8caf755f0f7ea7a4333418`.
Fresh-start gold light and candidate live light are pixel-identical.

### Reference limit and cleanup

The pinned headless backend has one `headless-0`; nested has one `winit`.
Neither has an output-count or output-hotplug test path (source locations in
the round-two limit survey above). The actual native witnesses are one output
at scales 1 and 1.5, actual-name override and absent-name default fallback.
This is the exact limit requested by the brief, not a claim that simultaneous
physical outputs or hotplug were exercised. Native retained-store/removal,
fractional fit, scope cancellation and resource-lease tests pass. One output
cannot exhaust the 512 MiB wallpaper texture budget with its two retained
3840² RGBA textures (112.5 MiB); budget overflow/release and renderer removal
are tested at the reusable native boundary rather than by lowering gold's cap.

`cleanup.json` confirms all 19 main-run PIDs and all seven fractional-run
PIDs exited, including broker; `managed-children.jsonl` verifies owned service
children stop as well. The settled run has its own cleanup receipt. All
earlier failed/debug runs remain historical evidence, never release attestation.

### CBC receipts and repair history

Final exact-source job **20261107-110154-76f0252d-a1**, cbc2:
**BUILD_RC=0**, 554 seconds, `mix tools/backdrop_accept_gate.mix`, no worker
diff. Saved **2734-line complete log read**, including preserved warnings:
`20261107-110154-cbc2-407e4617d0f0-20261107-110154-76f0252d-a1-c0b158d8.log`
in the CBC client cache. Seven Mix suites, app lint, exact rustfmt check,
debug/release artifact handoff, 23 raster tests, four controller tests,
two app-port tests, five session tests, normal Clippy and complete Viu/engine
tests pass. Viu unit tests: 85 passed; engine library: 98 passed, five ignored.
The separate native GPU process gate is also ignored. Those six existing
icon/offscreen/GPU requirements are not counted as passed.

Prepare **20261107-105350-b68c6ffc-a1** returned the test-fixture formatting
applied in the tested commit; its complete saved log was read. Earlier green
round-three full gates include `20261107-100151-171edf00-a1`,
`20261107-101948-4d96792b-a1`, `20261107-103724-d48be212-a1` and
`20261107-104949-61bad727-a1`. They are repair witnesses, not this release's
attestation. Failed CBC repairs `20261107-101322-a874fc60-a1` (callback-local
include resolution), `20261107-101613-7cde2bdc-a1` (duplicate existing regex
dependency) and `20261107-102804-cc529125-a1` (egui test API) were corrected.
The first appearance-retention repair passed unit tests but failed native
authority loss (`round3-recipe-5/pair.E01BON`); separate accepted appearance
state fixes that actual clearing behaviour. No assertion was weakened.
