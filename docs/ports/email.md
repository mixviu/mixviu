# Email inventory and round-5 list experiment

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, `apps/email`:
actions/commands/state/shell, account/mail/compose/calendar/contacts views,
reader/documents/drafts/attachments/writer and embedded JMAP client.
Email core remains open. The round-5 budget checkpoint adds a fixture-only
list/draft experiment, not a mail client or viability step-2 acceptance.
`viu run apps/email --fixture` admits an explicit synthetic message snapshot;
ordinary launch refuses snapshot injection. The status always says fixture-only.
The table uses the existing Viu virtual tree-table, with 2000-row/4 MiB input
bounds, unique IDs and bounded text cells. A per-row Mix loop and the existing
lib-mix `jq` bulk primitive project identical rows. No per-app Rust or shared
production Rust is added; no interpreter/dependency change.

Selection and an in-memory native draft editor are exercised; accounts and
mailboxes are synthetic labels. No real account/mailbox list, message fetch,
send, durable draft or real maild acceptance exists yet. Those remain next
work, not optional feature deferrals. Existing richer-feature deferrals below
remain explicit. The saved original snapshot is a layout reference with
different fixture content, not a same-data original-run comparison.

| Original capability | Disposition |
|---|---|
| Accounts, enabled/default selection, settings.apply fencing, secrets/CA and reconnect | Deferred until Email implementation; explicit scoped auth ownership without secrets in info replies |
| JMAP discovery, same-origin endpoints, token exchange, bounded HTTP/blob upload | Deferred shared native upstream wiring; embedded Mix has jmap/jmap_upload but Viu registers no JmapHandler |
| Mailbox roles/counts/hierarchy and selection | Next core target; not implemented |
| Paged message list, total, sort/query/search and stale-response fencing | Next core target; not implemented |
| Open headers/plain text body | Next core target; not implemented |
| Compose To/subject/body and explicit send/outcome/uncertainty | Next core target; not implemented |
| MIME upload, Email/set, submission, Sent filing and partial-failure receipt | Deferred core implementation; recipe policy in Mix, reusable byte/upload primitives in Viu |
| Separate composers, identities/signatures, Cc/Bcc/Reply-To, suggestions and reply/all/forward/quoting | Deferred richer compose after core |
| Durable drafts/debounce/restore, dirty-close/quit and mail documents with Ced | Deferred; durable atomic ownership and close gates together |
| Attachments/chooser, inspection/limits, inline preview/download/open | Deferred shared byte/image/chooser contracts and lifecycle |
| Markdown/HTML/source/plain reader, links and print | Deferred reusable rendering controls with policy in Mix |
| Read/unread/star/junk, move/delete/archive, labels/bulk selection | Deferred core follow-on; authoritative mutation receipts/paging |
| Mailbox CRUD and account preferences integration | Deferred core follow-on |
| Calendar/event and address-book/contact CRUD | Deferred separate PIM surfaces after mail core |
| Actions/menu/shortcuts/macros, theme/settings-follow, saved layouts, routing/full email.v1 | Deferred shared host and application integration |

The original explicitly uses JMAP HTTPS for data; its Bus carries application
automation/settings. `maild.submit` does not replace Mailbox/get, Email/query
or Email/get. Survey the unchanged maild personality wire with that distinction
intact. Do not use the old Email GUI as the new app's backend, read maild's
database directly or introduce a parallel transport. Next: wire the existing
JMAP seam to a reusable upstream and verify list/read/compose with an isolated
real mail service. No embedded interpreter update is proposed.

## List budget checkpoint results

Validated source `7c55df5d2de07b87e0ff6032fb9fade36b6a55b1`, CBC
`20261106-224820-566c0878-a1`: 112 tests, fmt/check/clippy/build passed; full
saved log read, empty worker diff. The initial adaptation job caught missing
native table flags, fixed in both paths; no test weakened. Existing warnings
remain. Zero per-app or shared production Rust added; one shared measurement
test added. Native binary hash matches the CBC artifact.

| 2000 messages, median ms | Mix loop | Existing jq bulk |
|---|---:|---:|
| CBC embedded callback wall time | 387.34 | 451.79 |
| CBC runtime mailbox acknowledgement | 391.30 | 455.80 |
| Fresh SceneView admission | 53.92 | 54.68 |
| Cold egui context first draw | 65.71 | 65.46 |
| Separate native full-snapshot Bus ACK | 562.13 | 636.96 |

CBC method: debug build, actual embedded Mix 0.117, 2000 synthetic JMAP-shaped
records, 3 warmups then 21 measured iterations per path, loop then bulk.
Every projected row is compared for equality. Timed callback includes input
decoding, validation, projection, copying and publication. Adapter and draw
are separate timings with fresh view/context each iteration, without GPU or
physical presentation; do not sum medians or call these steady-state frames.
Native method uses a separate private headless/nested run with 21 warm samples
per path, full JSON snapshot transfer through generic-host reply. The native
cohort predates only startup-guard/format/report changes. The final-source
fixture run also passed selection/native draft input; four owned processes
stopped and two captures inspected. The pinned original snapshot is a layout
reference, not an equivalent-content or timing baseline.

Bulk jq is slower in both cohorts. These numbers show substantial full-list
cost outside Dopus but do not attribute the same bottleneck. Dopus's retained
2000-row sort-size ACK medians (14.99/34.50/237.40 ms original/Viu Rust/Mix)
use a different operation and release build; there is no valid speed ratio.
The private Dopus results now carry this separate cohort beside those numbers.

Next required core work: account/settings/auth, native JMAP session and request
wiring, mailbox/query/get, explicit compose/send with uncertain outcomes, then
real isolated mail-service verification. Existing lib-mix HTTP builtins are
feature-gated and not enabled by Viu here; the JMAP seam has no upstream
handler. That is remaining implementation, not evidence that Mix cannot do it.
Do not install this fixture bundle as a replacement Email client.
