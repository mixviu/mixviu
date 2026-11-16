# Tungstenite 0.29.0: payload-free transport logs

Base: crates.io `tungstenite-0.29.0.crate`, SHA-256
`6c01152af293afb9c7c2a57e4b559c5620b421f6d133261c60dd2d0cdb38e6b8`.
Upstream licences and notices are preserved. The workspace selects this frozen
copy through `[patch.crates-io]`, including per-package builds.

Local changes and regression coverage match
[the 0.26.2 patch](../tungstenite-0.26.2/PATCHES.md): payload-free frame,
assembled-message, close and handshake-request diagnostics. No log level cap,
runtime filter or subscriber opt-in is needed.

See [the shared audit](../tungstenite-audit.md). Exit path: upstream payload-free
logging, then remove these patches after rerunning the regressions.
