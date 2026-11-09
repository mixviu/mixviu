# Scene documents: P0 reference

Baseline: `881e405162dc42a20361cce53a68ffe1f7225c39` (2026-11-09).
This describes the parser, loader and generic `viu --host` separately: accepting
a port does not prove that every renderer implements it. The schema is in
`libraries/scene/src/schema.rs:89`; the current host dispatch is in
`engines/viu/src/runtime.rs:925`. Later parity phases must update this baseline.

## Directory and lifecycle

The catalogue is `share/scenes/`; installations live at
`$SCENES_DIR/<name>/`, defaulting to `$XDG_CONFIG_HOME/mix/scenes` (or
`$HOME/.config/mix/scenes`). The loader is a `mix --serve` citizen named
`scenes`, targeting `shell` by default. It locks the root. Catalogue discovery
uses `SCENES_TEMPLATES`, then `MIX_SHARE/scenes`, then `MIX_ROOT/share/scenes`,
then a repository ancestor. (`engines/viud/scripts/scenes.mix:10`,
`engines/viud/scripts/lib/scenes_io.mix:38`.)

| File | Contract and evidence |
|---|---|
| `scene.mix` | Required regular file, at most 262144 bytes; envelope name must match directory. `engines/viud/scripts/lib/scenes.mix:544` |
| `template.conf.mix` | Required strict-data map, at most 65536 bytes. `title` supplies display title; metadata is not a capability sandbox. `engines/viud/scripts/lib/scenes.mix:566` |
| `behaviour.mix` | Optional regular executable Mix source, at most 262144 bytes. Its presence, rather than the metadata's behaviour label, determines execution. Requires citizen `scene-<name>`, with name at most 25 characters and no underscore. `engines/viud/scripts/lib/scenes.mix:573` |
| Local helpers | Behaviour may `require` files beside itself using `SCENE_DIR`; shared helpers come from `SCENES_LIB`, default catalogue `/lib`. `engines/viud/scripts/lib/scenes_io.mix:79`, `engines/viud/scripts/scenes.mix:14` |
| Settings | No generic scene settings schema or binding root is installed by the loader: the candidate reads the three files above. `content.settings` is a widget object port, not a scene settings service. `engines/viud/scripts/lib/scenes.mix:544`, `libraries/scene/src/schema.rs:131` |
| Loader state | `state.conf.mix` records enablement, origins, recovery and forks; unreadable state blocks destructive writes. At most 128 scenes. `engines/viud/scripts/lib/scenes.mix:360`, `engines/viud/scripts/lib/scenes.mix:384` |

Behaviours run in their scene directory as unrestricted `mix --serve`, with
`SCENE_NAME`, `SCENE_GENERATION`, `SCENE_DIR`, `SCENE_HOST`, `SCENES_SERVICE`.
They are children terminated with SIGKILL on invalidation. No declared
`requires` capability enforcement exists at this launch boundary.
(`engines/viud/scripts/lib/scenes_io.mix:81`,
`engines/viud/scripts/lib/scenes_io.mix:89`.)

## Document grammar

Minimal document (the outer four-backtick fence here is documentation only):

````text
---
scene: 1
name: hello
citizen: scene-hello
window: {"kind":"edge","edge":"bottom","panel":"hello"}
model: {"label":"Hello"}
---
```mix
root: {widget: text, text: '= $model.label'}
```
````

Source begins with literal `---\n`; headers are unindented `key: value`
lines ending at `\n---\n`. A header starting with `[` or `{` must be JSON,
not Mix map syntax. Required headers are `scene: 1`, `name`, `citizen`;
`window`, `subscribe`, `targets`, `model` are JSON-valued optional headers.
Exactly one `mix` fence supplies a map of node IDs to node maps. Each node
needs a string `widget`; every other key is a port. `@` is reserved for
generated instance IDs. (`libraries/scene/src/envelope.rs:28`,
`libraries/scene/src/lib.rs:395`.)

