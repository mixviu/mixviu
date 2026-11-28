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
then each XDG data directory's `applications/`; non-absolute XDG config
entries are ignored. Desktop-prefixed files supply defaults only: `[Added
Associations]` and `[Removed Associations]` are read from plain
`mimeapps.list`, and within one file Added is processed before Removed.
Defaults take precedence, then added associations, then entries declaring
the MIME type. A default counts only when its desktop file is associated
with the type, and a removal in a config directory voids it; the user's
remedy is their own `[Added Associations]`. Schemes use
`x-scheme-handler/<lowercase-scheme>`. Desktop filenames map to registry
ids without the `.desktop` suffix.

An OpenURI handler's Exec needs exactly one standalone `%u` or `%U`; there
is no implicit append on the open path, and entries with only `%f`/`%F`, or
with no field code at all, are not scheme handlers (rc 14 when no eligible
handler remains). The check runs on the expanded argv with a per-call
random sentinel standing in for the URI: the sentinel must survive as
exactly one whole argv element after the executable -- never inside a
larger element, never argv[0] (a leading `%d` or an empty `%i` expands away
and would shift the URI into the executable slot) and never immediately
after a `-c`/`-e`-style code flag or `--command`, `--eval`, `--exec` or
`--execute`. `%%` is a literal percent escape. URI-bearing START/LAUNCH
apply the same expanded-argv gate, keeping the spec's implicit append (at
least one whole-element URI), and refuse with rc 10 `invalid_request`
before spawning, recording a `refused` row.

Desktop entries are trusted code: an operator or a package installs them,
and a handler that evaluates its argument is not detected. The flag check
is a malformed-entry tripwire, not a security boundary; the guarantee is
the URI's placement in argv, nothing about what the handler does with it.

Known misses: `bash -cx`, `bash -c --`, `perl -E`, `php -r`, `node -p`.
Known false refusals: single-dash long options ending in c or e, such as
`thunderbird -compose` and `firefox -private`. Remedy: use the `--` form,
or reorder the Exec.

`Terminal=true` entries are excluded from OpenURI resolution. The current
term launcher forwards argv (`apps/term/term.mix`), but its startup policy
(`apps/term/startup-policy.mix`, `decide`) rejects `--` and command arguments;
it does not implement `term -- argv`. Until a verified terminal argv launch
path exists, `apps.handler` cannot advertise those entries as supported.

Open uses the existing managed START path and returns its reply plus
`scheme`: rc 14 for no handler, rc 11 `spawn_failed`, or rc 10 `limit`
(`RESOURCE_LIMIT`) at 120 handles. Each refusal records a `refused` row.
Success records an `open` row with scheme, handler id and handle, never the
URI; `apps`' own log lines omit it too (a handler's stderr is the
handler's). Argv goes directly to spawn, with no shell; that covers `apps`
only, not what the handler then does with the argument.
`apps.start` and `apps.launch` require each `uris` element to be a valid
URI of the shape above or an absolute path (at most 8192 bytes, no bytes
below 0x20 or 0x7f; spaces stay legal). START/LAUNCH with non-empty
`uris` is local-only: mesh origin returns rc 10 `forbidden`. Mesh
start-by-id without `uris` stays open until Bus 2c.

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
