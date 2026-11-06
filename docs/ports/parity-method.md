# Application parity method

An acceptance record covers behaviour, appearance, ABP compatibility and live
Prefs response. Passing one does not substitute for another. Use the pinned
gold original at `8779f33284b7d3f1178c3d57a699efb9b0867b22` and the exact CBC
artifact for the candidate. Record both source revisions and binary hashes.
Do not use a dirty source checkout as the oracle.

## Isolated capture recipe

1. Start an owned native Bus and headless/nested compositor with the existing
   native test harness. Give the run its own home, runtime, config, data,
   cache, settings root and Bus credentials. Remove live display/session
   variables. Never reuse a desktop service identity or change live Prefs.
2. Start a private settingsd. Create its instance/profile, then set identical
   appearance, framing, scale and app overrides for both applications. Force
   the same client decoration/menu/action placement for client readbacks;
   compositor-hosted chrome needs a separate whole-surface capture pair.
   Read back effective titlebar forms rather than assuming the request won.
3. Serve deterministic data through the native inter-process service contract.
   Monitor uses the pinned `monitord-snapshot.json` and
   `monitord-processes.json` fixtures from its gold contract tests. Record
   fixture hashes and every request/reply. A fixture used to measure pixels
   does not replace functional testing against the real daemon.
4. Capture gold first at 1040×680 logical client pixels and scale 1. Set
   Processes, default CPU sort, no selection/filter and paused state; wait
   for data and the applied state, then request `monitor.ui.capture`.
   Retain its PNG and width/height receipt. The current port's corresponding
   command is `viu.capture`; full ABP parity must also provide the gold name.
5. Stop the gold process, launch the candidate against the same services and
   fixtures, reproduce the state, verify readiness and absence of errors,
   then capture. Check equal decoded image dimensions. Do not resize either
   capture to conceal a geometry or scale mismatch.
6. Capture every further relevant state: Performance/resource pages, tree,
   sorting, selection/hover/focus, menus/dialogs, narrow windows, overflow and
   both colour modes. Record any state transition used before each capture.
7. Stop all owned processes in `finally`; verify that every PID and the
   private broker exited. Keep failed runs and logs. Store private scripts,
   captures and receipts in the private hub, outside this public repository.

Intra-app models, events and widget interactions use in-process channels.
Bus traffic in this recipe is between independent processes.

## Diff figures and inspection

The reusable comparison driver is `tools/parity_compare.mix`. Run it with
Mix, passing gold PNG, candidate PNG, a new output directory and optionally a
JSON region map such as `{"header":[0,0,1040,38],"status":[0,650,1040,30]}`.
It refuses unequal sizes and existing output directories, writes crops,
side-by-side/diff figures and `comparison.json`, and leaves its verdict as
`inspection-required`. Choose region rectangles from the matching gold state.

Run image tools through Mix `run_argv`, using argv arrays:

```mix
$side = run_argv(["magick", $gold, $candidate, "+append", $side_path])
$ssim = run_argv(["magick", "compare", "-metric", "SSIM", $gold, $candidate, $diff_path])
$rmse = run_argv(["magick", "compare", "-metric", "RMSE", $gold, $candidate, "null:"])
```

Require the montage command to succeed. Compare exit code 1 means differences;
it is not a tool failure. Other failures must be diagnosed. Save raw metric
output, tool version and the normalised metric in the receipt; ImageMagick
can print a quantum-scaled value followed by the normalised value in brackets.
The installed ImageMagick 7.1.2-35 reports SSIM as distortion: identical images
return `0 (0)`. Record that convention explicitly; lower is better for these
receipts. Do not relabel the raw normalised distortion as a similarity score
or compare it to a threshold defined for mathematical SSIM similarity.

Inspect the originals, side-by-side figure and difference figure. Repeat the
comparison for separately cropped regions: header/menu/actions/captions,
options, content table or resource rail, graphs, status and overlays. Use
identical crop rectangles derived from the intended gold geometry, and retain
them in the record. A whole-image score can hide a missing small control in a
large empty background.

There is no arbitrary SSIM pass threshold. Close region differences until only
anti-aliasing/font hinting remains, or record Fable's explicit approval and
reason. Never label a deferred capability or an unexplained low score as an
accepted deviation. New screenshots do not overwrite earlier evidence.

## Functional, ABP and Prefs acceptance

Inventory the pinned app's actions, commands, topics, properties, payloads,
return codes, errors and asynchronous delivery rules. Drive the gold and Mix
surfaces with the same requests, including refusals and stale replies. Match
structured replies rather than merely checking rc 0. Test callbacks through
native input as well as direct Bus calls. Retain daemon-backed behaviour
tests, representative large datasets and measured row-processing budgets.

While the candidate remains running, apply settingsd changes for mode, scheme,
style, accent, density, roundness, font family/weight/size, mono family, text
scale, icon stroke, app overrides, auto-mode state and relevant `tune.*` keys.
Wait for the accepted revision, capture the new appearance and verify the
PID stayed unchanged. Compare framing, geometry and typography with gold.
Also test invalid/refused changes, absent/restarted settings authority and
recovery, retaining the last accepted look where the gold contract does.

Rust checks/builds run only through CBC. Apply returned formatting/lock
patches, commit/push, then rerun the exact-source gate with no worker diff.
Read the saved complete log. A focused widget test is a repair check; it is
not the final app or full integration gate.

## Acceptance record

Each checkpoint records:

- Branch, tested source SHA, receipt commit, gold revision and binary hashes.
- CBC job IDs, commands, result and log locations.
- Fixture/service revisions and hashes; size, state, theme and scale.
- Capture filenames, native readback receipts, tool version and diff figures.
- Whole-image and region metrics with inspected differences listed by region.
- Behaviour/ABP/Prefs cases, expected and actual outcomes, including refusals.
- Every remaining difference, its owner and closure evidence; any Fable
  decision includes the question, options, evidence, answer and rationale.
- Explicit overall verdict and process-cleanup receipt.

Update `last.md` at the checkpoint. Its final line carries branch, head SHA,
frame state, Monitor evidence and CBC job IDs. A receipt-only documentation
commit must identify the exact tested code SHA separately.
