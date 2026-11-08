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

The host also preserves the existing in-process reply contracts: named
`viu-local requests.defer` / `requests.reply` tokens, and
`viu_reply("hold", native_token, {})` / `viu_reply("finish", native_token,
{rc, body, close})` receipts. Their tokens are distinct from `keep` handles;
the host delivers every form on its original native request. Broker origin
headers remain unchanged when a separate opaque caller handle is attached.

`contract.json.host.reply_timeout_secs` selects the ordinary native response
deadline, from 1 to 180 seconds. Its default is 180 seconds, preserving the
generic host and Ced contract. Prefs selects 95 seconds, covering its bounded
90-second named deferrals. Screenshot selects 10 seconds; `keep` explicitly
retains its response past that deadline. Callback execution limits and the
applications' own operation deadlines still apply independently.

Window settings have an independent `retain_window_on_loss` contract
(default `true`). A bundle selecting `false` uses the authority's empty
window map during loss, so automatic decoration fallback remains native.
Appearance palette retention is independent. Screenshot and Workshop select
this policy to preserve their accepted gold behaviour; other accepted window
bundles retain their last window choices. Appearance retention after loss is
separately opt-in for Backdrop. Exact-release native deadline/cancellation
witnesses are in [six-app-integration-acceptance.md](six-app-integration-acceptance.md).
