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

For an output-sized allowlisted take use
`target:{output:"headless-0",windows:[{id:7,generation:3}]}` with optional
`output_generation`. The explicit output distinguishes it from the existing
`target:{windows:[...]}` stitched canvas. Start requires 1–16 live targets;
duplicates are collapsed. The default means backdrop plus the listed windows.
Only those windows, their popup/subsurface trees,
the wallpaper and the optional cursor enter a separate offscreen picture.
The wallpaper is the `backdrop` namespace's Background layer surface tree;
other layer surfaces (including other Background namespaces), layer popups,
unlisted bottom-band windows, drag icons, notifications and OSD are excluded.
Namespace identifies scene routing, not authenticated peer authority.
Client cursor artwork, named shapes and hidden state are honoured only while
pointer focus belongs to a listed window's surface tree (including its popups
and subsurfaces) in the recorded view, and the state was set during that same
pointer enter epoch. A focus change uses the default until the newly focused
listed surface sets its cursor; leaving and returning cannot revive old state.
A cursor surface must also belong to the focused surface's client.
Otherwise the compositor default image
is drawn, including over another window of the same client and over compositor
forced cursor affordances. During a drag the take always uses the default,
even over a listed drop target: source cursor state and drag icons are never
included. An unknown cursor output draws no cursor, as does a cursor belonging
to another output.
The live screen is unchanged, including windows covering listed content.

Start refuses a window outside the target view as `window_not_on_output`,
with `id`, `generation` and `output`. viud maps windows onto every protocol
output, so this fence uses the output camera/view intersection, plus explicit
fullscreen/constrained output selection, rather than `wl_surface.enter`.
The renderer rechecks the same fence.

A listed window closing or changing generation removes it from the active
set and queues one `window_gone` marker with `data:{id,generation}`. Membership
is by identity, independent of the current view, camera, world or window position.
Leaving the view queues `window_left`; returning queues `window_returned`, both
with `data:{id,generation}`. Neither removes membership: returning windows draw again.
The take
continues, including when the set becomes empty; the canvas stays output-sized
through listed-window moves and resizes. Status reconciles membership before answering. A successful stop includes all
these markers, admitted after an image evaluated with the removed window
excluded; no later frame shows a gone window. Visibility and lock markers are
also admitted after the first image evaluated with the new state; that image
precedes its marker. Marker admission is retried under queue pressure; inability to admit
one within the bounded stop drain fails publication instead of silently losing
it. Existing single-window and stitched targets still fail when a target goes
away or changes dimensions. Status includes `windows:[{id,generation}]` for
the active listed set (an empty array for other targets or idle).
New windows are excluded. There is no `comp.record.windows` mutation command
in this version; restart the take to change its allowlist.

While session-locked, a listed take captures an opaque black blank only: no
backdrop, windows, lock surfaces or cursor, and no listed window frame callbacks.
Recording continues, with quiet locked ticks repeating the blank. `locked` and
`unlocked` markers (null data) record sampled lock transitions. Unlock restores
the current backdrop and visible listed windows without changing membership.

Filtered takes carry an explicit internal take ID, independent of file paths,
and force a real first image and a fresh image after observed scene transitions.
An unchanged-scene answer without a raw consumer fails explicitly.
Filtered takes track damage on their own composed elements. Changes solely in
unlisted content cause no readback; unchanged pictures use scheduled repeats.

| Action | Parameters | Reply |
|---|---|---|
| `comp.record.start` | `request_id`, `target:{output?,output_generation?,window?:{id,generation},region?:{x,y,width,height}}` or `target:{windows:[{id,generation}]}` or `target:{output,output_generation?,windows:[{id,generation}]}`, absolute `.vfl` directory `path`, `fps?` (30; 1–60), `cursor?` (true), `encoder?` (`vfl`), `max_seconds` (1–86400) | `session,path,recovery_path,width,height,fps,clock:"monotonic",t0_us,frames_path,markers_path` |
| `comp.record.marker` | `session,label,data?` | `session,seq,frame,t_us` |
| `comp.record.status` | `session?` | `session,state,windows,frames,repeated,dropped,dropped_capture,dropped_queue,readback_deferred,pending_ticks,duration_us,bytes,queue,encoders:["vfl"],readbacks,readback_us,readback_mean_us,readback_max_us,writer,encoder,error` |
| `comp.record.stop` | `session,discard?` | `path,recovery_path,bytes,frames,repeated,dropped,dropped_capture,dropped_queue,readback_deferred,pending_ticks,duration_us,frames_path,markers_path,encoder,readbacks,readback_us,readback_mean_us,readback_max_us,writer,error` |

Only VFL is available. Matroska paths, x264 and VAAPI requests are refused.
The [VFL specification](../libraries/media/vfl/VFL.md)
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

