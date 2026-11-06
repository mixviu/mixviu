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
