# PR #226 round 2 — rider stop, 2026-11-22

**STOP: Fable ruling A's path-length-1 rider rejects legitimate publications.**
PR https://github.com/mixviu/mixviu/pull/226 stays draft. Starting head
`56c645700f9074a7d6413890f36bbfdc7066f51e`. No guard implementation,
exemption or differ tuning follows. Opus must take this evidence back to Fable.

The existing corrected-probe CBC batch already contains 20 standalone real
runs with every operation path recorded: `20261121-234716-fee43dae-a1`,
BUILD_RC=0, 20/20 green, source `000b1d10` (same application/differ as this
head). Auditing that raw trace finds rider violations in **all 20 runs**.
This is existing run evidence, not a newly executed round-2 batch. The stop
condition is established before any new CBC submission, so no additional
notice fixture, implementation, controls or acceptance soak was run.

Counts below include every patch callback in the 200-peer discovery and
retained-data steady windows. Startup and deliberate seed initialisation are
excluded. There are **zero root `set`s**, but these length-1 `set`s:

| Path | Discovery | Steady |
|---|---:|---:|
| `["mesh_local_text"]` | 146 | 78 |
| `["mesh_age"]` | 20 | 4 |
| `["mesh_start_visible"]` | 20 | 0 |
| `["mesh_empty"]` | 20 | 0 |
| `["mesh_observing"]` | 20 | 0 |
| `["mesh_inspector_title"]` | 20 | 0 |

These are flattened top-level **scalar fields**, not whole compound sections:
`apps/workshop/mesh-view.mix:44` produces strings for `mesh_local_text`,
`mesh_age`, `mesh_inspector_title`, and booleans for the other three.
The scalar differ's ordinary terminal `set` is correct for these changes.
No >256-operation fallback or depth collapse is needed to trigger the rider.
The ruling explicitly defines the rider by path length, so this report does
not reinterpret “section” to exempt scalars.

Concrete examples from sample 1, after the first full acknowledgement:

- Discovery revision 4/base 3, `workshop.refresh`, 2,217.712 ms:
  `["mesh_local_text"]`, 108-byte `set` inside a 28,822-byte patch.
- Steady revision 11/base 10, `viu.tick`, 9,307.594 ms:
  `["mesh_local_text"]`, 111-byte `set` inside a 2,678-byte patch.

The steady example alone activates the explicit legitimate-run stop rule.
No new Rust check is needed to establish a recorded protocol operation's
path length. The earlier round's green checks are historical, not new
round-2 validation. The kind assertion, byte-telemetry conversion, revised
negative controls and ready status remain **unimplemented/pending** until
Fable refines the rider. The current 768 KiB assertion remains active.

Private evidence: saved log
`20261121-234718-cbc3-000b1d103a66-20261121-234716-fee43dae-a1-ea386d02.log`;
`_var/workshop-byte-guard/r2-rider-audit.mix` reproduces the counts and first
examples from that log; its JSON output is `r2-rider-audit.json` beside it.
Only documentation changed in the public branch for this stop receipt.

Fable's separate monitoring disposition is recorded in TODO-workshop and
the flake ledger: **post-#216 burst growth is not a defect, monitor**.
Main's 781,513-byte legitimate discovery patch versus 480,075 bytes before
#216 (and 896,750 bytes for a forced whole-model resend) defeats a single
size threshold as a discriminator. Bound growth only against a measured
Bus frame limit or a measured publish-latency tail; no new batch cap.

## Round 1 receipt (historical)

# Workshop publication byte guard — PR #226

PR: https://github.com/mixviu/mixviu/pull/226 (draft, author mixviu-agent).
Branch: `fix/workshop-tick-bytes-guard`, base main `5938fb023d726c912c0dc88e806659e84a6fe1a7`.
Validated diagnostic source: `000b1d103a66d223de893a4981e5bfd06b3d92bf`;
formatted source: `cfcb54825fcf03c9c62f8ae1a5b6f7cca6141063`.
The receipt commit is the PR head; no merge or deployment.

**Disposition: neither (i) nor (ii) established for the original failure; stop.**
The requested untouched-main batch passed 20/20. A separate corrected-probe
batch passed 20/20, as did the matching pre-#216 comparison. No ordinary
publication crossed 786,432 bytes in these batches. We therefore cannot say
whether #225's 872,845-byte publication was a justified patch or an oversized
full/section publication. No production fix or guard proposal is selected.
The guard's meaning and value remain unchanged; there is no change to send
to Fable for approval. This does not unblock #220/#225 or establish a fix.

## Measurements

Each batch ran `workshop_answers_during_discovery` exactly once per sample,
20 samples, alone as the gate does. Samples are numbered in each job's log;
failures are retained, never retried. The test's 4- and 200-peer cases remain.
The comparison below uses 200 peers. The pre-#216 test was otherwise identical
to main; its temporary diagnostic source was restored on that worker.

| Batch | Commit | Failures | CBC job (attempt 1) |
|---|---|---:|---|
| Untouched main | `5938fb02` | 0/20 | `20261121-232507-b7bb107c-a1` |
| Main, corrected probe | `000b1d10` | 0/20 | `20261121-234716-fee43dae-a1` |
| Before #216, same corrected probe | `6b4278c7` | 0/20 | `20261121-234718-101836a7-a1` |

