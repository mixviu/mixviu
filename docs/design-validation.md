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

The worker observes installed source snapshots and checks file-backed sources
for content changes. Hand edits, including generic editor saves, are a bypass
of write-time admission. They receive the background report without an
automatic restyle. A later context choice reads and validates the current
file snapshot before use. No save policy is imposed on the generic editor.
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
  through `shell.scene.validate` before replacing the installed tree. The host
  also admits the current desk design source in full. Keep retains the existing
  copy and performs no write.
- `tools/apps_bundle.mix` runs the sibling `viu --validate-design --embedded`
  before admitting the bundle and requires the validator's source commit to
  match the bundle. The validator is a headless compiler path. Build `viu`
  beside the bundle's `mix`; the bundle and environment gate modes do this.

Admission results have a bounded cache keyed by the digest of authored content.
Parsed-source digests include programmatic edits and exclude diagnostic labels.
File monitoring uses source-byte digests. A path or a context name alone never
identifies a cached validation result. Call and owner-stall deadlines are
unchanged by this policy.
