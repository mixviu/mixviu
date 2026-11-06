# Workshop parity round 2 — in progress

W1 resolved by the round-two brief: gold wins. Format now pretty-prints JSON,
refuses expansion over the 65,536-byte body limit without replacing the body,
and preserves invalid/blank text as gold does. Replies use gold headers,
pretty JSON and the 1,000,000-byte UTF-8 boundary and marker. A reusable opt-in
raw native reply envelope preserves body whitespace and separate error text.

Frame wiring code: 93c1578. Workshop adopts the shared gold frame, title and
server icon, action menus, standard toolbar tail, About/Shortcuts overlays,
window close, theme choice, settings/keymap callbacks and clipboard. Existing
content remains the bounded specimen pending the full feature pass. Macro,
mesh and agent actions are not yet implemented; no feature acceptance claimed.

Installed Mix text contract checks and embedded-function-aware lint pass.
CBC frame/text/native-bridge gate queued as 20261107-065902-b38ad05a at
93c1578; no CBC result yet. Captures and live Prefs witnesses remain pending.
Pinned gold: 8779f33284b7d3f1178c3d57a699efb9b0867b22. No install/VT work.

Further work is active: full node discovery/tree, Bus structured replies and
deferred completion, traffic/mesh, macro manager and agent workflows, then
exact release gate and isolated gold/candidate captures under parity-method.
No proposed deviation and no Fable question is pending. Overall NOT ACCEPTED.

Branch lab/parity-workshop; head SHA 93c1578 (frame code; receipt commit is HEAD containing this file); frame wired, CBC pending; features/Bus/Prefs incomplete; SSIM/regions unmeasured; CBC job 20261107-065902-b38ad05a pending.
