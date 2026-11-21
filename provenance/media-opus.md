# Opus codec dependencies

`libraries/media/opus` uses registry packages without source modifications:

| Package | Version | Licence | Registry package SHA256 |
|---|---|---|---|
| [opus-pure](https://crates.io/crates/opus-pure/0.2.3) | 0.2.3 | BSD-3-Clause | `5afacd236411dddcc284983066ef66b27e306e53ce4de85cd745b3da9be8a0be` |
| [cranpose-opus-pure](https://crates.io/crates/cranpose-opus-pure/0.2.2) | 0.2.2 | BSD-3-Clause | `523f02377ea3cd16781316c395b04b07b70a021063eba41b0ac46255e007c005` |

The second package is optional and selected only by `bench-cranpose`. Both
packages have no normal or build dependencies. Cargo.lock pins their registry
checksums; the foreign policy records the versions and licences.

Upstream opus-pure attributes its codec to rusty-opus 0.9.1, opus-rs and the
Xiph libopus reference implementation. It also attributes the SILK resampler
coefficient tables to libopus 1.6.1. These are Rust implementations/data: neither
package compiles or links the C codec. Cranpose is a patched fork of opus-pure
0.2.2. Package LICENSE and ATTRIBUTION files remain authoritative. The inherited
copyright, BSD conditions, disclaimer and patent notice are retained in
`opus-codecs.LICENSE`; include the notices when redistributing codec binaries.

Admission evidence: each implementation passed the 12 RFC 8251 vectors at
48 kHz in mono and stereo using the external reference quality comparator,
and generated-signal interop with libopus in both directions. The original
RFC 6716 streams were also measured; updated-reference compliance is the gate
per RFC 8251. The packages gave identical quality results in the spike.

RFC fixture archives and the pinned C comparator are downloaded and SHA256
checked by `tools/opus_test_setup.mix` on the worker. They are external test
data/oracles and are not runtime dependencies or committed vector data. Their
IETF Trust BSD terms are recorded in the crate's README and conformance test.
