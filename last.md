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

Branch: lab/app-ports; head: 2bcb71059da14e02c489b05056928d1da4d5b86a; steps done: none accepted; CBC jobs: 20261106-192006-425c8dff-a1 (lock update required), 20261106-192201-8c092192 (cancelled queued).
