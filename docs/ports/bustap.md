# Bus Tap

Bus Tap (`mixviu.bustap`, service `bustap`) is a hosted Mix app:
`viu run apps/bustap`. It shows this node's broker observation feed, oldest
first and newest at the bottom. The shared striped virtual table follows the
bottom until you scroll up; scrolling back to the bottom resumes following.
Rejected messages and replies with RC >= 10 use the theme danger colour.

Pause stops collecting rows; observation totals, service counts and broker loss
continue. Resume collects new traffic. Clear empties rows and pending pairings,
preserving lifetime totals and loss. History holds 2,000 rows; eviction is shown
separately from broker loss. Filters apply to retained rows as well as new ones.
Select a row and click Hide service to remove a noisy service in one click;
View > Show hidden services restores them. Own traffic is hidden initially.
The service summary counts observed messages before display filtering.

## Feed and subscription

On Bus connection, the app calls `noded.observe.start` with
`{filter:{services:[],commands:[],directions:[]},body:"none",capacity:1024}`.
Empty lists select all. Directions are `local`, `mesh_in`, `mesh_out`.
It consumes `noded.observe.event` metadata without replying to events:
`seq`, ISO millisecond `ts`, `direction`, `outcome`, `message_type`, `from`,
`to`, `command`, `size`, `correlation_id`, `rc`, `dropped_count` and
`payload_omitted`. Bodies are never requested or retained.

Changing broker filters starts the replacement before stopping the previous
subscription. Refusal preserves the previous filter and reports an error.
Disconnect clears the subscription and pairings; reconnect subscribes again.
Loss is cumulative within a subscription, added to a lifetime total and always
shown, including while paused or filtered. Replacement resets the loss baseline.
Models update at 250 ms intervals, rather than once for every observation.

## Bus controls

`bustap.actions` lists menu actions; `bustap.execute {id}` runs one.
`bustap.pause {on}` and `bustap.clear` control collection.
`bustap.filter {services,commands,directions,hide,own,text}` edits only supplied
keys. Text matches from, to and command without case sensitivity.
`bustap.rows {limit}` returns visible structured rows (0–2,000, newest last).
`bustap.stats` returns counts, messages/second, loss, eviction, filters and
subscription health. These commands are also described by `app.describe`.

## Limits and follow-ups

Anonymous endpoints display `(anon)`. Frames have no connection identity and
correlation IDs are only unique within a connection. Anonymous latency is
therefore always blank. Named pairs use (correlation ID, service, caller),
expire after 30 seconds and suppress latency for duplicate outstanding keys.
A reply arriving before its request (including rejection-first frames) remains
unpaired. A later named reply can still be ambiguous if different connections
reuse the same caller name; broker connection identity is a follow-up, outside
this app. There is no mesh-wide aggregation.

Pending pairs are capped at 4,096 and service counters at 256 names plus an
overflow bucket. Paused messages are intentionally omitted, not broker loss.
Filter handover can briefly receive overlapping observations; frames lack a
subscription identifier, so loss attribution across overlap is best effort.

The gold apps bundler discovers tracked `apps/` runtime files automatically;
Bus Tap uses that same route as monitor. No per-app Rust host is added. The
shared grid adds only `stick_to_bottom` and row `tone:"danger"` rendering.

Run `mix apps/bustap/core-test.mix` for pure policy checks. The Viu `app_ports`
test draws the actual scene offscreen; the `bustap` hosted runtime test checks
subscription, reconnect, filter refusal and Bus controls with a fake broker.
