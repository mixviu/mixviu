The reference CSS and deck runtime are the maintainer's 2026-11-19 deck
templates, retained as a reproducible oracle. Entrance CSS was revised on
2026-11-23 for cubic ease-in-out, longer durations and larger travel;
continuous effects are unchanged. They are under
the workspace's MIT OR Apache-2.0 licence. `capture.js` runs in Chrome, never
in a separate JavaScript runtime.

SHA-256 of `reference/anim.css`:
`a3eeb9f220bc0d1092fcfb6d74592b335250fd8d21e4eafc0d7c9a5be2ea2ddc`.
SHA-256 of `reference/deck.js`:
`26a24935798e3e4d4f49e0caac7d4ff75656fc1f13f4a28c8ec2fdf605ae8bf1`.

`tools/motion_oracle.mix` generates a specimen with every preset, runs the
real deck runtime, then samples each animation at 21 equally spaced times
using the runtime's `pause(); currentTime = t * 1000` seek. It reads actual
computed styles, including the underline pseudo-element, and retains both
raw CSS values and the scalar projection used by the library. Elements have
a 100×100 reference box. Ken Burns translation is divided by matrix scale
to recover logical pixels; renderers must scale the percentage translation
to their own box dimensions. Spin angles are unwrapped to -200..0 degrees.

Each fixture is JSON: `{schema, browser, width, height, samples_per_preset,
presets:[{name,duration,samples:[{t,values,raw}]}]}`. `values` uses the
library's property names. Flip scalar is the X angle in degrees, glow scalar
is shadow blur / 40, background scalar is first-layer X position / 18%, and
shine scalar is background X position / 100%. These are explicit renderer
projections, rather than claims of full visual equivalence.

The committed generated fixture was captured on cos, the media builder, as
user markc with Google Chrome 157.0.8103.78 on 2026-11-19. The orchestrator
copied the oracle inputs into a scratch directory, ran the unchanged
`tools/motion_oracle.mix`, retrieved its `MOTION_ORACLE_JSON=` output, and
removed the directory. This was not a CBC capture: no CBC worker currently
advertises the Chrome capability. The private runner is `run_on_cos.mix`.

The fixture contains all 19 presets and 21 samples each. CBC tests consume
this committed data without a browser; `chrome_oracle` is an ordinary gate.
The reference and fixture data are not modified to fit the native sampler.

The gate keeps the original tolerances: opacity and scale channels 1e-4;
translation, rotation, blur and dash offset 0.05 in their native units;
clip fractions 1e-4. Projected flip scalar uses 0.05 degrees; glow, bg and
shine scalars use 1e-4. All 19 presets pass. Full per-property maxima are
recorded in `../VALIDATION.md`.

Run mix tools/motion_oracle.mix --write on the Chrome host to regenerate
both chrome.json and entrance tracks in presets.json. Track endpoints and
intermediate keyframes come from captured times; easing comes from computed
CSS. title_samples additionally pins the title at ages 0, 0.05 and 0.45.
legacy-presets.json freezes the pre-change drawing and sting contract.
