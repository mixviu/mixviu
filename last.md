# Backdrop — document and retained-raster checkpoint

Overall parity: NOT ACCEPTED. This checkpoint is implementation work, not the requested final feature/visual acceptance. No deviation or Fable question is proposed.

Exact tested code: d825c2ae665c1d6fe06d73708242f84bb3682774. Gold: 8779f33284b7d3f1178c3d57a699efb9b0867b22. Receipt-only commit follows. docs/ports/backdrop.md records the inventory correction and exact checkpoint scope.

Gold has no chooser/settings window: only the background layer and standalone probe. No shared window frame is applicable. Gold does include the train view, which remains required.

Implemented: strict Mix document schema and canonicalisation, CLI config/image/colour/fit parsing and startup loading, structured invalid-set refusal, pure slideshow traversal/deadlines/failure handling, shared retained raster facility with the gold bounded decoder and provenance. Root/Jobs evaluators share the native handle store; image-fill renders the decoded handle without reopening paths, accounts resident/texture overlap, uses original dimensions for native sizing, and shares ownership with newly added output views.

CBC 20261107-090116-633bcd47-a1 passed at the exact tested code on cbc3: four Mix suites, eight lint checks, fmt check, 13 raster-related tests, three app-port tests and normal Clippy. No worker diff. Complete saved log read. Earlier failed/green job IDs are in the port record. No release artifact, install, VT action or native compositor test was performed.

Required next work:

- Wire retained handles into the Mix wallpaper transaction controller. Use Jobs for blocking decode/store; hold and finish native call receipts. Fence superseded work and release stale handles, including cancellation during decode. Current app still publishes path-based models and old model-only success replies.
- Preserve last applied document/pixels on IMAGE/STORE refusal, implement durable storage and exact requested/active/pending/state/output envelopes, source refresh at the same path, and full gold descriptors, debug, action/executed surface.
- Wire independent slideshow schedules with native one-shot deadlines, folder/list startup selection, failed-entry retries, manual next and half-second crossfade. schedule.mix is pure and tested but is not yet used by the running app.
- Port probe rendering and train 2D/3D, queue/history/gate, retained alerts, visibility/animation leases and view commands. Probe currently explicitly refuses.
- Replace hardcoded theme fallback and polling with live gold appearance/settings behaviour. Exercise all Prefs fields, authority loss/recovery and unchanged PIDs.
- Test simultaneous outputs, hotplug/removal, scale/work-area changes, texture starvation and wake-on-release. Native store/renderer tests are not output acceptance.
- Build and hash exact release bytes, run the isolated gold/candidate recipe, capture every required state, compare whole/regions with parity_compare.mix and inspect images; retain Bus replies and owned-process cleanup. Existing six-mode AE=0 is historical, not revalidated by this checkpoint.

Branch lab/parity-backdrop; head SHA d825c2ae665c1d6fe06d73708242f84bb3682774 (exact tested code; receipt HEAD is the commit containing this file); parity evidence: document/launch/schedule and retained-raster contracts pass, full features/Bus/Prefs incomplete, SSIM/regions pending; CBC job ids: 20261107-090116-633bcd47-a1, 20261107-085920-7963d5d5-a1, 20261107-085822-29188e7c-a1, 20261107-085523-b9e3c76c-a1; failed 20261107-085433-7d251078-a1, 20261107-090016-dbccaea6-a1.
