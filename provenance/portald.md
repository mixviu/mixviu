# portald derivation

Source: `markc/mixos`, immutable commit
`8779f33284b7d3f1178c3d57a699efb9b0867b22`. Imported with `git show` from
that commit, never from the source checkout. The original MIT OR Apache-2.0
licences and copyright notices are retained. `portald-files.txt` records the
source paths and Git blob hashes, including both documentation pages.

Adaptations: workspace paths reuse MixViu libraries; portald remains a standalone
binary. The default-on `screencast` feature owns PipeWire and Wayland client
dependencies. The protocol implementation moved to `screencast/enabled.rs`;
FileChooser's shared request/handle and caller identity helpers remain available
without ScreenCast. A build without the feature refuses `--screencast`.

Settings profile, consent policy, frame-rate ceiling and PipeWire remote name
come from registered installed defaults. Environment access uses `paths::env`;
cache paths use the paths state root. The unit uses layout placeholders and an
explicit per-session cache path, with no systemd implicit state/runtime roots.
The service account is `mix`. The template manifest includes the unit.

PipeWire 0.10.1 is pinned and admitted only for portald. Its native crates are
not strict-crate allowlist entries. On the CBC worker, libspa's build script
compiles `tests/pod.c`; libspa-sys runs bindgen and compiles generated static
function wrappers and `src/{type-info,command,node-command,meta}.c`.

## Protocol inventory

Unchanged bus name: `org.freedesktop.portal.Desktop`.
Unchanged interfaces: `org.freedesktop.portal.Settings`,
`org.freedesktop.portal.FileChooser`, `org.freedesktop.portal.ScreenCast`,
`org.freedesktop.portal.Request`, `org.freedesktop.portal.Session`.
Unchanged error namespace: `org.freedesktop.portal.Error`.
Unchanged settings namespace: `org.freedesktop.appearance`.
Unchanged object path: `/org/freedesktop/portal/desktop`, with request and
session objects below its `/request/` and `/session/` branches.
No `.portal` file exists anywhere in the pinned source tree. This source owns
the frontend bus name itself; it is not an `org.freedesktop.impl.portal.*`
backend behind another portal process. No protocol identifier was renamed.

## Compositor calls

The source has no compositor Bus calls. It connects to the session's Wayland
socket. `comp` remains viud's Bus service; portald does not call it.

| Client operation | viud counterpart |
|---|---|
| Registry enumeration, wl_output bind (up to v4) and output events | dispatcher Wayland output factory; outputs advertisement |
| zxdg_output_manager_v1 v2–3, get_xdg_output (optional) | dispatcher output factory uses OutputManagerState::new_with_xdg_output |
| wl_shm v1, create_pool/create_buffer and destroy | dispatcher shm protocol state (Smithay) |
| zwlr_screencopy_manager_v1 v3, capture_output | screencopy crate create_global and manager Dispatch |
| frame copy, copy_with_damage and destroy | screencopy crate frame Dispatch |
| buffer, flags, ready, failed, buffer_done events | screencopy crate frame service |

These are source counterparts, not a live compositor acceptance claim.

## Test limits

All Rust source tests and both Mix scripts are retained. The original visual
chooser script needs the frozen `tests/desktop/nested_host_lib.mix` and native
Dopus executable, neither present in MixViu. It fails explicitly when that
fixture is unavailable. It is not run by the headless gate and does not stand
in for visual chooser acceptance. Rust FileChooser tests use the test broker.

The restart-recovery Mix script needs a systemd user manager and creates an
isolated transient unit and private D-Bus. It is retained for explicit worker
execution; it must never be pointed at a running desktop/service. No desktop,
VT, install or production service action is part of this derivation.

The source also patches PipeWire 0.10.1 with Buffer::find_meta_mut, required
for producer PTS/sequence metadata. The same frozen vendor/pipewire tree is
retained, with its upstream archive checksum, patch explanation and MIT licence.
No PipeWire version or feature change is used to replace that patch. The old
files-spec documentation assertion now checks the imported portal contract page.
