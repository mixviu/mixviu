# MixViu app ports — round 1 checkpoint

Steps 0/1/2 are implemented and verified as bounded lab specimens.
No install, desktop/VT change, main merge or production acceptance.

- `viu run APP_DIR [ARGS]` runs Mix bundles through the shared native host.
  Gallery has no Rust; its historical comparator/tests are in gallery-reference.
  Dopus is unchanged; its future host inventory is in docs/ports/generic-host.md.
- Backdrop: colour, five image modes, an actual output override and a native
  Bus settings witness change passed headless capture checks. All six original/
  new wallpaper pairs have AE=0 at 1280×900. Mix owns document/settings policy.
- Monitor: real monitord CPU/memory/process data rendered; native focus/text
  filtering, CPU/memory sorts, pause stability, explicit refresh and tree
  queries passed. Captures were inspected beside the original Monitor.
- Per-app Rust added: **0**. Shared native imports/extensions and line counts
  are recorded in docs/ports/generic-host.md and provenance. The five added
  packages have MIT/Apache licences and no build scripts.

## Exact-source evidence

Validated source: `14943caf69394c93e66acdd7af878b7634707b7f`.
CBC `20261106-195953-c07e7f58-a1`, BUILD_RC=0: fmt check, locked scoped checks,
106 tests, clippy and debug build. Full saved log read; worker diff empty.
Clippy exits successfully with style warnings, including four collapsible-if
suggestions in the new host; this is not a warning-free claim.

Binary SHA256: `47f1e617aa204eba7bc1e9b8bac267247005a543496c4301a9eada3990e7ca3e`.
Final immutable-archive headless run: `native.CvXIFK`, 16 result records.
App archive SHA256: `78c3dc7ea567e10c11d0efacf589c12a875034c1e8beb7bde0068b3e5f65140e`.
Captures inspected directly or verified byte-identical to inspected images;
all nine owned processes confirmed stopped. Private captures stay outside this
public tree. Native source/binary and final archive match the validated SHA.

Pure Mix checks pass: backdrop refusals/output mapping and explicit 2001-process
refusal. A 2000-row installed-interpreter projection sample took 15.6 ms; this
is not embedded UI latency or a performance-parity result. Seed, external-pin,
native-Bus, Dopus-import and app-port integrity checks pass. Runtime-import
verification still hits jwt-rsa-test.pem, already absent at the base revision;
no test/verifier was weakened.

## Limits and deferrals

Full inventories: docs/ports/backdrop.md and docs/ports/monitor.md.
Reference headless compd exposes one output: simultaneous multi-output/hotplug
acceptance remains deferred. Backdrop lacks slideshow/train views,
decode-before-commit/store parity and zero-poll settings following. Monitor's
detailed resource/TCP charts, seeded interactive history, signals, shared action
menus/macros and complete original wire/UI compatibility remain deferred.
These specimens do not establish complete app parity or readiness to install.

Earlier CBC attempts: 20261106-192006-425c8dff-a1 (lock update),
20261106-193321-957e2f64-a1 (logging adaptation),
20261106-194334-64190f42-a1 (scene-colour fix),
20261106-194851-e7308ce5-a1 (exploratory green),
20261106-195718-70b492c0-a1 (scene-key quoting fix). Reporting-only commits
after the validated source change no runtime, bundle or test source.

Branch: lab/app-ports; head/source SHA: 14943caf69394c93e66acdd7af878b7634707b7f (validated source; reporting commit follows); steps done: 0/1/2 bounded lab specimens with listed deferrals; CBC job ids: 20261106-195953-c07e7f58-a1 (final green), 20261106-194851-e7308ce5-a1 (exploratory green).


Round 2 started from 11945c0. Source-derived screenshot/prefs inventories written before implementation. Embedded mix remains 0.117; no installs or desktop changes.
Branch: lab/app-ports; head SHA: 11945c0a2f96f1067aab79050b25a9ceeabd93c4; apps done: round-1 specimens, round-2 inventory only; CBC job ids: 20261106-195953-c07e7f58-a1 (round 1).

Round 2 implementation checkpoint: new Mix screenshot/prefs bundles; reusable generic-host jobs, explicit native timeout bounds, bounded image metadata/exclusive PNG export and native clipboard image submission. Pure Mix capture-shape and settings-fence/refusal checks pass. Native build/render gates pending. No libraries/mix changes.
Branch: lab/app-ports; head SHA: de075ba9c09ffd4ba9b8f6162959c02aaa2f9f0e; apps done: screenshot/prefs prototypes, acceptance pending; CBC job ids: round-2 pending.
