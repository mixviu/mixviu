# Ced round-3 inventory

Oracle: MixOS 8779f3328, apps/ced actions/app/view/bus/mail and edit wire
fixtures. Inventory committed before implementation. Per-app Rust: zero.

| Original capability | Disposition |
|---|---|
| Open/new, file tabs and active selection | Planned bounded Mix port over edit.open/get |
| Native editing buffer, selection and keyboard text | Planned reusable libraries/viu editor control, no file/session policy in Rust |
| Revision-fenced changes and save/save-as | Planned explicit Apply draft, then server save; no unfenced overwrites |
| Undo/redo/reload and dirty-close refusal | Planned editd commands, preserve draft on conflict |
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
