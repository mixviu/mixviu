# Tungstenite 0.26.2: payload-free transport logs

Base: crates.io `tungstenite-0.26.2.crate`, SHA-256
`4793cb5e56680ecbb1d843515b23b6de9a75eb04b66643e256a396d43be33c13`.
Upstream licences and notices are preserved. The workspace selects this frozen
copy through `[patch.crates-io]`, including per-package builds.

Local changes:

- Frame send/read/write and assembled-message TRACE logs retain opcode and
  length, without formatting the message or frame payload.
- Close receive/reply DEBUG logs retain length/opcode, without the close reason.
- The handshake request TRACE log retains length, without raw HTTP headers.
- A `log_redaction` test included by Bus integration targets tests text, binary, ping, pong,
  raw frame and close payloads in both client and server roles. It requires TRACE
  to be compiled in and verifies each redacted diagnostic still appears.

See [the shared audit](../tungstenite-audit.md). The same patch is carried for
0.29.0, which Axum's WebSocket server uses. Exit path: upstream payload-free
logging, then remove these patches after rerunning the regressions.
