# Email on the generic Viu host

Run `viu run apps/email`. Accounts come from settingsd's `email.accounts`
and `email.default_account`. Open Email Settings to manage them, or use the
app's Bus commands with these JSON argument objects (these are not shell commands):

```text
email.account.set {"name":"Personal","url":"https://mail.example.test","email":"user@example.test"}
email.account.password.set {"id":"<returned-account-id>","password":"<your-password>"}
email.account.default {"id":"<returned-account-id>"}
```

`email.account.set` returns the new account ID; supply that ID to update an
existing account. Optional fields include `ca_path` (an absolute PEM file
path), `enabled`, `signature` and `identities`. Passwords stay in the app's
private `secrets.conf.mix` file, separate from settingsd and account readbacks.
Keep that file private; do not put credentials in command history or logs.

The old positional accounts catalogue is a compatibility startup input,
not the current account authority: settingsd's live account snapshot replaces
it. Use Email Settings or the commands above for persistent configuration.
When settingsd is unavailable, cached accounts are shown read-only.

Select an account, mailbox and message. Reads and sends use authenticated
JMAP HTTPS; sending uploads MIME, creates the email, submits it and files it
in Sent. No `submit_service` setting is needed. Server replies distinguish
sent, filed and uncertain outcomes; acceptance does not confirm delivery.

Paged lists and reader modes are implemented. The current acceptance inventory
in [the port record](../../docs/ports/email.md#round-6-acceptance-record-2026-11-07-accepted)
records attachment transfers/previews, draft save/restore, document/close gates
and calendar/contact CRUD. These are separate from durable outbox/restart
reconciliation; this README does not promise that pending sends survive restart.
Older checkpoints retain their historical open items and the superseded
catalogue/submission route.

`viu run apps/email --fixture` is a separate synthetic-list measurement mode
that cannot contact a mail server.
