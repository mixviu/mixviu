# Bus history

The logger subscribes to `noded.observe` with body mode `none` and capacity
1024 on its own connection. The separate `log.props.*` service connection,
bounded observer queue and broker drop accounting are unchanged from the
2026-04-28 starvation design. There is no payload logging or redacted-body opt-in.

History is operator-private. The active file is `bus.jsonl` in the logging
library's noded log directory (normally `~/.local/log/mix/noded`). Files are
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
Sudden process death can lose buffered rows. Daily rotation uses the logger's
UTC clock, including while idle; size rotation happens before a row would exceed
the cap. Archives are `bus-YYYY-MM-DD.jsonl`, then
`bus-YYYY-MM-DD-00000001.jsonl`, etc. Existing archives are never overwritten.
A single row larger than either byte cap stops logging with an error.

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
All values must be positive. Retention runs at start, on writes and every second;
archives at least 30 calendar days old go first, then oldest dates and rotation
numbers until the total fits (including the active file and pending row).
Only recognised history archive names are removed. Retention does not touch old
`bus.log` files. The shipped units enable history and retain `--no-monitor`;
host rollout is a later gold operation.
