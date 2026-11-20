# Incremental model publication (proposed)

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

## Proposed contract

Add `scenes.model.patch` on the existing local scene endpoint. Its body carries
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

The first implementation targets model data in an ordinary root scene, as used
by Workshop. Scene source, host appearance, window topology and projected
multi-surface models continue to use full publication. The app must publish a
full model when switching those inputs. Other apps need no conversion.

The asynchronous local acknowledgement must distinguish queuing from host
acceptance. A refused batch must fence later patches until the app resynchronises
with a full model; it must not silently advance the base or apply against an
older model. The exact refusal notification is part of the native prototype.

## Derivation requirements

The sender should retain projected rows and format only changed rows. Building
the complete projection and then diffing it saves transport bytes but retains
the Mix build cost. Traffic row retention needs particular care: positional
presentation ids change when the bounded list drops its oldest entries.

The receiver should retain the model and unaffected ports. Reconstructing a
full candidate, serialising it repeatedly or cloning it for an unchanged frame
would retain the dominant host cost. Changed dependencies must conservatively
include containing list/map reads and whole-model reads.

Before landing: a different-family whole-specimen survey; captured-oracle
equivalence; atomic rejection and revision recovery tests; measured byte/apply
bounds with negative controls; all changed-crate targets, fmt, clippy and Mix
lint. Workshop leaves the serial list only after twelve strict width-four
whole-crate rounds pass with no serial entry, followed by the requested full CI.
