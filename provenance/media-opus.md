# Opus codec dependencies

`libraries/media/opus` uses registry packages without source modifications:

| Package | Version | Licence | Registry package SHA256 |
|---|---|---|---|
| [opus-pure](https://crates.io/crates/opus-pure/0.2.3) | 0.2.3 | BSD-3-Clause | `5afacd236411dddcc284983066ef66b27e306e53ce4de85cd745b3da9be8a0be` |

The package has no normal or build dependencies. Cargo.lock pins its registry
checksum; the foreign policy records its version and licence. The codec is
explicitly forbidden from the strict shell and service-engine dependency trees.

Upstream opus-pure attributes its codec to rusty-opus 0.9.1, opus-rs and the
Xiph libopus reference implementation. It also attributes the SILK resampler
coefficient tables to libopus 1.6.1. These are Rust implementations/data: the
package does not compile or link the C codec. Package LICENSE and ATTRIBUTION
files remain authoritative. The inherited
copyright, BSD conditions, disclaimer and patent notice are retained in
`opus-codecs.LICENSE`; include the notices when redistributing codec binaries.

Admission evidence: the implementation passed the 12 RFC 8251 vectors at
48 kHz in mono and stereo using the external reference quality comparator,
and generated-signal interop with libopus in both directions. The original
RFC 6716 streams were also measured; updated-reference compliance is the gate
per RFC 8251. The codec remains pinned and is not vendored.

RFC fixture archives and the pinned C comparator are downloaded and SHA256
checked by `tools/opus_test_setup.mix` on the worker. They are external test
data/oracles and are not runtime dependencies or committed vector data. Their
IETF Trust BSD terms are recorded in the crate's README and conformance test.
