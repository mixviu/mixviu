# Quoin P2 lab hand-off

Branch: `lab/quoin-p2` (P0 base, pushed; no PR or merge).
Branch head before this hand-off commit: `39e76473b00f12f23f589174623a1d51e17ffb16`.
The hand-off commit changes only this file; resolve the tip with `git rev-parse HEAD`.
Source under final CBC test: `113ae03bf2460c702e4edae318ce0f835bf4f7eb`.

Implemented the four prototype paths: generation-fenced `publish_patch`,
recursive whole-model path diff with atomic host batch, a separate pure `$src`
root with shared Mix clock/comp/audio/net sources, and compositor snapshot
refresh from watch events rather than the successful-connection 1 s poll.
The 50-property conformance test passes: one changed leaf evaluates one
dependent, whole-source publication evaluates 50, unrelated model writes zero.

Mix: 397 loader assertions, 8 path assertions, 6 source-distribution
assertions; lint with deny-warnings passes. No desk4, VT or `/opt` changes.

First panel measurement, debug CBC, actual shipped panel renderer with empty
authored tasks/tray, 10 warmups + 100 samples:

| Path | Re-evaluations | Patch median/p95 µs | CPU frame median/p95 µs |
|---|---:|---:|---:|
| whole model | 31 | 520.127 / 540.418 | 334725.266 / 375178.335 |
| source leaf | 1 | 402.445 / 415.739 | 333926.766 / 341936.713 |

Binding work falls; frame cost does not materially improve. Includes content,
template and image preparation. No GPU/presentation timings, live hotplug or
captured live-workload oracle: desktop access was excluded by the task.

CBC jobs (saved logs read):

- `20261109-155011-d6675eca-a1`: **BUILD_RC=0** at source `113ae03bf246`.
  All 179 executed Rust tests passed; 6 environment/GPU tests ignored.
  Only documentation changed after this tested source. Confirmatory panel:
  whole-model patch median/p95 520.717/551.426 µs, frame
  335143.767/372471.570 µs; source leaf patch 401.067/419.580 µs,
  frame 335261.925/341868.347 µs. Re-evaluations remain 31 → 1.

- `20261109-154110-c5b70fcb-a1`: rc 101, unseeded source fixture.
- `20261109-154238-b4e9c5f2-a1`: rc 101, same fixture before fix.
- `20261109-154427-68167e04-a1`: rc 101 overall; 73 scene and 100 host tests
  passed, 5 host tests ignored. Failed inherited registry owner filter:
  P0 says viu, test expected viud. Corrected in final source.
- `20261109-154623-904f3210-a1`: rc 0, CBC rustfmt diff applied locally;
  worker edits restored in the same job.
- Superseded queued formatting request `20261109-154437-92aa66e3` never ran;
  local client ended rc 143. A separate cancel request could not release
  the submitting client's interest; it was terminated while still queued.

The specimen's detailed shapes, failed approaches and proposed two-PR split
are in `docs/2026-11-09-quoin-p2-lab.md`. Production needs an independent
whole-specimen survey, source defaults/availability at load, coherent snapshot
and sequence semantics, async provider reads, watch release/retry, and one
dirty-set union per batch. No claim of production readiness.

P3: preserve native stable comp keys; define ordered bounded projections for
repeat and `instance.output` for per-output instances. Share one source owner
and one behaviour across those instances. Initial output snapshot and
add/remove/gap semantics must be specified before derivation.

Overlapping P1 hunks: scenes.mix handlers/publication wrapper, loader
model/reload, runtime worker and store receipts. P3: ResolvedScene/source
root/dependency policy, template context, metadata and panel document.
Do not merge the lab branches to resolve these overlaps.

## P2 release frame attribution follow-up (2026-11-09)

Measurement source: `28064c5cf3ac1c1f3aef475d82eec780e8bfd142`.
Driver: `tools/quoin_p2_frames.mix`, CBC heavy worker selection `any`,
`CARGO_BUILD_JOBS=2` explicitly (within either heavy worker release cap).
Same empty authored task/tray panel, 1920 x 52, 10 warmups and 100 samples
per whole-model/source-leaf path. Two runs: profile disabled, then enabled.
All stage instrumentation is cfg(test), opt-in and thread-local; no
production behaviour is repaired. Inclusive stages must not be added twice.
Tessellation is timed separately after the original CPU-frame boundary;
neither run includes GPU upload, compositor presentation or desktop I/O.

The original gold source has a headless scene-host layout assertion for this
panel, but no matching clock-mutation timing harness was found in its
scene-host tests. No gold performance number is claimed and no gold path
was installed, restarted or touched.

### Release results

Branch head before this report commit:
`4f053102d6066c36df0961507cb39dc1b4a521f5` (pushed). It differs from the
measured source only by CBC rustfmt changes. The report commit changes docs
only; `git rev-parse HEAD` gives its final tip without a self-referential hash.

Unprofiled release, same original timer boundaries, µs:

| Path | Re-evals | Patch median / p95 | Frame median / p95 |
|---|---:|---:|---:|
| whole model | 31 | 124.663 / 141.230 | 124943.506 / 132204.858 |
| source leaf | 1 | 94.730 / 122.092 | 124731.637 / 133019.091 |

