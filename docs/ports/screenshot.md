# Screenshot port inventory (round 2)

Oracle: MixOS `8779f3328`, apps/screenshot capture/actions/commands/shell/jobs,
annotation engine and docs/screenshot/README.md. Per-app Rust added: 0.
The shared gold frame is now selected. This only covers frame adoption;
the rows labelled ported below are historical specimen dispositions, not
gold acceptance. In particular, capture/save/export return admission rather
than gold's completion replies. See the Screenshot Fable question in last.md.
The inventory was committed before implementation. Dispositions below describe
the bounded lab implementation, with acceptance evidence recorded in last.md.

| Original feature | Round 2 disposition |
|---|---|
| Screen/output, fenced window identity and explicit region capture | Ported: Mix calls comp.capture.frame with id/generation or output/rectangle |
| Interactive region selection, output-generation fence | Ported command flow through comp.region.select; returned generation forwarded to capture |
| Hide/confirm/restore own window on all outcomes | Ported for completion/failure/cancel: confirmed native minimisation, root-owned restore and visible restore failure/retry |
| Delay, cursor, capture/cancel busy admission | Ported: delay 0–10 seconds and cursor via commands, cursor checkbox, one owned background job; stale completion fenced |
| Window/output enumeration and selection | Ported: native Bus enumeration, Mix combo controls; own window excluded |
| PNG open, preview, Save/Save As, no overwrite, copy | Ported: semantic open/export, fitted preview, Save/Copy buttons; exclusive shared PNG encoder and native blobd/clipd references |
| Save folder, saved topic/recent | Ported: launch folder or HOME/Pictures/Screenshots, timestamp collision handling, saved topic and bounded in-memory recent list |
| Settings follow, XDG user-dirs resolution and keymaps | Deferred: original uses shared session appearance/keymaps; generic host has no design/keymap follower yet. No original screenshot-specific settings store found; launch folder and live cursor/mode controls are explicit |
| Editable arrow/line/box/ellipse/pen/highlighter/redaction/text/number | Deferred: requires a reusable image-overlay editing primitive; no annotation policy moves to Rust |
| Object selection/move/delete, crop, undo/redo and raster export | Deferred with editable annotations; raw capture export remains lossless |
| Zoom/pan/fit and responsive annotation toolbars | Deferred until reusable image viewport editing is available |
| Unsaved annotation prompts/history budgets and original dialog registry | Deferred with annotations; raw source is retained until explicit export/replacement |
| Single instance handoff, Print shortcuts and compositor-scoped service names | Deferred integration; this lab uses explicit bundle service/comp argument |
| Full original HELP/actions/execute/macros/theme/CLI and deprecated aliases | Deferred compatibility surface; generic viu native automation remains available |
| Original seven-day/newest-20 raw prune and recent on-disk index | Deferred compatibility; this app owns only paths it creates in its private run directory |

This is a lab specimen, not annotation parity or installation acceptance.

Launch: `viu run apps/screenshot [COMPOSITOR_SERVICE [SAVE_FOLDER [RAW_FOLDER]]]`.
Commands are declared in contract.json. Capture/export/copy return admission;
`screenshot.info` exposes completion, errors and restore state. Exact original
blocking command replies and file dialogs remain deferred. PNG/JPEG/WebP open
uses the shared bounded decoder; export re-encodes PNG without annotations.
The UI does not yet expose a delay field or editable export destination.

Track B7 gap: the existing region command returns a final result, but exposes
no caller-owned selection handle plus cancel/status/removal acknowledgement.
Stopping a Mix job therefore cannot certify overlay removal; the UI must say
the selector may remain until Escape or its 55-second timeout. Native Escape
belongs to compositor input, not an application workaround. Add an owned
capture/selection lifecycle command in Track B7; do not reimplement overlays
or app capture policy in the host. Application quit during selection has the
same limitation. Explicit-region captures need no interactive overlay.
