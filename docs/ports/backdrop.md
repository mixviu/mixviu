# Backdrop port inventory

Oracle: MixOS `8779f3328`, `apps/backdrop/{src,crates/wallpaper}`, its README
and `docs/backdrop/README.md`. **Overall: NOT ACCEPTED.** The round-two
record below supersedes the historical implementation statuses.

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

## Round two — 2026-11-07: Fable decision required

**NOT ACCEPTED.** Retaining live Prefs is a proposed deviation from the
pinned gold application. The options and evidence are in `last.md`; no
answer or waiver is assumed. Further functional acceptance remains open.

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
