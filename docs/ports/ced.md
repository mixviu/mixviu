# Ced parity inventory

Activation results and caller deadlines: [Application control](../app-control.md).

Gold reference: MixOS 8779f3328. Acceptance: see [ced-acceptance.md](ced-acceptance.md) for the exact tested source, native witnesses and inspected capture regions.

Ced is a Mix application over the reusable in-process editor-model Mirror.
Mix owns editd traffic, retries, epoch reattachment, file/scratch sessions,
tabs, actions, menus, dialogs, recovery and mail save/send policy. Root and
Jobs share native document handles; scene publication carries metadata.
The shared frame was merged through lab/gold-parity 5ab4207.

| Surface | Implementation and evidence |
|---|---|
| Editing | Retained virtualised editor, Unicode/IME, cursor/selection, clipboard, wrap, line numbers, overwrite, indentation/comments, line operations and per-origin undo/redo |
| Files and tabs | Open/new, file/scratch tabs, Save/Save As/Save All, reload, dirty close, save-before-close and session restore; scratch Save waits for the native path dialog and disk completion |
| Recovery | editd recovery IDs, restored views, epoch reattachment, Keep mine/Take theirs, conflict Reinsert/Dismiss and recovered-buffer Open/Discard |
| Search and previews | Literal/regex/case find, previous/next, replacement captures, transactional Replace All, goto, source/split/Markdown preview |
| Panels and chrome | Gold grouped menus, Markdown/Theme submenus, toolbar, status, Problems/Output, keyboard shortcuts, About and retained macro popup |
| Mail | Inert classification, body mirror, compact optional header rows, thread metadata, attachment chooser/chips, header undo/redo, save, recovery and explicit Send through Email |
| Live Prefs | Retained theme/typography follower plus editor font/scale/padding/light surface; real settingsd changes and nine paired gold variants |
| Large files | Paged 16/64 MiB native flow and gold byte/line refusals; metadata-only scenes and bounded inline reads |
| Actions and Bus | 61 action IDs share the menu/key/Bus reducer; native tests dispatch every ID, use both successful mail contexts and non-mail guards, and inspect action/macro/selection topics |

Gold's windowed surface includes `ced.dialog`, `INFO`, sixteen native drive
operations and twelve macro operations. The previous seven-macro inventory
was incomplete. `ced.dialog` reads/opens/answers gold-shaped dialogs, including
deferred Open/Save As/Goto receipts. Gold `HELP` descriptors are retained for
all 53 commands. `ced.specimen`, `ced.command`, `ced.draft` and `ced.intent`
are additional specimen interfaces.

Native coverage includes Clipboard/IME, a real native scratch Save dialog,
mail row disclosure and attachment selection, isolated Email acceptance and
outcome-unknown without retry, crash recovery and process termination. The
test Email fixture never delivers external mail. Macro globals belong to
Workshop and leave Ced's catalogue when moved there, as in pinned gold.

The candidate shows both Replace rows under Fable C1: gold clips the second
row and hides part of its controls. The acceptance record retains both
captures and the gold bug note. Other differences are confined to edge
antialiasing. The shutdown-plugin window release restores gold Quit
behaviour; it is not a design deviation.

Gold source corrections include path:line[:col] Open, file directory defaults,
Bus dirty-close refusals without a modal, native dirty-close prompts,
last-holder editd decisions and native disk-overwrite confirmation.

Reusable widget contract: [editor-widget.md](editor-widget.md).
