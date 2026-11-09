# Quoin P3 discovery specimen

Approved as a lab on 2026-11-09. This branch is never merged. The report and
CBC receipts are in `last.md`, including the round-two addendum. Production
derivation waits for P2 sources and an independent whole-specimen survey.

Two entry points use the same output-feed seam and existing native Bus:

- `loader.mix`: `instances: per_output | primary | named` in
  `template.conf.mix`; `P3_SHAPE=shared|instance` chooses the D2 behaviour shape.
- `theatre-loader.mix`: strict-data `theatre.conf.mix` expands output rules
  into places, then casts registered scenes onto them. It is a Mix Bus citizen
  named `lab`, with `stages`, `cast`, `uncast`, `mode` and a lab preview bridge.

`instances.mix` lowers bounded `repeat` into P0 list templates and substitutes
reserved model sections for the future read-only `$instance` and `$src` roots.
`theatre.mix` supplies stage expansion, persistent cast overrides and placement
fallback. No stage-specific Rust host implementation was needed.

The panel retains the shipped panel's complete node composition, with bounded
authored defaults for service-dependent content. The Network fixture is the
unchanged shipped scene. This does not activate the shipped device behaviours.

Run the bounded adapter checks with Mix:

```text
mix apps/quoin-p3-lab/check.mix
mix apps/quoin-p3-lab/theatre-check.mix
```

Build through CBC, after pushing the chosen SHA:

```text
mix ~/.mix/_bin/cbc-build.mix --repo mixviu --commit SHA -- mix apps/quoin-p3-lab/cbc.mix
```

The gate runs the Rust formatter, builds viu/viud and the pixel probe, runs
headless tests and the main identity-collision reproducer, and saves a snapshot.
Apply any printed formatting diff locally. Read `BUILD_RC=` and the saved log.
The collision test deliberately asserts existing faulty behaviour, as evidence;
it is not a production regression test to merge.

`live.mix HARNESS BIN-DIR PRIVATE-RUN-DIR` requires the native desktop harness
module beside its `smoke-settings.json` and the CBC snapshot containing
`viu`, `viud`, and `testkit-panel-pixels`. It creates an isolated native broker,
two headless outputs and a timed unplug, and cleans up its children. Run once
per D2 shape. `theatre-live.mix` takes the same arguments followed by a P1
`viu --preview` binary and its scene-preview validator. P1 remains read-only.

The live checks wait for a presented scene revision before capturing. Each
output has its own name, current-workspace highlight and footer colour; repeated
captures reject blank or crossed panels. Native workspace switching currently
accepts only the default output, so the specimen shows workspace 2 there and
workspace 1 on its peer. That native policy limit is reported, not replaced.

The stage specimen supports one active cast per stage. Preview uses P1's passive
window and a private materialised transport copy to pass its installed-scene
validator; authored scene files remain unchanged. Generic scene behaviour
lifecycle, capabilities, stage ownership and final vocabulary remain survey
and Fable decisions.
