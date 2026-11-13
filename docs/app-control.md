# Application control

`<app>.show` asks an application's Mix worker to activate its own native
window through the compositor. Callers must check both the return code and
the reply's `shown` field.

- `rc == 0` and `shown == true`: the window was activated.
- `rc == 0`, `shown == false` and
  `reason == "compositor_unavailable"`: the compositor has not registered.
  The application is running, but activation has not happened. Wait for native
  service registration before asking again. This result schedules no later
  focus and must not be reported as a shown window.
- A nonzero return code reports an activation failure. Retain its diagnostic
  body and show the message to the user.

Ced holds its activation reply for 65 seconds. Its worker can make four
sequential compositor requests, each bounded to 15 seconds; the hold includes
five seconds for completion delivery. A caller wishing to await the complete
result should allow at least 70 seconds. A shorter caller deadline can expire
before ced finishes and does not establish whether activation succeeded.

Ced retains the latest failed job's kind, return code and complete body in
`ced.specimen` under `last_job_error`. Success of the same job kind clears the
visible failure notice, while an unrelated successful background job leaves
it visible. Clearing the notice does not remove the diagnostic record.
