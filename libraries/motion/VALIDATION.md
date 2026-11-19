# Validation status

The shared sampler is pending its browser parity gate. The native build queue
has no configured slot satisfying `needs: ["chrome"]` (2026-11-19).
Consequently all nineteen presets have **unmeasured** maximum browser errors:
up, down, left, right, pop, zoom, fade, wipe, drop, flip, spin, glow, kb,
float, draw, blink, bg, under and shine. No generated fixture is committed.
The ignored `chrome_oracle` test fails if explicitly invoked without a real
fixture; ordinary unit tests cannot establish this gate.

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
