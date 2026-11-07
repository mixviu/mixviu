# Integration close-out: cancelled gate needs clarification

Runtime source: 1804bb98a5ab305111bd35412bf6f3902bc24013. CBC 20261107-123118-5c7ba675-a1 on cbc2 is no longer running: its authoritative receipt says cancelled, rc 143, cancel requested. The session timeout interrupted the workspace test stage. Read the full saved log: 823,451 bytes, 14,422 lines, SHA256 ab8bea10e5e302071cdb137e726bc91e84a693882b5236803b63c728fc96d11a; 397 completed batches and 8,444 passes, no reported test failure. This is an incomplete gate, not acceptance.

The latest brief says do not requeue unless rc 75. No replacement was submitted after finding rc 143; an exception has been requested. Final release builds, four-app release-byte reruns, furniture/Track A completion and the final same-worker 2,000-row comparison remain pending. Native preflight at 4083f51 closes Screenshot's menu IDs and authority-loss fallback pixel-exactly; the formatting freeze at 1804bb9 changes no runtime behaviour.

See docs/ports/desk4-integration.md for historical four-engine hashes and docs/ports/import-hash-debt.md for all 22 inherited import mismatches, when and why they arose, and the required production-derivation reconciliation. The historical 4755e9e artifacts must not be described as final 1804bb9 acceptance. No install or VT work.

The Backdrop/Workshop follow-on brief is read, remote heads verified, and not merged pending this close-out as instructed. Scoped checks will use --light; full acceptance/release builds stay on cbc2/cbc3.

Branch lab/gold-parity; tested head 1804bb98a5ab305111bd35412bf6f3902bc24013 (this receipt commit follows); final artifacts PENDING, historical cbc3:/build/es-artifacts/4755e9e10bd2071cded1d18c7865c29d754a1009/ SHA256SUMS=9a70a839e3b4629c8a6ab8ccd6fa7748dd1c403fe4935bd8e37878b5e165c199; gates INCOMPLETE; CBC 20261107-123118-5c7ba675-a1 CANCELLED rc143, earlier 20261107-105201-a587a3c3-a1 PASS at4755e9e.