Per-publication bytes across the guarded discovery/steady windows, excluding
startup and deliberate retained-model initialisation:

| Version / phase | n | Min | Median | p95 | p99 | Max |
|---|---:|---:|---:|---:|---:|---:|
| Main / discovery | 146 | 6,308 | 97,282 | 630,398 | 736,116 | 781,513 |
| Before #216 / discovery | 161 | 3,620 | 272,860 | 402,911 | 465,240 | 480,075 |
| Main / steady | 80 | 2,208 | 5,912 | 7,542 | 8,637 | 8,637 |
| Before #216 / steady | 80 | 1,543 | 6,798 | 6,798 | 6,799 | 6,799 |

All publications in these windows were patches. Full startup/seed
publications are recorded separately in the same raw logs. The live
`$s.discovery!=nil` phase is recorded too, so terminal discovery updates can
be distinguished from the test's retained-data steady window.

Untouched main's existing per-phase summaries report discovery maxima from
387,529 to 759,849 bytes, and steady maxima from 7,227 to 8,413 bytes.
They do not provide a pooled publication distribution without instrumentation.
After-fix failure rate: **not applicable — no fix was made**. The second
0/20 is a diagnostic batch, not a post-fix claim.

#214 reported roughly 380 KB discovery maximum. Even pre-#216 now reaches
480,075, so that historical sample was not a workload-independent ceiling.
On the same worker, the corrected main probe's maximum is 1.63 times the
pre-#216 maximum; its median is lower. #216 changed the tail/batching shape,
not every publication uniformly.

Main's largest sample is run 19, revision 9/base 8, a 781,513-byte patch,
98 operations, at 7,592.689 ms. It records 107 events/1 tick since the last
acknowledgement and 130 events/1 tick since the preceding publication build,
including work accumulated while pending. Its retained model is 2,142,399
bytes (patch 36.5%). Traffic accounts for 763,885 bytes: 13 inserted rows and
one removal, with no whole-Traffic `set`; tree edits account for 14,229 bytes.
This is a legitimate observed burst, just 4,919 bytes below the guard.
It is not a trace of the original failing publication. Dividing by ticks
would not help this one-tick burst.

Every instrumented publication records revision/base, full/patch, bytes,
operation count and individual operation paths/sizes/insertions/removals,
retained-model size, both coalescence intervals, phase and time from start.
Raw logs retain all records, including startup, seed and 4-peer publications.

## Negative control and checks

Forced `$full=true` in the existing publisher, restored after the run:
`20261121-235128-7431ba86-a1`, driver BUILD_RC=0. The target failed as intended
at `libraries/viu/src/app_host.rs:2976`, with 896,750 bytes against 786,432
and pump 150,808 us. Thus the existing guard still rejects whole-model
publication. A size close to #225's 872,845 bytes can also be a full model;
the original payload type cannot be inferred from size alone.

`20261121-234717-09781181-a1`, BUILD_RC=0: both all-target clippy configurations;
54 test binaries plus doctests per configuration, strict width-eight pools,
650 default / 660 test-support passed, zero failed, 12 existing ignored each.
`app_mix` is green in both pools. Existing `workshop_bus` serial registration
is unchanged; all six support tests passed. CBC rustfmt diff was applied.
`20261122-000221-b7321e34-a1`, BUILD_RC=0: final fmt check and the remaining
`terminal_ime_driver` example target in both configurations, bringing coverage
to every 55 non-doc targets plus doctests per configuration. No new app test,
MIX_* variable, dependency, retry, ignore, serial entry or deadline change.
The 1 s call / 2 s owner-stall limits remain. Mix driver lint: zero diagnostics.

Probe: `libraries/viu/tests/workshop_bus.rs:26`; opt-in
`PROFILE_PUBLICATIONS` is false in ordinary checks. Driver:
`tools/workshop-publication-sample.mix:1`. Unchanged byte guard:
`libraries/viu/src/app_host.rs:2976`; publication/fallback:
`apps/workshop/app.mix:30`; batched redaction: `apps/workshop/traffic.mix:24`.

## Superseded diagnostics and evidence

The initial probe encoded the entire previous Traffic list once per operation.
That was excessive observer overhead. Main `20261121-232943-9bf5b600-a1`
failed 13/20 calls; pre-#216 `20261121-233225-c88af4fb-a1` failed 14/20 calls.
All failures were one-second ping/info deadlines, not the reported byte guard.
These are retained diagnostic failures, not a main failure-rate estimate.
The corrected probe removes old-target encoding and passes both 20-run batches.
Unstarted setup submissions were cancelled at attempt zero, not retried tests:
`232355-1390982e`, `232405-8309e3b2`, `232426-8b28c4d1`,
`232506-1e33fb1c`, `232916-8b9a0b69`, `233242-788504a0` (20261121 prefix).

Private evidence index (never copied into public source):
`_var/workshop-byte-guard/` contains the Mix summariser and JSONL summaries;
saved CBC logs are indexed by the job IDs above. Flake ledger and
`TODO-workshop` retain the original failure and this unresolved disposition.
Next evidence needed is a low-overhead trace of an actually over-bound
ordinary publication, including its full/patch type and operation payloads.
Do not change the unit or raise 768 KiB on the basis of these passing samples.
