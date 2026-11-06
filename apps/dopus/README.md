# Dopus comparison laboratory

This is an active three-way port, not a completed or accepted replacement.
The retained original Rust application is the reference. This bundle adds
independent Mix and Rust drivers using one app-owned scene and the same shared
Viu controls. Only the Rust comparator calls the imported Dopus core.

Build and test through the authorised remote build service. Run the resulting
lab artifact with `viu-dopus mix APP_DIR PROFILE_DIR` or
`viu-dopus rust APP_DIR PROFILE_DIR`, against the existing native Bus and a
native Wayland compositor. `PROFILE_DIR/config.conf.mix` holds independent
settings; use disposable files for mutation tests. No gold installation follows.

`core.mix`, `operations.mix`, `reducer.mix`, `presentation.mix` and `dopus.mix`
own the Mix application's navigation, selection, file operations, dialogs and
publication. `src/native.rs` is the separately requested Rust comparator.
`scene.mix` and `contract.json` are shared application composition/data.
Reusable rendering, input geometry, image decoding and evaluator facilities
belong to `libraries/viu`; the existing compositor retains window authority.

The header uses app-authored `band` groups over the shared native design solver.
Menu, title, action and caption alignment follow the desktop profile. Native
caption widgets and automatic backend framing detection are reusable Viu code;
the owner retains quit, persistence and reply-delivery rules. Menu form/popup
queries and native geometry are available through the same Bus surface.
The native host adapter offers the existing compd revisioned icon-bar contract.
A registered host query acknowledges support before the ordinary client row
is detached. Menu and nested action popups remain the same shared Viu controls.
The historical `dev.mixos.<service>` application-ID namespace and three service
command aliases are retained solely for the current compositor protocol.
An absent host leaves usable client controls; unacknowledged offers make at most six
attempts within a bounded startup window, then become idle until state changes.
The canonical app commands remain `dopus.*`. `dopus.dialog` accepts the original
open/answer writes and typed `dialog` envelope alongside the Viu form fields.
An unacknowledged offer reports its effective client row honestly. The separate
`dopus.dialog.answer` adds an explicit form-generation fence. Window-local title-bar
and caption-position choices are application-owned commands in both drivers;
Auto resumes the desktop profile. Their effective metadata feeds the same
shared framing, caption and native-host controls.

`HELP`, `dopus.describe`, `dopus.actions` and `dopus.ui.tree` describe the actual
Bus surface. Semantic intents use `dopus.intent`; native input uses the
`dopus.ui.*` commands. Application GPU readback and compositor captures are
separate endpoints, neither proving physical presentation.

`dopus.state {compact:true}` returns the applied pane summaries without the
full row model/geometry. Use this for outcome observations in Bus benchmarks;
retain the default response for complete UI inspection. An execute reply is
not proof of physical presentation. Viu bounds model data at 8 MiB independently
of the scene library's unchanged 256 KiB authored-document limit. Legacy list
templates retain their 500-row limit; native virtual tree tables use their own
row validation and still must fit the model and evaluator budgets.

Opaque file identities use ordinary absolute text paths where UTF-8 is exact,
and local percent-encoded `file:` URIs for other byte names. The reusable
`viu_files` primitive preserves bytes; Mix still owns scanning visibility,
tree recursion, navigation, selection and operation preflight/order. Run
`core-test.mix` through the exact built client's `--macro` interpreter because
the installed shell's ordinary `ls` returns display strings. No installed
binary is selected as the new engine. Copy/rename retain native filesystem
semantics; application preflight is not a filesystem transaction against
concurrent external changes.

The native pilots exercised real selection, byte-equal file copying and
previews in all three versions. The two Viu clients also produced byte-identical
application GPU captures. Shared appearance resolves the original design-token
source, including all twelve schemes. File-open and mail use an in-process Mix
effect job; mail launches only after an explicit absent-service reply, and only
an explicit BUSY compose refusal permits attachment fallback. Both independent
controllers use this common edge integration.

Complete command/feature coverage, appearance matching, native effect delivery,
macros and chooser integration,
stress/lifecycle measurements and independent whole-specimen reviews remain
in progress. The final comparison report must retain failures and measured
limits. A pilot screenshot or a passing scoped test is not full-port acceptance.