Release reduces the earlier debug frame from ~335 ms to ~125 ms, which is
still far too expensive. P2 saves **29.933 µs (24.0%) of patch time**, about
0.024% of a frame this size. The 0.17% difference between frame medians is
smaller than their observed variation; there is no meaningful frame win.
These are CPU-path elapsed times, including synchronous filesystem lookup,
not a CPU-cycle profile or a presented-frame latency measurement.

Profiled release frame median/p95 was 124661.098/131319.888 µs for whole
model and 125073.837/131670.860 µs for source leaf. Attribution does not
change the overall conclusion. Source-leaf breakdown below uses **means**
(100 retained frames) so nested durations can be subtracted correctly:

| Stage | Mean µs | Scope |
|---|---:|---|
| Image preparation | 125868.551 | inclusive; four failed icon lookups |
| Layout | 128.140 | Taffy tree construction, text measurement and layout |
| Template preparation | 95.106 | binding recompilation and list preflight |
| Content replacement excluding images | 46.509 | clone/Arc, local-state checks, assignment; difference of means |
| Paint shapes | 33.530 | egui draw commands and interaction |
| Other egui run work | 23.664 | egui_run minus layout and paint; difference of means |
| Tessellation | 4.597 | extra; outside the original frame timer |

`replace_content` is 125915.060 µs inclusive of images; `egui_run` is
185.334 µs inclusive of layout and paint. Do not add these parents to their
children. Theme access totals 0.294 µs/frame, nested in the draw work.
Image preparation consumes ~99.74% of the measured replacement + template
+ egui path. The top three **disjoint frame stages** are therefore image
preparation, layout and template preparation, not binding evaluation.

### What repeats, and evidence

1. **Four failed icon resolutions dominate.** Each replacement repeats
   `dev.mixos` (35.579 ms mean), `preferences-system-notifications-symbolic`
   (30.378 ms), `audio-volume-high-symbolic` (29.954 ms), and
   `network-wired-symbolic` (29.949 ms). Every retained frame records four
   asset-cache misses and four missing resolutions, zero asset-cache hits,
   zero SVG reads/rasterisations and zero raster decodes. This is failed
   lookup, not images being repeatedly decoded. `SceneUi::update` calls
   preparation on every Replace (`engines/viu/src/view.rs:503`). The asset
   cache inserts successes only (`engines/viu/src/images.rs:205`), as does
   the path resolver (`engines/viu/src/freedesktop.rs:490`). A miss repeats
   theme-directory enumeration (`engines/viu/src/freedesktop.rs:317`).
   Preparation visits all non-template images (`engines/viu/src/images.rs:216`)
   even though these four images are hidden in the authored empty model
   (`share/scenes/panel/scene.mix:11`, `:29`, `:33`, `:36`). A successful
   decoded asset already has a cache; SVG textures also reuse asset/size keys
   (`engines/viu/src/view/paint.rs:549`). Their warm-path performance is not
   measured by this missing-asset workload.
2. **Layout rebuilds each draw.** `engines/viu/src/view.rs:485` calls
   `flex::layout`; `engines/viu/src/view/flex.rs:916` creates a fresh Taffy
   tree and `:964` computes layout. This is repeated even on a text-only
   clock update. It is ~0.128 ms, three orders below the image problem.
3. **Template preparation rebuilds per accepted revision.** The harness calls
   it at `engines/viu/src/view/tests.rs:71`, matching store preflight on a
   content update. `engines/viu/src/templates.rs:49` reconstructs the
   BindingSet from resolved expressions even with zero task/tray rows.
   Authored parsing/initial scene resolution happens before the sample loop;
   it is not repeated each frame. Compiled templates could be retained by
   authored revision, with instances dirtied only by their dependencies.

### Recommended derived fix order (no fixes made here)

1. Remove filesystem discovery from unrelated render updates: skip hidden
   images, reuse prepared assets when image inputs are unchanged, and add
   bounded negative lookup caching with explicit theme/filesystem
   invalidation. Validate recovery when a missing icon appears. First add
   hermetic installed-icon and missing-icon benchmark cases: this CBC
   workload has no resolved panel assets and does not represent the deployed
   gold icon environment. Do not claim a gold regression from it.
2. Retain compiled template bindings and prepared list instances across
   unrelated source patches; pass the accepted binding set into preflight.
   This removes guaranteed repeat work before trying a more complex layout
   cache. Preserve the shared template budgets and last-good transaction.
3. Then profile a populated, installed-asset panel and decide whether layout
   invalidation/caching is worthwhile. Current layout/paint/tessellation are
   small; leave raster/SVG decode optimisation and further binding tuning
   behind the missing-lookup fix. P2 remains useful dependency infrastructure,
   but cannot solve this frame bottleneck.

### CBC receipts

- `20261109-155941-ce15e8c5-a1`: **BUILD_RC=0**, release measurement at
  `28064c5cf3ac1c1f3aef475d82eec780e8bfd142`, 498.1 s including compilation.
  Both unprofiled and profiled test invocations passed, 104 other tests
  filtered each time. Two Cargo jobs, standard fat-LTO release profile.
  Complete saved log read; numeric records are preserved in
  `docs/quoin-p2-release-frames.json` without machine paths.
- `20261109-160013-5e088cd7-a1`: **BUILD_RC=0**, CBC formatting, 2.1 s;
  complete saved log read, diff applied locally, worker edits restored in
  that same job. No production code repair, PR, desktop or install changes.
