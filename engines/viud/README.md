# viud

## Durable handover diagnostics

Each native instance appends `handover-<socket>.jsonl` under
`slots::persist::path::base::state_dir()`: currently `$XDG_STATE_HOME/mix/viud`,
or `/home/user/.local/state/mix/viud` with the default environment. The socket
separates instances sharing a home. No configuration is required. Rows use
`write_all` and `sync_data`; rotation at 1 MiB keeps one `.jsonl.1` file.
An I/O failure warns once and disables the trail for that instance.

Rows contain `ts` (UTC RFC 3339 milliseconds), `boot_ns` (CLOCK_BOOTTIME),
`pid`, `socket`, `seq`, `event`, `phase`, nullable `connector`, `crtc`, `mode`
(`WxH@Hz`), and `ok`, `error`, `elapsed_ms` on end rows. History envelope
fields are included (`ts_utc`, `source`, `kind`, `class`, `ref`, `data`);
`node` is null until a history reader supplies node provenance. Sequence
numbers restart per process. Recorded rows are private operator data.

`PauseSession` marks receipt; `CrtcClear` records each disabling attempt,
including revoked-master errors. `DisplayPaused` and `InputSuspended` mark
completion. All pause rows stay in memory until the display/input pause
transaction returns, so trail I/O cannot delay CRTC release. A reset during
that transaction can therefore leave no pause rows.

`ActivateSession` covers initial attachment and resume. `ReclaimModeset`
starts for every connector are durable before the device-wide activation;
their ends share that manager call's result. `SurfaceReset` records each
surface reset. `FirstVblank` marks watchdog retirement and includes
`since_activate_ms`. `InitializeOutput` records startup and runtime mode
fallback attempts; `SubmitModeset` covers real modeset submissions outside
steady page flips. `ModeChange` covers provisional changes and reverts.
`OutputDrop` records output removal. `DpmsOn`, `DpmsOff` and
`DpmsReclaimModeset` cover panel power recovery. `Blank`/`Unblank` record
logical requests; `BlankTest`/`BlankCommit` record individual atomic work,
including cleanup and restoration.

After a crash, preserve both files before restarting repeatedly. Read `.1`
then `.jsonl`; group by socket and PID, and compare `boot_ns` within the same
host boot (retain the host boot identity alongside the files). A final start
without an end bounds the unfinished operation; it does not prove its cause.
Pause rows are retrospective, with their original timestamps. Existing
journal messages remain useful for error detail. This adds no history verb.

The MixViu Wayland compositor: the gold MixOS compositor (compd) without the in-process
egui widget toolkit. Furniture and applications run out of process in `viu`.
`cargo tree --locked -p viud -e normal` must show no `egui`, `egui-wgpu`,
`egui-winit` or `toolkit`.

Its private crates live in `crates/`, its units in `units/` (`viud@`,
`viud-headless@`). Configuration lives in `~/.config/mix/viud`, environment
variables are `VIUD_*` (and `MIX_VIUD_CONFIG`). The Bus service is still
`comp` with its `comp.*` commands: that is the protocol name.

`comp.window.place` accepts persistent visual attributes alongside geometry:
`opacity` (number 0–1), `z` (signed 32-bit integer), `visible` (boolean), and
`clip` (`{x,y,width,height}` in window-local logical coordinates, non-negative
extents). Omission keeps the previous value; `null` resets it to 1, 0, true,
or no clip respectively. Attribute-only requests also work on constrained
windows and do not move, resize or unmap them. Opacity does not alter input.
Hidden windows retain their mapping and size, contribute no pixels, receive
no input, and use the existing invisible-window frame-callback policy.

Higher `z` stacks windows forward within their existing workspace band;
ties retain the normal raise order. Compositor-marked drawables are barriers,
and layer-shell, lock, OSD and screen UI bands are never promoted across.

The OSD band is owned by a compositor-launched viu host, authenticated by
Wayland credentials and the child's process incarnation. Its Mix behaviour
serves `osd.show/update/hide/list/actions/execute`; see the
[OSD contract](../../docs/docs/spec/bus/osd.md). Client namespaces and window
placement cannot grant OSD authority. Pointer input is opt-in; OSD surfaces
never take keyboard focus.
`comp.windows.list` and `windows.s<ID>` expose `opacity`, `z`, `clip`, and
`requested_visible`; `visible` remains the effective visibility including
workspace and minimisation. State lasts for the window role's lifetime and
can be re-queried after a shell restart. Scale and rotation need a later
transform contract and are outside this command's current scope.

Hidden windows can still be raised; the order applies when they are shown.
Unregistered same-layer drawables are stacking barriers too. `z` requires a
JSON integer token (`1.0` is refused).

Hiding a window cancels the whole touch sequence across all active slots when any slot targets that window (Smithay touch-grab semantics).

Caller identity step 1 is audit-only. The port prefers verified Unix Bus
registration and keeps supervised re-registration and subscription replay.
A missing native endpoint falls back to TCP with a warning; `comp.callers`
reports `protected_recipient: false`. No caller needs a role or grant in this
step. See [the caller audit contract](../../docs/comp-callers.md).

Step 2 introduces desk-rooted roles and revocable grants after design review.
Step 3 migrates legitimate callers before enforcement; reads stay open and
input is refused while the session is locked. Capture has its own class for
future grants because it exposes screen pixels.
