# Shared motion

Pure time sampling for hosts, compositors and widget renderers. No clock,
display or thread is consulted. `presets()` parses the canonical `presets.json`
once; `sample(&preset, &timing, seconds)` evaluates it without allocation.
Existing waypoint, timeline and velocity helpers retain their API after moving
from the compositor's private crate.

The table is a JSON array of `{name, timing, tracks}`. Each track has a
`property` and `keyframes: [{offset, value, easing}]`. Easing belongs to the
segment starting at that keyframe, including intermediate CSS keyframes.
Hosts can decode the same table as a Mix value or call `load_presets` for
validated Rust data. Time and duration are seconds, translations and blur
are logical pixels, rotation is degrees, clip insets are fractions. Infinite
iteration is an explicit tagged value rather than a non-JSON number.

CSS renderer responsibilities and approximations:

| Preset | Renderer contract |
|---|---|
| flip | `scalar` is rotate-X in degrees. Apply perspective distance 900 and top transform origin; a 2D renderer can only approximate it. |
| glow | `scalar` is intensity 0–1: shadow blur 40×, spread 6×, colour rgba(60,200,216,0.35×). This is not opacity of the card. |
| kb | Table translations assume a 100×100 reference box. Multiply tx by width/100 and ty by height/100, then apply scale before translation as CSS does. Origin is 30% 30%. |
| bg | `scalar` is progress 0–1: two backgrounds move (0%,0%)→(18%,10%) and (100%,100%)→(82%,90%). |
| shine | `scalar` is background-position X as a fraction, −2→2. Keep the second background static. |
| draw | Dash offset assumes path length 1000; multiply by actual length/1000. |
| wipe | Inset clip right fraction; renderer supplies rectangular clipping. |
| zoom | Renderer supplies Gaussian blur in logical pixels. |
| spin, under, blink | Renderer supplies CSS-equivalent transform origin (centre, left, centre respectively). |

All other properties directly express CSS scalar values. Apply opacity clamp
0–1 at the renderer boundary when an overshooting curve is used.

The arrow's wipe duration override (0.5 seconds), underline's extra delay
(0.3 seconds), and count-up's cubic ease-out (duration 1.4 seconds) are caller
timing/track choices, not extra named keyframes.

Determinism: the new sampler uses ordered IEEE-754 binary64 operations, no
`mul_add` or fast-math. Spring transcendental functions use pinned pure-Rust
`libm` with default architecture features disabled, avoiding platform system
libm. Finite validated inputs under the standard floating-point environment
therefore follow the same arithmetic path across supported targets. Legacy
route/spring helpers retain their earlier numerical contracts; use
`SpringCurve` for the shared sampler's deterministic spring contract.

Validation happens before sampling. The hot path returns a fixed-size Values
array with a presence mask; an unfilled animation contributes no properties.
Retargeting preserves sampled position; callers needing velocity continuity
can use the existing Hermite helper.

Run crate tests and clippy through the native build service. The ignored
`scene_throughput` test reports samples/second for thirty properties in an
optimised test build. Browser capture instructions and status are in
`oracle/README.md`; browser equivalence must not be claimed without its fixture.
