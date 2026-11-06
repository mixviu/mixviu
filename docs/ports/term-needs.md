# Future Term native primitives

No Term implementation or VT/PTY work is started in round 2. This is an input
inventory from the shared Viu input/render/job path and Prefs terminal settings,
not a completed survey of the original Term source. Fable D6 requires reusable
facilities in libraries/viu; no per-app Rust host.

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
