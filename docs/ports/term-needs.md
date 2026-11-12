# Term native widget requirements (round-5 checkpoint)

This is the historical extraction requirements inventory. The completed
[Term acceptance record](term.md#acceptance-2026-11-08) identifies the tested
release and its evidence; installation remains separate integration work.
Fable D6 requires reusable
facilities in libraries/viu; no per-app Rust host. The round-2 input inventory
is now supplemented by pinned MixOS 8779f3328 Term Cargo manifests,
term-core native_session/native_lane and the frontend IME ownership code.
Gold Term remains until a separately accepted replacement exists.

- An owned, bounded PTY/session interface: spawn argv with explicit environment,
  read/write backpressure, resize, exit status, cancellation and reaping.
- A reusable VT parser and terminal grid, alternate screen and bounded
  scrollback; incremental damage, attributes, wide/combining Unicode and fonts.
- A native terminal view: cell metrics, cursor shapes/blink, selection,
  search highlights, scrolling, IME/preedit, focus, and terminal mouse protocols.
- Keyboard/paste encoding including bracketed paste and modified keys; native
  clipboard integration and copy-on-select without exposing terminal escapes as
  application commands. Hyperlinks/bell/title changes should be typed intents.
- Observable headless geometry/readback and input automation for correctness,
  throughput, large scrollback and representative latency tests.

Mix must own tabs/panes, process-launch choices, profiles, actions, settings,
close prompts and session lifetime policy. Prefs already exposes terminal
family/size/weight/padding, scrollback/speed, cursor, copy-on-select, TERM and
pane/focus appearance settings; the future primitive must make those usable
without importing Term application policy. Interpreter unification is P2 work.

## Placement and reuse candidates

The original term-core is not a ready-made generic widget: it also owns tabs,
panes, settings and application control. Extract only the native mechanisms.
Its pinned rio-vt/teletypewriter revision is
`932c1a7d9e07b4db5924f7a0dd689e823c3a1442`; the child-only descriptor handoff
patch and licences/dependency closure must be inspected before import. Reuse
the original terminal/parser, Unicode cluster/raster and term-render atlas/grid
work where separable. Do not copy app.rs/actions/layout into a Rust host.

The native widget needs typed create/input/resize/snapshot/close operations,
stable session handles with generations, explicit byte/scrollback budgets and
event-driven damage/exit notifications. Keep output bytes in native storage;
do not publish the full grid/scrollback through Mix on every PTY read. Mix
chooses when to create/close a session and how to compose tabs/panes, while
native close cancels readers and joins/reaps the specific owned child.

The original native-session lane adds requirements beyond a PTY and parser:
verified broker attachment, child launch descriptor ownership, connection and
pane generations, bounded dispatch queues, reconnect/resumption and shutdown
ordering. Preserve the existing Bus authority semantics without a new broker
or ambient credentials. Widget extraction must keep stale requests from
reaching a newly attached pane. Shell launch and layout choices stay in Mix.

IME composition belongs to a pane. Focus change/close must ask the platform to
cancel and suppress stale preedit/late commits until reset acknowledgement;
neither a timer nor returning to the old pane may redirect text. Clipboard and
primary selection similarly need ownership and bounded asynchronous reads.

Acceptance before replacing gold: parser/Unicode/raster tests retained, real
PTY resize/input/output/exit and leak tests headlessly on CBC, native-session
reconnect/stale-generation/descriptor tests, IME pane-switch tests, selection
and clipboard checks, GPU grid comparison, and measured latency/throughput
with large scrollback. No desktop trial or installation follows this inventory.
