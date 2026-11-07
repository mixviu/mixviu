# Integration round 4: detached gate queued

Mark authorised requeueing the rc 143 session-closure cancellation. The replacement full gate is submitted with --detach, job 20261107-125735-43beb330, pinned to cbc2 at runtime source 1804bb98a5ab305111bd35412bf6f3902bc24013. It is queued behind an existing worker job; no duplicate submitted. Full gates and release builds use --detach from this checkpoint onward. Scoped checks use --light.

The full gate includes formatting, clippy, all tests, Monitor policy tests, bulk primitives, four release engines, P2 furniture and Track A. Four-app release-byte suites (including deferred-reply deadlines), the same-worker 2,000-row comparison and final hashes follow its completion. No final release acceptance is claimed yet.

All 22 inherited import-hash mismatches are explained, with files, original/current hashes and first divergence, in docs/ports/import-hash-debt.md. This remains production-derivation provenance debt. The previous interrupted gate and historical artifacts are recorded in docs/ports/desk4-integration.md.

After this close-out, merge Backdrop 250f87d and Workshop 493ae42, reconcile shared facilities, wire Workshop's New macro editor to Ced's shared widget, and repeat all six release suites as required by the follow-on brief. No install or VT work.

Branch lab/gold-parity; runtime head 1804bb98a5ab305111bd35412bf6f3902bc24013 (receipt commit follows); artifacts PENDING, final gates QUEUED; CBC 20261107-125735-43beb330 DETACHED cbc2, previous 20261107-123118-5c7ba675-a1 CANCELLED rc143.
