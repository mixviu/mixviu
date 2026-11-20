# toolsd Mix service port

Source: the deployed Mix tools service assets, captured read-only on
2026-11-20. MIT OR Apache-2.0, copyright 2026 Mark Constable. Original
SPDX and copyright notices remain. These hashes identify the source bytes
without publishing host identity, private configuration or credentials.

| Source relative to the toolsd library | SHA-256 |
|---|---|
| `toolsd.mix` | `86486f783c5b6f0be2d9236ab4f3cc60ad88b9a5bd9cd90f71db418d49daa46f` |
| `toolsd_drain.mix` | `143ce8bba4bcd6ef415c25564bfdef311272c27795f61784d37744063b4de4a0` |
| `lib/tools_exec.mix` | `7ff0d6a4713ff520d675e8f529606558f2a061196101da73c9c7db942e58f22a` |
| `lib/tools_registry.mix` | `075f0e93db8188e53d1ec47fe9e49ce230411aef572b2db7b28646eec28a7af2` |
| `tools/shwho.mix` | `5498a036a074c6c92d415df5320276776d92d22dd676e0e2c3dd4e6f9149227b` |

Adaptations: canonical MixViu paths, script-relative lexical includes,
MIX root environment, version headers, bundle templates, Unix Bus endpoint,
sysusers and runtime tmpfiles. Existing include-analysis warnings are locally
acknowledged to preserve the lexical library contract; runtime tests exercise
that code. No argv, role ladder, tool limits or claim/report protocol change.
The immutable registry is checked against webd and the bundle copies webd's
source into the toolsd destination. Unit hardening, polkit action, credential
split, eight-run/600-second budget and five-minute backstop are retained.
Host endpoint/configuration, address exceptions and secret material are excluded.
