## Summary

Clear 11 of the 18 inherited advisory ignores without changing Hickory or mail-auth versions or their APIs. Base: `395aab71` on `infra/pr-ci`.

| Advisory | Resolution |
| --- | --- |
| RUSTSEC-2026-0346, 0347, 0348, 0349 | rustls 0.23.46 |
| RUSTSEC-2026-0382 | rustls-webpki 0.103.16 |
| RUSTSEC-2026-0381 | webpki-roots 1.0.11; 0.26.11 is its unaffected compatibility re-export |
| RUSTSEC-2025-0134 | Remove rustls-pemfile; use the maintained parser in rustls-pki-types 1.15.1 |
| RUSTSEC-2026-0386 | Tokio 1.53.3, including the exact application/library pins |
| RUSTSEC-2026-0385 | ctutils 0.4.8 |
| RUSTSEC-2026-0194, 0195 | quick-xml 0.41.0; adapt REPORT text/entity events |

Remove unused mail-send. Its default DKIM feature enabled mail-auth’s default report feature, the remaining path to quick-xml 0.38.4, ZIP 6 and compression dependencies. Existing source has no uses of mail-send. This removal allows the XML fix without migrating mail-auth. Dependency resolution and formatting were performed through CBC and the resulting diff applied locally.

The PEM adapter preserves certificate filtering, private-key zeroisation and JWT rejection of a certificate before a private key. REPORT parsing preserves predefined/numeric entities in hrefs and sync tokens; unknown entities and invalid UTF-8 mark the request malformed.

Deferred debt remains in `deny.toml`:

- RUSTSEC-2026-0118 and 0119: the resolver and mail-auth migration is preserved on `fix/security-hickory`, with its own review budget. Neither migration is included here.
- RUSTSEC-2026-0367, 0368, 0369 and 0429: the remaining ZIP 7.4.0 is required by exactly pinned candle-core 0.10.2. Fixes require ZIP 9; there is no compatible 0.10 parent update. A Candle 0.11 stack upgrade needs separate indexd/model compatibility validation. The unused ZIP 6 path is removed here.
- RUSTSEC-2023-0071: RSA has no patched release listed by RustSec. Replacing its signing/key-generation use is a separate crypto change.

## Risk tier

Tier 2: TLS/trust, network services, runtime dependencies and advisory policy. The code owner must approve. No deployment or data migration.

## How tested

- Run: all five affected-crate jobs passed with `--light` at `43a7c7b8`; the final JWT tests passed at `dc699500`; final REPORT tests, security, formatting and environment checks passed at `b664878c`. The final full CI test stage passed at `b664878c`: 10,525 Rust tests passed, 0 failed, 90 ignored, followed by all five Mix policy checks. Clippy and the viud dependency gate passed. Mix lint fails identically at the base and this branch; no baseline was weakened.

| CBC job | Check | Result |
| --- | --- | --- |
| `20261107-215710-64ee1d7a-a1` | Resolve dependency removal and format adapters | Pass |
| `20261107-220026-acacbff6-a1` | maild + webd tests | Pass |
| `20261107-220026-e6a5c094-a1` | config + daemon tests, TLS enabled | Pass |
| `20261107-220026-6d4dae71-a1` | lib-mix tests, DKIM enabled | Pass |
| `20261107-220026-62cd04d9-a1` | bus + props tests | Pass |
| `20261107-220026-3d85fbf6-a1` | lib-viu tests | Pass |
| `20261107-221648-810111e3-a1` | Final JWT tests, including both PEM compatibility regressions | 9 passed |
| `20261107-221713-6e136c4d-a1` | cargo deny check; CI fmt + env stages | Pass; env registry 196 names, 0 problems |
| `20261107-221945-3122f461-a1` | Earlier full CI test attempt | Cancelled (143) after final XML fix; superseded |
| `20261107-222839-4c5d81a1-a1` | Final REPORT tests, including split invalid UTF-8 regression | 26 passed |
| `20261107-222839-b8cd5ee7-a1` | Final cargo deny check; CI fmt + env stages | Pass |
| `20261107-223327-64e473bf-a1` | Final full CI test stage, `--worker any` | Pass; BUILD_RC=0 |
| `20261107-223349-8cb1d811-a1` | CI clippy and viud dependency gate | Pass; subsequent Mix lint fails on eight inherited regression entries |
| `20261107-223537-fa2ce977-a1` | Base `395aab71` Mix lint comparison | Same eight regression entries, same counts |

- Observed: no Hickory/mail-auth version changes or DNS adapter changes in the split diff. The security command reports all four checks OK, alongside permitted policy warnings and cargo-deny workspace-span diagnostic messages. Existing ignored tests remain ignored. Deprecated Tokio AsyncFd calls still warn; the Mix calls move owned cloned sockets into AsyncFd, and noded passes an OwnedFd; these types retain their descriptor lifetimes.
- Inherited gate failure: Mix lint reports seven Backdrop/Workshop MIX-E1102 entries and one Backdrop MIX-W2401 entry. The base comparison reproduces all eight with identical counts. These files, the linter and its baseline are unchanged here; the full `all` recipe is therefore not claimed green.
- Not tested: production deployment, public mail traffic, or a newer Candle stack. Known deferred vulnerabilities are not claimed fixed.

## How to undo

Revert this branch's dependency, parser-adapter and deny-policy commits together. No data change is expected. Existing keys, certificates, messages, configuration and model files keep their formats.

## Authorship and review

GPT-6.1 Sol via the mixviu-agent bot. Mark requested this split after the combined branch exceeded the review diff-growth limit. The original `fix/security-advisories` history is unchanged. Round 1 reviewed the final source `b664878c`: Gemini 3.1 Pro Preview and DeepSeek V4 Pro both returned no blocking findings. DeepSeek’s non-blocking malformed-ECHCONFIG nit is closed as pre-existing: the base already locks rustls-pki-types 1.15.1, and rustls-pemfile 2.2.0 calls `pem::from_buf(rd)?` before filtering section kinds, so malformed recognised sections already fail decoding. No source change followed the reviews. Code-owner approval remains required. No PR opened before cut-over.
