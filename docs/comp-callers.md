# comp caller audit

`comp.callers` is a read-only Bus command with no required arguments, roles or
grants. Step 1 observes calls and refuses none on identity grounds. Existing
argument validation and capacity limits still apply.

The recipient prefers the existing verified Unix native-session profile. One
profile ping is added per connect/reconnect, with no identity RPC per call.
TCP remains the default for other services. When the native endpoint is
missing, comp uses TCP and warns that its recipient is unprotected. Native
verification failures are not silently treated as verified connections.

The reply contains `mode: observe`, current `protected_recipient`, saturating
`total` and `legacy` counters, counts by class and known command, and `recent`:
the latest 128 calls. Each recent entry has a sequence number, command, class,
connection generation, recipient protection status and caller. The caller is
either `legacy/no principal` or a summary of broker-attested assurance, uid,
pid, optional session role and registered name. The name remains self-selected
and is not an application authorisation claim. No arguments, screen pixels,
keys, full headers, grants or session identifiers are retained.

The five classes are `read`, `window`, `input`, `capture` and `debug`.
`capture.frame` is `capture`; `debug.set` is `debug`; single input operations
and input sequences are `input`. The catalogue explicitly classifies every
command. Discovery, truth and unknown commands share an additional `other`
audit bucket. Unknown command names share one bounded counter bucket. Topic
fan-out is excluded. The audit query counts itself. Counters survive broker
reconnects while the comp worker lives, and reset when it restarts.

Principals come from the verified delivery, never by trusting incoming header
text. TCP has no principal even if a client supplies a principal header. Every
reconnect negotiates and registers afresh; queued deliveries retain only their
own connection context. No term identity or session allocation is used.

Step 2 adds roles rooted in the desk launcher and time-limited, revocable agent
grants, after independent design review. Screen capture will need a grant.
Step 3 migrates legitimate callers before enforcing role/grant requirements on
privileged classes. Read-only commands stay open. Input must be refused while
locked; that policy is future work, not enforcement claimed by step 1. Remote
comp authority waits for mesh authentication.
