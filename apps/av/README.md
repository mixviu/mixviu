# av Bus citizen

Run `mix --serve apps/av/app.mix --name av` as the media user's own account.
The explicit name is required because Mix otherwise derives `app` from the
filename. The CLI remains independent of the Bus; only paths and JSON control
data cross the Bus, never frames.

`av.actions` returns typed action schemas. Call `av.execute` with
`{id:"av.render",params:{timeline:"/home/user/media/runs/demo/timeline.json",out:"/home/user/media/runs/demo/out.webm"}}`.
Probe is synchronous; render, audio and decode return `{job}` immediately.
`av.status` and `av.cancel` are action IDs used through `av.execute`, rather
than separate commands. Status without a job returns all jobs since startup.
There is no queue: `AV_BUSY` refuses an overlapping job.

`av.state` publishes JSON envelopes with `event: start`, `progress`, or `end`
and a job ID. Progress includes the parsed AVP record verbatim, at the CLI's
existing cadence. End includes status, receipt, exit code, signal and metrics:
`wall_s`, `cpu_s`, `memory_peak_bytes`. Missing systemd accounting is null.
Decode currently has no AVP output; it still publishes start and end.
Malformed progress is retained as a diagnostic; capture loss marks the job
failed. Receipts are capped at 1 MiB and systemd diagnostics at 64 KiB.

Set `MIX_AV_CONFIG` to an absolute strict-data config file. Defaults are:

```mix
{
  av: ["/opt/mix/bin/av"],
  roots: ["/home/user/.viu/media/runs"],
  cpu: 1,
  mem: "8G",
  max_jobs: 1
}
```

Roots must exist. `av` is an argv prefix, allowing a development binary
without installing it. `runner` (default `["systemd-run"]`) and `control`
(default `["systemctl"]`) are host-only edge configuration, also used by the
test fixtures. No Bus parameter changes these settings. Render can override
cpu and mem, audio can override cpu, decode uses the configured memory cap.
All jobs use user transient units with `--collect --wait --pipe`,
`MemoryAccounting=yes` and `MemoryMax`; render/audio also receive CLI ceilings.

All supplied paths must be absolute and canonicalise inside an allowed root.
New outputs require an existing canonical parent; dangling symlinks and
non-regular files are refused. This is a path admission policy, not a sandbox
against concurrent filesystem changes by the service user. Run as an account
whose filesystem access is appropriate; Bus callers exercise that account's
authority. The CLI owns validation of sources inside timelines.
The citizen holds a per-user host lock in `XDG_RUNTIME_DIR` (or `~/.cache`),
refusing duplicate instances. Admission also counts existing active job units
so a restart cannot bypass the job cap. Cancel active jobs before restarting
or reloading to retain their receipts: systemd owns the units and can outlive
the client. Job history is in memory for the evaluator lifetime; reload resets it.

`mix apps/av/core-test.mix` exercises the real process-watch event path with a
Mix av stub and a Mix systemd edge stub, without a broker or user systemd on
workers. The same test is discovered by `lib-viu`'s `app_mix` target.
