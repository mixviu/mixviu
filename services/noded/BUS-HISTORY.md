# Bus history

The logger subscribes to `noded.observe` with body mode `none` and capacity
1024 on its own connection. The separate `log.props.*` service connection,
bounded observer queue and broker drop accounting are unchanged from the
2026-04-28 starvation design. There is no payload logging or redacted-body opt-in.

History is operator-private. The active file is
`default_log_dir("noded")/<node>/bus.jsonl` (normally
`~/.local/log/mix/noded/<node>/bus.jsonl`) unless `LOGS_DIRECTORY` supplies
a single absolute history root. Empty or absent uses the default; relative
paths and systemd colon-separated multi-directory values disable history
with a warning. The node name is always appended.

The shipped units provision their roots differently:

- `noded.service`: `host.tmpfiles.conf` creates `@{var}/log` as root:root
  0755 and `@{var}/log/noded` as noded:noded 0700 before startup. The unit
  lists this history root as an optional write path and requires its mounts.
  If the directory is missing, the broker still starts and history fails soft.
  With the shipped
  var root, history is `/opt/mix/var/log/noded/<node>/bus.jsonl`. Operators
  overriding `User=`/`Group=` must also override the history tmpfiles owner
  and change ownership of any existing history tree before starting noded.
- `noded@.service`: installers create `@{var}/session/<instance>` as the
  session account, mode 0700, under the same contract used by monitord.
  The unit sets `LOGS_DIRECTORY=@{var}/session/%i/log/noded` before its
  session environment file. The logger creates missing children with mode
  0700, so history lives at
  `@{var}/session/<instance>/log/noded/<node>/bus.jsonl`. The path is rendered
  at install time and a mesh-profile `MIX_VAR` does not move it. Host-owned
  identity drop-ins keep working through the installer ownership contract.
  A missing or unwritable session directory disables history without adding
  a broker startup dependency.

The logger creates the private per-node child. Rows written under the old
`$MIX_VAR/log/noded` are not migrated and are outside retention at the new
location. Direct invocations retain the default path
unless the caller supplies `LOGS_DIRECTORY`; its ownership checks still apply.

A `<svc>.history` reader must never recompute the history path. It asks noded:
`log.props.*` already serves the active `bus.jsonl` path.

Each logger holds an exclusive,
non-blocking advisory `flock` on the pinned per-node directory fd itself for
its lifetime, through writer destruction. There is no lock file pathname to
replace: concurrent starts lock the same directory inode. A leftover `bus.lock`
is ignored, never opened, written or unlinked, even if it is a symlink.
Lock contention or failure warns and disables history for that instance;
no active file is opened or modified without the lock. Files are
created with mode 0600. The per-node directory is created with mode 0700 and
opened with `O_DIRECTORY|O_NOFOLLOW`. Its handle is owner-checked against the
effective UID with `fstat`; group- or world-writable directories are refused,
even when owned by that UID. An accepted owned directory is tightened to 0700.
Symlink directories, foreign owners and failed checks warn and disable history.
Files are reached relative to that pinned directory handle with `openat`,
`linkat`, `renameat2` and `unlinkat`, never by resolving the parent again.
The parent directory is created 0700 when missing; an existing parent's mode
is unchanged. The parent path is not pinned before the node directory opens. A writable
ancestor can cause denial of service during that lookup: owner checks disable
history if a foreign directory is substituted, and nothing is captured.
Active files must be regular, owned by the effective UID and opened
with `O_NOFOLLOW`. Existing files with any group/world permission bits are
never reused for new rows: a loose active file is fchmodded to 0600 through its
already-open handle, then renamed to the next archive intact. Fresh files use exclusive creation with
mode 0600 and are fstat-verified. Recovery writes a gap with
`reason: "active_file_mode_loose"`. Old held descriptors receive no new rows
and cannot affect the directory lock. Foreign-owned files are never taken
over. The old
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
equal to the last timestamp in the active file (or preserved linked archive
after interrupted rotation), or null. Incomplete crash tails
are discarded before appending. A rising drop counter writes a `kind: "dropped"`
row whose `count` is the increase (initial baseline zero). A counter decrease
writes a gap with `reason: "dropped_counter_reset"` and rebases the counter.
Malformed observation warnings share the once-per-minute retry throttle.
Observer closure or
graceful shutdown writes `kind: "gap"`, `reason: "observer_closed"`. Gaps and
drops use the logger clock. No global ordering across nodes is promised.

One buffered writer stays open, flushes every second and on graceful shutdown.
Disk operations run on the blocking pool so they cannot wedge the broker's
async worker. Shutdown allows two seconds, then aborts the logger; runtime
shutdown does not wait for a wedged disk syscall. The final `observer_closed`
row may be lost; sudden death or a wedged disk can also lose buffered rows.
Transient maintenance and flush errors warn at most once a minute and retry.
Failed event writes are counted; the next successful row write first emits
`kind: "dropped"`, `reason: "write_failed"` and the accumulated `count`.
The count remains pending if the marker fails or cannot fit the byte caps.
Daily rotation uses the logger's
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
Rotation tracks its phase explicitly: a successful archive link is remembered
across unlink failures, and after unlink the next attempt only opens the fresh
active file and resets its recording day and byte count. Retries create no
duplicate archive links and cannot append to the completed archive.

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
History loading never stops the broker. The broker's own mesh-config policy
is unchanged: an explicit mesh file that cannot be read or parsed is fatal.
Retention runs at start, on rotation, every second and before the running total
would exceed the cap; ordinary message rows do not scan the directory.
The running total includes buffered rows. Archives at least 30 calendar days
old go first, then oldest dates and rotation
numbers until the total fits (including the active file and pending row).
Only recognised history archive names are removed. Retention does not touch
files with noncanonical suffixes: a suffix must be exactly eight unsigned
decimal digits with a nonzero value, and the date must be exactly `YYYY-MM-DD`.
The zero suffix `-00000000` is rejected rather than aliasing the base archive.
It also leaves old
`bus.log` files. The shipped units enable history and retain `--no-monitor`;
host rollout is a later gold operation.
