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
The service summary counts observed messages before display filtering. Own
frames contribute only broker loss unless Show own traffic is enabled; the
same predicate governs rows, totals, rate and service counts. Scene models are
published only when their projection changes. The summary shows the top eight
services by count, with alphabetical ties. Evicting a selected row clears selection.

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
If the old subscription is live, a refused replacement keeps Live status.
Successful replacement discards pending latency measurements, seeding their keys
into ambiguity tombstones. Clear also preserves these tombstones. Tick expiry
removes them after 30 seconds; queued retired replies can poison current pairings.
Disconnect clears the subscription and pairings; reconnect subscribes again.
Refused connection subscriptions retry on tick at most once every two seconds.
Display filter edits always apply locally; broker filter edits made while
disconnected are stored for reconnect without changing the disconnected status.
Closing or quitting attempts to stop the current subscription (best effort,
one-second timeout). noded also reaps all subscriptions with their connection.
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
`bustap.intent` is the validated widget command: `traffic` requires action
`select` and `{key:string}`, `filter` takes a string, `pause`/`own` a bool,
and `clear`/`hide` a string or null button value. Malformed and unknown intents
return RC 10 ARGUMENT without changing state. Anonymous services cannot be hidden.

## Limits and follow-ups

Anonymous endpoints display `(anon)`. Frames have no connection identity and
correlation IDs are only unique within a connection. Anonymous latency is
therefore always blank. Named pairs use (correlation ID, service, caller),
expire after 30 seconds and suppress latency for duplicate outstanding keys.
A reply arriving before its request (including rejection-first frames) remains
unpaired and leaves a 30-second tombstone. Error replies poison pending pairs;
completed keys also leave tombstones so duplicate replies cannot create a later
false latency. A later named reply can still be ambiguous if different connections
reuse the same caller name; broker connection identity is a follow-up, outside
this app. There is no mesh-wide aggregation.

Pending pairs and reply tombstones are each capped at 4,096; expiry sweeps run
on tick. Service counters are capped at 256 keys total: 255 service names plus
an overflow bucket. Paused messages are intentionally omitted, not broker loss.
Filter handover retains up to 64 retired subscription records with independent
loss and sequence cursors. Failed stops retry on tick (up to two every two
seconds, one-second timeout each); a tick that retries a start skips the stop
batch. The warning persists until all stops succeed. Only stopped entries
can be evicted at the cap; replacement is refused if all 64 are still live.
Queued retired frames account loss and pairing ambiguity without adding rows. Every subscription
deduplicates by sequence before accounting; new subscriptions reset the sequence.
The broker always stamps `subscription_id` in the event header; headerless and
unknown notifications are ignored. Local row IDs survive broker sequence restarts.

The gold apps bundler discovers tracked `apps/` runtime files automatically;
Bus Tap uses that same route as monitor. No per-app Rust host is added. The
shared grid adds only `stick_to_bottom` and row `tone:"danger"` rendering.

Run `mix apps/bustap/core-test.mix` for pure policy checks. The Viu `app_ports`
test draws the actual scene offscreen; the `bustap` hosted runtime test checks
subscription, reconnect, filter refusal and Bus controls with a fake broker.
Capacity tests inject small internal limits into the same policy branches and
assert the production defaults. This exercises full, overflow and expiry cases
without thousands of copy-heavy test events or increased harness deadlines.
