# Integration round 4: full gate passed


## Detached full-gate pass at 1804bb9

CBC `20261107-125735-43beb330-a1`, cbc2, runtime source `1804bb98a5ab305111bd35412bf6f3902bc24013`: **PASS (rc 0)**. The complete saved log was read: 1109729 bytes, 19266 lines, SHA256 `0c7da20c8e8abe20b5a9711ce86d3aa97b667f69e0fc633471fc4174e6a39ec5`. All 614 reported test-result batches are green. The gate includes exact formatting, workspace Clippy and tests, Monitor parity policy, bulk primitive tests, P2 furniture and Track A (`SMOKE_FAILS=0`). The worker diff is empty. Existing warnings and ignored tests remain visible in the log.

The four engines are saved together on **cbc2** at `/build/es-artifacts/1804bb98a5ab305111bd35412bf6f3902bc24013/`. `SHA256SUMS` SHA256: `afd278113a2a7be85e047ab5a72cbbae1a8832b041d7a0616c429454ca1e2801`.

| Engine | SHA256 |
|---|---|
| mix | `5faf85a042c9547cbe042d4d0aa4bd5987109cd81c75b23fae5f9a0b5dd22f5e` |
| mixd | `ad1e4f8f86ac1cb58294831fd21055b69b7281966117fdd45a9e97402d9a38ff` |
| viud | `c9994a0567d3f5fd262c8eaa13aab3f6126b1bb97eb2596c68b519d40d85b2a6` |
| viu | `6a995927feba6d70dfb028fc43e972e964f0b9009dfe4fdff7968f9142389299` |

Four-app release-byte revalidation and the same-worker comparison are in progress. This gate receipt alone does not grant final integration acceptance. All 22 inherited provenance mismatches remain explained in [import-hash-debt.md](import-hash-debt.md); they predate this integration and must be reconciled before production derivation.

All four apps' native execution groups now pass on the saved release bytes, including all 11 Ced suites, all 11 Screenshot scripts, ten Prefs groups, Monitor quick checks and deferred reply deadlines. Final comparison/figure collectors remain in progress. Same-worker comparison CBC 20261107-144123-01357bfa-a1 passed 510 checked samples on cbc2. Dopus ACK medians are close to 4c1d715; Email was 118.9 vs 114.1 ms, so a repeat (20261107-144437-22b9e8f4) is queued before deciding the performance finding. Backdrop and Workshop merge follows this close-out.

Branch lab/gold-parity; runtime head 1804bb98a5ab305111bd35412bf6f3902bc24013 (receipt commit follows); artifacts cbc2:/build/es-artifacts/1804bb98a5ab305111bd35412bf6f3902bc24013 SHA256SUMS=afd278113a2a7be85e047ab5a72cbbae1a8832b041d7a0616c429454ca1e2801; full CBC/P2/Track A PASS, native acceptance IN PROGRESS; CBC 20261107-125735-43beb330-a1 PASS, 20261107-144123-01357bfa-a1 comparison PASS; 20261107-144437-22b9e8f4 repeat DETACHED QUEUED.
