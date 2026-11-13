# Inactive VT desks

The KMS backend starts its Wayland socket, clients and `comp` Bus service on
a render-node output before acquiring a seat. It never switches consoles.
The primary connector and mode are discovered with read-only DRM queries when
available. Configured output mode preferences are the next fallback; otherwise
the output is 3840×2160 at the configured scale and 60 Hz.

`MIX_VIUD_OFFSCREEN_SIZE=3840x2160` overrides the initial offscreen dimensions.
After the first KMS attachment, the output's live mode and scale are retained
on every switch away. A discovered connector keeps its normal output identity;
when discovery is unavailable the desk starts with `headless-0`.

The first activation opens libseat and attaches KMS. The render-node GLES
renderer moves into the native GPU manager, preserving its EGL context, UI
textures and capture registries. The logical output and windows survive this
handover. Later libseat pause events release KMS and suspend physical input;
the headless frame path borrows the same renderer to present offscreen again.
The VT watcher blocks on sysfs notifications, with no polling or console switch.

Offscreen rendering uses the headless backend's damage-driven, one-shot frame
timer, capped at the current output refresh. Idle desks have no frame timer.
The pointer retains its position while inactive and is included when capture
requests `cursor: true`. Physical input is unavailable while the seat is paused.

`comp.session.get` reports `active` and `render_state` (`active` or `offscreen`).
`comp.capture.frame` supports output and window targets in either state; its
reply reports `source: offscreen` for inactive captures. The renderer continues
to service the existing scene/capture registry hooks while inactive.

The inactive-seat integration fixture deliberately prevents any seat open. It
maps a real Wayland client and captures output/window frames over an owned
private native Bus, including a retained-cursor pixel comparison. CBC can run
this render-node test but cannot prove physical VT activation, DRM master
handover, modesets or physical input resumption. Those require a real VT trial.
