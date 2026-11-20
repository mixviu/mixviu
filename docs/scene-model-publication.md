# Incremental model publication

Workshop currently rebuilds and publishes its complete scene model on each dirty
tick. The host then validates and copies that model, including unchanged data.
Whole-model `scenes.model` publication remains supported for every app.

## Discovery

The disposable `lab/viu-incremental-model` specimen starts at PR #213's merge.
CBC job `20261121-090653-fae26aac-a1` captured real private-Bus Workshop models
with 200 peers. Late models were about 2.25 MB; the Traffic control held most
of the bytes. Replacing changed top-level keys would still send almost all of
that data. Nested map edits and list splices replayed the captures exactly:
late patches were 5.9–11.7 KB, and representative discovery patches 168–301 KB.
Mix diffing took about 2–3 ms and data replay under 0.5 ms. These are diagnostic
data-replay results, not native rendering or responsiveness acceptance.

Instrumentation itself adds cost. The original one-second fixture failed in
the initial instrumented runs. The separate capture specimen uses longer
diagnostic deadlines; those runs cannot establish the responsiveness claim.
Native validation, binding application and rendered equivalence remain required.

## Contract

`scenes.model.patch` uses the existing local scene endpoint. Its body carries
`generation`, `base`, `revision`, and `operations`. Revisions are monotonic
integers scoped to one hosted evaluator generation. A full publication may
establish a revision; legacy whole publications require no revision and do not
opt into patches. A patch must name the last successfully applied revision and
advance it by one. Rejected batches change neither model nor revision.

Operations use an array of path components: strings for object keys and integers
for list positions. Dots and slashes in keys are literal. There is no text-path
parser, expression evaluation or toolkit-specific field in this contract.

- `set`: replace the addressed value, or add a key to an existing object.
- `remove`: remove an existing object key.
- `splice`: replace a bounded list range (`index`, `remove`) with `items`.

Operations run in order as one transaction. Missing parents, wrong types,
invalid ranges, unknown fields/operations, oversized batches and aggregate
models over the existing 8 MiB budget refuse the whole batch. A later operation
cannot make an invalid earlier operation valid. Model, bindings, controls,
templates and a mounted frame must commit together. Existing control and
expression limits remain in force.

A batch contains 1–512 operations, at most 1 MiB of encoded JSON, and paths
of 1–32 components with keys no longer than 256 UTF-8 bytes. A list splice
requires an existing list and a range wholly inside it. `remove` requires an
existing object key; list deletion uses `splice`. An empty batch is refused.
Revisions are positive integers no greater than 2^53−1, exact in Mix.

Example after a full publication at revision 1:

```mix
send "viu-local" "scenes.model.patch" body={
  generation:$ctx.generation,base:1,revision:2,
  operations:[{op:"splice",path:["rows"],index:0,remove:1,items:[]}]
}
```

The first implementation targets model data in an ordinary root scene, as used
by Workshop. Scene source, host appearance, window topology and projected
multi-surface models continue to use full publication. The app must publish a
full model when switching those inputs. Other apps need no conversion.

`viu_context().incremental_models` advertises owner support. Without it, an app
continues using legacy full publication. A successful local send means queued,
not applied. The host later sends `viu.model.applied` with `generation` and
`revision`, or `viu.model.refused` with those fields and `message`. Acceptance
means validated model state, not a frame drawn on a display. Workshop keeps
one publication outstanding and coalesces changes until that receipt arrives.

At most one revisioned publication may be queued per callback, including a
mixture of full and incremental sends. Conflicting sends fault the callback
and discard its publication. A failed callback publishes no partial model.
A refused batch fences later patches in that generation until an accepted full
publication resynchronises it. Full resynchronisation advances the revision;
it does not reset it within a generation. A new evaluator generation starts a
new revision sequence. Legacy full publication still needs no revision.

## Implementation and checks

The sender retains projected rows and formats only changed rows. Building
the complete projection and then diffing it saves transport bytes but retains
the Mix build cost. Traffic row retention needs particular care: positional
presentation ids change when the bounded list drops its oldest entries.

The receiver retains the model and unaffected ports. Reconstructing a
full candidate, serialising it repeatedly or cloning it for an unchanged frame
would retain the dominant host cost. Changed dependencies must conservatively
include containing list/map reads and whole-model reads.

The native edit transaction moves displaced values into rollback receipts and
tracks exact encoded size from the edited values, including object/list
punctuation. It does not clone or serialise the unchanged aggregate model.
Bindings are compiled once per scene and only affected roots are prepared;
template and control validation still runs before the frame/root commit.
Scenes without legacy list templates do not copy a model for template work.

Workshop retains unchanged action/menu projections and formatted Traffic rows.
Traffic collapse ids follow retained rows rather than list positions. Identical
duplicate rows are matched by occurrence; their identities are interchangeable
when an identical occurrence leaves the window. Filtering and placement changes
continue to show complete redacted bodies.

The 200-peer private-Bus fixture keeps its original one-second calls, two-second
owner stall check, silent peer and departing peer. A bounded steady read phase
also requires a full retained model larger than 1 MiB, above the 768 KiB tick
wire bound. Native incremental apply CPU is bounded at 100 ms. Controls force
full publication, full application and skipped binding updates to prove the
byte/time/equivalence checks detect regressions. Fresh full native scene/frame
and prepared-template state are compared with incremental state in the
equivalence recipe.

Validation recipes live in `libraries/viu/tests/workshop-probe.mix`: `validate`,
`equivalence`, `incremental-negative`, `numeric-negative`, `guard-negative` and
`pool-normal`. The last runs twelve strict width-four whole-crate rounds with
an empty serial list and no retry. Workshop leaves the normal serial list only
after that soak passes, followed by the requested full CI. The independent
whole-specimen survey preceded this derivation; the lab branch is never merged.