Fence contents use strict data: optional top-level braces, newline/comma
separators at top level, comma separators inside maps/lists, trailing commas,
bare or quoted keys, bare word string values, `nil`, booleans, f64 numbers,
quoted/raw strings and heredocs. Duplicate map keys fail. No variables,
calls, interpolation, command substitutions or semicolons execute here.
Use raw strings for binding source containing interpolation syntax.
(`libraries/strict/src/lib.rs:5`, `libraries/strict/src/lib.rs:28`.)

There must be a node `root`. Children reference IDs, never inline node maps.
Cycles, dangling children and multiple parents fail; unreachable nodes warn.
List row roots and descendants are templates. Legacy `{cells[N]}` substitution
is confined to template `text.text` and `image.src`; index validity is checked
against available cells. (`libraries/scene/src/lib.rs:608`,
`libraries/scene/src/lib.rs:743`, `libraries/scene/src/lib.rs:988`.)

Window metadata has `kind: edge|dialog|toast` (header defaults to edge),
`edge: right|left|top|bottom`, optional string title, numeric nonnegative
`w`/`h`, and chrome. Dialog requires w/h in 240..2048 and forbids edge/panel;
toast also forbids edge/panel. Window nodes and headers must agree.
Managed pages and edges must be static. The loader reserves dialog/toast for
the editor and shipped system scenes, and reserves page `scene-editor`.
(`libraries/scene/src/lib.rs:232`, `libraries/scene/src/lib.rs:249`,
`libraries/scene/src/lib.rs:750`, `engines/viud/scripts/lib/scenes.mix:551`.)

## Widget ports

Notation: `!` required; `?` optional with no default; `=v` default. Types:
`s` string, `n` number, `b` bool, `l` list, `o` object. Numeric ports below
are nonnegative unless noted; `row_height` is strictly positive. These are
schema ports, including families used by ordinary app scenes. All nineteen
families are selected at `libraries/scene/src/schema.rs:279`.

| Family | Ports | Evidence |
|---|---|---|
| window | kind:s! (edge/dialog/toast), edge:s=right, title:s?, w:n?, h:n?, chrome:b=true | `libraries/scene/src/schema.rs:94` |
| column | children:l!, gap:n=0, padding:n=0, fill:b=false, align:s=stretch, height:n?, radius:n?, background:s?, scroll:b=false | `libraries/scene/src/schema.rs:109` |
| row | children:l!, wrap:b=false, gap:n=0, padding:n=0, fill:b=false, align:s=start, height:n?, radius:n?, background:s?, hover:s?, on_click:s?, on_context:s? | `libraries/scene/src/schema.rs:158` |
| section | children:l!, titles:l!, frame:s=plain (plain/dock), on_change:s?, actions:l?, on_action:s?, focused:b? | `libraries/scene/src/schema.rs:122` |
| content | children:l!, settings:o={}, scroll:b=true, surface:s=panel (panel/editor), keyboard_scroll:o? | `libraries/scene/src/schema.rs:131` |
| panes | children:l!, left_fraction:n=0.18, right_fraction:n=0.42, right_visible:b=true, clip:b=true, focus:s=center (left/center/right), on_change:s?, compact_below:n?, narrow_below:n?, narrow_left_range:l?, narrow_right_range:l?, keyboard:b=false, focus_targets:o?, pane_labels:o? | `libraries/scene/src/schema.rs:138` |
| sides | children:l!, wrap:b=false | `libraries/scene/src/schema.rs:154` |
| text | text:s!, size:n=13, bold:b=false, mono:b=false, color:s?, elide:b=false, wrap:b=false, max_lines:n=3 (integer 1..1024), width:n?, fill:b=false, hidden:b=false, align:s=left (left/center/right) | `libraries/scene/src/schema.rs:180` |
| field | value:s!, placeholder:s?, width:n?, password:b=false, on_change:s?, on_submit:s? | `libraries/scene/src/schema.rs:194` |
| button | label:s!, tone:s=normal (normal/danger/primary), width:n?, on_click:s?, on_context:s? | `libraries/scene/src/schema.rs:202` |
| toggle | value:b!, label:s!, on_change:s? | `libraries/scene/src/schema.rs:217` |
| list | rows:l!, row:s!, row_height:n!, gap:n=0, max_rows:n? (>=1), fill:b=false, hidden_if_empty:b=false, on_click:s?, on_context:s?, flow:s? (vertical/horizontal), align:s? | `libraries/scene/src/schema.rs:222` |
| image | src:s!, w:n?, h:n? | `libraries/scene/src/schema.rs:238` |
| spacer | size:n? | `libraries/scene/src/schema.rs:243` |
| control | definition:o!, on_intent:s? | `libraries/scene/src/schema.rs:244` |
| grid | children:l!, columns:n! (integer 1..16), identity:s?, column_gap:n=12, row_gap:n=3, min_column_width:n=0 | `libraries/scene/src/schema.rs:248` |
| dock | children:l!, placements:o!, clip:b?, on_change:s?, outline:s?, outline_width:n?, passthrough_single:b=false, focus_target:s? | `libraries/scene/src/schema.rs:256` |
| band | children:l!, groups:o! | `libraries/scene/src/schema.rs:266` |
| split | children:l!, ratio:n=0.5, axis:s=horizontal (horizontal/vertical), divider:n=6 (>=1), minimum:n=0.1, maximum:n=0.9, on_change:s?; ratio/minimum/maximum <=1 | `libraries/scene/src/schema.rs:270`, `libraries/scene/src/schema.rs:431` |

