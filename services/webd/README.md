# webd

See the manual (manual not included in this source tree) and the preserved
[Mix CMS example](examples/mix-cms/README.md).

## Vhost secrets

Service credentials follow `@{etc}/credstore/<svc>.<name>`: one file per
service secret, delivered by the operator from the authoritative secrets
store, never through git or shared config. Systemd gives only the owning
service a private copy through `$CREDENTIALS_DIRECTORY`.

Move JMAP passwords out of the shared `node.conf.mix` into
`/opt/mix/etc/credstore/webd.vhost-secrets`, owned by `root:root` with mode
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
Unknown or duplicate names and supplied passwords without their corresponding
email refuse startup. Only supplied password fields are checked by the overlay;
unrelated node-config rows keep their existing validation behaviour.
Credential parse errors never print source text or values.

The unit loads this file as the systemd credential `webd.vhost-secrets`.
Its empty same-ID `SetCredential` fallback lets nodes without the file keep
using passwords from the node config. Without a credential directory or
credential file, webd also keeps its existing behaviour. This compatibility
path supports unmigrated nodes; migrated nodes keep all password values in
the credstore, never in shared config.

An empty or whitespace-only credential, or `{vhost:[]}`, applies no overlay
and does not validate or change node-config rows. When set,
`CREDENTIALS_DIRECTORY` must be an absolute path to an existing directory.
The credential must be a regular file (no symlinks, FIFOs or devices), UTF-8,
and at most 64 KiB; invalid files refuse startup without printing content.

The same-ID fallback requires systemd 247 or later, where it is
[documented in the original systemd 247 manual](https://github.com/systemd/systemd/blob/v247/man/systemd.exec.xml).
Runtime probes use systemd 262; older versions were checked against the
documented contract rather than tested locally.

Create the credential file first, then remove the corresponding passwords
from `node.conf.mix`. Restart webd after changing either file; credentials
are read once at startup. The startup log reports the vhost count and field
names, never passwords. An email whose password was removed requires its
replacement in the credential before restarting.
## Static sites

A redirect vhost needs its usual TLS or ACME settings but no document root:

```text
webd: {
  vhost: [{
    host: "www.example.com",
    redirect: "https://example.com",
    acme: { provider: "letsencrypt_staging", challenge: "http01",
            contact_email: "ops@example.com" }
  }]
}
```

`redirect` must be an HTTPS origin without a trailing slash, path, query,
fragment or user information. It cannot name this vhost or an alias, or coexist
with `cms_db_path`, `jmap_upstream` or `noded_ws`. Responses use 308, retaining
the method, path and query. Plain HTTP keeps its ACME and autoconfig branches.

Docroot vhosts can set response defaults:

```text
{ host: "example.com", www_dir: "/srv/www/example",
  headers: { "content-security-policy": "default-src 'self'",
             "referrer-policy": "strict-origin-when-cross-origin" } }
```

The `headers` string map applies to docroot files and redirects only. Mix and
CMS responses own their headers. Existing response headers win. Header syntax
is validated; framing, hop-by-hop, content-encoding and content-range overrides
are refused. Unknown vhost fields remain forward compatible.

Docroot files negotiate adjacent `.br` and `.gz` representations using
`Accept-Encoding`; no compression runs at request time. Responses vary on
`accept-encoding`, including identity and conditional responses. Shared assets
and media retain their existing serving paths. ServeDir uses Last-Modified/304;
this change adds no ETag generator.

Explicit `webd.listener` rows can set `http2: true` (default false) to offer
`h2, http/1.1` over TLS. HTTP/2 needs authority-based host routing as well as
ALPN, so adoption is opt-in. Plain listeners remain HTTP/1.1. Extended CONNECT
is disabled; WebSockets use a separate HTTP/1.1 connection. Certificate renewal
retains the listener's ALPN policy.

Before routing an h2 request, webd copies `:authority` to `Host`. If both
are supplied they must agree (case-insensitive, including the port); duplicate
Host fields are refused. This gives JMAP forwarding metadata, Mix `$HOST` and
`$HEADERS.host`, authentication/origin checks and routing the same host view.
The JMAP upstream's own Host remains its upstream authority; the public host is
forwarded as `X-Forwarded-Host`. HTTP/1.1 Host handling is unchanged.

