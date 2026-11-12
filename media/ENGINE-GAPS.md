# Daily media pipeline: engine gaps

Capabilities the pipeline needed that `mix` or `mixd` does not provide, the
workaround in use, and whether each is a candidate for a small engine
addition. Nothing here changed an engine; each addition needs its own
approved PR.

| # | Needed | Workaround | Engine candidate? |
|---|---|---|---|
| 1 | Validate a value against a JSON Schema (the brief commits `media/schemas/*.json`) | `services/daily/scripts/lib/jsonschema.mix`: the draft 2020-12 subset the schemas use; unknown keywords are refused, not ignored | Maybe. `validate()` covers API boundaries with its own spec form; a JSON Schema builtin only pays off if more components publish JSON schemas |
| 2 | Match a path against a glob string (`engines/mix/**`) | `util.glob_regex` turns the glob into an anchored regex | Yes, small: `glob_match(path, pattern)` beside `glob()`, sharing its `**` rules |
| 3 | Today's date in a named time zone (`Australia/Brisbane`) | `date +%Y-%m-%d` by argv with `TZ` in its environment | Yes, small: a `tz` option on `date_format` |
| 4 | Parse ISO 8601 times with offsets (`2026-12-01T03:00:00+10:00`, gh's `closedAt`) | `util.iso_epoch` parses the form with a regex and `date_parse` of the date part | Yes, small: `date_parse` accepting RFC 3339 |

## Deviations from the brief (not gaps)

- Configuration and the area table are strict-data `.conf.mix`
  (`media/config.default.conf.mix`, `media/areas.conf.mix`), not YAML: that is
  the repository's configuration form, read with `load_data` and never
  executed.
- The service runs as `mix --serve` (which hands `--serve` to mixd), like
  every other Mix service unit in the repository.
- A tutorial's metadata block sits between `-- tutorial-meta` and
  `-- end-tutorial-meta` as `$meta = {...}` in strict-data form, so the
  manifest can read `covers` without running the tutorial.
