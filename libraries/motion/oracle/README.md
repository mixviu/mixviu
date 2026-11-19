The reference CSS and deck runtime are the maintainer's 2026-11-19 deck
templates, copied without changes for a reproducible oracle. They are under
the workspace's MIT OR Apache-2.0 licence. `capture.js` runs in Chrome, never
in a separate JavaScript runtime.

SHA-256 of `reference/anim.css`:
`3c3bf518cff42f18f583ab4cd17b28d89811a9bf5b7c92658d2b2a3662c9e326`.
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

Submit the script through the native CBC queue with `needs: ["chrome"]`,
`kind: "check"`, the pushed commit and command
`["mix", "tools/motion_oracle.mix"]`. It emits `MOTION_ORACLE_JSON=` followed
by one JSON line into the saved CBC log; copy that generated line to
`libraries/motion/oracle/chrome.json` only after a successful run. The script
uses a temporary directory and does not dirty the shared worker checkout.

Capture is currently blocked: on 2026-11-19 the native queue rejected the
Chrome capability request with `BUILDD_SLOT: no configured slot satisfies
this job`. No Chrome fixture or passing oracle result is claimed. Restore
an authorised worker with the `chrome` capability and rerun before accepting
this library's CSS parity gate.
