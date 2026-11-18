# Bus TLS lab gaps — 2026-11-18

- Existing Bus test port race: `libraries/bus/src/native_port.rs:82`,
  `typed_success_is_ok_value`, failed with `AddrInUse` in CBC job
  `20261118-134517-7c4d549f-a1`; the same suites previously passed. A separate
  fix should retain an ephemeral bound listener through test-server startup,
  rather than probe, release and rebind. Serial rerun recorded in `last.md`.
- Lab-only configuration: `services/noded/src/lib.rs` still points built-in
  monitor/logger clients at the main TCP listener, which becomes TLS. A2
  must use `--no-monitor --no-log`; production should use Unix ingress for
  those local clients and expose typed TLS/local listener configuration.
- Revocation timing tests call the real accepted-inventory reload entry point;
  they do not measure the filesystem watch/debounce path.
  A2 must replace the signed inventory on disk and record both boundaries.
- CBC auto dispatch chose a worker outside the brief's permitted set once.
  Repeated the check on an explicitly pinned permitted worker. Pin workers
  for this lab; the dispatcher needs a separate allow-set scheduling fix.
- CBC wrapper cancellation without the submission owner returned a successful
  control receipt while leaving a job queued. The native cancellation with
  the known owner completed cancellation. A separate wrapper fix should retain
  ownership and verify terminal cancellation, rather than report success early.
- Existing `services/noded/src/session.rs:64` uses deprecated `AsyncFd::new`;
  CBC reports its I/O-safety warning. Migrate this to `AsyncFd::register` in a
  separate native-session fix; transport lab scope did not alter that code.
- A2 still needs isolated CT provisioning, a remote hostile probe launcher,
  and a Mix echo citizen. This branch tests loopback in-process brokers;
  it does not claim routed-network or hostile-CT results.
