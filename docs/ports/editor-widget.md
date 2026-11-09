# Reusable Viu editor

Current source: `libraries/editor`, `libraries/editor-model`, `libraries/edit`
and `libraries/syntax`. The paths in the gold reference below belong to its
historical MixOS revision.

Gold reference: MixOS 8779f3328, `libs/editor`, `libs/editor-model`, `libs/edit`
and `libs/syntax`. Ced composes these facilities from Mix; there is no Ced
Rust controller or editd connection inside the widget.

`editor` with `properties.native_document: true` accepts a model value
`{document: handle}`. The handle names a retained in-process Mirror and
EditorModel. Scene updates contain metadata rather than the document text.
Mix opens/closes handles through `viu_document`, drains outgoing requests,
chooses when to apply them, and delivers editd replies/events. Root and Jobs
share the same document owner. Application-local traffic stays in process.

The retained editor supplies virtualised lines, cursor and selection,
Unicode/grapheme motion, mouse selection, IME preedit/commit, clipboard events,
soft wrap, line numbers, overwrite, indentation/comments and line operations.
The mirror supplies revision fencing, optimistic edits, per-origin undo/redo,
remote carets, retries, recovery and conflicts. Mix owns their transport and
the person's recovery decisions. Native clipboard paste is serial and has
gold's two-second wait.

Syntax highlighting uses the retained language rules. Mix lexing and Markdown
preview parsing run on Jobs; diagnostics remain source-tagged. Find supports
literal/regex, case, wrap-around, previous/next, replacement captures and one
undoable Replace All transaction. Gold's 10,000-operation and 1 MiB replacement
limits remain in force.

The service's 64 MiB and two-million-line limits apply. Paged reads are fenced
by view generation; inline `ced.state` text is bounded at 4 MiB. Native tests
exercise 16/64 MiB files and refusals without copying full text into scenes.

The host follows the retained settings/theme logic. Ced supplies live editor
font, scale, padding, light surface and view choices. Nine real settingsd
variants have paired 1000×720 gold captures. Additional shared controls are
`document-tabs`, `search-bar`, `notice-bar`, `header-form`, `diagnostic-list`, `macro-menu`
and gold-styled modal forms. Header visibility, attachment policy, dialog
answers, tabs, sessions and mail Send stay in Mix.

The legacy string-valued editor remains bounded at 64 KiB for existing callers.
It is separate from the document-handle flow described here.

The Replace rail measures both rows under Fable C1 instead of reproducing
gold’s clipped second row. [Ced acceptance](ced-acceptance.md) records the
matched captures, gold bug note, regional differences, final native witnesses
and CBC receipt.
