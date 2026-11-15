# comp caller audit

`comp.callers` is a read-only Bus command with no required arguments, roles or
grants. Step 1 observes calls and refuses none on identity grounds. Existing
argument validation and capacity limits still apply.

The recipient prefers the existing verified Unix native-session profile. One
profile ping is added per connect/reconnect, with no identity RPC per call.
TCP remains the default for other services. When the native endpoint is
missing, comp uses TCP and warns that its recipient is unprotected. Native
verification failures are not silently treated as verified connections.

The reply contains `mode: observe`, current `protected_recipient`, saturating
`total` and `legacy` counters, counts by class and known command, and `recent`:
the latest 128 calls. Each recent entry has a sequence number, command, class,
connection generation, recipient protection status and caller. The caller is
either `legacy/no principal` or a summary of broker-attested assurance, uid,
pid, optional session role and registered name. The name remains self-selected
and is not an application authorisation claim. No arguments, screen pixels,
keys, full headers, grants or session identifiers are retained.

The five classes are `read`, `window`, `input`, `capture` and `debug`.
`capture.frame`, `record.start`, `record.marker` and `record.stop` are `capture`;
`record.status` is `read`; `debug.set` is `debug`; single input operations
and input sequences are `input`. The catalogue explicitly classifies every
command. Discovery, truth and unknown commands share an additional `other`
audit bucket. Unknown command names share one bounded counter bucket. Topic
fan-out is excluded. The audit query counts itself. Counters survive broker
reconnects while the comp worker lives, and reset when it restarts.

Principals come from the verified delivery, never by trusting incoming header
text. TCP has no principal even if a client supplies a principal header. Every
reconnect negotiates and registers afresh; queued deliveries retain only their
own connection context. No term identity or session allocation is used.

Step 2 adds roles rooted in the desk launcher and time-limited, revocable agent
grants, after independent design review. Screen capture will need a grant.
Step 3 migrates legitimate callers before enforcing role/grant requirements on
privileged classes. Read-only commands stay open. Input must be refused while
locked; that policy is future work, not enforcement claimed by step 1. Remote
comp authority waits for mesh authentication.

Screen recording uses the following Bus methods. Takes have no overlay or
interactive keepalive prompt. Window targets use offscreen rendering and exclude
other windows and compositor overlays. Multiple windows are arranged left to
right on a fixed canvas.

| Action | Parameters | Reply |
|---|---|---|
| `comp.record.start` | `request_id`, `target:{output?,output_generation?,window?:{id,generation},region?:{x,y,width,height}}` or `target:{windows:[{id,generation}]}`, absolute `.vfl` directory `path`, `fps?` (30; 1–60), `cursor?` (true), `encoder?` (`vfl`), `max_seconds` (1–86400) | `session,path,recovery_path,width,height,fps,clock:"monotonic",t0_us,frames_path,markers_path` |
| `comp.record.marker` | `session,label,data?` | `session,seq,frame,t_us` |
| `comp.record.status` | `session?` | `session,state,frames,repeated,dropped,duration_us,bytes,queue,encoders:["vfl"],readbacks,readback_us,readback_mean_us,readback_max_us,encoder,error` |
| `comp.record.stop` | `session,discard?` | `path,recovery_path,bytes,frames,repeated,dropped,duration_us,frames_path,markers_path,encoder,readbacks,readback_us,readback_mean_us,readback_max_us,error` |

Only VFL is available. Matroska paths, x264 and VAAPI requests are refused.
The [VFL specification](../engines/viud/crates/graphics/src/capture/encode/VFL.md)
defines the directory, LZ4 key/delta/repeat records, CRCs and crash recovery.

The destination must not exist, including a dangling symlink. An asynchronous
preflight checks no-replace publication support. The writer creates a 0700
private sibling `<path>.<64 hex characters>.part`, with exclusive no-follow 0600
files inside. Successful stop drains and syncs the writer, publishes the whole
directory in one no-replace rename and syncs its parent directory. Failed and
interrupted takes remain at `recovery_path` for recovery; explicit discard
removes the private take. Failed stop replies have null sidecar paths.
If publication renamed the take but parent sync failed, the error names the
published path whose durability could not be confirmed.

A window resize ends the take because its canvas dimensions are fixed.
Reusing a `request_id` with different parameters is refused. Completed retries
are retained for the 128 most recently used takes; evicted session IDs are
refused. A concurrent different request is refused with `record_busy`.
Stop cancels a pending first frame, and `max_seconds` bounds that phase too.
Stop retries return the terminal reply. During finalisation, a stop with a
different `discard` value is refused as `already finishing`. State edges
arrive on `<service>.record.state`.

The renderer maps its shared export interface synchronously. Recording budgets
readback to one quarter of the frame period: after an over-budget read,
following damaged ticks repeat the previous frame and increment `dropped`
until the debt is repaid. Idle ticks repay debt without counting a dropped
capture. Each read itself can still exceed that budget. Single-target pixels
share one immutable allocation through capture, session and encoder. RGB to
BGRA conversion, comparison, compression and file I/O run on the writer thread.

Frame/marker admission never waits for disk. Four frame allocations can be
queued, with four reserved control slots for markers. Full frame admission
increments `dropped`; the writer fills slot gaps and the dropped tail with
repeats. A full marker queue explicitly refuses the marker. `frames` includes
all scheduled slots; `duration_us = floor(frames * 1000000 / fps)`.
`bytes` reports binary record bytes while recording and all take file bytes
after successful finalisation. Markers are acknowledged when queued, and
successful stop guarantees they have been synced.

The origin is CLOCK_MONOTONIC microseconds at first capture. Marker timestamps
are relative to that origin; marker frame numbers identify the last scheduled
slot. Frame sidecar timestamps describe the constant-rate timeline, starting
at zero. Sidecars are `<path>/frames.jsonl` and `<path>/markers.jsonl`;
their paths become usable at successful stop. Content can lead the first
capture timestamp by the measured readback time.

Whole-output capture includes every client window and optionally the pointer:
`{target:{output:"headless-0"},cursor:true,fps:30}` records 3840×2160 on a 4K
output. Odd extents are padded with zero pixels to even dimensions.
Unchanged ticks share the previous source allocation and avoid GPU readback.
`readback_us` is the latest tick's readback cost (zero for a repeat), with mean
and maximum over `readbacks` actual reads. Window measurements include the
offscreen render. These are microseconds, not encoder latency.
