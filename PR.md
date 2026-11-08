## Summary

Clear 11 of the 18 inherited advisory ignores without changing Hickory or mail-auth versions or their APIs. Updated onto `main` at `60cb1270` by merge commit `1862f9d7`; no rebase or force-push. CBC resolution `a6434f41` starts from main’s lock graph and reapplies the advisory versions.

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

Tier 2: TLS/trust, network services, runtime dependencies, advisory policy and a narrowly scoped scanner false-positive exception. The code owner must approve. No deployment or data migration.

## How tested

Merged-tree validation (2026-11-08):

- CBC `20261108-153157-b139388e-a1`: targeted Cargo updates, build the merged tree’s own Mix, regenerate the lint baseline; PASS. Applied the generated diff locally. The baseline only shrinks against main; no counts increased.
- Affected-crate light jobs at `a6434f41`: maild/webd (`20261108-153359-f8acf742-a1`), TLS config/daemon (`20261108-153400-7256afc2-a1`), lib-mix DKIM (`20261108-153401-4b361290-a1`) and bus/props (`20261108-153403-980add31-a1`) PASS. Viu (`20261108-153404-dc557563-a1`) PASS. These five saved logs contain 4,873 passing tests and zero failures.
- Full CBC security + CI all-stage gate (`20261108-153405-e638aa26-a1`, worker any) PASS, BUILD_RC=0. Rust tests: 10566 passed, 0 failed, 90 ignored; all Mix policy checks and the final lint ratchet passed. After this gate, changes are confined to Gitleaks policy, documentation and test-only audio fixtures; release Rust behaviour and Cargo.lock are unchanged.
- GitHub security-ok PASS at `c171e033`: cargo-deny, Gitleaks and workflow checks. GitHub run 42453980666 at `a12966ff` subsequently failed one inherited audio test; the follow-up below corrects its fixture rather than weakening its assertions. Final-head checks remain required before handoff.
- Observed: seven deferred advisory ignores retained; Hickory/mail-auth versions and DNS APIs unchanged. Main’s Mix-first Dopus removal, four-engine layout and environment registry retained. No tests weakened.
- Not tested: production deployment, public mail traffic, or a newer Candle stack. Deferred vulnerabilities are not claimed fixed.

The audio test assumed that a single pipe write produced a single read, then dropped the raw reader rather than calling `audio_unwatch`. A delayed normal batch could therefore remain queued. Follow-up `c8b15ac9` keeps the content, process-reaping and empty-queue assertions, explicitly injects a pending normal batch, and invokes real unwatch through a test-only owner fixture. No production audio implementation changed. CBC `20261108-171553-37a3846c-a1` proves that omitting slot retirement makes the retained assertion fail, then restores the code and passes the full lib-mix DKIM suite: 2107 passed, 0 failed, 7 ignored. Formatting came from CBC `20261108-171202-1a7a06b0-a1`. The earlier symbolic-HEAD job selected the configured checkout and is not acceptance evidence.

The final documentation head is rechecked by full CBC job `pr8-tls-final-20261108` and GitHub. Both results are verified before handoff.

Gitleaks initially flagged historical terminal keyboard-shortcut help and audit notes quoting it. The added exception requires both an exact path (the historical help file, PR notes or Gitleaks audit notes) and the captured value equal to the public shortcut literal. It does not exempt arbitrary values or change the workflow. A pinned Gitleaks 8.30.1 full-history scan passes locally; GitHub confirms the result.

## How to undo

After landing, revert this PR’s squash commit. While it is a branch, revert its dependency, parser-adapter, baseline and policy commits together; retain main’s cut-over merge. No data change is expected. Existing keys, certificates, messages, configuration and model files keep their formats.

## Authorship and review

GPT-6.1 Sol via the mixviu-agent bot. Mark requested this split after the combined branch exceeded the review diff-growth limit. The original `fix/security-advisories` history is unchanged. Original round 1 reviewed the pre-cut-over source `b664878c`: Gemini 3.1 Pro Preview and DeepSeek V4 Pro both returned no blocking findings. DeepSeek’s non-blocking malformed-ECHCONFIG nit is closed as pre-existing: the base already locks rustls-pki-types 1.15.1, and rustls-pemfile 2.2.0 calls `pem::from_buf(rd)?` before filtering section kinds, so malformed recognised sections already fail decoding. The main refresh retains those adapters. Gemini and DeepSeek reviewed the new Gitleaks exception and its corrected delta in two rounds with no blocking findings; non-blocking future-path housekeeping is deferred. Existing broader help exceptions are unchanged. PR #8 is open; current-head code-owner approval remains required.
