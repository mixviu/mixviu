## Summary

Park the Hickory 0.26.3 and mail-auth 0.11 migration separately from the smaller TLS/security update. This branch preserves the full original advisory branch history and the uncommitted DNS message, record, generic lookup and authentication adapters. Non-Hickory fixes are being isolated on fix/security-tls.

## Risk tier

Tier 2. Unfinished: do not merge. The migration changes dnsd, Mix DNS builtins and mail authentication. The exact mail-auth 0.11.0 pin retains the configurable DMARC domain-suffix policy; its lockfile still needs resolution. Scoped IPv6 DNSBL resolvers are explicitly refused because the new upstream configuration cannot represent their scope IDs.

## How tested

- Run: initial TLS maild/webd tests passed in CBC job 20261107-204531-0eb24e83-a1. Subsequent DNS and mail compile attempts exposed upstream API changes.
- Observed: the latest adapters are preserved at source commit 2dfcb234. Cargo.lock still resolves mail-auth 0.11.2 while the manifest pins 0.11.0; this branch is not buildable with --locked yet.
- Not tested: the final adapters, a final cargo-deny pass, or the full workspace gate. Old diagnostic jobs 20261107-211707-a72fecd8 and 20261107-212635-39b09297 were submitted before the split; their results are not acceptance evidence for this branch. No deployment.

## How to undo

Revert the migration commits together, including manifests, lockfile, adapters and advisory ignores. No data change is expected. No configuration or wire-format migration is intended.

## Authorship and review

GPT-6.1 Sol via mixviu-agent. Mark chose this split under REVIEW.md after the combined diff exceeded the growth limit. This branch has a fresh three-round review budget; reviews of the combined branch do not approve this split. Code-owner approval and two independent review families remain required. No PR opened.
