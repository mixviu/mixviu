# maild

JMAP, SMTP, IMAP, CalDAV and CardDAV mail service, with private authentication,
rule-engine and Bayesian crates, using the shared metadata store. Maild ships
as its own binary; `src/main.rs` calls the library entry point.

The server package carries the `maild` binary, service/socket units and sysusers
declaration under `services/maild/units`, and the patched parser's licences under
the installed prefix's mail-parser licence directory. `maild@INSTANCE` requires the matching
`noded@INSTANCE` and `<etc>/session/INSTANCE.env` broker configuration.
Its service config is `<etc>/maild/config.conf.mix`. State is owned by
`maild` under `<var>/maild` and `<var>/jmap`. Units retain the interim bind
capability for config listeners; socket templates require explicit host
addresses, service selection and enablement before removing it with a drop-in.
Configured TLS certificates and DKIM keys must be readable by that account.

See the manual (manual not included in this source tree) and
storage API (manual not included in this source tree). The [test accounting](tests/README.md)
records every ported test and the inherited gates. Run Rust tests on a build
worker; parser nesting tests require the workspace's patched mail-parser.

## JMAP front doors

In the node's Mix config, `maild.jmap_origins` optionally lists HTTPS origins
that may appear in session URLs:

```mix
maild: {
    jmap_origins: ["https://mail.example.com", "https://mail.example.org:8443"]
}
```

Standalone maild config uses the same `jmap_origins` key at its top level.
Entries must be `https://host[:port]`, without a path (including a trailing
slash), query, fragment or userinfo. Hosts are lowercased and port 443 is
omitted. Invalid entries fail config loading.

webd replaces client forwarding headers with the request's own host and HTTPS
scheme on every proxied JMAP route. maild uses only `X-Forwarded-Host`, and only
when its HTTPS origin matches this list, for all four session URLs. Other
headers never select the origin. A missing or unapproved host falls back to
`base_url`; an empty list (the default) preserves existing session behaviour.
