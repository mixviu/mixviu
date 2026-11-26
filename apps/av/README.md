# av Bus citizen

Run `mix --serve apps/av/app.mix --name av` as the media user's own account.
The explicit name is required because Mix otherwise derives `app` from the
filename. The CLI remains independent of the Bus; only paths and JSON control
data cross the Bus, never frames.

`av.actions` returns typed action schemas. Call `av.execute` with
`{id:"av.render",params:{timeline:"/home/user/media/runs/demo/timeline.json",out:"/home/user/media/runs/demo/out.webm"}}`.
Probe is synchronous; render, audio and decode return `{job}` immediately.
`av.status` and `av.cancel` are action IDs used through `av.execute`, rather
than separate commands. Status without a job returns every running job and
the last 32 completed jobs since startup, evicting the oldest completion first.
Public status fields are `job`, `action`, `unit`, `status`, `started`, `ended`,
`exit_code`, `signal`, `cancelled`, `progress`, `receipt`, `metrics` and `stderr_tail`.
There is no queue: `AV_BUSY` refuses an overlapping job.

`av.state` publishes JSON envelopes with `event: start`, `progress`, or `end`
and a job ID. Progress includes the parsed AVP record verbatim, at the CLI's
existing cadence. End includes status, receipt, exit code, signal, `stderr_tail` and metrics:
`wall_s`, `cpu_s`, `memory_peak_bytes`. Missing systemd accounting is null.
Decode currently has no AVP output; it still publishes start and end.
Malformed progress is retained as a diagnostic; capture loss marks the job
failed. Receipts are capped at 1 MiB and systemd diagnostics at 64 KiB.
The public stderr tail is UTF-8 lossy, capped at the last 4 KiB or 20 lines,
whichever is smaller, excluding all `AVP ` lines. It includes CLI failure reasons
even when there is no receipt. Cancel requests only label signal termination
(TERM/KILL) without a receipt as cancelled; normal exits during stop retain
their exit-code/receipt outcome.

Set `MIX_AV_CONFIG` to an absolute strict-data config file. Defaults are:

```mix
{
  av: ["/opt/mix/bin/av"],
  roots: ["/home/user/.viu/media/runs"],
  max_cpu: 1,
  max_mem: "8G",
  max_output_bytes: 68719476736,
  decode_max_bytes: "64G",
  max_jobs: 1
}
```

Roots must exist. `av` is an argv prefix, allowing a development binary
without installing it. `runner` (default `["systemd-run"]`) and `control`
(default `["systemctl"]`) are host-only edge configuration, also used by the
test fixtures. No Bus parameter changes these settings. `max_cpu` is a positive
integer; `max_mem` accepts positive bytes or uppercase K/M/G (powers of 1024).
Render and audio can request `cpu` and `mem` up to these ceilings; omitted
values use the host maximum. Higher requests fail with `AV_MAX_CPU` or
`AV_MAX_MEM` before a unit starts. Decode uses the host memory ceiling.
Legacy host `cpu` and `mem` keys are accepted as aliases for the respective
ceiling when the new key is absent from the config file.
Decode also passes `--max-output-bytes` as plain bytes, converted from
the smaller of `max_output_bytes` (positive integer bytes) and
`decode_max_bytes` (positive bytes or uppercase K/M/G, powers of 1024).
The optional positive-integer `max_output_bytes` action parameter may lower
this disk cap; values above the host cap are refused with `AV_MAX_OUTPUT_BYTES`.
Limits must fit Mix's
exact integer range (at most 9007199254740991 bytes). The cap covers combined
Y4M and WAV output. Both audio and decode support optional `wav` paths.
All jobs use user transient units with `--collect --wait --pipe`,
`MemoryAccounting=yes`, `MemoryMax` and `LimitFSIZE=max_output_bytes` in bytes.
Render/audio enforce output size through systemd's hard per-file `LimitFSIZE`,
including optional WAV files. `av` writes through pending temporary files and
renames them on success: a failure never leaves a partial published output,
but may orphan a pending temporary file, up to one cap per file.
The `LimitFSIZE` signature is signal 25 (`SIGXFSZ`) or forwarded exit 153.
If stderr is empty, the citizen supplies an explicit output-ceiling diagnostic
in `stderr_tail` on status and the end envelope. Decode's CLI limit additionally
caps combined Y4M and WAV bytes. Render/audio also receive CPU and memory
ceilings through the CLI.

All supplied paths must be absolute and canonicalise inside an allowed root.
Outputs require an existing canonical parent and must not exist: any existing
file, directory or symlink is refused with `AV_PATH` / `output exists`.
Outputs must differ from inputs and from each other after canonicalisation.
This is a path admission policy, not a sandbox
against concurrent filesystem changes by the service user. Run as an account
whose filesystem access is appropriate; Bus callers exercise that account's
authority. There is no per-job ownership until 2c; this surface is for the
local operator only. The CLI owns validation of sources inside timelines.
Deferred, pre-existing: `av.probe` runs synchronously outside any transient
unit or host resource ceiling; bounding probe is outside this change's scope.
The citizen holds a per-user host lock in `XDG_RUNTIME_DIR` (or `~/.cache`),
refusing duplicate instances. Admission also counts existing active job units
so a restart cannot bypass the job cap. Cancel active jobs before restarting
or reloading to retain their receipts: systemd owns the units and can outlive
the client. Job history is in memory for the evaluator lifetime; reload resets it.

`mix apps/av/core-test.mix` exercises the real process-watch event path with a
Mix av stub and a Mix systemd edge stub, without a broker or user systemd on
workers. The same test is discovered by `lib-viu`'s `app_mix` target.
