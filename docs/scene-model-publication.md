# Incremental model publication — v1, provisional

Workshop previously rebuilt and published its complete scene model on each dirty
tick. The host then validated and copied that model, including unchanged data.
Whole-model `scenes.model` publication remains supported for every app.

## Normative contract (v1, provisional)

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
requires an existing list with index <= len and remove <= len - index.
An append (index == len, remove == 0) is allowed. `remove` requires an
existing object key; list deletion uses `splice`. An empty batch is refused.
Revisions are positive integers no greater than 2^53−1, exact in Mix.
Once revision 2^53−1 is accepted, patches and revisioned full publications are
refused for that generation. Recovery means a new generation, or continuing
with legacy full publications; a legacy full does not reopen the retired
revision sequence.

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
full model when switching those inputs. A change to independently derived host
layout metrics also refuses a patch and requires full resynchronisation. That
comparison reads only the small host-input projection. Other apps need no
conversion.

`viu_context().incremental_models` advertises owner support. Without it, an app
continues using legacy full publication. A successful local send means queued,
not applied. Every revisioned publication gets exactly one applied or refused
receipt, or the generation ends. Publication here means a send admitted by the
local endpoint; malformed sends are refused synchronously before admission.
The host later sends `viu.model.applied` with `generation` and
`revision`, or `viu.model.refused` with those fields and `message`. Acceptance
means validated model state, not a frame drawn on a display. Workshop keeps
one publication outstanding and coalesces changes until that receipt arrives.
Acceptance immediately flushes any queued input; refusal schedules a full
resynchronisation on the next tick. The owner retains acceptance/refusal
receipts when its evaluator mailbox is full and admits them before ordinary
event producers. The retention bound is non-normative: this implementation
uses the runtime configured mailbox width (currently 32 for the app host).
Saturation pauses all completion retirement, replies included, until receipts
drain; further completions remain in the existing result queue.

At most one revisioned publication may be queued per callback, including a
mixture of full and incremental sends. Conflicting sends fault the callback
and discard its publication. A failed callback publishes no partial model.
A refused batch fences later patches in that generation until an accepted full
publication resynchronises it. Full resynchronisation advances the revision;
it does not reset it within a generation. A new evaluator generation starts a
new revision sequence. Legacy full publication still needs no revision.

## Appendix: implementation, discovery and checks (non-normative)

### Discovery

The disposable `lab/viu-incremental-model` specimen starts at PR #213's merge.
The first capture mixed peer-count phases and is excluded from acceptance
evidence. A fresh 200-peer capture in CBC job `20261121-091529-dcb5cbe3-a1`
reached 2.64 MB, mostly retained Traffic bodies. Top-level replacement still
sent almost all those bytes; nested map edits and list splices replayed the
captured data exactly. The largest patch was 386.5 KB. Native mounted-frame
replay reached 146.9 ms versus 428.5 ms for a full apply. That diagnostic capture
later failed its refetch assertion; its replay establishes only a captured
state comparison, not responsiveness. The independent whole-specimen survey
identified that limit before production derivation.

Instrumentation itself adds cost. The original one-second fixture failed in
the initial instrumented runs. The separate capture specimen uses longer
diagnostic deadlines; those runs cannot establish the responsiveness claim.
Native validation, binding application and rendered equivalence remain required.



### Implementation and checks

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
The template early exit is structural: no declared template roots and no
reads_item binding. It does not infer template absence from widget families.

Workshop retains unchanged action/menu projections and formatted Traffic rows.
Traffic collapse ids follow retained rows rather than list positions. Identical
duplicate rows are matched by occurrence; their identities are interchangeable
when an identical occurrence leaves the window. Filtering and placement changes
continue to show complete redacted bodies.

Workshop also compares presentation section inputs with the inputs captured by
the last acknowledged publication. Unchanged tree, macro and Traffic sections
reuse that model; pending publications do not advance the acknowledged inputs.
Changes accumulated while awaiting acceptance are compared on the next flush.
Mesh ages/animation and agent permission expiry labels continue to project each
turn. The full presentation projection remains an independent callable oracle.
The Mix differ refuses root replacements and changed root keys over 256 bytes;
callers use full publication for those cases.

Traffic notice callbacks perform no recursive redaction. They enqueue raw rows
in private state for one ordered native Job at a time, using the same recursive
redaction function as before. Only a completely redacted result enters the
Traffic ring, so display, filtering and export cannot expose a pending body.
Observation visibility is asynchronous. Pending inputs are capped at 1,024 rows
and 8 MiB including active work; overflow drops the oldest waiting rows, and
failed Jobs drop their row. Both increment the existing visible drop count.
This queue is separate from the existing bounded redacted display ring.

The 200-peer private-Bus fixture keeps its original one-second calls, two-second
owner stall check, silent peer and departing peer. After those checks, a native
owner event retains 48 representative escaped JSON bodies through the real
Traffic retention code, producing a model larger than 2 MiB. Seed publication
is initialisation, outside the per-tick guard. Eight further one-second reads
and ordinary ticks enforce a 768 KiB publication bound and 100 ms native
incremental apply CPU bound against that retained model. Controls force
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