For window-only targets, a resize ends the take because its canvas dimensions
are fixed. Output-sized listed targets keep their canvas through window resizes.
Reusing a `request_id` with different parameters is refused. Completed retries
are retained for the 128 most recently used takes; evicted session IDs are
refused. A concurrent different request is refused with `record_busy`.
Stop cancels a pending first frame, and `max_seconds` bounds that phase too.
Stop retries return the terminal reply. During finalisation, a stop with a
different `discard` value is refused as `already finishing`. State edges
arrive on `<service>.record.state`.

The renderer maps its shared export interface synchronously. Recording budgets
readback to one quarter of the frame period: after an over-budget read,
following changed ticks can defer readback until the debt is repaid. Deferral
alone does not count as dropped; a sampled source state superseded before
capture counts once as `dropped_capture`. Idle ticks repay debt without counting a dropped
capture. Each read itself can still exceed that budget. Single-target pixels
share one immutable allocation through capture, session and encoder. RGB to
BGRA conversion, comparison, compression and file I/O run on the writer thread.

Frame/marker admission never waits for disk. Four frame allocations can be
queued, with four reserved control slots for markers. Full frame admission
increments `writer.queue_overruns`; a new changed image refused by the queue
also increments `dropped_queue`. The writer fills slot gaps and the tail with
repeats. Unchanged ticks and repeated queue pressure are not lost changes. A full marker queue explicitly refuses the marker. `frames` includes
all scheduled slots; `duration_us = floor(frames * 1000000 / fps)`.
`bytes` reports binary record bytes while recording and all take file bytes
after successful finalisation. Markers are acknowledged when queued, and
successful stop guarantees they have been synced.

The origin is CLOCK_MONOTONIC microseconds when the first capture completes
(the last callback for a multi-window capture), before writer setup. It is
not start/admission time. Marker timestamps
are relative to that origin; marker frame numbers identify the last scheduled
slot. Frame sidecar timestamps describe the constant-rate timeline, starting
at zero. Sidecars are `<path>/frames.jsonl` and `<path>/markers.jsonl`;
their paths become usable at successful stop. Content can lead the first
capture timestamp by the measured readback time.

`comp.record.start` also accepts optional `max_bytes` (positive integer;
omitted means no byte ceiling). It caps total file-content bytes, including
sidecars, manifests and receipts; it excludes filesystem allocation overhead.
Preflight requires at least 1 GiB free for the calling user. Every writer
write checks that its bytes would leave at least 1 GiB free. This headroom
protects other session work and exceeds the maximum 256 MiB raw frame.
These checks are not a reservation against concurrent writers. A floor or
byte-limit failure fails the take and prevents publication.

Failed takes remain at the `recovery_path` returned by start/failed stop:
`<path>.<token>.part`, beside the requested destination. After a compositor
crash, find that sibling by its destination prefix and `.part` suffix. Once
the take is terminal (or viud has exited), inspect or recover it, then remove
that exact directory recursively, for example with Mix
`run_argv(["rm","-r","--",$recovery_path])` using the returned path as a value.
Do not delete an active take. `discard:true` removes the active take on stop.

Recording continues offscreen while the VT is switched away or the display
is parked, if session clients keep updating. It records only viud's scene.
Intermediate destination path symlinks resolve normally. The caller is a
trusted session client of the same user and must trust the parent directories.
An idle status retains every status key; counters are zero and `writer` and
`encoder` are null until a writer exists.

Whole-output capture includes every client window and optionally the pointer:
`{target:{output:"headless-0"},cursor:true,fps:30}` records 3840×2160 on a 4K
output. Odd extents are padded with zero pixels to even dimensions.
Unchanged ticks share the previous source allocation and avoid GPU readback.
`readback_us` is the latest tick's readback cost (zero for a repeat), with mean
and maximum over `readbacks` actual reads. Window measurements include the
offscreen render. These are microseconds, not encoder latency.

Recording accepts either the canonical output id returned by comp.windows.list
(e.g. `o_hdmi_a_1`) or its backend name (`HDMI-A-1`). Targets are normalised
to the backend name internally for generation fencing. Recording samples
the selected output's submitted scene changes; capture-only redraws and
presentation completions do not make an idle output look changed. Paused or
parked presentation (including DPMS off) uses a conservative scene-request
clock instead. Changes on another output can cause an extra offscreen sample;
the writer still compares accepted pixels before deciding whether to repeat.

`dropped = dropped_capture + dropped_queue`. `readback_deferred` counts
budget waits, and `pending_ticks` counts waits for an in-flight capture. The
`writer` object reports conversion, XOR/LZ4, file write/seal and queue wait
timings plus record/payload byte counts. These fields remain in stop replies.
Uncaptured state pixels cannot be inferred; source loss accounting counts
distinct observed states, not every client commit between 30 Hz samples.
