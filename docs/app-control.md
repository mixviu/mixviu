# Application control

`apps.handler {"scheme":"https"}` is a local-origin, read-only lookup. It
returns `{id}` for an allowlisted scheme with an installed handler, or rc 14
`not_found`. Scheme names are case insensitive. Both it and `apps.open`
reject mesh origin with rc 10 `forbidden` (`error_code: FORBIDDEN`).

`apps.open {"uri":"https://example.invalid/","activation_token":"..."}`
validates before resolution: at most 8192 bytes, no bytes 0x00–0x20 or 0x7f,
and an ASCII scheme beginning with a letter, followed by letters, digits,
`+`, `.` or `-` and a colon. HTTP(S) requires `//` and a non-empty host.
`file:` is always refused; local files belong to the future OpenFile path.
Invalid URIs or tokens return rc 10 `invalid_request`. Tokens must be at
most 256 printable ASCII bytes; they reach the child as
`XDG_ACTIVATION_TOKEN`. There is no caller-directed compositor focus.
`ask`, `writable` and `parent_window` are ignored and grant no authority.

`apps.open_schemes` defaults to `["http","https","mailto"]`. Read it through
`apps.props.watch`; change it through the existing local-only
`apps.props.set {"path":"apps.open_schemes","value":["https","mailto"]}`.
Values must be lists of lowercase scheme names, excluding `file`. An empty
list disables OpenURI. Unlisted schemes return rc 10 `forbidden` from open
and rc 14 from handler. Only the local operator may widen this policy;
hostile mail content must not reach arbitrary registered scheme handlers.

Resolution rereads desktop-prefixed then plain `mimeapps.list` at each of
XDG_CONFIG_HOME (default `~/.config`), XDG_CONFIG_DIRS (default `/etc/xdg`),
then each XDG data directory's `applications/`. Defaults take precedence,
then added associations, then entries declaring the MIME type, respecting
higher-priority removals. Schemes use `x-scheme-handler/<lowercase-scheme>`.
Desktop filenames map to registry ids without the `.desktop` suffix.

Open uses the existing managed START path and returns its reply plus
`scheme`: rc 14 for no handler, rc 11 `spawn_failed`, or rc 10 `limit`
(`RESOURCE_LIMIT`) at 120 handles. Each refusal records a `refused` row.
Success records an `open` row with scheme, handler id and handle, never the
URI; journals also omit it. Argv goes directly to spawn, with no shell.
`apps.start` and `apps.launch` now require each `uris` element to be a valid
URI of the shape above or an absolute path (paths may contain spaces).
These START verbs retain their existing origin policy.

A Firefox child may hand off to an existing Firefox and exit 0; `apps.exited`
then reports `reason: "exit"`. Success means the handler spawned, not that a
tab opened. Cold Firefox remains managed under its id and can be stopped.
Exited transient handles free their slots. Only an activation token may
request focus; there is no special Firefox focus call.

`<app>.show` asks an application's Mix worker to activate its own native
window through the compositor. Callers must check both the return code and
the reply's `shown` field.

- `rc == 0` and `shown == true`: the window was activated.
- `rc == 0`, `shown == false` and
  `reason == "compositor_unavailable"`: the compositor has not registered.
  The application is running, but activation has not happened. Wait for native
  service registration before asking again. This result schedules no later
  focus and must not be reported as a shown window.
- A nonzero return code reports an activation failure. Retain its diagnostic
  body and show the message to the user.

Ced holds its activation reply for 65 seconds. Its worker can make four
sequential compositor requests, each bounded to 15 seconds; the hold includes
five seconds for completion delivery. A caller wishing to await the complete
result should allow at least 70 seconds. A shorter caller deadline can expire
before ced finishes and does not establish whether activation succeeded.

Ced retains the latest failed job's kind, return code and complete body in
`ced.specimen` under `last_job_error`. Success of the same job kind clears the
visible failure notice, while an unrelated successful background job leaves
it visible. Clearing the notice does not remove the diagnostic record.
