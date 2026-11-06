# Email inventory, core port and list measurement

## Round 6 measured list rerun

Step 1 is measured, not merely prepared: CBC
`20261106-225448-9bf578e7-a1` at `4988d293933406931fd2bb2904ddc7a8c6f3e2ef`
reran the actual embedded interpreter/scene experiment. Full saved log read;
worker diff empty. Debug build, 2000 identical JMAP-shaped messages, three
warmups then 21 retained samples per path, ordered loop then jq; nearest-rank
p95/p99 (with 21 samples p99 is the maximum). All rows asserted equivalent.

| Milliseconds median / p95 / p99 | Per-row Mix | lib-mix jq bulk |
|---|---:|---:|
| Callback wall time | 387.95 / 393.06 / 400.93 | 450.90 / 454.90 / 455.37 |
| Runtime mailbox ACK | 391.93 / 396.76 / 404.86 | 454.88 / 458.86 / 459.13 |
| Fresh adapter admission | 53.82 / 54.76 / 54.82 | 53.12 / 54.47 / 55.04 |
| Cold context draw | 65.52 / 67.94 / 68.94 | 65.68 / 67.18 / 67.34 |
| Prior native snapshot ACK, raw samples rechecked | 562.13 / 566.52 / 568.02 | 636.96 / 643.48 / 644.36 |

Dopus's recorded 2000-row sort-size ACK medians are original 14.99 ms,
Viu Rust 34.50 ms, Viu Mix 237.40 ms. Email likewise has substantial
row-scaled behaviour/publication cost independent of Dopus's file policy;
existing jq bulk is slower and does not remove it. This supports the shared
runtime/value-copy/publication concern, but does not isolate the same cause:
Email replaces a full input snapshot, Dopus sorts retained rows, and the
Dopus reference cohort is release rather than debug. No direct speed ratio
or performance gate pass is claimed. Adapter/draw stages are separate fresh
objects, not live frame latency; callback includes validation/copy/publication.
Real-mail verification below is separate from these synthetic timings.

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, `apps/email`:
actions/commands/state/shell, account/mail/compose/calendar/contacts views,
reader/documents/drafts/attachments/writer and embedded JMAP client.
Round 6 ports the bounded account/mailbox/list/read/compose core. Run
`viu run apps/email /absolute/path/accounts.json`; the bundle README documents
the explicit account catalogue and native submission route. Mix owns account,
JMAP, paging, reader, draft and receipt policy. Fixture mode remains separate
and cannot contact a server; ordinary launch refuses fixture injection.

| Original capability | Disposition |
|---|---|
| Accounts, enabled/default selection, settings.apply fencing, secrets/CA and reconnect | Ported explicit catalogue (up to 32), account selection, password-file/optional CA and explicit reconnect. Deferred settingsd registry/migration, enable/default preferences and token exchange: catalogue is the bounded authority |
| JMAP discovery, same-origin endpoints, token exchange, bounded HTTP/blob upload | Ported Basic-auth discovery and same-origin maild /jmap via shared bounded HTTP. Deferred token flow, arbitrary advertised paths and binary/blob upload until their lifecycle contracts are ported |
| Mailbox roles/counts/hierarchy and selection | Ported Mailbox/get, names/roles/total/unread and selection (up to 256). Deferred hierarchy/tree composition; flat labelled choices retain unique IDs |
| Paged message list, total, sort/query/search and stale-response fencing | Ported 200-row pages, total, receivedAt descending, explicit refresh/previous/next, busy/generation ownership and exact query/get ID matching/order. Deferred custom sort/search, live push and paginated cross-page selection until authoritative update policy |
| Open headers/plain text body | Ported listed-ID Email/get and full bounded text (64 KiB); truncated/invalid parts refused. UI excerpt 600 characters with notice, full text via email.message. Deferred rich full-body reader layout/scrolling |
| Compose To/subject/body and explicit send/outcome/uncertainty | Ported native in-memory editor, 1..32 bare ASCII recipients, duplicate/header validation, explicit configured Bus route, accepted receipt, same-ID manual retry and held draft. No automatic retry. Deferred full RFC address parser and durable restart reconciliation |
| MIME upload, Email/set, submission, Sent filing and partial-failure receipt | Ported existing maild.submit pipeline/receipt; maild owns MIME/Sent/delivery. Deferred client attachment uploads and direct JMAP mutation recipe; no duplicate submission implementation |
| Separate composers, identities/signatures, Cc/Bcc/Reply-To, suggestions and reply/all/forward/quoting | Deferred richer compose after core |
| Durable drafts/debounce/restore, dirty-close/quit and mail documents with Ced | Deferred; durable atomic ownership and close gates together |
| Attachments/chooser, inspection/limits, inline preview/download/open | Deferred shared byte/image/chooser contracts and lifecycle |
| Markdown/HTML/source/plain reader, links and print | Deferred reusable rendering controls with policy in Mix |
| Read/unread/star/junk, move/delete/archive, labels/bulk selection | Deferred core follow-on; authoritative mutation receipts/paging |
| Mailbox CRUD and account preferences integration | Deferred core follow-on |
| Calendar/event and address-book/contact CRUD | Deferred separate PIM surfaces after mail core |
| Actions/menu/shortcuts/macros, theme/settings-follow, saved layouts, routing/full email.v1 | Deferred shared host and application integration |

The original uses JMAP HTTPS for reads; maild's existing Bus submission command
owns sends. Both wires are preserved. The app neither drives the old GUI as a
backend nor reads maild's database. No embedded interpreter change: Mix 0.117
remains. This is a functional bounded specimen, not full-feature/visual parity
or approval to install it.

## Shared native addition and real-mail witness

The reusable `viu_http` text primitive in `libraries/viu` supplies GET/POST,
Basic headers, optional CA trust, HTTPS (loopback HTTP for tests), 1..30-second
deadlines, 1 MiB request/4 MiB UTF-8 response budgets and explicit HTTP status.
It disables redirects and ambient proxies; failures exclude credentials and
request bodies. Authentication/protocol/retry decisions remain in Mix.
The shared worker now enables Tokio I/O as well as timers. No app-specific
Rust or JMAP handler was added. Existing workspace reqwest 0.12.28 with
json/rustls-tls is reused; only the viu lockfile dependency edge changes,
with no new package/version/C build dependency. Embedded lib-mix is untouched.

Initial private nested/headless witness at `248958af0d3551dfd3b27c647127e1fff444975e`
passed against actual pinned maild: CLI-created temporary account, seeded local
message via maild.submit, authenticated session/account/mailbox catalogue,
list/read, native compose/Send and JMAP readback of exactly one delivered mail.
Wrong credentials were refused and explicit reconnect recovered. Seven PNGs
inspected including the pinned original reading the same message/server.
Subject/body Unicode agrees; composition/layout is deliberately simpler.
All six owned processes stopped. Final-source receipts follow below.

An initial embedded call exposed disabled Tokio I/O; fixed in the shared
worker with an actual embedded socket regression. Shared tests also cover
off-origin session refusal, wrong method/call identity, incomplete text,
query/get ID mismatch and query-order preservation. No test was weakened.

## Historical round-5 fixture checkpoint

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

At that checkpoint the mail connection and core were still absent. Round 6
implements the core above using generic native HTTP, rather than enabling the
feature-gated lib-mix HTTP/JMAP handlers or changing the embedded interpreter.