Additional ports (no defaults unless shown):

| Applies to | Ports | Evidence |
|---|---|---|
| Every non-window | window_slot:s, slot_only:s, hidden:b (text already defaults false), clip:b, enabled:b, clip_below:n, align_self:s (auto/start/center/end/stretch), grow:n, shrink:n, basis:n, min_width:n, max_width:n, min_height:n, max_height:n, reserve_height:n | `libraries/scene/src/schema.rs:332` |
| row | packing:s (normal/toolbar), end:s, fit:l, flex:s | `libraries/scene/src/schema.rs:302` |
| row/column/button | tooltip:o; intended shape title, optional subtitle/lines | `libraries/scene/src/schema.rs:360` |
| row/column | scroll:b, justify:s (start/center/end/between/around/evenly), bar:s, padding_x:length, padding_y:length, native_spacing:b, native_frame:b, group_frame:b, group_title:s, group_key:s, scroll_key:s, scroll_ready:b, reserve_panel_after:b, row_gap:n, column_gap:n, padding_top:n, padding_right:n, padding_bottom:n, padding_left:n | `libraries/scene/src/schema.rs:366` |

Row/column align and list align are start/center/end/stretch. `length` accepts
nonnegative finite numbers or `design.bar_inset`, `design.bar_padding_y`,
`design.tool_margin`. The schema appends duplicate scroll/clip declarations;
do not interpret duplicates as separate ports. `describe` returns path, type,
mutable, sensitive, description, enum/default/min/max; field.value is sensitive.
(`libraries/scene/src/schema.rs:90`, `libraries/scene/src/schema.rs:317`,
`libraries/scene/src/schema.rs:397`, `libraries/scene/src/schema.rs:505`.)

Explicit grow/shrink/basis conflicts with authored fill (even false), spacer
size, or fixed row height. Min width/height cannot exceed max; list max_rows
cannot coexist with max_height. Rows are objects containing unique nonempty
string id and string-list cells (additional fields can be read through item).
(`libraries/scene/src/lib.rs:329`, `libraries/scene/src/schema.rs:525`.)

## Pure bindings and model patches

A string port whose trimmed source starts `=` is compiled as one expression.
`children`, `row`, `widget` cannot bind. Only `$model` and template-only
`$item` roots are allowed. There is no `$src`, `$instance`, sources declaration
or structural `repeat`. The nineteen-family schema has no repeater.
(`libraries/scene/src/bindings.rs:155`, `libraries/scene/src/bindings.rs:199`,
`libraries/scene/src/bindings.rs:222`, `libraries/scene/src/schema.rs:279`.)

