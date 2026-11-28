# Bus history

## Command manifests and authority

`noded.commands.set` accepts a JSON array of names in the body, for example
`["example.read", "example.write"]`, replacing the connection's registration's
previous set. The register body is unchanged. Native clients send after a
successful named register and when their command names change; anonymous clients
never publish. `mix --serve` publishes declared handlers and runtime commands
automatically, alongside its existing HELP implementation. Reconnect registers
again and republishes. An old broker's unknown-verb rc 10 is logged once at debug
per connection; registration and serving continue.
Other non-zero manifest return codes warn once per rc per connection. On rc 25,
the publisher marks its set dirty, registers again with the original authenticated
register body, then republishes current names. Refusals 20–24 and 26 are not
retried. Recovery is bounded so a broker repeatedly returning 25 cannot cause
an automatic registration loop; a later publication may attempt recovery again.

Every refusal clears the previous manifest:

| rc | Meaning |
| --- | --- |
| 0 | Accepted, including a set ignored for history |
| 20 | Body is not a JSON array of strings |
| 21 | More than 512 names (duplicates count), or body exceeds 198145 bytes |
| 22 | A name exceeds 64 bytes |
| 23 | A name exceeds six dot-separated segments |
| 24 | Empty segment or character outside `[A-Za-z0-9_]`, including `@` |
| 25 | More than 16 sets per registration, including refused attempts |
| 26 | No current registration owned by this connection |

Decoding happens outside the registry lock. The byte cap is
`512 * (64 * 6 + 3) + 1 = 198145`: six-byte JSON escapes for each ASCII name
byte, quotes, commas and brackets. Whitespace consumes the same wire budget.
The decoder stops at element 513 without allocating that name. Return-code
precedence remains ownership, set budget, then decoding and name limits.
A fresh registration resets the budget.
Deregister, connection loss and replacement of a registration remove its set.
There is no manifest relay across bridges.

The node-local `noded.history_allowlist` key in `node.conf.mix` is an array of
service identities. Omission ships these defaults:

```mix
noded: {
  history_allowlist: ["maild", "webd", "dnsd", "blobd", "toolsd", "filesd", "provisiond", "sshm", "mon"]
}
```

An explicit array replaces the defaults; `[]` honours no service manifests.
A malformed value or reserved identity refuses broker startup. The same broker
reservation helpers reject session shapes, `bridge-*`, `anon-*`, `log`,
`log-observe-*`, `noded` and `noded-*`; tokened `mon` remains allowed.
Operators extend this list per node.
Sets are honoured by exact allowlisted name only, never by pattern, prefix or
registration origin. Anonymous callers, observers, sessions and bridges do not
vouch for commands. `log` is not allowlisted: `log.props.get`, `log.props.list`
and `log.props.describe` are static broker control verbs. BROKER-024 reserves
the logger, observer, anonymous, bridge and monitor namespaces; in-process task
tokens and registration origin remain broker-controlled. A valid set
from another registered identity receives rc 0 but cannot vouch for names.
The internal `vouched_command` query checks the union of honoured sets of
currently registered services.

These facilities supply the subsequent history projection; this change does
not change history rows.

A service, instance, peer or `bus-svc` name is infrastructure. Never derive
one from an account, customer, mailbox or domain.

The logger subscribes to `noded.observe` with body mode `none` and capacity
1024 on its own connection. The separate `log.props.*` service connection,
bounded observer queue and broker drop accounting are unchanged from the
2026-04-28 starvation design. There is no payload logging or redacted-body opt-in.

## Reserved broker identities

Ordinary registrations refuse `noded`, `noded-*`, `log`, `log-observe-*`,
`anon-*`, `bridge-*` and `mon` with rc 10 (`reserved_name`). BROKER-017's
session-name shape rule is unchanged; `t0-x` and `c0-x` remain client names.
Only the in-process logger and monitor receive their separate per-boot,
256-bit OS-random registration tokens. Comparison is constant time. Disabled
tasks have no token. Tokens never enter provenance or logs. Tap output removes
only the capability from valid JSON provenance and clears malformed bodies;
observe continues to omit register payloads under its authentication policy.

An admitted mesh link may register its own `bridge-<peer>` name. Forwarded
reserved task service assertions are refused. Each ServiceEntry records its
creation origin: `client`, `logger`, `monitor`, `session` or `bridge`. Names
cannot confer origin; history projection does not consume origin yet.

A service, instance, peer or `bus-svc` name is infrastructure. Never derive
one from an account, customer, mailbox or domain. Examples use names such as
`bridge-example`.

The logger and monitor run functions are crate-private: only noded startup
may hand them a capability. Repository-wide caller search found only noded
startup uses. Embedded run/run_on brokers have no task tokens by default.
NodedClient never reconnects internally; each new task connection must send
the capability again. A body-less re-registration is refused (fail closed).

## History storage

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
`$MIX_VAR/log/noded` are not moved automatically. An operator can move the old
`bus.jsonl` into the new `<node>/` directory as `bus-<last UTC day>.jsonl`,
using the last row's UTC day. Never overwrite an existing archive: check that
the target does not exist and use a no-clobber move (`mv -n`, or a
`renameat2(RENAME_NOREPLACE)` equivalent). If the name is taken, use the next
free index, `bus-<day>-<NNNNNNNN>.jsonl`. The naming scheme cannot place older
rows before index 0, so that day's archives are then out of order.

The moved file must be a regular file owned by noded's effective UID, mode
0600. `retain()` skips a file it cannot open, so a wrong-owner copy is never
expired and is never counted against the byte cap. Recognised archive names
are expired by date: if the last UTC day is already outside the retention
window, the next `retain()` run deletes the moved file.

The gap rows mark a discontinuity, not a path change. The old file ends with
`observer_closed`; on the first start in a new directory, `logger_start` has
`since_last: null`. The operator records the path change separately, for
example in a journal. Direct invocations retain the default path unless the
caller supplies `LOGS_DIRECTORY`; its ownership checks still apply.

A `<svc>.history` reader must never recompute the history path.
It asks noded with `log.props.get` and reads `config.log_path`.

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
