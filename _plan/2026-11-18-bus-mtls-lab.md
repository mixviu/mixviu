# Bus mutual TLS prototype A

## Charter

Question: can two native brokers authenticate by inventory key on a shared
plain network, refuse outsiders, and keep call overhead small?

Mode: novel, lab branch only. Budget: one working day, stopping at a measured
hermetic specimen and report. No merge, PR, installation or live-host work.

Principles: TLS 1.3 mutual possession proofs; accepted signed inventory as
authority; no source-address identity; Mix automation; CBC Rust execution;
no new dependency packages. Extend the native broker and mesh dialler.

Evidence: real noded handlers with genesis-signed synthetic fixtures,
negative handshakes and frame gates, live revocation, socket ciphertext tap,
release comparisons with today's plaintext path. These generated fixtures
are unit/acceptance evidence, not captured Proxmox oracle traffic.

Whole-specimen independent survey is the next stage. No derived production
implementation is authorised by this lab. Results and A2 requirements are
at the beginning of `last.md`; unresolved gaps are in
`_doc/TODO-bus-mtls.md`.
