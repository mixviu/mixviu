# Dopus gold parity

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Method: [parity-method.md](parity-method.md). Overall verdict: **OPEN**.
The generic-host migration is a lab specimen, not an accepted replacement.

## Gold inventory

Gold source owns the contract, including the window's overrides of the
headless command dispatcher. In particular `App::serve` handles `dopus.dialog`
before `commands::serve_command`; the latter's UNAVAILABLE reply is not the
window contract.

| Area | Required gold behaviour | Current migration evidence |
| --- | --- | --- |
| Frame | Folder icon, active path followed by ` — MixOS DOpus`, File/View/Go/Help menus, sidebar actions, toolkit tail, status | Shared gold frame mounted; native visual comparison pending |
| Panes | Two locations, active outline, resizable split, summaries, name/size/modified, directory-first sorting | Existing Mix reducer and bulk projections retained |
| Tree | Expansion, collapse, double-click, keyboard navigation, selection, Ctrl/Shift ranges, context menus | Existing Mix/native widgets retained; complete gold witness pending |
| Navigation | Back/forward/parent/home, location edit/submit/cancel, Places and bookmarks, pane target | Existing Mix policy; targeted open now preserves active pane |
| Sidebars | Resizable Places/Properties; lend space below 640px and restore choices | Existing scene policy retained; gold geometry pending |
| Metadata | Names, kind, counts/size, three dates, permissions, owner/group, symlinks, raw path display | Existing Mix projection retained |
| Previews | Local image preview, cache, refusal, toggle, stale completion fence, remote policy | Generic `viu.job.completed` now enters existing reducer |
| Files | Open, mail selection and busy-compose fallback, mkdir, rename, copy/move, delete, drag/drop choice | Existing owned Mix jobs retained; native acceptance pending |
| Dialogs | Pinned prompts, queued confirmations, native Enter/Escape, old Bus dialog envelope | Existing Mix forms retained |
| App furniture | Search, shortcuts, About, macros, native/hosted menu and action placement | Generic macros and frame wiring; full gold restoration pending |
| Persistence | Config migration, poison-pill refusal, atomic write, panel/split/location state, orderly quit | Existing Mix persistence retained; exit fences pending |
| Appearance | All gold themes/styles, window overrides, live settings/keymap, authority recovery | Generic session/theme/keymap events wired; live matrix pending |
| Launch | Canonical identity, service/comp/URL overrides, path arguments and single instance, chooser service | Generic overrides available; legacy profile argument retained; gold path/chooser migration pending |

## Bus inventory

Gold app commands: `dopus.ping`, `dopus.describe`, `dopus.info`,
`dopus.state`, `dopus.dialog`, `dopus.execute`, `dopus.actions`,
`dopus.theme.set`, `dopus.open`, `dopus.show`, `dopus.quit`, plus `HELP`
and `app.describe`. Add the full toolkit drive/window/capture/menu surface,
debug/debug.set/debug.recent, all twelve macro commands and the executed,
titlebar and macro completion topics. The chooser service has its own
contract and lifecycle and remains part of the migration.

Ping/state/info/actions/open/theme/dialog/quit handlers now exist in Mix.
Gold open accepts an empty paths list, reports the number supplied, applies
at most two paths without a target, and requires exactly one with a pane
target. Describe/HELP/activation, complete action replies, topic delivery,
launch and shutdown still need matched native acceptance. No catalogue-only
claim counts as Bus parity.

## Scan work budget

The integration baseline already introduced a default budget of 65,536
enumerated names and depth 64. Every root enumeration, visible directory
child-count enumeration and expanded subtree spends the same remaining work
allowance. Hidden entries also spend work. Symlinks are not followed. The
depth bound is inclusive; expansion beyond it refuses. Caller budgets are
validated (rows 1..65,536; depth 0..64). Exhaustion raises `SCAN_BUDGET`, never
a truncated successful listing. `dp_request` reduces a copied candidate, so
failure in the second pane discards the entire navigation request.

CBC `20261107-112139-11c64519-a1` at `c10e6f725a39` passed both scan tests
and the 2,000-row profile (three tests total). The complete saved log was
read. Debug callback medians were 328.02 ms sort and 327.90 ms refresh;
these are debug measurements, not the release performance gate. Bulk
`stat_many`/`rows_project` and row-cache paths remain in place. A new full
bundle test checks frame mounting and refusal across both panes; its gate
is pending. Release remeasurement and native applied-model witnesses remain
required before acceptance.

## Frame checkpoint

Code `c10e6f725a39` adopts the shared frame. Follow-up `8fd7120` adds
generic event/Bus wiring and the bundle refusal test. No per-app Rust was
added to the runtime; retained comparator sources are not the generic app's
execution path. Private diagnostic captures, scripts and cleanup receipts
live outside this tree. Exact-source release hashes, paired visual figures,
SSIM/region inspection and live Prefs acceptance are pending. No Fable
deviation is proposed at this checkpoint.

## Diagnostic visual pair (not acceptance)

An isolated native Bus/compositor run captured clean pinned gold and the
generic-host specimen sequentially at 1040×680. Gold and candidate use the
same real filesystem fixture. The candidate used the saved integration
executable; later shared-widget changes have not been captured. This run
did not include the required private settingsd matrix, so it cannot attest
Prefs or exact-source release parity. All owned processes were confirmed
stopped. Earlier failed diagnostic runs remain retained.

The whole original, candidate, side-by-side and difference images were
inspected, along with header/Places/status crops. ImageMagick 7.1.2-35 reports
normalised **distortion**, zero identical:

| Region | x,y,w,h | SSIM distortion | RMSE distortion |
| --- | --- | ---: | ---: |
| Full | 0,0,1040,680 | 0.0466528 | 0.0870047 |
| Header | 0,0,1040,38 | 0.00241008 | 0.017971 |
| Places | 0,38,147,612 | 0.0278773 | 0.0657722 |
| Panes | 147,38,677,612 | 0.053698 | 0.0830627 |
| Properties | 824,38,216,612 | 0.0529945 | 0.119515 |
| Status | 0,650,1040,30 | 0.0315301 | 0.0935127 |

Remaining differences: macro icon; Places header/row layout; sidebar widths
after resize; pane outline colour, location/header/footer geometry; metadata
wrapping; unwanted status icon. Reusable navigation-list and text-only status
facilities address two of these; both await the new artifact and comparison.
No difference is accepted or referred to Fable as a proposed deviation.

Gold PNG SHA256: `d7a116de48c3fb15bc95ec9b50bcae64cd0276fd5633993a7ed6ef0162614b14`.
Diagnostic candidate: `f8975ae8055625ccc85a7969b0f4f6125394ab64552af4cacb41f0bfc65afa14`.
The private `parity-dopus/smoke.x7IJs2` archive retains figures, metrics,
queries, logs and the all-stopped receipt.

## Pending gate

Source `d1b655a` includes native navigation-list, optional status icon,
generic reply-fenced quit, Mix-owned describe replies, deduplicated explicit
Bus descriptors and the pinned gold command/action inventories. CBC
`20261107-114033-a67ba1a0` is queued for formatting preparation, focused
tests, all-target checks/Clippy, release 2,000-row measurements and release
host artifact. It must return its formatting patch, followed by a clean
exact-source gate. Superseded queued jobs `20261107-112707-102d6304`,
`20261107-112921-92b8ec74`, `20261107-113446-ec29f120` and
`20261107-113825-38f97644` were explicitly cancelled before execution.
