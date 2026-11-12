# daily

The daily media pipeline as a Mix Bus citizen. Each day it cuts a
`daily/YYYY-MM-DD` tag on `main` and builds the day's manifest: every change
between the previous cut and this one, with areas, stats, diff-card hunks,
tasks and tutorials. Later phases derive the post, the video and
`agents.txt` from that manifest; nothing downstream may state a fact the
manifest cannot back.

This is phase 1 of the pipeline: the service skeleton, configuration, cut
tags and `manifest.build`. The other stages are listed in `daily.actions` as
disabled, and `run` stops cleanly before the first of them.

## Run it

```
mix --serve services/daily/scripts/daily.mix --name daily
mix services/daily/scripts/daily-cli.mix actions
mix services/daily/scripts/daily-cli.mix run --dry-run
mix services/daily/scripts/daily-cli.mix manifest.build --date 2026-12-01 --sha 1a2b3c4
mix services/daily/scripts/daily-cli.mix status --date 2026-12-01
```

`daily-cli` calls `daily.execute` over the Bus; `--local` runs the same
`execute` in its own process. Agents call `daily.actions` and
`daily.execute {id, params}` directly.

## Actions

| id | params | does |
|---|---|---|
| `run` | `date?`, `dry_run?`, `sha?` | stages in order from `manifest.build`; stops at the first failure or unbuilt stage |
| `resume` | `date?`, `dry_run?` | continues from the first stage that is not `ok` |
| `manifest.build` | `date?`, `dry_run?`, `sha?` | stage 1 alone |
| `status` | `date?` | the day's `status.json` |
| `config.show` | | merged configuration and its sources |

`date` defaults to today in the configured time zone. A date with no cut
tag is cut from `repo.ref` when it is today, or from `sha` otherwise; an
existing tag is reused, and a different `sha` is refused (`CUT_CONFLICT`).
`dry_run` writes the artefacts but fetches, tags and pushes nothing, and
marks the window `provisional`. A refused action returns rc 10 with
`{error_code, message}`; a failed stage adds a `resume` hint.

## Configuration

Public defaults are `media/config.default.conf.mix`. The real file is
`<hub>/media/config.conf.mix` (hub from `MIX_DAILY_HUB`, default `~/.viu`) or
whatever `MIX_DAILY_CONFIG` names; it is deep-merged over the defaults.
`MIX_DAILY_MEDIA` names the `media/` directory when the service is
installed away from a checkout. Pushing cut tags (`cut.push_tags`) is off by
default.

## Artefacts

`<hub>/media/runs/YYYY-MM-DD/` (mode 0700): `manifest.json`, `status.json`
and `log.jsonl`. A rerun for the same window writes byte-identical
`manifest.json` and leaves an unchanged file untouched. The schema is
`media/schemas/manifest.v1.json`; it is checked before the file is written.

PR titles and bodies, commit subjects, labels and author names are
contributor text. The manifest strips their markup, bounds their length and
lists them in `untrusted_fields`; prompts in later stages must treat them as
data.

## Tests

```
mix services/daily/tests/manifest_test.mix   # fixture repo + fake gh, no Bus
mix services/daily/tests/serve_test.mix      # private broker, needs noded
```

`tools/ci.mix test` runs the manifest test with the mix built from the tree.

## Units

`units/daily.service` (the citizen), `units/daily-run.service` and
`units/daily-run.timer` (03:00 Australia/Brisbane) are user units: the
pipeline reads the maintainer's hub, a checkout and the gh login.
