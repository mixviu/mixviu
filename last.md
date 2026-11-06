# Backdrop round two — Fable decision required

**NOT ACCEPTED.** Stop for Fable under the standing brief's “propose to
deviate from gold” rule. No waiver is assumed. Opus can take the question
below to Fable; the other open acceptance items remain required afterwards.

Exact release/test source: `d34633db087f1e6d6f3281d5f5e207a65057af99`,
branch `lab/parity-backdrop`. This file and `docs/ports/backdrop.md` form a
receipt-only documentation commit after that source. Gold remains
`8779f33284b7d3f1178c3d57a699efb9b0867b22`. Gold has no chooser/settings
window; its shared-window-frame adoption is inapplicable to the background
and standalone probe layers.

## Question for Fable: live Prefs versus the pinned gold behaviour

May Backdrop keep the live native settings follower, deliberately differing
from the gold application's startup-only appearance?

- **B — recommended:** retain the port's live Prefs response and approve
  this specific behaviour/visual deviation. Then complete the remaining
  appearance matrix against the gold toolkit's tokens, using fresh-start
  gold captures where necessary. The current implementation is B, awaiting
  approval; it is not accepted merely because it exists.
- **A — strict gold:** remove the live follower for Backdrop, preserve the
  startup-only look and explicitly waive this app's standing live-Prefs
  acceptance criterion. Re-run the native matrix with that behaviour.

Evidence: pinned `apps/backdrop/src/main.rs` passes `Theme::load()` once to
`Backdrop::new`; its program installs the theme at setup. The pinned layer
loop has no settings-session follower. This is confirmed by two isolated
native gold runs, not inferred solely from source. The release pair's gold
PID 2945883 remains alive while private settingsd accepts/resolves light
(revision 3). Its before/after PNG SHA256 is identical:
`a761cc8b8692b55e8eff32604d1520bfec0a6eb56178ac861110c6db327baf26`;
SSIM/RMSE distortion is 0/0. Candidate PID 2946263 remains alive at its
light revision 5 and produces
`a03f8a4eb4f1e9445ebff5adaed673435e7e862cfc5718dc47f86c25afafbbc9`.
The light gold/candidate pair differs across the background: normalised
SSIM distortion 0.418086, RMSE 0.887584. Both were dark and pixel-identical
before the live change. Full state, settings CAS/readbacks, captures and
figures are retained. C1 explicitly concerns evident layout defects that
hide function; applying it to an absent settings follower would be a new
interpretation, so no C1 exemption is presumed.

## Implemented checkpoint and evidence

Retained-raster decode-before-commit, durable set, held Bus replies,
supersession scopes, same-path refresh, independent slideshow timers,
failed-slide skipping, manual next and crossfade ownership are wired into
the running Mix controller. Train views/readers/history/gate/alerts and the
offline probe variant are implemented in Mix over reusable native widgets.
Generic layers now expose native output/configure/presentation/input
events and a settings session; app policy remains in-process Mix. No
per-app Rust, install or VT change.

Release SHA256:
`5df196a34e6ddf6ccc96ffa3f80ba6fff6eda39d0f9fb0c3749b78fcb292187c`,
fetched and verified against CBC's source/profile receipt before use.
Private archive: `_var/backdrop-parity-20261107/pair.6PjeW0` in the private
hub. It retains scripts, receipts/hashes, native Bus traffic, source archive,
PNGs and ten comparison sets. Gold/candidate use canonical `backdrop`
sequentially on an owned native broker, compd and settingsd, 1040×680,
scale 1, Studio dark, comfortable. All five owned PIDs are confirmed exited.

Whole/wallpaper SSIM and RMSE distortions are both **0** for colour, fill,
fit, centre, tile, stretch and default dark theme. All 12 HELP descriptors
match exactly. Both train renderers' unavailable states match exactly in
header, queues and scene. History differs only at the rounded “Now” legend
background corners: SSIM distortion 0.0000246615 / RMSE 0.000493558;
whole-image 0.00000689071 / 0.000260892. Originals, side-by-side and diff
figures were inspected; that small difference is still open. ImageMagick
7.1.2-35 reports SSIM as distortion (zero identical, lower better).

Native decode refusal preserves active config/revision. Runtime/Jobs tests
also cover same-path replacement, failed slide traversal, source-document
retention and fade ownership. Full feature/Bus/Prefs acceptance is not
implied by HELP or those focused witnesses.

CBC `20261107-094413-6a3e7ac6-a1` on cbc3: **BUILD_RC=0**, exact source
above, `mix tools/backdrop_accept_gate.mix`, 647.2 seconds. Six Mix suites,
lint, rustfmt check, both artifacts, 23 raster-related tests, five app/controller
tests, normal Clippy and complete Viu/engine tests pass, with no worker diff.
The full saved 2717-line log was read, including warnings and six existing
ignored engine tests (installed icon tree / offscreen snapshots / native GPU
process gate). Those ignored cases are not claimed as passed. Saved log:
`20261107-094413-cbc3-d34633db087f-20261107-094413-6a3e7ac6-a1-c479a8c5.log`
in the CBC client cache. Earlier green prepare/artifact job
`20261107-093600-9a0a97f4-a1` returned formatting, applied before this exact
gate; repair receipts are preserved in the port record.

## Open after the ruling

- Complete native Bus payload/refusal/topic/debug/action-executed audit.
  Observed port train output state still reports underlying `theme` rather
  than `train`; view-frame counters also need gold-compatible semantics.
  Invalid-view error wrapping and nullary executed params need comparison.
- Close the train legend corners and exercise populated deterministic and
  real train/build services, retained alerts, stale reads and large histories.
- Capture timed slideshows/crossfades, supersession and storage failures;
  exercise native probe/input and CLI refusal/exit-code cases.
- Complete the approved Prefs matrix, authority loss/recovery, fractional
  scale, work areas and native texture starvation/wake-on-release.
- Simultaneous outputs/hotplug is unverified. Pinned headless backend owns
  exactly one `headless-0`; nested owns exactly one `winit`; CLI has no
  output-count flag. The actual witness is one 1040×680 scale-1 output.
  Shared output-store/removal tests do not substitute for compositor evidence.

Branch lab/parity-backdrop; head SHA d34633db087f1e6d6f3281d5f5e207a65057af99 (exact tested code; receipt HEAD is the commit containing this file); OPEN: Fable live-Prefs ruling, full features/Bus/Prefs and train legend acceptance; CBC job ids: 20261107-094413-6a3e7ac6-a1, 20261107-093600-9a0a97f4-a1, 20261107-093016-40e41e4e-a1; repair failures listed in docs/ports/backdrop.md.