Allowed: numbers, strings, booleans, nil, list/map literals, variables,
field/index access, interpolation, unary minus/not/!, arithmetic, comparison,
concatenation, coalescing, ternary and `if … then … elif … else … end`.
No calls (even pure builtins), assignments, loops, handlers, shell or Bus forms.
Precedence low to high: ternary; or; and; comparisons; ??; ..; + -; * / %;
**; unary. Binary operators including ** associate left; ternary associates
right. (`libraries/expr/src/parser.rs:3`, `libraries/expr/src/parser.rs:35`,
`libraries/expr/src/ast.rs:1`, `libraries/scene/src/evaluator.rs:95`.)

`and`, `or`, `??` short circuit and return operands. Missing object fields and
out-of-range indices yield nil; accessing a field on a non-object fails.
Negative indices count from the end. Prefer `..` for text: `+` numerically
adds coercible scalars, otherwise joins scalars, but refuses nil/collections.
Division/modulo by zero fails. (`libraries/expr/src/eval.rs:216`,
`libraries/expr/src/eval.rs:251`, `libraries/expr/src/eval.rs:285`.)

Dependencies use dotted model paths; indexing stops path precision at the
collection, e.g. `$model.rows[$model.index].label` depends on model.rows and
model.index. Patch paths are `model` or `model.a.b`, map keys only, no array
indices or escaping dots. Nil at root resets to {}; nil at a key deletes it;
intermediate maps may be created but traversal through a scalar fails.
Re-evaluate only dependencies equal to, ancestors of, or descendants of the
patch path. (`libraries/expr/src/analysis.rs:71`,
`libraries/scene/src/bindings.rs:273`, `libraries/scene/src/bindings.rs:551`.)

Nil binding results restore a port default or remove an optional port. Wrong
types/ranges produce binding-type. A model patch keeps last-good ports on
individual evaluation failures and reports warnings; layout conflicts refuse
the complete patch. (`libraries/scene/src/bindings.rs:244`,
`libraries/scene/src/bindings.rs:318`, `libraries/scene/src/bindings.rs:494`.)

The host accepts `shell.scene.patch {scene,path,value,generation?}`. A managed
model requires the current nonzero generation. Literal node.port patches
change the authored document and undergo acceptance again. Accepted changes
advance revision and digest; mount moves via model patch fail. Behaviours
currently publish a **whole model**, not fine-grained patches:
`publish_model` -> `scenes.model` -> `shell.scene.patch {path:"model"}`.
Equal models are suppressed. (`engines/viu/src/store.rs:574`,
`share/scenes/lib/runtime.mix:92`, `engines/viud/scripts/lib/scenes.mix:2179`.)

Lists instantiate row templates with `$item`; legacy cells substitution runs
after binding evaluation and never reinterprets bound results as substitution
source. The host refuses nested lists and preflights templates before commit.
(`engines/viu/src/templates.rs:38`, `engines/viu/src/templates.rs:102`.)

## Bounds and diagnostics

| Bound | Current value and evidence |
|---|---|
| Document / core model patch | 256 KiB; 2000 authored nodes; 500 rows/list. `libraries/scene/src/lib.rs:36`, `libraries/scene/src/bindings.rs:253` |
| Strict data nesting | 200. `libraries/strict/src/lib.rs:70` |
| Expression nesting / tree depth | Parser 200; analysed AST depth 256. `libraries/expr/src/parser.rs:15`, `libraries/expr/src/analysis.rs:10` |
| Evaluation | 250 ms across pass, at most 50 ms/expression; constructed string 1 MiB, list 500, map 1024. No step cap in scene adapter. `libraries/scene/src/bindings.rs:18`, `libraries/scene/src/bindings.rs:476`, `libraries/scene/src/evaluator.rs:125` |
| Templates | Shared revision budget: 16384 instantiated nodes and 250 ms across all lists/rows. `libraries/scene/src/bindings.rs:369` |
| Loader | 128 installations; metadata 64 KiB; behaviour/document/model 256 KiB. `engines/viud/scripts/lib/scenes.mix:395`, `engines/viud/scripts/lib/scenes.mix:544`, `engines/viud/scripts/lib/scenes.mix:2189` |

