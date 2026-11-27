# Guarded mail submission and inbound routing

The `maild.accounts` properties `allowed_recipients` and `send_rate` apply
to JMAP EmailSubmission, native Bus submission and SMTP-AUTH submission.
Set them through `maild.props.set` with namespace `accounts`, the account
email as key, `merge: patch` and the current `if_version`.

- `allowed_recipients: null` (or unset) permits any envelope recipient.
  A list contains anchored, ASCII case-insensitive globs: `*` matches any
  sequence and `?` one byte. An empty list permits none.
- `send_rate: null` (or unset) is unlimited. A non-negative integer caps
  messages admitted in a rolling hour; zero permits none.

For example, patch an enclave account with
`{"allowed_recipients":["mc@example.com"],"send_rate":20}`.
All envelope recipients must pass before any local delivery or queue write.
JMAP's automatic envelope includes To, Cc and Bcc; delivery copies omit the
Bcc header. An explicit JMAP envelope supplies the delivery recipients.

Rate reservations are atomic and persisted in SQLite, shared by all submission
paths. A rejected recipient list consumes no reservation. A later delivery
failure or client resubmission does consume one; reservations are not refunded
after crashes. Queue retries continue the admitted submission without another
reservation. Entries expire exactly 3600 seconds after admission; a backwards
clock retains future reservations.

Recipient refusals return SMTP `550 5.7.1 recipientNotAllowed` or JMAP
`notCreated` with type `forbiddenToSend`. Rate refusals return SMTP
`451 4.7.0 sendRateExceeded` or JMAP type `tooManyRequests`.
Storage failures refuse admission.

The inbound Mix filter also receives:

| Variable | Value |
| --- | --- |
| `$AUTH` | `local-auth` or `external`, from the SMTP session |
| `$AUTH_USER` | Authenticated account email; empty for external mail |
| `$SPF` | Computed SPF result, lowercase |
| `$DKIM` | Computed aggregate DKIM result, lowercase |
| `$DMARC` | Computed DMARC result: pass, fail or none |
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
when SPF is `fail` and either the envelope sender or any parsed header From
address claims an enabled authoritative domain with `spf_policy: strict`.
Authenticated submissions are exempt. Other domains and SPF results retain
existing behaviour.

Remaining work: wider SPF enforcement and DMARC policy enforcement, including
the existing DMARC result computation issue. Relay and the other doorway gaps
are separate work.
