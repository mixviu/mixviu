# Backdrop port inventory

Oracle: MixOS `8779f3328`, `apps/backdrop/{src,crates/wallpaper}`, its README
and `docs/backdrop/README.md`. This is a bounded lab port, not parity acceptance.

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
