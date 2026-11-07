# Desk4 integration receipts

## Bulk and Monitor integration

Tested source `9643f9235819d33fc8e398ea0abfcc29d84192cf` merges `4c1d7153f6e13da0e1f67317af9c3b4518ff5e09` into `lab/gold-parity`. All four release engines are together on **cbc2**, `/build/es-artifacts/9643f9235819d33fc8e398ea0abfcc29d84192cf/`. SHA256SUMS hash: `2becc4885ba92e1e2ec9164a664911fc01c6f0ea5fa31eff269c64959e4a8869`.

```text
2dd3b73fb913c32c9106275e63c5a4455966824a5bb81573e6f639d5abe43ca3  mix
4795719e5b8ff1f8a53e37c431349ad462b85ab7afb26e53f8f4f2757f2ee138  mixd
f32cc6cfb3162955f67a9938a56dcc9c564ee699be861e55e26c4f1db602e205  viud
f7c4f97ca82c989340ba7060ad1130aa164de3b0d986dc9ae7680db8d7796a7b  viu
```

CBC `20261107-064343-f05d5cfb-a1` completed fmt, both isolated-profile Clippy runs, all workspace/shell test stages (10,649 passed, zero failed, 100 existing ignores), four Monitor Mix tests and explicit bulk profiles. It was interrupted during the furniture-oracle build; it is not recorded as an entirely successful job. Continuation `20261107-075258-c6071acd-a1` returned BUILD_RC=0: oracle build, the previously ignored P2 furniture test, Track A SMOKE_FAILS=0, and SHA256SUMS. No tests were weakened.

Monitor's exact release SHA256 is the viu entry above. All 12 native runs pass: 55 visual comparison sets, 46/46 matched structured Bus replies, complete actions/macros/native hosting, disposable-child signals and real daemon, 19 live Prefs states on unchanged PIDs, authority recovery and cleanup. Processes and all seven Performance pages retain whole/header/options/content/status SSIM and RMSE distortion 0; resource rail/dashboard, command line and CPU charts/grids also remain 0. Two Prefs raster cases reproduce the accepted node-chevron/Close antialiasing exactly (maximum whole SSIM distortion 9.74852e-8); content/status remain exact. P1 remains the sole approved non-visual deviation. The previous Monitor acceptance's region rectangles and figures remain applicable. Private receipt: `gold-parity-20261107/integration/9643f9235819d33fc8e398ea0abfcc29d84192cf`.

### Same-worker 2,000-row comparison

CBC `20261107-075901-fe888f44-a1` passed on CBC2 using the saved release bytes for both sources. Thirty warm rounds per cell, Latin-rotated Dopus order and alternating Email order; 510 checked samples. Every sort, selection, revision and row count is checked; refresh creates an actual 2,001st witness row. Original gold is pinned to 8779f33. The private native Bus measures execute acknowledgement, followed separately by verified model readback. These are not physical frame-presentation times.

| Action | 4c1d715 median/p95/p99 ms | Integrated median/p95/p99 ms |
| --- | ---: | ---: |
| view.sort-size | 115.03/124.15/126.83 | 113.88/119.34/128.84 |
| view.sort-name | 108.94/119.92/122.57 | 109.60/118.98/155.30 |
| view.refresh | 100.00/111.32/112.69 | 100.54/110.52/117.20 |
| selection.first | 64.78/72.39/76.95 | 62.87/75.00/81.75 |
| selection.last | 63.61/71.47/73.13 | 64.28/68.85/80.50 |
| email.list | 114.58/120.64/129.92 | 115.43/132.24/136.31 |

Median changes range from -2.9% to +1.1%; all original bulk hot paths are retained. Tail values are preserved rather than rounded into an equivalence claim. The existing roughly-100 ms viability work remains separate; this merge does not claim a new latency target. An earlier 480-sample witness (`20261107-075542-24a1c263-a1`) also passed and is retained. All benchmark processes and brokers are confirmed stopped; a later audit resolves the broker that was still exiting at the initial cleanup readback.

## Accepted Ced integration

Merged `d4e19db5259e448190f3e877a0021f735195635a` at `4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0`. Only last.md conflicted. The merged source is now **ACCEPTED** for this integration handoff. All four release engines are together on **cbc3**, `/build/es-artifacts/4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0/`. SHA256SUMS hash: `302124de8d98426a199daebd9341535cecd6e20bc7486d18abee81f5fada22b9`.

