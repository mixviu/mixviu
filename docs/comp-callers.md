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

Screen recording uses the following Bus methods. Recordings have no overlay or
interactive keepalive prompt. Window targets use offscreen rendering and exclude
other windows and compositor overlays. Multiple windows are arranged left to
right on black, with a fixed canvas for the take.

| Action | Parameters | Reply |
|---|---|---|
| `comp.record.start` | `request_id`, `target:{output?,output_generation?,window?:{id,generation},region?:{x,y,width,height}}` or `target:{windows:[{id,generation}]}`, absolute `.mkv` `path`, `fps?` (30; 1–60), `cursor?` (true), `encoder?` (`x264` default or `vaapi`), `max_seconds` (1–86400) | `session,path,width,height,fps,clock:"monotonic",t0_us,frames_path,markers_path` |
| `comp.record.marker` | `session,label,data?` | `session,seq,frame,t_us` |
| `comp.record.status` | `session?` | `session,state,frames,repeated,dropped,duration_us,bytes,queue,readbacks,readback_us,readback_mean_us,readback_max_us,encoder,error` |
| `comp.record.stop` | `session,discard?` | `path,frames,repeated,dropped,duration_us,frames_path,markers_path,encoder,readbacks,readback_us,readback_mean_us,readback_max_us,error` |

Recording refuses existing destinations, including sidecars and symlinks. Private
0600 temporary files stay in the destination directory until a successful stop
publishes them without replacing existing files. Failure and discard remove the
temporary files; absent sidecars have null paths in terminal replies. A window
resize ends the take because its canvas dimensions are fixed. Reusing a
`request_id` with different parameters is refused; completed retries are cached
for at most 128 takes. Stop cancels a pending first frame, and `max_seconds`
also bounds that pending phase.

The renderer's shared export interface maps synchronously. Recording budgets
readback to one quarter of the frame period: after an over-budget read, following
damaged ticks repeat the previous frame and increment `dropped` until the debt
is repaid. Idle ticks also repay the debt without counting a dropped capture.
Each read itself can still exceed that budget. Single-target
pixels share one immutable allocation through capture, session and encoder.
Set `MIX_VIUD_RECORD_VAAPI_DEVICE` to select the external encoder's render device;
the default is `/dev/dri/renderD128`. Frame timestamps are taken after readback,
so content can lead its timestamp by the measured readback time.
`bytes` is cached at finalisation; it is zero while the temporary take is open.

Staging files use `<path>.<64 hex characters>.part`, with `.frames` and
`.markers` appended for the sidecars. They have mode 0600 and are removed on
failure or discard. A compositor crash can leave these `.part` files behind;
remove them after checking no take is active. Publication support is probed in
the destination directory before accepting a start. Successful publication
renames the sidecars first, then the video, without replacing existing files.
A crash between those renames can also leave final sidecars without a video.

`start` retries use the same request ID. A concurrent different request is
refused with `record_busy`. Stop retries return the terminal reply. During
finalisation, a stop with a different `discard` value is refused as
`already finishing`. The 128 most recently used completed requests are retained
for retries; evicted session IDs are refused. State edges
arrive on `<service>.record.state`. The clock is CLOCK_MONOTONIC microseconds;
sidecar and marker times are relative to the first frame, numbered from zero.
`<path>.frames.jsonl` contains `{frame,t_us}` and `<path>.markers.jsonl` contains
`{seq,frame,t_us,label,data}`. Keep marker replies as the authoritative timestamps.

The intermediate is silent Matroska using ffmpeg/libx264, ultrafast, CRF 14,
GOP 30 and yuv420p. No hardware encoder is required. The queue holds four
frames; overruns increment `dropped` and the encoder worker fills their time
slots with the previous image. `frames` and the frame sidecar include those
repeats, so overload does not shorten the video. Finalisation runs on a separate thread
and kills ffmpeg after ten seconds. Encoder failures retain stderr in `error`.

The tutorial target is the whole output at its native size, including every
client window and the pointer: `{target:{output:"headless-0"},cursor:true,
fps:30}` records 3840×2160 when that is the output mode. Odd extents are padded
with black to even dimensions for 4:2:0. Frames of unchanged content share the
previous buffer; no GPU readback or full-frame copy occurs on repeated ticks.
`readback_us` reports the latest tick’s readback cost (zero for a repeat), with
mean and maximum over `readbacks` actual reads. Window measurements include
the offscreen window render. These are microseconds, not encoder latency.

`encoder:"vaapi"` is optional. An asynchronous, bounded ffmpeg encoder probe
must report `h264_vaapi`; ffmpeg then uses its VAAPI render device with NV12
hwupload and QP 18. An unavailable device fails the take with captured stderr.
The compositor links no libav API and requires no hardware encoder for x264.
