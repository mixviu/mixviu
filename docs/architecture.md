# Direction and boundaries

## What MixViu is

MixViu is an agent-first operating environment: a shell and scripting
language, a headless service host, a Wayland compositor and a widget toolkit,
all in one repository. People and AI agents use the same surfaces. One human
maintainer builds it with several AI agents. Application behaviour is written
in Mix. Rust holds the native foundations, and an application moves to Rust
only after a measured, diagnosed performance failure.

- **mix, the shell.** The login shell, REPL, script runner and manual, and
  the language for automation and application behaviour. Values are
  structured, not text streams, and external programs run by argv. mix does
  not interpret natural language. Agents do that, and they act by running Mix
  and calling Bus commands.
- **mixd, the headless host.** One multi-call binary: the Mix engine's
  service profile plus every headless personality. `mixd serve` runs Mix
  services as Bus citizens. It is built for servers and contains no display
  code.
- **viud, the compositor.** Only the compositor: scanout, rendering, input,
  capture, window and output authority (including global corner and edge
  triggers) and the `comp` Bus service. It contains no widget toolkit.
- **viu, the toolkit.** Widgets, scene rendering and the host for Mix
  behaviours. Scene documents describe composition and bindings; Mix
  behaviours make the decisions. The same widgets serve shell surfaces and
  ordinary application windows, as Wayland clients of viud.

**How agents reach it.** There is one transport, the Bus (`owner.verb`
commands, routed by the node daemon). Each application exposes its actions as
data behind two commands, `<app>.actions` (list) and
`<app>.execute {id, params}`, with typed parameters. Menus, keybindings, the
CLI and agents all end in the same `execute`. MCP is an edge adapter for AI
clients, written in Mix. It exposes a few generic tools, never one tool per
action, and never bypasses the Bus.

**Boundaries.** The engines share only the Mix library and the Bus crate. No
engine links another, and every other boundary is the Bus.

**Direction, not yet built.** Agent-generated UI: an agent composes a scene
document from the existing widget catalogue, binds it only to declared
actions, and viu shows it as a clearly marked agent surface. Today agents
drive the UI through actions and by editing scene files.

## Principles

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
