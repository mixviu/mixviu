# OSD scenes

`osd` is a Mix behaviour on a privileged viu scene host. viud launches the
host directly and authenticates its Wayland peer against the child's PID and
process start time. The `viu-osd` namespace alone grants nothing: another
client claiming it is closed. The compositor attaches the OSD mark to the
surface resource; neither an app ID nor `comp.window.place` can supply it.

Marked surfaces compose in the OSD band above client and screen UI bands.
The compositor draws an amber provenance mark at the output's upper left
while OSD content is mapped. Session lock uses its separate lock-only scene.
The host takes no keyboard focus, including when pointer input is enabled.

| Command | Body |
| --- | --- |
| `osd.show` | `{id, scene, anchor, enter?, exit?, duration_ms?, pointer_input?}` |
| `osd.update` | `{id, scene?, anchor?, enter?, exit?, duration_ms?, pointer_input?}` |
| `osd.hide` | `{id, exit?}` |
| `osd.list` | `{}` |
| `osd.actions` | `{}` |
| `osd.execute` | `{id:"osd.show"|"osd.update"|"osd.hide", params:{...}}` |

The initial literal scene catalogue is
`{kind:"title"|"caption"|"card", text, title?, width?, height?}`.
These are reviewed viu scenes with text supplied as model data. Requests
cannot introduce executable Mix, expressions, bindings or arbitrary widgets.
Text is limited to 8192 characters per field; at most 32 items may exist.

An output anchor is `{output, edge, margin?}`. `edge` is `top`, `bottom`,
`left`, `right`, or a corner such as `bottom-right`. Margin defaults to 16
logical pixels and may be a number or `{top,right,bottom,left}`.
A window anchor is `{window:{id,generation},offset?:{x,y}}`. It follows the
window's projected output-local position and draws nothing while that role
is hidden, minimised or gone. A stale generation is refused before mutation.

`enter` and `exit` use the shared motion library's `fade`, `up`, `down`,
`left`, `right` and `pop` presets. Exits sample the entrance in reverse.
`duration_ms` is an optional lifetime in milliseconds, including zero;
`null` removes the deadline. `pointer_input` defaults to false. Opting in
adds only that item's region to the host's pointer input region. The choice
is visible in `osd.list`.

Validation precedes mutation. Refusals carry an `error_code`, including
`INVALID_ARGS`, `INVALID_SCENE`, `INVALID_ANCHOR`, `INVALID_MOTION`,
`ALREADY_EXISTS`, `NOT_FOUND`, `BUDGET_EXCEEDED`, `WINDOW_NOT_FOUND`,
`STALE_GENERATION` and `UNKNOWN_ACTION`.

Ordinary output captures include the OSD because it is part of the output
scene. Listed-window captures exclude it with all other non-listed surfaces.
There is no recorder band selector here. With no visible items the host
detaches its buffer; no OSD frame timer or surface work remains.
