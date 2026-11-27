# Guarded mail submission and inbound routing

The `maild.accounts` properties `allowed_recipients` and `send_rate` apply
to JMAP EmailSubmission, native Bus submission and SMTP-AUTH submission.
Set them through `maild.props.set` with namespace `accounts`, the account
email as key, `merge: patch` and the current `if_version`. Both guards are
preserved when omitted from a full Replace (including password rotations).
Clear a guard explicitly with null to restore its unrestricted default. An empty
recipient list still denies all recipients, and a zero rate still denies sends.

- `allowed_recipients: null` (or unset) permits any envelope recipient.
  A list contains anchored, ASCII case-insensitive globs: `*` matches any
  sequence and `?` one byte. An empty list permits none.
- `send_rate: null` (or unset) is unlimited. A non-negative integer caps
  messages admitted in a rolling hour; zero permits none.

For example, patch an enclave account with
`{"allowed_recipients":["mc@example.com"],"send_rate":20}`.
All envelope recipients must pass before any local delivery or queue write.
JMAP's automatic envelope includes To, Cc and Bcc; delivery copies omit the
Bcc and Resent-Bcc headers. An explicit JMAP envelope supplies its complete
rcptTo list to admission. SMTP submission retains its existing header handling.

Rate reservations are atomic and persisted in SQLite, shared by all submission
paths. SMTP/JMAP submissions cost one message slot even with multiple recipients;
separately queued remote Sieve redirects cost one slot per target. The whole
redirect fan-out is reserved atomically: either every message fits or none is sent.
Vacation and account-authored rejection DSNs each cost one slot.
A rejected recipient list consumes no reservation. A later delivery
failure or client resubmission does consume one; reservations are not refunded
after crashes. Queue retries continue the admitted submission without another
reservation. Entries expire exactly 3600 seconds after admission; a backwards
clock retains future reservations. A forward wall-clock jump can expire them early.
Unlimited accounts also reserve, preserving the window if a cap is enabled later.

Recipient refusals return SMTP `550 5.7.1 recipientNotAllowed` or JMAP
`notCreated` with type `forbiddenToSend`. Rate refusals return SMTP
`451 4.7.0 sendRateExceeded` or JMAP type `tooManyRequests`.
Storage failures refuse admission (JMAP `serverFail` for internal/storage errors).

Account-authored vacation (Sieve and legacy), rejection DSNs and operator-enabled
remote Sieve redirects use the same account recipient/rate admission. Denied
automatic replies are dropped and logged; denied redirect plans keep the original
message. External redirect configuration never overrides account guards. Aliases
remain local-only. Server-generated transport failure reports are separate.

The inbound Mix filter also receives:

| Variable | Value |
| --- | --- |
| `$AUTH` | `local-auth` or `external`, from the SMTP session |
| `$AUTH_USER` | Authenticated account email; empty for external mail |
| `$SPF` | pass, fail, softfail, neutral, none, temperror or permerror |
| `$DKIM` | Computed aggregate: pass, fail, none, temperror or permerror |
| `$DMARC` | Computed pass, fail or none; known computation defect can report none instead of fail, so it is not a reliable guard until fixed |
| `$REMOTE_IP` | Socket peer address |

These facts are never taken from sender-supplied Authentication-Results.
Local authentication is independent of SPF/DKIM/DMARC results.
For example:

```mix
if $AUTH == "local-auth" and $AUTH_USER == "mc@example.com" then
    return "Inbox"
end
return "junk-if-missing:Quarantine"
```

A plain mailbox name preserves the existing Inbox fallback. The explicit
`junk-if-missing:` prefix instead routes to the account's Junk role when
the named mailbox is absent. If both are absent, that recipient cannot be
delivered. It never falls through to Inbox. The same policy applies to SMTP
Sieve local redirects. Filter execution errors retain the existing behaviour.
JMAP local delivery still runs recipient Sieve rather than the global filter.

Unauthenticated SMTP mail is refused with `550 5.7.1 ownDomainSpfFail`
when the failing SPF identity belongs to the same strict own domain: MAIL FROM
for a nonempty reverse path, or HELO for an empty one. The verifier's failed
domain must match that identity. This is exact-domain only; subdomains do not
inherit the policy. Header From alignment is DMARC work and is not checked by G3.
Authenticated submissions are exempt. Other domains and SPF results retain
existing behaviour. All-token sessions preserve their immediate 250 unless a
DNS-free policy lookup identifies an opted-in own strict domain. Only those
sender identities undergo verification before the 250; recipient token validity
does not influence this choice.

SPF and REMOTE_IP assume direct socket exposure. There is no PROXY/XCLIENT
support; putting a proxy in front would change the peer used for SPF.
Missing Junk during fail-closed routing causes recipient delivery failure.
On the post-250 token path this can silently lose external mail; the transport
failure reporter only reports to local senders, not remote senders. Keep and monitor
quarantine/Junk provisioning. Synchronous failures may instead return 451 if no
recipient was accepted.

Strict SPF refusal requires a published hard-fail policy such as `-all`; SoftFail
and None do not reject. Check direct socket exposure and SPF records when enabling
strict policy. The `junk-if-missing:` prefix is case-sensitive and reserved, so a
literal mailbox name beginning with that prefix cannot be selected literally.
The recipient glob `?` matches one byte, including in UTF-8 local parts.

Rate limits count messages rather than recipient instances; duplicate RCPTs still
cost one slot for a submission. Queue retries, including messages queued before
guard deployment, are not re-admitted after a policy tightening. Operators must
inspect/cancel already queued messages separately during incident response.

Remaining work: wider SPF enforcement and DMARC policy enforcement, including
the existing DMARC result computation issue. Relay and the other doorway gaps
are separate work.
