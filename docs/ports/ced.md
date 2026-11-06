# Ced parity inventory

Current checkpoint: reusable gold editor integration, **not full parity**.
Gold reference remains MixOS 8779f3328. No deviation below is approved.

| Capability | Current evidence / remaining work |
|---|---|
| Editing, cursor, selection, undo/redo | Retained editor/model, native Unicode typing and local undo regression pass. UI Undo/Redo stay local; legacy ced.command undo/redo still target editd history. |
| Clipboard, IME, Unicode and mouse/keyboard bindings | Gold widget code reused; Unicode selection/input tested. OS clipboard, native IME and complete binding acceptance remain open. |
| Syntax, gutter, wrap | Gold syntax/model/rendering wired; Rust highlighting inspected, wrap and line-number options supplied by Mix. |
| Find/replace | Literal find, wrap-around Next and single replace. Regex, case/word options, all-replace and gold navigation remain open. |
| Large files | Widget admits 64 MiB and uses gold visible-row rendering. Ced still refuses snapshots/drafts above 64 KiB. Paged loading, delta publication, bounded search and representative performance remain open. |
| Tabs, open/save | Named tabs retain independent drafts; reopening preserves drafts/fences. Open dialog, explicit Apply, Save/Save As and authoritative language/dirty metadata. Tab close furniture and gold chooser/prompt behaviour remain open. |
| Shared frame | Gold frame adopted with Mix-authored menus, toolbar and status; no frame implementation changed. Menu/action completeness and status details remain open. |
| Collaboration, sessions and recovery | Revision-fenced Apply retained. Gold mirror reconciliation, remote cursors, reply-loss recovery, session restore and persisted local drafts remain open. |
| Prefs | Widget resolves palette/style each frame; Ced uses gold's default 13-point editor text. Live settings subscription, font selection, text scaling and keymap compatibility remain open. |
| Other gold views | Diagnostics/problems/output, previews, mail documents/send, macros and complete dialogs remain open. |

Gold Bus surface still to reproduce compatibly: ced.open/new/tabs/focus/show,
ced.state/type/select/execute/actions/wait/layout/stats/diagnostics/problems,
ced.macros.list/reload/run/status/output/stop/new, app.describe/quit, window
drive/debug commands, command manifests, topics and properties. Existing
ced.ping/info responses also retain specimen shapes rather than gold DTOs.
ced.command/draft/intent are specimen interfaces, not compatibility aliases.

Paired 1000x720 gold/Mix captures were inspected with perceptual-difference
figures. They remain visibly different; header, tabs, editor inset and status
regions are open. Private receipts retain the captures. No full functional,
Bus, Prefs, visual or large-file acceptance is claimed.

Reusable widget contract and limits: [editor-widget.md](editor-widget.md).

## Historical round-3 inventory

Oracle: MixOS 8779f3328, apps/ced actions/app/view/bus/mail and edit wire
fixtures. Inventory committed before implementation. Per-app Rust: zero.

| Original capability | Disposition |
|---|---|
| Open/new, file tabs and active selection | Ported bounded Mix edit.open/get; semantic open with create=true provides new documents |
| Native editing buffer, selection and keyboard text | Ported reusable libraries/viu editor control, no file/session policy in Rust |
| Revision-fenced changes and save/save-as | Ported explicit Apply draft then server save; optional semantic destination; no unfenced overwrites |
| Undo/redo/reload and dirty-close refusal | Ported editd commands, preserve draft on conflict; original close/quit prompts remain deferred |
| Search/goto, line operations and commands | Deferred rich commands until the shared model/mirror integration is derived |
| Optimistic typing, remote cursors, OT history/gap/reply-loss reconciliation | Deferred: editor-model mirror is the appropriate reusable owner; this first specimen uses a bounded explicit CAS draft, not a replacement mirror |
| Large paged documents and recovery/epoch reattachment | Deferred; refuse truncated reads rather than presenting partial editable buffers |
| Highlighting, diagnostics/problems/output, markdown previews | Deferred reusable editor-model/syntax/view integration |
| Mail documents/send/headers | Deferred Email/document workflows; never copied into native widget |
| Menus/macros/shortcuts, settings/keymaps/theme/session restore | Deferred original compatibility and generic session host work |

The existing edit Text gap buffer depends on its vendored msedit closure;
editor-model additionally depends on syntax and its mirror protocol. Importing
these wholesale just for a bounded draft would pretend to provide the original
collaboration semantics. The initial native control therefore uses the retained
egui text buffer; future model/mirror integration must retain the original
fixtures, dependencies/licences and convergence tests. This is a bounded editor
specimen, not full Ced parity.

Launch: `viu run apps/ced`. `ced.command {op,path?,create?}` admits open,
apply, save, undo, redo, reload, refresh and close. `ced.info` reports the job
state/tabs; `ced.draft {text}` edits the draft. UI open/path, numeric tab picker,
editor and Apply/Save/Undo/Redo/Refresh/Discard/Close route to Mix policy.
Apply is explicit: typing never silently overwrites a server revision. Refresh
requires an applied or discarded draft. Refusals retain the draft; no automatic
write replay. Generic host quit does not provide original unsaved-draft prompts.

The native control admits up to 65536 UTF-8 bytes and limits interactive text
insertion to 16384 Unicode scalars. Partial/paged snapshots are refused.
Shared production Rust is +12/-5 lines at the initial checkpoint (widget kind,
limits/code-editor drawing and native selection admission), plus +10/-1 shared
test lines. No dependency, interpreter or per-app Rust additions. Native OS
clipboard integration and full per-document editing-model/undo/mirror state
remain deferred; the tested Undo/Redo buttons use editd history.
