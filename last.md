# Quoin P2 lab hand-off

Branch: `lab/quoin-p2` (P0 base, pushed; no PR or merge).
Branch head before this hand-off commit: `39e76473b00f12f23f589174623a1d51e17ffb16`.
The hand-off commit changes only this file; resolve the tip with `git rev-parse HEAD`.
Source under final CBC test: `113ae03bf2460c702e4edae318ce0f835bf4f7eb`.

Implemented the four prototype paths: generation-fenced `publish_patch`,
recursive whole-model path diff with atomic host batch, a separate pure `$src`
root with shared Mix clock/comp/audio/net sources, and compositor snapshot
refresh from watch events rather than the successful-connection 1 s poll.
The 50-property conformance test passes: one changed leaf evaluates one
dependent, whole-source publication evaluates 50, unrelated model writes zero.

Mix: 397 loader assertions, 8 path assertions, 6 source-distribution
assertions; lint with deny-warnings passes. No desk4, VT or `/opt` changes.

First panel measurement, debug CBC, actual shipped panel renderer with empty
authored tasks/tray, 10 warmups + 100 samples:

| Path | Re-evaluations | Patch median/p95 µs | CPU frame median/p95 µs |
|---|---:|---:|---:|
| whole model | 31 | 520.127 / 540.418 | 334725.266 / 375178.335 |
| source leaf | 1 | 402.445 / 415.739 | 333926.766 / 341936.713 |

Binding work falls; frame cost does not materially improve. Includes content,
template and image preparation. No GPU/presentation timings, live hotplug or
captured live-workload oracle: desktop access was excluded by the task.

CBC jobs (saved logs read):

- `20261109-155011-d6675eca-a1`: **BUILD_RC=0** at source `113ae03bf246`.
  All 179 executed Rust tests passed; 6 environment/GPU tests ignored.
  Only documentation changed after this tested source. Confirmatory panel:
  whole-model patch median/p95 520.717/551.426 µs, frame
  335143.767/372471.570 µs; source leaf patch 401.067/419.580 µs,
  frame 335261.925/341868.347 µs. Re-evaluations remain 31 → 1.

- `20261109-154110-c5b70fcb-a1`: rc 101, unseeded source fixture.
- `20261109-154238-b4e9c5f2-a1`: rc 101, same fixture before fix.
- `20261109-154427-68167e04-a1`: rc 101 overall; 73 scene and 100 host tests
  passed, 5 host tests ignored. Failed inherited registry owner filter:
  P0 says viu, test expected viud. Corrected in final source.
- `20261109-154623-904f3210-a1`: rc 0, CBC rustfmt diff applied locally;
  worker edits restored in the same job.
- Superseded queued formatting request `20261109-154437-92aa66e3` never ran;
  local client ended rc 143. A separate cancel request could not release
  the submitting client's interest; it was terminated while still queued.

The specimen's detailed shapes, failed approaches and proposed two-PR split
are in `docs/2026-11-09-quoin-p2-lab.md`. Production needs an independent
whole-specimen survey, source defaults/availability at load, coherent snapshot
and sequence semantics, async provider reads, watch release/retry, and one
dirty-set union per batch. No claim of production readiness.

P3: preserve native stable comp keys; define ordered bounded projections for
repeat and `instance.output` for per-output instances. Share one source owner
and one behaviour across those instances. Initial output snapshot and
add/remove/gap semantics must be specified before derivation.

Overlapping P1 hunks: scenes.mix handlers/publication wrapper, loader
model/reload, runtime worker and store receipts. P3: ResolvedScene/source
root/dependency policy, template context, metadata and panel document.
Do not merge the lab branches to resolve these overlaps.
