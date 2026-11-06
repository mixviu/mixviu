# App ports round 1

Base: `5f573b22a2a108e0f81a5303680e29946725074a`. Isolated lab only;
no installs, real displays, VT changes or main merge.

Generic host prototype and native layer backend pushed. Current follow-up adds
Mix backdrop and monitor bundles, complete source-derived feature inventories,
generic background hosting and image-fill, plus real-evaluator/scene tests.
The old gallery comparator/tests move to gallery-reference; gallery has no Rust.
Application policy stays in Mix. Dopus host inventory is in docs/ports/generic-host.md.

CBC 20261106-192006-425c8dff-a1 (cbc3) stopped on required Cargo.lock update;
full saved log read, worker formatting applied locally. Obsolete queued job
20261106-192201-8c092192 cancelled before running (workers occupied by other
sessions). No green gate or run/capture claim yet. Mix lint passes all five
behaviour/core files with the host-provided globals/functions declared.

Open: exact-source CBC, real headless/nested captures and inspection, native
settings/monitord gates. Feature inventories state current parity deferrals.

Second CBC attempt 20261106-193321-957e2f64-a1: compile stopped on two
retained logging crate references in the layer import. Full saved log read;
lockfile and applicable formatting diffs applied; calls adapted to Viu tracing.
Corrected startup so bound scenes wait for their initial model; native window
and per-output layer views retain independent adapter state. Added literal
argument round-trip tests, layer presentation counters and explicit refusal of
eframe-only capture on layer surfaces (use native compositor capture).
Pure Mix tests pass: backdrop RGB/gradient, six refusals, settings output
mapping; monitor node admission, empty state, 2000-row projection, 2001 refusal.
Installed-interpreter projection sample: 15.89 ms, not embedded UI latency.
Private Dopus-style headless harness prepared; not yet run.


CBC 20261106-194334-64190f42-a1 compiled successfully; 50 library unit tests
and preliminary integration tests passed. New scene test correctly refused
f64-normalised image-fill RGB channels. Validator/drawing fixed without
changing tests; literal bundle argument and backdrop callback tests passed.
Full saved log read. Static seed, external-pin and native-Bus provenance checks
pass. Runtime import verifier hits a missing jwt-rsa-test.pem already absent
at the base revision; no verifier was weakened.


CBC 20261106-194851-e7308ce5-a1 passed scoped check/tests/build (118 tests),
full log read, worker fmt applied. Artifact SHA256
47f1e617aa204eba7bc1e9b8bac267247005a543496c4301a9eada3990e7ca3e.
Owned headless run passed gallery callbacks, real monitord sampling, backdrop
colour/five modes/output override/settings change; screenshots inspected.
All six original/new backdrop comparison captures have AE=0 at 1280x900.
Original Monitor inspected; differences are in the feature inventory. Visual
inspection found hidden empty search fields and reversed CPU sorting; corrected
in Mix and awaiting fresh harness run. All owned processes confirmed stopped.
Final gate now requires fmt-check/locked deps and zero worker diff.

Branch: lab/app-ports; head: a6d5e1177e9f712b7589c444e2f60a1b68234c56; steps done: 0/1/2 bounded specimens implemented, final acceptance pending; CBC jobs: 20261106-194851-e7308ce5-a1 green (earlier attempts above).
