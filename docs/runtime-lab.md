# First client-runtime lab

This branch is a discovery specimen. Keep it isolated, survey the whole
specimen and derive production source afterwards. No lab merge or gold install.

`viu-client apps/quoin/volume` starts a native Wayland client. The first slice
uses a safe in-process audio fixture; it never changes the session's audio.
The window says so. Existing `behaviour.mix` remains the unmodified reference;
`viu.mix` adapts only host lifecycle/context and the named fixture boundary.
Volume decisions, write guards, clamping, selection and readback remain Mix.
The fixture implements two outputs, inputs and streams. It is backend test
data, not evidence of production Bus or audio integration.

The evaluator is created on one worker thread. UI input uses a bounded ordered
mailbox with `try_send`; accepted distinct operations are never coalesced.
Full input is refused visibly. A caller may retry a refused operation but must
not pretend it was accepted. A bounded completion mailbox backpressures the
worker. Each completion wakes the client once; idle behaviour has no UI poll.
Output retention is limited to 64 KiB per stream/request, with truncation
marked. Interpreter containers/strings and callback duration are also limited.

A successfully enqueued reload immediately advances the visible generation.
Old results are rejected even if their worker call subsequently completes.
The new script gets a fresh evaluator. Failed loads leave that generation
without callable behaviour; a further successful load recovers. Faulted
callbacks discard any partial UI model. Script state or external effects
already performed by a callback are not rolled back. UI model bindings and
templates are prepared before replacing the last accepted tree/revision.

The client uses the interpreter's existing `BusHandler` send/reply contract
for compatible local `scenes.ready`, `scenes.model` and description dispatch.
Other destinations are passed to an injected native handler. No wire protocol,
broker or general task dispatcher is added. The executable currently injects
the labelled fixture only; a real external Bus adapter remains unverified.

Native egui controls provide rendering and hit testing. Template preflight is
copied from scene-host, including shared template budgets and nested-list
refusal. The client renderer covers volume's row/column/text/button/list/spacer
families. Its geometry/theme differs from the original flex renderer, and it
is not evidence of identical widget maturity or full application parity.

Drop cancels interpreter work, disconnects both mailboxes and joins the worker.
Async sleep cancellation is checked by tests. A synchronous backend builtin
or syscall cannot be interrupted mid-call: its own bound governs the join.
Trusted source is assumed in this lab; this is not an untrusted-code sandbox.

Verification uses CBC only: runtime tests exercise real Mix callbacks,
semantic ordering, delay, queue refusal, stale completion, errors, reload,
drop and idle wakes. The client test sends actual egui pointer events into the
volume widgets, then checks the resulting Mix callback and fixture readback.
Native session capture is separate from these headless widget tests. GPU
submission, model readiness and headless capture do not prove physical display
presentation. Performance and edit-loop targets remain unaccepted until their
named equivalent workloads have measured receipts.