```text
52b7cc69bb445bf95e07649267770b41cbe2aa4b03b91163091a4ae9b6681a41  mix
f3b3b5d7b22f2c2cfca9ad5a28e5fd5653cdd5ed5bc62964242db9a422e9a659  mixd
3fd391f4bed21d0b9cbcd6ed6a674842bd5f232d328c398b50262b967fd21de8  viud
8c2eb07cbef36eed50ddcaacdb8b4664554fdfed999b0b42878aea2e3abe1735  viu
```

CBC `20261107-080318-8bce8633-a1` passed fmt, normal Clippy, 59 scoped Rust tests, four Monitor Mix tests, the four locked release builds and clean worker diff. The complete saved log is archived. Ced's release-byte acceptance passes 11 native suites and 35 paired captures: 31 exact, three unchanged raster-edge cases and Fable C1 Replace. The full [revalidation record](ced-acceptance.md#release-byte-revalidation-after-integration) includes every nonzero region and the gold caret-phase distinction.

Monitor quick recheck on these same release bytes passes 46/46 structured replies and 10 comparison sets covering Processes and every Performance page. Whole/header/options/content/status SSIM and RMSE distortion are zero. This is a quick recheck, not a claim to have repeated the first set's entire live Prefs matrix on the editor merge. All owned services/apps/brokers are stopped. Private archive: `gold-parity-20261107/editor-integration/4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0`.

Use the second set for the integrated Ced/bulk/Monitor handoff; the first CBC2 set is retained as a separately verified checkpoint. No installation, desktop or VT work was performed.


## Prefs and Screenshot close-out: gate interrupted

Tested source awaiting completion: `1804bb98a5ab305111bd35412bf6f3902bc24013`. The worker receipt for CBC `20261107-123118-5c7ba675-a1` on cbc2 records **cancelled, rc 143**, detail `cancel requested`. This is not rc 75 and is not a passing gate. The complete saved 823,451-byte, 14,422-line log was read; its SHA256 is `ab8bea10e5e302071cdb137e726bc91e84a693882b5236803b63c728fc96d11a`. It records 397 completed test batches, including preparation, with 8,444 passes and no reported test failure before cancellation. The remaining workspace tests, release build, furniture and Track A stages did not complete. These totals do not imply a full workspace pass.

The latest brief permits requeueing only for rc 75. No replacement job was submitted after establishing rc 143; resuming this cancelled gate requires that instruction to be clarified. No four-engine release acceptance is claimed for 1804bb9. Its release-byte app suites and same-worker performance comparison remain pending.

The previous 4755e9e checkpoint has an available four-engine set on cbc3 at `/build/es-artifacts/4755e9e10bd2071cded1d18c7865c29d754a1009/`, SHA256SUMS `9a70a839e3b4629c8a6ab8ccd6fa7748dd1c403fe4935bd8e37878b5e165c199`. This is historical evidence, not the final repaired integration. Its Monitor, ced and Prefs rechecks passed; Screenshot exposed menu-ID and authority-loss regressions subsequently fixed at 36a65f4/4083f51 and verified in native debug preflight.

| Historical 4755e9e engine | SHA256 |
| --- | --- |
| mix | `ee513d7d447d2aec70b43f623d1d7d15f65d86eb350c9054397822d3c3d4dc15` |
| mixd | `c363b520026ed8301267d53983a7bcdc3b3247e32dec7f4ed405b5dcbe6cdacd` |
| viud | `036fe93c9ebc367aeb8eac5607e4b6c51420c7a1b214b273d5962ea13a4d9b8a` |
| viu | `bf16c811b8a99d1a93bad0a55a5ac920307b8d8ba08bccaf00e4e2495fab972d` |

The [22 inherited import-hash mismatches](import-hash-debt.md) are fully enumerated with both hashes, source pins, divergence dates and adaptation commits. They are unchanged from 4f2c0fd. The import verifier remains failing until the overlapping inventories are reconciled; behavioural acceptance does not clear that provenance debt before production derivation.

The subsequent Backdrop/Workshop integration brief has been read and both requested remote heads verified (250f87d and 493ae42). They have not been merged because that brief explicitly follows this close-out. Workshop's shared editor field and integrated acceptance remain next work.

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
