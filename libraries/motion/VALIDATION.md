# Validation status

The generated Chrome fixture was captured on cos (the media builder), not
CBC, as user markc with Google Chrome 157.0.8103.78 on 2026-11-19. The
orchestrator ran the unchanged Mix oracle script in a scratch directory and
retrieved its JSON output. All 19 presets have 21 samples. The committed
fixture now feeds the ordinary, non-ignored `chrome_oracle` test on CBC.
All 19 presets pass without modifying the sampler, fixture or tolerances.
Maximum absolute errors over each preset's 21 captured times:

| Preset | Maximum absolute error per property |
|---|---|
| up | `opacity` 0.0000004617445881205384, `ty` 0.000029059921924812215 |
| down | `opacity` 0.0000004617445881205384, `ty` 0.000029059921924812215 |
| left | `opacity` 0.0000004617445881205384, `tx` 0.00004405732465428969 |
| right | `opacity` 0.0000004617445881205384, `tx` 0.00004405732465428969 |
| pop | `opacity` 0.0000006376099630500676, `scale` 0.000004767690791496193 |
| zoom | `blur` 0.000004617445881427429, `opacity` 0.0000004617445881205384, `scale` 0.0000033872614400021206 |
| fade | `opacity` 0.0000004617445881205384 |
| wipe | `clip_bottom` 0, `clip_left` 0, `clip_right` 0.0000005128816590316276, `clip_top` 0 |
| drop | `opacity` 0.0000005318001210508072, `scale` 0.0000004849939378992474, `ty` 0.000032901192014378466 |
| flip | `opacity` 0.0000004617445881205384, `scalar` 0.019293141774696654 |
| spin | `opacity` 0.0000004617445881205384, `rotate` 0.000026707931020553133, `scale` 0.0000005604559043526081 |
| glow | `scalar` 0.0000011302796083523248 |
| kb | `scale` 0.000005020819348056094, `tx` 0.000009634177912998254, `ty` 0.000009634177912998254 |
| float | `ty` 0.000004535483112633187 |
| draw | `dash_offset` 0.00048711834085679584 |
| blink | `scale_y` 0.00000002343646540725608 |
| bg | `scalar` 0.00000000000000011102230246251565 |
| under | `scale_x` 0.0000005128816590316276 |
| shine | `scalar` 0.0000036268212504353414 |

Units are pixels for translation/blur/dash offset, degrees for rotation and
flip scalar, fractions for clip, and dimensionless otherwise. The gate uses
1e-4 for opacity/scales/clip and glow/bg/shine scalars; 0.05 for pixel/degree
channels including flip. The largest error, flip scalar 0.0192931418 degrees
at t=0.76 s, follows from the computed matrix serialising m22 as exactly 1,
so the captured acos(m22) projects that small angle to zero. This is below
0.05 degrees; no tolerance relaxation or curve change is needed.

The shine renderer note now matches CSS list repetition: both background
layers receive the same animated position. Capture and projection are unchanged.

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
