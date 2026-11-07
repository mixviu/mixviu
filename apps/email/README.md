# Email on the generic Viu host

Run `viu run apps/email /absolute/path/accounts.json`, then choose Connect,
a mailbox and a message. Mail data uses maild's existing authenticated JMAP
wire. Compose uses an explicitly configured native Bus `maild.submit` route.

```json
{
  "accounts": [{
    "id": "personal",
    "label": "Personal",
    "url": "https://mail.example.test",
    "user": "user@example.test",
    "password_file": "/home/user/.config/email/password",
    "submit_service": "maild"
  }]
}
```

Keep the catalogue and password file private (mode 0600). An optional
`ca_file` names a PEM trust certificate. URLs are origins without a trailing
slash/path; HTTPS is required except for loopback HTTP test servers. Only
the same origin's `/.well-known/jmap` and `/jmap` are supported. Credentials
are read in the background worker and excluded from `email.info` replies.
No settingsd migration or credential editor is implemented yet.

The submission route is optional for reading. Configure it only when that Bus
service owns the selected account. Acceptance is a maild receipt, not delivery
confirmation. A lost reply retains the draft and operation ID; explicit Send
retries that same ID. Drafts and pending IDs are in memory: do not quit with
an unresolved submission. Durable outbox/restart reconciliation is deferred.

Lists page 200 rows at a time through the shared virtual table. Plain message
bodies are bounded at 64 KiB and shown in a native read-only scrollable editor;
`email.message` also returns the full admitted text. Header labels are excerpts.
HTML, attachments and richer reader/compose workflows remain deferred.

`viu run apps/email --fixture` is a separate synthetic-list measurement mode
that cannot contact a mail server. See [the inventory](../../docs/ports/email.md)
for all limits, original-source comparison and 2,000-row measurements.
