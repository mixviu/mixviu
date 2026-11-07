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
4. Use the same canonical service identity sequentially (Monitor: `monitor`),
   so per-app Prefs, menu hosting, macros and theme lookup see the same app.
   Capture gold first at 1040×680 logical client pixels and scale 1. Set
   Processes, default CPU sort, no selection/filter and paused state; wait
   for data and the applied state, then request `monitor.ui.capture`.
   Retain its PNG and width/height receipt. The generic host also supplies
   `viu.capture` for diagnostics; acceptance exercises the gold command name.
5. Stop the gold process, launch the candidate against the same services and
   fixtures, reproduce the state, verify readiness and absence of errors,
   then capture. Check equal decoded image dimensions. Do not resize either
   capture to conceal a geometry or scale mismatch.
6. Capture every further relevant state: Performance/resource pages, tree,
   sorting, selection/hover/focus, menus/dialogs, narrow windows, overflow and
   both colour modes. Record any state transition used before each capture.
   Seed a full constant 60-sample history when comparing chart geometry;
   startup/tick timing must not accidentally become different chart data.
   Use a separate varied/gapped history to test chart behaviour.
   For restart/selection-reveal pairs, seed the same initial saved window
   size before launching both processes. Equal final dimensions do not imply
   equal reveal geometry if one process centred its selection before the
   compositor resized its first frame. Retain startup sizing and in-progress
   scroll captures as diagnostics; compare the settled, matched state.
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

Preserve caller identity across operations whose contract depends on it.
For example, Ced edits and their undo use one persistent native Bus citizen;
a separate citizen tests cross-origin refusal. Spawning a new anonymous
client for every request changes the contract under test. Verify the gold
identity behaviour before changing a fixture, and retain its original
assertions and deadlines. Run launch/interoperability witnesses without
unrelated app suites competing for the same machine; retain any concurrent
admission failures separately from the isolated paired result.

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

Save the exact accepted release executable under `/build/es-artifacts/<sha>/`
with its SHA256 and source/profile receipt. Fetch and verify that hash before
running the final native evidence suite. Debug witnesses do not attest the
release bytes. `tools/gold_parity_gate.mix` performs the scoped checks and
saves both profiles; its release receipt is `viu.json`. The artifact handoff
does not install it or restart a desk.

## Monitor accepted reference

The [Monitor acceptance record](monitor.md#acceptance-2026-11-07) is the
completed example: exact release source/bytes, 1040×680 whole/region figures,
46/46 matched Bus replies, live Prefs on unchanged PIDs, inspected raster
differences, P1's approved non-visual fence and native cleanup receipts.
Its private archive includes the executed scripts, seeded fixtures, all
55 comparison sets and a machine-readable index. Earlier failed receipts
below remain historical evidence, not the current verdict.

## Monitor round-two record: failed

Tested code: `bb950a59f82ef5c400fe89eaf1175a42bef6f93c`; pinned gold as
above. Candidate binary SHA256:
`2044c741983b90bdc48ae2bc6ce7c07f461f779341a13dd8df9044ef6659ce51`.
CBC `20261107-012348-cc3a95cf-a1` passed the scoped Viu/viu-engine gate
with no worker diff; the four-engine integration gate was not rerun.

Both captures are 1040×680, scale 1, paused Processes with the same six-row
native fixture, Studio Dark, CSD, right captions, bar menus, row actions and
Inter 400 at 14.667. Gold PNG SHA256:
`800c28506c24d9bc9c755c740d4cbca72486ce67aee48597e548461d12da19fc`;
Mix PNG SHA256:
`9121f639c803f49d8f08f210a6e33e4963c67114561e6c4c4adb741cc9c41e69`.
The private receipt `gold-parity-20261107/round2/matched-monitor-pair`
contains the native harness, service data, captures, figures and raw metrics.
All owned processes were confirmed stopped.

ImageMagick 7.1.2-35 reports these normalised **distortions** (zero is identical):

| Region | Rectangle x,y,w,h | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| Full | 0,0,1040,680 | 0.0957189 | 0.118710 |
| Header | 0,0,1040,38 | 0.119769 | 0.111125 |
| Options | 0,38,1040,42 | 0.258504 | 0.199630 |
| Content | 0,80,1040,570 | 0.0859221 | 0.113603 |
| Status | 0,650,1040,30 | 0.125844 | 0.147344 |

The side-by-side and full/header/options/status differences were inspected.
Remaining differences belong to the Monitor parity job:

- Header: missing View/Process/Help menus, gold node-aware title and actions.
- Options: missing gold tabs, filter/tree/node controls and placement.
- Content: legacy stacked title/summary/chart, missing command-line column,
  incorrect table proportions, striping, dividers and typography. Legacy
  content's hardcoded text colours also need conversion for light mode.
- Status: missing filtered/total count, uptime and the gold layout.
- Further states: Performance resources/charts, process tree/actions/dialogs,
  node discovery and the complete 50-command ABP surface remain unaccepted.

At that historical checkpoint no difference had Fable approval. P1 has since
been decided; [monitor.md](monitor.md) records the approved invisible process
identity fence and corrects the old inventory's UID/confirmation claim.
Five live Prefs variants plus settings authority loss/recovery and Mix-owned
close passed on one candidate PID; that separate frame test does not grant
Monitor visual, functional or ABP acceptance. Overall verdict: **FAIL**.
