## Summary

Migrate the DNS stack to Hickory 0.26.3 and mail authentication to mail-auth =0.11.0, split from the smaller TLS/security update (fix/security-tls). Clears the hickory-proto advisories and consolidates the tree on one Hickory and one quick-xml. The branch keeps the full original advisory history.

- Hickory 0.26: dnsd resolver/serve/wire, mix DNS builtins, maild rDNS, MX delivery, envelope IDNA and DNSBL, webd MX resolve.
- mail-auth =0.11.0 (0.11.3 is yanked): Box<str> TXT cache keys, DmarcParameters with the configurable domain-suffix function, Error::Dns(DnsError::...).
- quick-xml 0.41 in DAV REPORT parsing; tokio =1.53.3; rustls-pemfile replaced by rustls-pki-types; unused mail-send removed.
- logging caps system-configuration(-sys), pulled in by Hickory 0.26 on macOS.
- .gitleaksignore for the known terminal-help false positive on fix/mix-hygiene.

## Behaviour changes (release notes)

- **DKIM:** a DKIM-Signature that mail-auth 0.11 cannot parse (missing required tags, bad base64) is reported as `dkim=neutral`, as under 0.7 and per RFC 8601 section 2.7.1. mail-auth 0.11 itself reports nothing for these. The synthesized results follow mail-auth's own outputs, so under `max_dkim_signatures` they are the first dropped, and in Authentication-Results they appear after verified signatures. DMARC is unaffected (neutral never aligns).
- **DAV REPORT:** predefined and numeric XML entity references are decoded; an unknown entity rejects the REPORT as malformed. Before, the affected text chunk was silently dropped, truncating hrefs.
- **DNSBL resolvers:** a scoped IPv6 resolver address (link-local with a zone id) is refused at startup, because Hickory 0.26's configuration cannot carry the scope id. Use a global address or a local forwarder.

## Risk tier

Tier 2 (dnsd, maild, webd, the Bus library, deny.toml).

## How tested

- CBC `tools/ci.mix` stages fmt, env, viud, mix-lint, clippy and the full test suite green at fa52b33e (job 20261108-173323-19ccf584-a1) apart from a logging cap fixed in 76857d20; GitHub CI green on afd973f9 and 27d47130.
- maild-auth tests for the DKIM change: upstream behaviour pinned, maild's neutral result, no double reporting of Error::Dkim parse errors (light CBC job 20261108-185600-a42961eb-a1).
- Merge gate: main's ci.mix `all` on CBC against the head, two review families, DeepSeek security pass and a Fable verdict, posted as `gate-ok`.
- Not tested: a live mail or DNS deployment. Nothing is installed by merging.

## How to undo

Revert the squash commit. It reverts manifests, lockfile, adapters, advisory ignores and the DKIM synthesis together. No data or configuration migration.

## Authorship and review

GPT-6.1 Sol (adapters) and Claude Opus (finish, merge with main, DKIM decision implementation) via mixviu-agent. DKIM neutral-vs-permerror decided by Fable. Reviews: Sol and GLM-5.3 approve with no blocking findings; security pass and Fable verdict recorded in the gate.
