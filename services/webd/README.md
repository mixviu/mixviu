# webd vhost secrets

Move JMAP passwords out of the shared `node.conf.mix` into
`/opt/mix/etc/webd/vhost-secrets.conf.mix`, owned by `root:root` with mode
`0600`. Keep the corresponding email fields in `node.conf.mix`.

```text
{
  vhost: [{
    name: "example.com",
    public_read_password: "replace-with-password"
  }]
}
```

Each `name` must exactly match a `webd.vhost` row's `host` in the node
config. The optional `dev_session_password`, `public_read_password` and
`system_sender_password` fields fill or override that row's passwords.
Unknown or duplicate names and incomplete email/password pairs refuse startup.
Credential parse errors never print source text or values.

The unit loads this file as the systemd credential `webd-vhost-secrets`.
Its empty same-ID `SetCredential` fallback lets nodes without the file keep
using passwords from the node config. Without a credential directory or
credential file, webd also keeps its existing behaviour.

Create the credential file first, then remove the corresponding passwords
from `node.conf.mix`. Restart webd after changing either file; credentials
are read once at startup. The startup log reports the vhost count and field
names, never passwords. An email whose password was removed requires its
replacement in the credential before restarting.
