# Screenshot port inventory (round 2)

Oracle: MixOS `8779f3328`, apps/screenshot capture/actions/commands/shell/jobs,
annotation engine and docs/screenshot/README.md. Per-app Rust added: 0.
Initial inventory before implementation; verification statuses follow below.

| Original feature | Round 2 disposition |
|---|---|
| Screen/output, fenced window identity and explicit region capture | Planned port: Mix calls existing compositor Bus commands |
| Interactive region selection, output-generation fence | Planned port through comp.region.select; lifecycle gaps belong to Track B7 |
| Hide/confirm/restore own window on all outcomes | Planned Mix policy over comp.window commands |
| Delay, cursor, capture/cancel busy admission | Planned Mix state machine using reusable owned jobs |
| Window/output enumeration and selection | Planned data/UI port over existing compositor commands |
| PNG open, preview, Save/Save As, no overwrite, copy | Planned Mix flow with shared bounded image/file/clipboard primitives |
| Settings follow, save folder, saved topic/recent | Planned bounded Mix port; original folder/prune/index compatibility assessed separately |
| Editable arrow/line/box/ellipse/pen/highlighter/redaction/text/number | Deferred: requires a reusable image-overlay editing primitive; no annotation policy moves to Rust |
| Object selection/move/delete, crop, undo/redo and raster export | Deferred with editable annotations; raw capture export remains lossless |
| Zoom/pan/fit and responsive annotation toolbars | Deferred until reusable image viewport editing is available |
| Unsaved annotation prompts/history budgets and original dialog registry | Deferred with annotations; raw source is retained until explicit export/replacement |
| Single instance handoff, Print shortcuts and compositor-scoped service names | Deferred integration; this lab uses explicit bundle service/comp argument |
| Full original HELP/actions/execute/macros/theme/CLI and deprecated aliases | Deferred compatibility surface; generic viu native automation remains available |
| Original seven-day/newest-20 raw prune and recent on-disk index | Deferred compatibility; this app owns only paths it creates in its private run directory |

This is a lab specimen, not annotation parity or installation acceptance.
