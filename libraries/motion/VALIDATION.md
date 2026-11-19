# Validation status

The generated Chrome fixture was captured on cos (the media builder), not
CBC, as user markc with Google Chrome 157.0.8103.78 on 2026-11-19. The
orchestrator ran the unchanged Mix oracle script in a scratch directory and
retrieved its JSON output. All 19 presets have 21 samples. The committed
fixture now feeds the ordinary, non-ignored `chrome_oracle` test on CBC.
Per-property errors and final parity results follow after evaluation.

An optimised native worker test of ten objects with three Bézier-eased
properties each measured 852,821 scenes/second, 25,584,616 property
samples/second, or 1,173 ns per thirty-property scene (one million scenes).
This is a micro-benchmark, not an end-to-end compositor frame measurement.
It includes ten calls to the sampler per scene and excludes data loading.
Spring tracks currently prepare their settling envelope at each sample and
are not represented by this Bézier benchmark. Callers directly sampling a
prepared `SpringCurve` avoid this repeated setup.

The foreign dependency gate passed: nineteen existing foreign packages, all
listed, strict engine closures clean. This change adds no foreign package;
serde, serde_json and pure-Rust libm were already in the workspace lock.

Scope: no compositor integration, scene-contract changes, OSD, backdrop,
Mix builtins, desktop access, deployment or full workspace gate.
