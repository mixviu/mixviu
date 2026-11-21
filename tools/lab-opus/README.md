# Throwaway Opus spike

Lab only. Do not merge this directory or its branch into the product.

The harness probes pure Rust packet codecs against RFC vectors and an external
`opus_compare`, and uses generated PCM and external ffmpeg/libopus for interop.
It is not an Opus API, muxer or benchmark engine for applications.

Run the Mix drivers through the native build service. `run.mix` downloads the
candidate metadata and vector archives, records hashes and dependency trees,
builds the adapters, and compiles the standalone C comparator as an oracle.
`measure.mix` covers the first four candidates; `measure-more.mix` covers the
remaining supported candidates and repeats serial interop measurements for all
of them. `conformance.mix` bounds parallel vector comparisons to four processes.

The runner's exit code means that orchestration finished. Individual JSON logs
contain the actual decoder and comparison outcomes. A decode error, a failing
comparison and a missing measurement are different results.

Limits: generated signals cover ten seconds each; the Ogg wrapper is lab code,
uses a fixed pre-skip, and does not flush the encoder tail. Quality comparisons
align delay and use common samples. Production code must be derived separately
after the spike gate, including correct lookahead, frame and loss contracts.
