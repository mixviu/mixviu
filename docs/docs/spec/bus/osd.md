# OSD scenes

`osd` is a Mix behaviour on a privileged viu scene host. viud launches the
host directly over a private AF_UNIX socketpair passed as `WAYLAND_SOCKET`,
without `WAYLAND_DISPLAY`. Only that inserted connection has the private
client-data token; PID and /proc are not authentication. The `viu-osd` namespace alone grants nothing: another
client claiming it is closed. The compositor attaches the OSD mark to the
surface resource; neither an app ID nor `comp.window.place` can supply it.

Broker-attributed citizens may show content. Each item records its `owner`
in `osd.list`; only that owner may update or hide it (`NOT_OWNER` otherwise).
Unattributed mutations refuse `UNAUTHENTICATED`. Local owners use `local:<citizen>`;
mesh owners use `mesh:<citizen>@<peer>` from broker-stamped headers, never body fields.
Local mutations use the broker's Unix ingress. Unadmitted legacy TCP calls
have no attested peer identity and are refused, even with a registered name.
Each caller has eight of the 32 slots (`CALLER_BUDGET_EXCEEDED`) and 30
successful mutations (show, update and hide) per one-second window (`RATE_LIMITED`). Failed validation
does not spend the budget. Hide shares the mutation budget; expiry still removes items without a caller mutation.
Caller authorisation belongs to the Bus authorisation work shared with
`comp.record` and `comp.capture.frame`; surface authentication does not
authorise the caller or endorse its content.
Ownership prevents accidental cross-talk; names are self-chosen and can be reused after
disconnect. Caps and rate limits provide fairness, not a security boundary until
Bus admission supplies a real caller principal. Adoption of broker_principal is tracked.
Future Bus allow/deny checks govern admission to these commands; the reducer's
owner, slot and update limits still apply after admission. A grant does not
allow one citizen to modify another citizen's items.

viud supervises the host with bounded restart backoff (250 ms to 30 s).
Each restart inserts a new token; the old token is revoked on exit.
Items are lost on restart and callers must show them again.

Marked surfaces compose in the OSD band above client and screen UI bands.
The compositor draws an amber provenance mark at the output's upper left
while OSD content is mapped. Session lock uses its separate lock-only scene.
The host takes no keyboard focus, including when pointer input is enabled.

| Command | Body |
| --- | --- |
| `osd.show` | `{id, scene, anchor, enter?, exit?, duration_ms?, pointer_input?, backing?}` |
| `osd.update` | `{id, scene?, anchor?, enter?, exit?, duration_ms?, pointer_input?, backing?}` |
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
`null` removes the deadline. For `osd.hide`, an absent `exit` uses the item's
configured exit; explicit `exit:null` removes it immediately.
`pointer_input` defaults to true: the animated drawn bounds consume pointer
input without changing keyboard focus. `backing` defaults to `"card"`.
Click-through (`pointer_input:false`) requires explicit `backing:"none"`,
which draws text without a card background. Updates validate the combined
item, so switching back to a card also requires pointer input. Opaque items retain input through the entire exit until removal, with no opacity threshold.
Transparent backing requires pointer_input:false and never consumes input, including during exit. Both choices appear in `osd.list`.

Validation precedes mutation. Refusals carry an `error_code`, including
`INVALID_ARGS`, `INVALID_SCENE`, `INVALID_ANCHOR`, `INVALID_MOTION`,
`ALREADY_EXISTS`, `NOT_FOUND`, `BUDGET_EXCEEDED`, `WINDOW_NOT_FOUND`,
`STALE_GENERATION`, `COMP_UNAVAILABLE`, `INVALID_STATE` and `UNKNOWN_ACTION`.
Updating an item that is already exiting is refused; callers can show a new
item after its removal.

Ordinary output captures include the OSD because it is part of the output
scene. Listed-window captures exclude it with all other non-listed surfaces.
There is no recorder band selector here. With no visible items the host
detaches its buffer; no OSD frame timer or surface work remains.
