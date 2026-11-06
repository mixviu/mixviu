# Ced parity inventory

Gold reference: MixOS 8779f3328. Full parity is not yet accepted.

The current Mix application uses the retained paged editor-model Mirror through
a reusable in-process Viu document handle. Mix owns editd transport, retries,
epoch reattachment, session policy, tabs, actions and mail save/send policy.
Scene publication carries metadata, never complete large documents.

Native evidence covers 64 MiB documents and gold byte/line refusals, file and
scratch recovery across Ced and editd restarts, real settingsd changes, native
clipboard and IME preedit/commit, file save/undo/redo, regex replacement,
diagnostics, macros and inert mail body/header save and crash recovery.
Tabs, search controls, dialogs and Markdown previews use shared Viu widgets.
The shared frame is merged from lab/gold-parity through 0807e73.

Gold DTOs are implemented for the application commands and native window
drive/debug surfaces. Gold macro transport exposes twelve operations, including
get/put/remove/move/copy; the older seven-command inventory was incomplete.
The 61-action inventory is registered. Registration alone is not acceptance:
complete action scenarios, topics, cancellation and visual states still need
final coverage. ced.specimen is the diagnostic surface; ced.info uses gold DTOs.
ced.command/draft/intent remain specimen interfaces.

Remaining acceptance includes successful mail Send through an isolated Email
fixture, complete action/panel interactions, matched live-Prefs gold captures,
full topic/property comparison and closure of every visual region. No deviation
is approved. Current native witnesses and CBC receipts are recorded in last.md.

Reusable widget contract: [editor-widget.md](editor-widget.md).

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
