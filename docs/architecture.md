# Direction and boundaries

Build useful application behaviour in Mix first, then measure representative
workloads. A particular application or compound widget may be rewritten in
Rust after diagnosed, repeatable performance failures and bounded remediation.
Density/complexity alone is not evidence for choosing Rust.

Existing native rendering/input facilities remain useful. Extending a runtime
does not justify moving arbitrary application policy behind a native command.

The intended engine split is:

- mix: shell, CLI and scripts.
- mixd: native host for server-side Mix work; split from mix later.
- viu: reusable widgets, scene rendering and Mix application behaviour hosting.
- viud: compositor authority, including global corner/edge triggers and the
  input, window and output responsibilities that require compositor ownership.

Scene documents describe composition/bindings; executable Mix behaviours own
application decisions. Viu serves shell surfaces and ordinary application
windows using the same widget implementations. Behaviours must keep UI input
and rendering responsive. The native Bus remains available to agents/services.

Start with the required runtime/widget slice and one useful Quoin surface.
Then exercise Dopus and Email with captured operations. A full registry, server
rewrite and complete engine extraction are not prerequisites.

Measure about 95% or better native Rust median performance on named equivalent
workloads. For reciprocal costs this means at most Rust/0.95; throughput means
at least 0.95*Rust. Keep memory, tails and startup separate. The development
target is at least halved representative edit-to-tested-usable time against
the existing gold path, with scoped Rust timings reported separately.

These are targets, not results from this source seed. Missing evidence stays
unknown. Benchmarks must include required workers and shared host costs.
Correctness and content-bearing visible-response receipts matter alongside
median timings. Do not infer app parity from a synthetic stage-cost proxy.

Reuse the published native facilities and wire contracts. Introduce no
parallel broker/transport. Preserve session/output independence and a working
reference/rollback while new desktop components are developed.
