# Deferred replies from Mix callbacks

`viu_reply("keep")` retains the current request and returns an opaque handle
bound to this runtime generation, request and caller. Return from the handler
without calling `reply`. The normal callback deadline still applies to active
evaluation; it does not expire the retained request.

From a later callback in the same application, use
`viu_reply("send", handle, rc, body)`. The host replies on the original native
request, rather than on the job, timer or input callback that completed it.
The body is a typed Mix value. The handle is consumed once. Applications own
operation policy, including whether cancellation replies with an error, and
must settle retained requests on every success, failure and cancel path.

```mix
on begin
  $held=viu_reply("keep")
  -- Start in-process work, retaining $held in the operation state.
end
on finished
  viu_reply("send",$held,0,{ready:true})
end
```

Only requests can be retained; notifications cannot. A request cannot be both
retained and immediately answered. Altered caller/request fields, stale or
consumed handles, invalid reply codes, and more than 32 retained requests are
refused. Callback faults discard staged replies and restore the handle ledger
so a later callback can settle them. Reload creates a new ledger and identity.
The host retires handles when its native response receiver closes; transport
loss aborts old native response tasks. The native broker owns caller-side
correlation and drops orphan replies, as it does for gold applications.

There is no second connection, broker or application service. The existing
native endpoint bounds the total number of outstanding replies. Applications
must not use retained replies as an unbounded work queue.