Diagnostics are `{severity: Error|Warning, code:string, line:number,
message:string}`. Core catalogue below comes from
`libraries/scene/src/lib.rs:43`; **port-max** is also emitted by
`libraries/scene/src/schema.rs:431` but is missing from that catalogue.

| Area | Codes |
|---|---|
| Envelope/source | document-too-large, envelope, missing-header, scene-version, invalid-name, fence-count, mix-parse, strict-data, duplicate-id, root-type, node-type, missing-widget, invalid-id, header-json |
| Schema/tree | node-limit, unknown-family, unknown-port, port-type, enum-value, port-min, port-max, layout-conflict, missing-port, dangling-child, child-type, window-kind, row-limit, row-type, invalid-template, cell-substitution, missing-root, multiple-parents, window-disagreement, window-dialog-edge, orphan-node, cycle |
| Bindings/patch | invalid-binding, binding-policy, binding-not-allowed, binding-eval, binding-type, model-path |

Loader failures use uppercase `SCENES_*` and reply rc 10 with
`{error_code,message,context}`; they are separate from scene lint codes.
Behaviour parsing and `mix lint --deny-warnings` gate normal behaviour;
the shipped editor may report lint findings instead. Host validation returns
diagnostics, and watch exposes revision/digest/applied_revision/diagnostics.
(`engines/viud/scripts/lib/scenes.mix:2231`,
`engines/viud/scripts/lib/scenes_io.mix:50`, `engines/viu/src/store.rs:492`,
`engines/viu/src/store.rs:547`.)

## Reload and current limits

Filesystem batches reconcile changes to installed scene.mix, behaviour.mix,
template.conf.mix and scene directories; overflow/root/state changes rescan.
Shared or user helper lib edits do not trigger dependent reload. Editor copy
handling is special. (`engines/viud/scripts/lib/scenes.mix:991`.)

Reconcile checks changed behaviour before mounting. Scene edits load against
the last accepted runtime model (or authored model initially); behaviour-only
edits restart the behaviour. Forced `scenes.reload` rechecks and restarts even
without digest changes. File refusal retains the mounted last-good scene and
a load diagnostic with attempted digest and last-good revision; a later ready
or model publication does not clear that file refusal. Successful load clears
it. (`engines/viud/scripts/lib/scenes.mix:729`,
`engines/viud/scripts/lib/scenes.mix:748`,
`engines/viud/scripts/lib/scenes.mix:2153`.)

This is kill/spawn, without behaviour state hand-off or an error overlay.
Reloading user copies does not copy updated catalogue templates; reset does,
with a recovery copy. There is no reload transaction over shared imports.
(`engines/viud/scripts/lib/scenes_io.mix:81`,
`engines/viud/scripts/lib/scenes.mix:2053`,
`engines/viud/scripts/lib/scenes.mix:991`.)

Each page has one output seat; loader loads do not specify an output and
host dispatch selects one. There is no per-output instancing. The host polls
compositor outputs at one-second ticks. (`engines/viu/src/runtime.rs:878`,
`engines/viu/src/runtime.rs:1048`, `engines/viu/src/runtime.rs:750`.)

Current host limits matter to consumers: scene.layout returns
SCENE_NOT_RENDERED; dialog, menu and panel.order paths fall through to
NOT_IMPLEMENTED; props.get ignores path and returns panels as an array.
The loader popup code requires a panels map, a live contract mismatch.
Registration does not imply those operations work.
(`engines/viu/src/runtime.rs:933`, `engines/viu/src/runtime.rs:1000`,
`engines/viu/src/runtime.rs:1062`,
`engines/viud/scripts/lib/scenes.mix:2204`.)
