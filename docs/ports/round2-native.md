# Round 2 shared native changes

Screenshot and Prefs are Mix bundles under the existing generic `viu run`
host. Each adds **zero per-app Rust lines**, no Cargo package and no dependency.
`libraries/mix` remains the embedded 0.117 interpreter; P2 owns unification.
Dopus and the original application sources are unchanged.

Shared production Rust diff against round-1 reporting head 11945c0: **90 lines
added, 10 removed**, excluding the 91-line integration test. This counts
changed lines, not a newly imported engine or application host.

| Shared file | Change and reuse rationale |
|---|---|
| app_host.rs | Wire existing bounded Jobs into every bundle, deliver tagged completions through the root evaluator, and admit timer ticks during request traffic. Any app can perform IO without blocking its UI handler. |
| jobs.rs, runtime.rs | Dedicated owned jobs have a 90-second deadline; root callbacks retain 20 seconds. Human selection and slow native IO use the existing evaluator facility, without a second broker or service. |
| native_bus.rs | Honour explicit 0.1–90-second native command deadlines; omit timeout from wire headers. Reusable for interactive native commands, with the existing supervised connection and bounded owner queues. |
| native_primitives.rs | `viu_image("info", source)` and `viu_image("export", source, destination)` reuse the bounded decoder and exclusive PNG writer. Applications own filenames, folders, documents and save/copy choices. |
| image_view.rs, capture.rs | Expose the existing decoder/encoder inside libraries/viu; their bounds and exclusive create-new behaviour remain intact. |

Decoder limits remain regular files, PNG/JPEG/WebP formats, 32 MiB encoded,
8192 pixels per side, 128 MiB decode allocation and sixteen megapixels.
Export requires an absolute destination, creates mode 0600, refuses overwrite
and removes an incomplete newly created file on encoding failure.

Clipboard needs no new Rust. Mix requests blobd's existing path ingestion,
passes its reference to clipd's clipboard.set and releases its temporary pin.
The native private-harness check reads image/png back from the compositor's
clipboard and compares its bytes with the captured image.

The app inventories record the remaining policy/composition work. In
particular, the host contains no settings registry, transactional edit policy,
capture target selection, minimise/restore policy, save naming, annotation
state or region overlay implementation. Term's future primitive requirements
are listed separately in term-needs.md; no Term implementation was started.
