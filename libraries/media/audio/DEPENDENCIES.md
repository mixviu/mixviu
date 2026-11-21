# Audio dependency provenance

Exact direct pins: ebur128 0.1.11 (MIT), rubato 5.1.0 (MIT OR Apache-2.0),
hound 3.5.1 (Apache-2.0). Cargo.lock records registry checksums. Sources were
inspected on the build worker before implementation. No third-party source
is copied into this crate. Workspace licence remains MIT OR Apache-2.0.

| Newly locked crate | Version | Licence | Purpose / upstream |
| --- | --- | --- | --- |
| ebur128 | 0.1.11 | MIT | [BS.1770 measurement and FIR true peak](https://github.com/sdroege/ebur128) |
| rubato | 5.1.0 | MIT OR Apache-2.0 | [Sinc resampling](https://github.com/HEnquist/rubato) |
| hound | 3.5.1 | Apache-2.0 | [WAV I/O](https://github.com/ruuda/hound) |
| audioadapter | 5.0.0 | MIT OR Apache-2.0 | [rubato buffer traits](https://github.com/HEnquist/audioadapter-rs) |
| audioadapter-buffers | 5.1.0 | MIT OR Apache-2.0 | [rubato buffers](https://github.com/HEnquist/audioadapter-buffers-rs) |
| audioadapter-sample | 5.2.0 | MIT OR Apache-2.0 | [rubato sample conversion](https://github.com/HEnquist/audioadapter-sample-rs) |
| audio-codec-algorithms | 0.8.1 | 0BSD OR Apache-2.0 | [sample conversion support](https://github.com/karip/audio-codec-algorithms) |
| dasp_frame | 0.11.0 | MIT OR Apache-2.0 | [ebur128 frame types](https://github.com/rustaudio/dasp) |
| dasp_sample | 0.11.0 | MIT OR Apache-2.0 | [ebur128 sample types](https://github.com/rustaudio/sample) |
| visibility | 0.1.1 | Zlib OR MIT OR Apache-2.0 | [rubato proc macro](https://github.com/danielhenrymantilla/visibility.rs) |
| windowfunctions | 0.1.1 | MIT | [rubato window functions](https://github.com/HEnquist/windowfunctions-rs) |

These are pure Rust, with no C/assembler build in the enabled closure.
ebur128's C/reference test features and C API are disabled. rubato's default
FFT feature is disabled: the implementation uses its public sinc interpolator.
Transitive adapter crates remain dependencies of rubato even though this
crate does not call its streaming adapters. `dependencies.mix` prints licences
and repositories from the locked worker metadata, then the foreign gate checks
the whole workspace. Strict packages explicitly cannot reach media-audio or
its three direct dependencies.
