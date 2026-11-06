# Desk4 integration: ACCEPTED

Tested merged source: `4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0` on `lab/gold-parity`. Includes bulk primitives from 4c1d715, accepted Monitor and accepted Ced d4e19db. This is a receipt-only commit; runtime bytes remain those of the tested source.

[Full handoff](docs/ports/desk4-integration.md) records both four-engine sets, all SHA256s, CBC gates and performance figures. [Ced release acceptance](docs/ports/ced-acceptance.md#release-byte-revalidation-after-integration) records 11 native suites, 61 actions, 53 commands, nine live Prefs states and 35 paired captures (31 exact, three unchanged raster cases, C1 Replace). Monitor's quick recheck matches 46/46 replies and has zero whole/region distortion on Processes and all seven Performance pages. All owned processes stopped.

First integration 9643f92 on CBC2: full test stages 10,649 passed; Monitor's complete release revalidation passed; P2 furniture and Track A passed in continuation; 510 same-worker Dopus/Email samples preserve the bulk medians. Its complete artifact set remains at `/build/es-artifacts/9643f9235819d33fc8e398ea0abfcc29d84192cf/`, SHA256SUMS `2becc4885ba92e1e2ec9164a664911fc01c6f0ea5fa31eff269c64959e4a8869`. The initial job was interrupted during the oracle build, not during a failing test; its completed stages and passing continuation are distinguished in the full receipt.

**Final four-engine set:** cbc3:`/build/es-artifacts/4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0/`. SHA256SUMS `302124de8d98426a199daebd9341535cecd6e20bc7486d18abee81f5fada22b9`, verified against all four files.

```text
52b7cc69bb445bf95e07649267770b41cbe2aa4b03b91163091a4ae9b6681a41  mix
f3b3b5d7b22f2c2cfca9ad5a28e5fd5653cdd5ed5bc62964242db9a422e9a659  mixd
3fd391f4bed21d0b9cbcd6ed6a674842bd5f232d328c398b50262b967fd21de8  viud
8c2eb07cbef36eed50ddcaacdb8b4664554fdfed999b0b42878aea2e3abe1735  viu
```

Final CBC `20261107-080318-8bce8633-a1`: BUILD_RC=0, fmt/Clippy, 59 scoped Rust tests, four Monitor Mix tests, all four release builds, no worker diff. Complete saved log read and archived. No new deviation, install, desktop or VT work.

Branch lab/gold-parity; tested head SHA 4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0 (receipt HEAD is the commit containing this file); artifact cbc3:/build/es-artifacts/4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0/ mix=52b7cc69bb445bf95e07649267770b41cbe2aa4b03b91163091a4ae9b6681a41 mixd=f3b3b5d7b22f2c2cfca9ad5a28e5fd5653cdd5ed5bc62964242db9a422e9a659 viud=3fd391f4bed21d0b9cbcd6ed6a674842bd5f232d328c398b50262b967fd21de8 viu=8c2eb07cbef36eed50ddcaacdb8b4664554fdfed999b0b42878aea2e3abe1735 SHA256SUMS=302124de8d98426a199daebd9341535cecd6e20bc7486d18abee81f5fada22b9; gates PASS (first full test stages + furniture/Track A + 510-sample 2,000-row witnesses; merged Ced release acceptance + Monitor quick + scoped CBC); CBC 20261107-064343-f05d5cfb-a1 (interrupted after test stages), 20261107-075258-c6071acd-a1, 20261107-075901-fe888f44-a1, 20261107-080318-8bce8633-a1.


Prefs merge checkpoint: shared deferred replies, settings preview, galleries and settings-browser retained alongside Ced and Monitor. Integration gates pending.

Branch lab/gold-parity; head is this Prefs merge checkpoint; per-app revalidation, final artifacts and CBC pending.

Screenshot merge checkpoint: all four app contracts retained; shared host reconciliation and full CBC/release-native gates pending.

Branch lab/gold-parity; head is this merge checkpoint; all per-app revalidation and final four-engine artifacts pending; CBC pending.
