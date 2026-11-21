# Design validation timing

The compiler has two entry points. `design::compile_design` validates every
reachable context and remains the admission gate for source writes and builds.
`design::compile_design_context` validates the source structure, legacy
crosswalk and the requested context synchronously. An artifact is never used
before that context passes validation. Other contexts are compiled before
their first use; errors refuse that context and leave the current validated
artifact in place.

Native app hosts start one report worker after the first ready frame. It
checks the remaining source contexts without using the behaviour thread or
blocking the rendering thread. A background error is logged and retained in
`<app>.info.design_validation.errors`, keyed by source content digest. Reporting
does not abort, restyle or interrupt an app. A switch attempt that fails also
returns a refusal and shows a dismissible notice. Information retains the
diagnostic after the notice is dismissed.
Malformed choice arguments return `ARGUMENT` without creating a design notice
or error entry. A compile refusal returns `DESIGN_VALIDATION`. Adapter failures
remain ordinary host errors. A repaired source clears its report and notice;
repair alone does not restyle the running app.

The worker observes installed source snapshots and checks file-backed sources
for content changes. Hand edits, including generic editor saves, are a bypass
of write-time admission. They receive the background report without an
automatic restyle. A later context choice reads and validates the current
file snapshot before use. No save policy is imposed on the generic editor.
The observation identity includes both the source-byte digest and its optional
file path. A host watches its current file only; switching to another file or
embedded defaults unregisters the former path. Deletion means embedded defaults,
as at startup. The native task queue has four slots, the reply queue eight,
the pending queue 32, the observation history 128 and the diagnostic map 64.
`pending` counts submitted acknowledgements, not spontaneous file reports.
`skipped` counts unprocessed sources and `evicted` counts retired history or
diagnostic entries, so a bounded queue's degradation is visible in app info.
Pending observations are delivered in FIFO order. A full worker channel puts
the unsent observation back at the front and ends submission for that frame;
newer sources cannot overtake it. Worker failure counts subsequent observations
as skipped instead of leaving a permanently pending queue. A refusal notice
belongs to its source digest, so a successful repair clears it even when the
background report used a different diagnostic label or format.
Scene hand edits retain the scene loader's existing parse/resolve diagnostics
and last-good scene behaviour; a scene document is not a design-source grammar.

Startup preserves the existing file-theme fallback. If the requested file
context cannot compile, `Theme::load` logs the error and starts with the
embedded Studio/Dark theme. The app exposes the error in its info and a visible
notice. Fatal startup versus a formal fallback remains a separate policy
decision. Later failures cannot replace a running app's current context with
this startup fallback.

Write-time gates retain whole-source validation:

- `appearance.source` is admitted by the settings registry before the profile
  is written. A bad unused context refuses the entire source.
- Scene install, upgrade and Reset validate their private staged candidates
  through `shell.scene.validate {source, write:true}` before replacing the
  installed tree. Only that explicit staging call also admits the current desk
  design source in full. Read-only validation, reload and remount are independent
  of errors in unused desk-theme contexts. Scene parsing validates the scene
  being written; it is not a design-source grammar. Keep performs no write.
- `tools/apps_bundle.mix` runs the sibling `viu --validate-design --embedded`
  before admitting the bundle and requires the validator's source commit to
  match the bundle. The validator is a headless compiler path. Build `viu`
  beside the bundle's `mix`; the bundle and environment gate modes do this.

The `shell.scene.validate` Bus body accepts either bare scene source or
`{source, write?}`. Bare source and an omitted or false `write` are read-only;
only `write:true` opts into whole-source desk-theme admission. Both forms still
parse and validate the scene. The native host and all test hosts implement
both forms; the boot-upgrade test crosses the Bus with the staging envelope
and checks that a refused write leaves no installed tree or staging copy.

Admission results have a bounded cache keyed by the digest of authored content.
Admission's parsed-document digests include programmatic edits and exclude
diagnostic labels. Runtime diagnostics use source-byte digests consistently:
the failing snapshot supplies its own digest, never the last accepted theme.
File parsing, compilation and fallback attribution share one read. A path or a context name alone never
identifies a cached validation result. Call and owner-stall deadlines are
unchanged by this policy.
