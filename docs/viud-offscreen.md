# Inactive VT desks

The KMS backend starts its Wayland socket, clients and `comp` Bus service on
a render-node output before acquiring a seat. It never switches consoles.
The primary connector and mode are discovered with read-only DRM queries when
available. Configured output mode preferences are the next fallback; otherwise
the output is 3840×2160 at the configured scale and 60 Hz.

`MIX_VIUD_OFFSCREEN_SIZE=3840x2160` overrides the initial offscreen dimensions.
Malformed values warn and use 3840×2160 rather than stopping the desk.
After the first KMS attachment, the output's live mode and scale are retained
on every switch away. A discovered connector keeps its normal output identity;
when discovery is unavailable the desk starts with `headless-0`.

The first activation opens libseat and attaches KMS. The render-node GLES
renderer moves into the native GPU manager, preserving its EGL context, UI
textures and capture registries. The logical output and windows survive this
handover. Later libseat pause events release KMS and suspend physical input;
the headless frame path borrows the same renderer to present offscreen again.
The VT watcher blocks on sysfs notifications, with no polling or console switch.
Seat-open failures retry on one-shot timers with exponential backoff from
250 ms to 5 seconds while the VT is active. A failed watcher is re-armed with
the same bounded backoff; `render_state: offscreen-retrying` exposes this
condition until its source is listening again. Once attached, libseat owns
the transitions and the first attachment runs the same resume/announcement
transaction as later activations.

Offscreen rendering uses the headless backend's damage-driven, one-shot frame
timer, capped at the current output refresh. Idle desks have no frame timer.
The pointer retains its position while inactive and is included when capture
requests `cursor: true`. Physical input is unavailable while the seat is paused.

`comp.session.get` reports `active` and `render_state` (`active` or `offscreen`).
`comp.capture.frame` supports output and window targets in either state; its
reply reports `source: offscreen` for inactive captures. The renderer continues
to service the existing scene/capture registry hooks while inactive.
A purely headless desk also reports `render_state: offscreen`, even though
its session is active.

The legacy GPU recorder is stopped and discarded before any KMS buffer or
plane release. This is separate from PR #101's Bus recorder. Frame captures
finish GPU synchronisation and synchronous readback on the event-loop thread
before replying; their owned CPU pixels retain no KMS buffer reference. The
Bus recorder consumes those independent frames, so a later pause can release
scanout without invalidating an encoder's pixels. No native/Bus capture API
returns a borrowed KMS framebuffer.

The inactive-seat integration fixture deliberately prevents any seat open. It
maps a real Wayland client and captures output/window frames over an owned
private native Bus, including a retained-cursor pixel comparison. CBC can run
this render-node test but cannot prove physical VT activation, DRM master
handover, modesets or physical input resumption. Those require a real VT trial.

The inherited-descriptor transition fixture additionally retries two failed
seat attempts without another activation edge, transfers the actual renderer
through `take_renderer`/`create_with`, exercises `GPU_BINDING` borrowing across
repeated fake KMS attachments/pauses, retires a queued timer on activation,
recovers a missing binding, and exposes/recovers watcher failure. Its KMS
presentation is explicitly a stand-in: no seat or primary-device ioctl runs.
The two inactive-seat tests are not ignored; when no render node or built
viud/noded binaries exist they print the missing-prerequisite skip reason.
CBC acceptance builds the native binaries first, sets `MIX_TEST_MIX_EXE` to
the matching Mix binary, and runs both fixtures. A missing matching Mix binary
is also reported as a fixture prerequisite.
