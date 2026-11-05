# mixviu source guide

mixviu is a Mix-first rebuild informed by measured MixOS experience. Mark's
latest directions take precedence. This repository owns the eventual mix,
mixd, viu and viud sources, shared libraries and Mix applications. Earlier
desktop-only ownership is superseded. Mix/mixd separation is future work.

## Current scope

Mark resumed implementation on 2026-11-05. The isolated lab branch implements
the first bounded client-runtime/Quoin specimen. Prototype code stays on the
lab branch for an independent whole-specimen survey; do not merge it into main.
No production install, engine migration or desktop replacement is authorised.

Mark subsequently authorised hands-on evaluation of the Quoin specimen and
requested a Dopus Mix example adapted from the comparison work. This branch
adds that bounded example for the existing scene host; it remains lab code,
with no main merge or production promotion. See apps/dopus/README.md.

Application/compound-widget composition and behaviour start in Mix. Do not
assign future UI to Rust because it looks dense or complicated. A particular
Rust rewrite needs repeatable representative benchmark failures, diagnosis,
bounded repair and a scoped recorded decision. Existing native primitives are
available; runtime wiring must not hide untested application policy in Rust.

## Layout

- engines/: eventual thin mix, mixd, viu, viud hosts; import as needed.
- libraries/: reusable native code; currently scene, expr and strict only.
- apps/: Mix-first shell/application bundles; currently a Quoin volume specimen.
- tools/: small Mix checks; no copied deployment/orchestration framework.
- docs/: public architecture, development and contracts.
- provenance/: immutable source revisions, file hashes and adaptation notes.
- Tests stay with their owning library/app. Add shared tests only when needed.

Private plans, journals, machine configuration and captures live in the
maintainer's private hub. They never belong in this public tree. Use /home/user,
example.* and RFC 5737 for public examples, and preserve all licences.

## Work and checks

Use Mix for scripts/automation and run_argv for external tools. Never Python,
Bash or sh as a fallback. Probe unfamiliar Mix syntax against its live binary.
Rust builds/tests/checks/fmt/clippy run only through the maintainer's native
build service, never on developer desktops. Do not create local target trees.

Preserve unrelated changes; use isolated worktrees for branch work. Inspect
diffs, run relevant checks, commit and push scoped work. No force pushes or
history resets. Prototype novel features, capture behaviour, obtain an
independent whole-specimen survey and derive production code afterwards.

Use native Bus and host facilities; do not add parallel brokers/transports.
Future runtime imports require dependency/licence inspection and provenance.
The existing source versions/schema names are retained intentionally; rename
only when the owning compatibility contract and migration have been assessed.
