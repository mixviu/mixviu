# Scene motion

Scene composition declares motion; Mix behaviours change the bound model.
The motion library samples tracks at the presentation timestamp. The toolkit
adapter paints those samples. No widget or model is retained after removal:
exits retain paint only.

Optional ports on visual nodes (no authored defaults):

| Port | Shape |
| --- | --- |
| `enter`, `exit` | `"up"` or `{preset:"up", delay_ms:0, stagger_ms:0}` |
| `transition` | `{prop:"opacity", ms:160, ease:{kind:"ease_out"}}`, or a list |
| `motion` | `false` disables node motion |
| `opacity` | Number in 0..1, optionally bound |
| `offset` | `[x,y]` in scene units, optionally bound |

Finite presets: `fade`, `up`, `down`, `left`, `right`, `pop`, `drop`.
Unknown effects and fields are refused. Durations and delays must be in
0..60000 milliseconds. Easing uses the motion library's tagged curve objects:
`linear`, `ease`, `ease_in`, `ease_out`, `ease_in_out`, `bezier`, `steps`,
and `spring`, with the library's parameters. Omitted easing is linear.
Duplicate transition properties are refused.

Bound transitions: opacity, offset and literal `#rrggbb` or
`#rrggbbaa` colour/background/hover ports. Retargeting begins at the current
sample. Session reduced motion snaps transitions and removes retained exits.
`highlight` interpolates the actual foreground and background colours painted
for hover, press, selection and toggles, using the declared duration and curve.

A container declaration with nonzero `stagger_ms` delegates its preset to its
direct children: child `i` starts at `delay_ms + i * stagger_ms`. Descendant
paint follows its containing child's transform. Entrance and exit sampling
uses the shared table; no model update waits for motion to finish.

Live transforms affect paint and input together, including scale. Exits retain
the last paint and references to its managed image textures, and run no widget
code or event route. Those references are released when the exit completes.

Current default changes: shared switch knobs slide over 160 ms; shared push
buttons travel one scene unit over 90 ms; furniture toggles and buttons use
those same curves. Native menus and floating popups fade on open/close; list
Primary and secondary app buttons use the shared push button's own travel;
the app wrapper does not apply a second travel transform.
rows fade on insertion/removal; tab content fades on switches. Default app
toggles use the shared switch. These defaults honour global reduced motion and
`motion:false` on a scene node (including its subtree). Ordinary scene nodes
gain no entrance/exit by default.

The Gallery includes a Motion page with a visibility button, switch, seven
entrances, stagger, bound opacity/offset/colour, interaction highlights, native
menus/popups, tabs and list insertion/removal. App behaviour is Mix.

All samplers use the frame's supplied timestamp. Delayed entrances schedule
their next start; active motion requests frames; settled motion requests none.
The headless runtime also accepts explicit presentation timestamps for replay
and tests. The cost test includes layout, painting and tessellation, without a
GPU submission; its receipt belongs to the PR rather than the contract.
