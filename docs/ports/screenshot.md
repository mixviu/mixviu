# Screenshot parity inventory

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Mix owns application policy and composition. Reusable Viu facilities own raster
documents, painting, viewport gestures, native controls and deferred replies.

The earlier bounded specimen inventory is superseded. Gold is the contract;
implementation is not acceptance. The final native, Prefs and visual gates are
still outstanding; see last.md for exact tested revisions and receipts.

| Gold inventory | Implementation and remaining acceptance |
| --- | --- |
| Screen/output, fenced window, explicit and interactive region | Mix capture worker; native compositor requests and output-generation fence; native candidate conformance pending |
| Delay, cursor, cancellation, hide and restoration | Cooperative cancellation and restore retries; original request held through completion; native full-app cancellation/restoration checks pending |
| Capture/open/export/save completion replies | Reusable identity-bound `viu_reply` handles; primitive passed native 22-second and cancellation witnesses |
| Arrow, line, rectangle, ellipse, pen, highlighter, redact, text, number | Imported reusable gold document/painter; all nine gold oracle cases captured; candidate export comparison pending |
| Selection, move, delete, crop, undo/redo | Shared document/history and canvas gestures; native candidate conformance pending |
| Zoom, pan, fit, gesture guides and selection outlines | Shared image canvas driven by Mix; native input and narrow-state checks pending |
| Open, Save As, save/discard, help and shortcuts dialogs | Mix dialog policy; shared controls; matched dialog captures pending |
| PNG/JPEG/WebP, exclusive PNG export and clipboard | Gold document decoder/painter and in-process clipboard submission; full native clipboard checks pending |
| Paths, XDG user dirs, collision naming, recent ledger and raw pruning | Mix storage policy, durable native replacement, seven-day/newest-20 pruning; native edge-case checks pending |
| HELP, describe, actions, execute and edit aliases | Pinned descriptors and dynamic action registry; initial native actions matched gold; complete refusal/side-effect checks pending |
| Macro commands/windows and debug/UI/window drive | Shared native facilities and deferred ownership; native full surface checks pending |
| Saved and action-executed topics | Mix saved events and bounded action journal through in-process jobs; native subscriber checks pending |
| CLI, headless, Print handoff and compositor namespaces | Mix launcher and generic headless/host options; native launcher/headless acceptance pending |
| Live Prefs, keymaps and local theme choices | Shared session/frame plus content design roles; full live candidate variants pending |
| Matched visual states | Preliminary captures retained privately; not accepted |

Launch through Mix: `mix apps/screenshot/launch.mix [OPTIONS] [IMAGE]`.
The generic host also accepts `viu run apps/screenshot [COMPOSITOR_SERVICE
[SAVE_FOLDER [RAW_FOLDER]]]` for isolated tests.

Gold corrections: capture/save/export return finished state rather than
admission; cancellation is cooperative and waits for cleanup. Gold accepts
relative Bus file paths against its working directory. UI path entry also
trims whitespace and expands a leading home-directory abbreviation. Raw
unannotated captures do not trigger the unsaved-annotations prompt. Fixed
revision-one annotation inks are independent of live chrome appearance.

The old proposed B7 cancellation workaround is removed. Gold itself waits for
the compositor's region-selection result before checking cancellation. The
port follows that sequence; it does not destroy a worker that owns restoration.
