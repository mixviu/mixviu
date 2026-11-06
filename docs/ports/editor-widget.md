# Reusable Viu editor

The `editor` control opts into the retained gold editor through
`properties.native_document: true`. The default control remains compatible
with the existing mail reader. Gold source reference: MixOS 8779f3328,
`libs/editor`, `libs/editor-model`, `libs/edit` and `libs/syntax`; these
libraries were already imported in this repository. No new third-party
dependency or application Rust controller is introduced.

The adapter owns one in-process document per `(widget id, document)` pair.
`document` must identify a tab uniquely. `language` chooses highlighting;
`soft_wrap`, `line_numbers` and `read_only` select view options. Monospace
size and palette resolve from the active Viu style on every frame; optional
`font_size` (8..72 points) overrides the size. The gold
widget owns mouse selection, grapheme motion, indentation, comments, line
operations, clipboard events, IME composition and visible-row rendering.
Undo/redo remain local and survive model echoes. Ctrl+F opens literal find,
with wrap-around next match and single-selection replace. The find bar also
opens through `find_bar: true`.

`command: {serial, name}` applies a local command once per increasing serial:
undo, redo, select-all, duplicate-line, delete-line, move-line-up/down,
toggle-comment, indent and outdent. Integer-valued Mix numbers are admitted
without accepting fractional or unsafe serials. Read-only views hold pending
commands until editing is enabled. These commands never contact editd.

The only editing output is the existing local `set` intent containing text.
Mix chooses whether and when to apply it through editd. This adapter performs
no file or Bus I/O. Model updates acknowledge equal text without replacing
the document; different text explicitly reloads it. Native document admission
is bounded at 64 MiB; legacy editor admission remains 64 KiB.

This is an integration checkpoint, not full parity acceptance. Full-text
intent publication and model comparison still cost O(document bytes) after
edits; visible-row rendering alone does not establish large-file interaction
performance. OS clipboard and IME require host acceptance. Regex/all-replace,
gold find options, closed-tab document eviction, command/state introspection,
live settings subscription and deterministic paired captures remain open.
Ced's bounded editd snapshot and explicit Apply policy are unchanged at this
checkpoint. No visual acceptance is claimed.
