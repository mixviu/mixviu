# Essential import scope

Source: markc/mixos at 884fdd8ef2c548cc42017b9170593ec67ed30e57.
File hashes and reasons are in imports.json. No source history was transplanted.

## Included now

- libraries/scene: existing renderer-neutral scene document/validation/binding
  machinery and its tests/fixtures.
- libraries/expr and libraries/strict: that library's complete first-party
  dependency closure, with tests/fixtures.
- apps/volume-lab/volume: one existing useful shell specimen, its template and the
  seven helper modules its current module-level dependency graph requires.

The helpers remain a captured baseline; their current broad models module
requires several helpers that can later be separated by measured use. There
is no new application logic. The volume helper-root default changes to the
copied relative layout, retaining the explicit override. One trailing blank
line is removed from taskbar.mix; the adaptation is recorded separately.

Cargo.lock contains only reachable packages, with external checksums unchanged.
The crate files keep their API/schema/source identity; the root integration
supplies package metadata and lint defaults.

## Deferred until needed

Mix interpreter/shell and Bus dependency closures, renderer/toolkit extraction,
compositor tree, server personalities, Dopus/Email source and future Mix/mixd
split. The repository owns these future components, but copying all of them now
would reintroduce the old workspace before the first runtime slice needs it.

No old workspace root, apps/services tree, production settings, build artifacts,
captures, journals, plans, keys, vendor tree or deployment machinery is copied.
Historical schema identifiers and public MixOS references in retained source
are provenance, not the new project's strategy.

Imported files are unmodified except the adaptation recorded per file.
Required future imports retain their licences/tests and get the same inventory.
