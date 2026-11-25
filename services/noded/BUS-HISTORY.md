# Bus history

The logger subscribes to `noded.observe` with body mode `none` and capacity
1024 on its own connection. The separate `log.props.*` service connection,
bounded observer queue and broker drop accounting are unchanged from the
2026-04-28 starvation design. There is no payload logging or redacted-body opt-in.

History is operator-private. The active file is
`default_log_dir("noded")/<node>/bus.jsonl` (normally
`~/.local/log/mix/noded/<node>/bus.jsonl`). Each logger holds an exclusive,
non-blocking advisory `flock` on that directory's `bus.lock` for its lifetime.
Lock contention or failure warns and disables history for that instance;
no active file is opened or modified without the lock. Files are
created with mode 0600; an existing active file is tightened to 0600. The old
human-readable `bus.log` is neither read nor migrated.

Each line is a JSON object with `v: 1`, `ts_utc` (RFC 3339 UTC with milliseconds),
`node` (the served node name) and `kind`. Message rows retain `seq`, `direction`,
`outcome`, `message_type`, `from`, `to`, `command`, `size`, `correlation_id`,
`rc` and cumulative `dropped_count`; `peer` is present only when set. Nullable
metadata stays null. Event timestamps are normalised to UTC; empty or invalid
timestamps use the logger's UTC clock.

```json
{"v":1,"ts_utc":"2026-11-25T01:02:03.456Z","node":"beta","kind":"message","seq":42,"direction":"local","outcome":"delivered","message_type":"request","from":"ced.beta.bus","to":"dopus.beta.bus","command":"dopus.open","size":123,"correlation_id":"example-id","rc":null,"dropped_count":0}
```

Start writes a `kind: "gap"`, `reason: "logger_start"` row with `since_last`
equal to the last timestamp in the active file, or null. Incomplete crash tails
are discarded before appending. A rising drop counter writes a `kind: "dropped"`
row whose `count` is the increase (initial baseline zero). Observer closure or
graceful shutdown writes `kind: "gap"`, `reason: "observer_closed"`. Gaps and
drops use the logger clock. No global ordering across nodes is promised.

One buffered writer stays open, flushes every second and on graceful shutdown.
Disk operations run on the blocking pool so they cannot wedge the broker's
async worker. Shutdown allows two seconds, then aborts the logger; runtime
shutdown does not wait for a wedged disk syscall. The final `observer_closed`
row may be lost; sudden death or a wedged disk can also lose buffered rows.
Transient maintenance and flush errors warn at most once a minute and retry.
Sudden process death can lose buffered rows. Daily rotation uses the logger's
UTC clock, including while idle; size rotation happens before a row would exceed
the cap. Archives are `bus-YYYY-MM-DD.jsonl`, then
`bus-YYYY-MM-DD-00000001.jsonl`, etc. Existing archives are never overwritten. New indices follow the highest surviving
index, so deletion of old archives cannot make a new file sort oldest.
A row larger than either byte cap is skipped with a `kind: "dropped"`,
`reason: "row_too_large"`, `count: 1` marker; subsequent rows continue.
If even that marker exceeds the configured cap, it is skipped too.
On open, an active file with more than one hard link is treated as a completed
archive from an interrupted rotation: only the active name is unlinked, and
a fresh file is opened before crash-tail recovery or appending.

Configure the `history` map in noded's `config.conf.mix` (the explicit
`--mesh-config` file if supplied, otherwise `mix:etc/noded/config.conf.mix`):

```mix
history: {
  retention_days: 30
  max_file_bytes: 67108864
  max_total_bytes: 1073741824
}
```

These are the defaults: 30 UTC calendar days, 64 MiB per file, 1 GiB total.
All values must be positive. Parse errors, unknown history keys, type errors and
non-positive limits warn and use history defaults. An unreadable explicit file
(including missing files) disables history with a warning. An absent implicit
default file uses defaults. A readable malformed file uses history defaults.
These failures never stop the broker: if the explicit mesh file also cannot
be loaded by the broker, it warns and starts without configured mesh peers.
Retention runs at start, on rotation, every second and before the running total
would exceed the cap; ordinary message rows do not scan the directory.
The running total includes buffered rows;
archives at least 30 calendar days old go first, then oldest dates and rotation
numbers until the total fits (including the active file and pending row).
Only recognised history archive names are removed. Retention does not touch old
`bus.log` files. The shipped units enable history and retain `--no-monitor`;
host rollout is a later gold operation.
