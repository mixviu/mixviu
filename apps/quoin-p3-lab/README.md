# P3 discovery specimen

Lab branch only; never merge this implementation. This is a small panel
specimen, not a port of the shipped panel's audio, tray, task and popup policy.

`loader.mix` is an opt-in alternative entry point using the existing native
Bus and `shell.scene.load/unload` API. It does not modify the installed loader.
`template.conf.mix` selects `per_output`, `primary` or `named` (with `output`
set to a stable output name). Each page is `NAME@OUTPUT`; the backing scene
name is a deterministic hash because P0 scene names cannot contain `@`.
The output feed is isolated in `instances.mix`. It subscribes before taking
the compositor snapshot, uses P2's watch-as-invalidation contract, and
refreshes the snapshot on notifications/reconnect. Clock updates remain 1 s.

Run with the native Mix interpreter:

```text
mix apps/quoin-p3-lab/check.mix
mix apps/quoin-p3-lab/measure.mix HARNESS PRIVATE-RESULT-DIRECTORY
mix apps/quoin-p3-lab/live.mix HARNESS CBC-BINARY-DIRECTORY PRIVATE-RUN-DIRECTORY
```

HARNESS is the maintainer's native desktop test harness. It creates and stops
an isolated noded; nothing connects to the desktop Bus. The live test uses
only supplied CBC binaries. `cbc.mix` builds and snapshots them on a worker.
It prints formatting/lock changes for local application.

The headless backend's lab-only environment controls are
`MIX_VIUD_LAB_OUTPUTS=1..3` and optional
`MIX_VIUD_LAB_UNPLUG_SECS=1..300` (requires two outputs). The latter removes
the last output once. These are a bounded experiment, not a production
virtual-output management API. No renderer or physical seat is replaced.

Set `P3_SETTINGS` to a private file when running the loader directly.
`P3_SHAPE=shared|instance` chooses one Mix behaviour process or one per output.
`mode {output,mode}` stores mode by output **name** and rejects absent outputs.
The specimen exposes mode in the model; it does not apply panel visibility.

## Repeat and P2

The lab accepts a node with `repeat: {over: '= $src.comp.workspaces', as: ws,
key: id, max: 64}` and `child: template_id`. It lowers this to the existing
keyed list/template machinery. Bounds are 1..128, default 64; exceeding the
cap, duplicate/empty/non-string keys, shared templates or nested repeat/list
refuse the candidate. Variable-token rewriting preserves literal strings.
The host still validates the lowered scene and owns evaluation budgets.

This lowering intentionally supports only dotted model/source collection
paths. It re-lowers on changes, rather than extending core structural dirty
tracking. Existing list row identity/events supply the row key. Arbitrary
pure `over` expressions, keyed widget-state retention across reorder, precise
structural invalidation and diagnostics mapping need a production design.

P2 at `35260eb5` exposes read-only `$src` patches and a watch watermark with
snapshot invalidation. Workspaces are a map with `.list` rows containing
numeric `index`, optional `name`, and output-specific `current` fields.
The Mix adapter projects rows to `{id,index,label}`; `repeat` needs a keyed
list, not that unmodified map. P0 does not understand `$src`/`$instance`, so
the specimen lowers these to reserved model sections. P2 integration must
keep real read-only roots and invalidate both structural and item bindings.
No code from the concurrent P2 branch is copied or edited here.

## Measurement limits

`measure.mix` uses real native Bus calls and real behaviour processes with
equivalent synthetic 1/2/3-output snapshots. Three trials, 20 synchronous
updates each; per-instance processes receive only their output section.
It records summed behaviour VmRSS, spawn-to-first-reply time, successful
request count and JSON request-plus-reply payload bytes. Protocol overhead,
registration/startup retries, loader, compositor, host/GPU memory and frame
latency are excluded. Results cannot establish whole-panel parity or visible
startup latency. Code-size figures cover these tiny behaviours, with policy
shared in `panel-state.mix`; they say little about a full panel rewrite.

`check.mix` covers selection, stable page identity, unchanged-update silence,
bounded keyed repeat, and removal preceding pending model fanout. This is
an adapter test, not evidence of unplug during a live Bus transaction.
The live harness separately captures both outputs and checks one surviving
identity after compositor removal. Inspect its images and receipts before
calling rendered acceptance complete. The host's settings and hidden-page
behaviour remain separate questions for the independent survey.
