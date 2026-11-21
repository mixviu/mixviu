# Workshop publication guard — PR #226 round 3

PR: https://github.com/mixviu/mixviu/pull/226, branch
`fix/workshop-tick-bytes-guard`, author mixviu-agent. Fable ruling A and its
amended container-only rider (2026-11-22) are implemented. Final source
`5b5f78e993af7f838d2253e123eba6104387e1ac`; the receipt commit follows.
No merge, installation or deployment.

## Final behaviour

The test observes actual `scenes.model` / `scenes.model.patch` commands at the
native local endpoint, before decoding. After the initial full acknowledgement,
another full requires a recorded owner refusal and consumes its single recovery
permission. A patch consumes that permission too, so it cannot be saved for a
later full. Normal 200-peer runs require zero refusals.

Root sets and whole top-level container sets are rejected. Top-level scalar
sets remain allowed. A rider failure prints path, value type, encoded value
bytes and the fallback inferred from before/after values: type change,
over-256-operation list fallback, long-key parent, depth-8 collapse or unknown.
The list diagnostic recomputes the differ's count with only the affected
list's cap lifted; child limits and depth remain represented. At a length-1
path the message explicitly says depth-8 collapse is unreachable. No operation
field was added; `Patch::decode` and its unknown-field rejection are unchanged.

768 KiB is telemetry, not an assertion. A test failure prints the largest
publication's protocol kind, operation count, total JSON bytes and section
bytes. For patches, section totals group encoded operations by their root key;
envelope overhead remains in total bytes. The 800 ms pump, 100 ms steady apply
CPU, one-second call and two-second owner-stall assertions remain unchanged.

The retained-model fixture still creates 48 representative rows and a model
over 2 MiB. It seeds eight rows per acknowledged patch instead of deliberately
forcing a second full. This is fixture preparation, not a production batch cap.
It waits for each batch's actual final row, so ordinary observation rows cannot
falsely satisfy its wait. The original five-second total seed deadline remains.
No production Workshop script, differ limit, retry, ignore, serial entry,
dependency or MIX_* variable changed. No new app `*-test.mix` was added.

## Validation

- Final seed-identity fixture: **20/20 complete binaries green, 160 tests**, CBC
  `20261122-004550-a90bc0b5-a1`, source `fd1d33a3`. Later changes only format
  that check and simplify an equivalent failure-print condition. The 200-peer
  cases recorded 20 initial fulls, 380 patches and zero refusals; all 20 notice
  fixtures passed. Discovery max 781,890 bytes; steady max 8,633. Guarded pump
  max 135,577 us and apply CPU max 73,936 us. Largest publication including
  seed preparation: 973,583-byte patch, 15 operations (Traffic 973,437 bytes,
  mesh canvas 76). No amended-rider stop; scalar updates remain allowed.
- Earlier complete soak, `20261122-003054-da1742fb-a1`, source `02482d2d`:
  20/20 binaries green, 160 tests, including 20 notice fixtures and 20 recovery
  fixtures. The 200-peer cases observed 20 initial fulls, 381 patches and zero
  refusals. No amended-rider violation. Largest publication: 837,326-byte patch
  in seed preparation, revision 12, 14 operations; Traffic 837,181 bytes and
  mesh canvas 76 bytes. Discovery maximum 783,120 bytes; steady maximum 575,012
  (the stronger final-row wait removes preparation/steady overlap).
- Scoped CBC `20261122-003056-acc1a028-a1`, source `02482d2d`: all 55 non-doc
  targets plus doctests in both configurations; 650 default / 663 test-support
  passed, zero failed, 12 existing ignores each. Both strict eight-wide pools
  include green `app_mix`. Fmt and both all-target clippy configurations pass.
- Final source fmt, both all-target clippy configurations and all exact
  controls pass in `20261122-004708-b1178927-a1` on `5b5f78e9`.
- Env registry: 277 names, zero problems. Workspace Mix lint ratchet: zero
  regressions; both changed Mix recipes lint with zero diagnostics.

Exact controls (all sources restored by `finally`):

1. Forced full after first acknowledgement fails at revision 2 with
   `Workshop publication kind: expected=scenes.model.patch actual=scenes.model`.
   It fails on kind even at approximately 24 KB, not on a size threshold.
2. Forced top-level map set fails with
   `Workshop container rider: path=["values"] value_type=map encoded_bytes=7504 fallback=unknown (depth-8 collapse unreachable at path length 1)`.
   The driver requires the path/type, numeric bytes, fallback and largest-
   publication diagnostic fields.
3. A real injected owner refusal permits exactly one recovery full in the
   positive fixture. A second full fails at revision 4 with the kind assertion;
   the diagnostic retains two permitted fulls and one refusal.
4. Forced top-level scalar set passes the complete discovery/steady test.

Initial exact controls also passed in `20261122-003053-508b0401-a1`. The pure
fallback/scalar diagnostic unit and all eight Bus tests passed focused CBC
`20261122-002624-a9a3e79d-a1`.

## Evidence and monitoring

Native command recorder: `libraries/viu/src/runtime.rs:139`; test policy and
fallback inference: `libraries/viu/src/app_host_publication_audit.rs:18`,
`:82`, `:116`, `:132`; failure print: `libraries/viu/src/app_host.rs:2955`.
Real refusal and notice fixtures: `libraries/viu/tests/workshop_bus.rs:65`,
`:107`; guards attach before the first owner pump. CBC recipes:
`tools/workshop-publication-check.mix`. Superseded bound and ruling:
`docs/scene-model-publication.md:165`.

Post-#216 growth is **not a defect, monitor**, per Fable: round 1 measured
781,513-byte legitimate discovery patches on main versus 480,075 before #216;
a forced whole resend was only 896,750 bytes. Bound growth only on a measured
Bus frame limit or publish-latency tail; no new production batch cap. The flake
ledger and TODO-workshop record the ruling, resolved scalar rider stop and
validation. The original old-threshold failure's exact payload was not
recovered; this PR replaces that wrong-unit test policy under Fable's ruling.

All Rust ran on CBC. Saved logs are indexed by these job IDs; private evidence
under `_var/workshop-byte-guard/` retains both soak summaries and earlier round
receipts. No full-workspace CI was run; the merge gate owns that run.

The first implementation check `20261122-001725-d4e8e07e-a1` failed two new
fixture assertions (expected refusal temporarily rejected, seed missing a
Traffic dirty mark); both were fixed in this PR before the green validations.
The first green soak was repeated after strengthening the seed wait; it was
not a retry of a failed test. Superseded unstarted submissions were cancelled
at attempt zero, not counted as test results. No pre-existing normal-shape
failure was bypassed.
