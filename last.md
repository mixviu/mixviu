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

Branch: lab/app-ports; head: 1e180c493c45c887c508a0e5fbdeec130f158800; steps done: none accepted; CBC jobs: 20261106-192006-425c8dff-a1 (lock update required), 20261106-192201-8c092192 (queued owner stopped), 20261106-193321-957e2f64-a1 (logging import fix required).
