# Four-app integrated release revalidation

Runtime source `1804bb98a5ab305111bd35412bf6f3902bc24013` on `lab/gold-parity`; following commits are documentation receipts. Monitor, Ced, Prefs and Screenshot release-byte revalidation **PASS**. No installation or VT work was performed.

## Exact bytes and gates

The four engines are together on **cbc2**, `/build/es-artifacts/1804bb98a5ab305111bd35412bf6f3902bc24013/`.

| Engine | SHA256 |
|---|---|
| mix | `5faf85a042c9547cbe042d4d0aa4bd5987109cd81c75b23fae5f9a0b5dd22f5e` |
| mixd | `ad1e4f8f86ac1cb58294831fd21055b69b7281966117fdd45a9e97402d9a38ff` |
| viud | `c9994a0567d3f5fd262c8eaa13aab3f6126b1bb97eb2596c68b519d40d85b2a6` |
| viu | `6a995927feba6d70dfb028fc43e972e964f0b9009dfe4fdff7968f9142389299` |

`SHA256SUMS`: `afd278113a2a7be85e047ab5a72cbbae1a8832b041d7a0616c429454ca1e2801`. Full detached CBC `20261107-125735-43beb330-a1` passed formatting, normal workspace Clippy, complete tests, Monitor policy, bulk primitives, P2 furniture and Track A (`SMOKE_FAILS=0`), with an empty worker diff. The entire 1,109,729-byte, 19,266-line log was read; SHA256 `0c7da20c8e8abe20b5a9711ce86d3aa97b667f69e0fc633471fc4174e6a39ec5`. It contains 614 green test-result batches, including preparation duplicates; ignored cases and warnings are not counted as passed.

## App and visual acceptance

The original frozen recipes ran in isolated native/headless sessions on the exact release viu above. Captures use matched state, dimensions, settings and data; raw PNGs, regions, perceptual figures and cleanup receipts are retained. Figures were inspected: eight Prefs sheets covering all 91 pairs, five Ced sheets, six Screenshot sheets, and all ten Monitor pairs. Normalised SSIM distortion is zero for identical images; no threshold was substituted for inspection.

| App | Native checks | Captures and remaining differences |
|---|---|---|
| Monitor | Release Bus 46/46, quick Processes/Performance contract, owned PID cleanup | Ten 1040×680 pairs; full, header, options, content and status SSIM/RMSE all **0**. Existing P1 PID-start-time fence retained. |
| Ced | All eleven release native suites, actions, mail, files, recovery, 16/64 MiB, input and live Prefs | 35 pairs, 31 exact. Problems/Output full SSIM 0.00000215932; enlarged light/compact full 0.000301469. Existing raster edges unchanged. Replace C1 whole 0.0111743, content 0.0120445, gutter 0.027558, tabs/search 0.0138192; these equal the accepted 4f2c0fd figures. A comparison with the older initial capture differs only through the already documented gold caret phase. |
| Prefs | Ten native groups, **69/69** aggregate checks including the 40-case functional matrix, all 42 pages, CLI/input/mutations/events/live authority | 91 pairs; 88 exact. Authority-recovered, search-edited and compact each whole SSIM 0.0000927844; header 0.00166035 or 0.00191192, sidebar/content **0**. Inspected difference is the retained Macros glyph raster edge. |
| Screenshot | Eleven native suites, **55/55** Bus commands, 53 strict comparisons plus 27 extended cases, 14 annotation/export/clipboard replies, native lifecycle and CLI | 47 region/overlay comparisons; 42 exact. About whole SSIM 0.000808611 (overlay 0.00624508): truthful own source stamp, S2. Guide whole 0.00000000178112. Fonts/compact whole 0.0000917354 / 0.000090203: Macros glyph raster edges only. All other regions **0**, including authority present/lost/recovered and all menu IDs. |

Screenshot's two debug-filter readings retain the port's ten real logging targets under S2. Every shared directive and level/clamp matches; the raw differences and classification are retained. No source stamp, process reading or logging target was fabricated. Ced's C1 Replace repair and Monitor's P1 fence are existing approved deviations; no new design deviation is introduced.

Deferred-reply witnesses use each bundle's actual contract: Prefs 95 seconds, Ced/default 180 seconds, Screenshot 10 seconds with explicit keep. Named and Calls reply paths survive the 22-second native witness with pings and cancellation; Screenshot's own keep witness also passes. This tests liveness past the shorter ordinary deadline, not a sleep through every maximum.

## Same-worker 2,000-row comparison

Both detached comparisons ran on **cbc2** against `4c1d7153f6e13da0e1f67317af9c3b4518ff5e09`: jobs `20261107-144123-01357bfa-a1` and `20261107-144437-22b9e8f4-a1`, both rc 0. Each has 510 checked native samples: 30 Latin-rotated rounds, 2,000 actual files, refresh witness row, applied-model predicates and native Bus ACKs. No compiler was active. App and bulk-HOF source paths are unchanged from the bulk baseline.

The first run's Email median shift triggered a repeat rather than being hidden. Times are measurements, not pixel equality. All correctness/bounds checks pass; small timing shifts remain explicitly recorded. The repeat's before/after ACK figures are:

| Operation | Median before → after, ms | p95 before → after, ms |
|---|---:|---:|
| view.sort-size | 115.61880505178124 → 113.90215088613331 | 122.29212885722518 → 120.37366000004113 |
| view.sort-name | 109.04230899177492 → 107.46755090076476 | 120.65018899738789 → 115.29486207291484 |
| view.refresh | 101.9651381066069 → 102.56626806221902 | 110.80346489325166 → 109.5840020570904 |
| selection.first | 63.46162187401205 → 63.95585194695741 | 75.24032588116825 → 72.82672985456884 |
| selection.last | 63.28818597830832 → 65.0609239237383 | 69.79305017739534 → 68.22144612669945 |
| email.list | 114.26326492801309 → 116.54402245767415 | 137.34837295487523 → 142.52687990665436 |

Email's first median was 114.120 → 118.904 ms, repeat 114.263 → 116.544 ms; its repeat p95 is 137.348 → 142.527 ms. These small residual shifts are not described as identical timings or concealed by a new threshold. The generic host remains slower than the gold per-app host; that inherited baseline is also retained in the raw results. ACK and applied-model readback do not claim physical presentation latency. Both raw runs, distributions and median/p95/p99 summaries are saved.

## Provenance and handoff

[All 22 inherited import-hash mismatches](import-hash-debt.md) list the affected files, expected/current hashes, first-divergence revisions and reasons. All match the earlier 4f2c0fd integration: 20 originate with the A1 import of newer MixOS source under an older runtime manifest, one with A5 script version handling and one with the bulk-HOF import. The manifest entered later without reconciliation. Subsequent adaptations are listed per file. `verify_runtime` remains red for this inherited provenance debt; passing behavioural gates does not clear it. Reconcile source and adaptation provenance before production derivation, rather than blindly replacing expected hashes.

Private evidence archive: `gold-parity-20261107/four-app-integration/1804bb98a5ab305111bd35412bf6f3902bc24013/`, including executed recipes, captures, region metrics, inspection sheets, cleanup, complete CBC logs and paired performance summaries. Worker `HANDOFF.json` and `HANDOFF.md` accompany the four engines and manifest. The next checkpoint integrates Backdrop and Workshop and must revalidate all six apps on its successor bytes.
