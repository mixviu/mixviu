# Bus transport logging audit — 2026-11-15

PR #158 audits the locked Bus WebSocket paths, rather than relying on a binary's
subscriber configuration. Cargo selects patched Tungstenite 0.26.2 and 0.29.0
through workspace patches. This covers `bus` clients, Mix/mix-shell, Viu hosts,
settings clients, mesh clients, and noded's Axum WebSocket server, whether built
alone or together. Neither versions nor protocol behaviour change.

Inspected all logging call sites in Tungstenite and tokio-tungstenite 0.26.2 and
0.29.0, plus Bus/noded ingress, routing and client logging. Findings:

- Tungstenite formats whole frames on send/read/write, assembled messages on
  receive, close reasons at DEBUG, and raw handshake requests at TRACE. Both
  vendored copies replace those arguments with opcode/type and byte counts.
- Frame header parse diagnostics contain flags, opcode, length and masking
  metadata only. Remaining handshake/TLS diagnostics contain state, endpoints,
  certificate counts and errors, not WebSocket message bodies.
- tokio-tungstenite logs polling/handshake state and Display-form errors.
  Tungstenite's error Display does not format retained messages (including
  WriteBufferFull) or close reasons. No wrapper patch is needed.
- Axum 0.8.9 delegates frame I/O to tokio-tungstenite 0.29.0. Hyper 1.12.0's
  HTTP/1 upgrade/dispatch/I/O logs describe state, lengths and HTTP errors;
  post-upgrade WebSocket data goes through the patched transport. Tokio 1.54.1
  and hyper-util 0.1.21 provide socket/upgrade I/O, not WebSocket payload logs.
- Bus's receive-loop parse-error DEBUG log and noded's invalid-message DEBUG
  log formatted ParseError, which includes raw input prefixes. They now report
  a fixed invalid-envelope diagnostic. Other Bus/noded session/routing logs
  use metadata, lifecycle state and Display-form transport errors, not bodies.
- The shared `logging` crate and Viu's subscriber forward events; they do not
  need opt-in redaction or a special filter. Native-session keys and proofs are
  WebSocket payloads and are removed before either log bridge can see them.

Guards: isolated noded bootstrap capture (Bus allocation/renewal and server
traffic), isolated mix-shell capture (Bus allocation and real shell attach),
and each vendored version's `log_redaction` test. Captures install LogTracer
explicitly, assert a legacy TRACE witness reaches the buffer, and assert native
tracing still works. Subprocess parents require exactly one passing test.

Limits: this is transport payload confidentiality, not application-wide secret
scrubbing. Endpoint URLs, routing metadata and HTTP validation diagnostics are
still logged; callers must not put secrets there. Explicit application output,
caller-formatted Debug/Display values, packet captures, external logging tools,
and future transport versions are outside this patch. Viu/settings coverage is
through the common patched dependency; no desktop visual run is claimed.
