# Term and the Rust reference (term-rs)

`apps/term` is the Mix terminal, named `term`. Its Bus service and authored
commands both use `term` / `term.*`; HELP descriptions name the actual service.
Its app id is `mixviu.term` and its window title is `Term`. The numeric
`instance` in replies remains a fresh process token used to reject stale input.
Schemas and that token contract are unchanged.

```text
/opt/mix/bin/term
/opt/mix/bin/viu run /opt/mix/share/mixviu/apps/term --service term --app-id mixviu.term --title Term
```

The launcher forwards arguments, including `--comp NAME`, `--noded-url URL`
and `--headless`. Put application arguments after the host options.

## Bundle and migration

`tools/apps_bundle.mix` produces the complete shared bundle:

| Source | Installed destination | Mode |
| --- | --- | --- |
| `apps/term/term.mix` | `/opt/mix/bin/term` | 0755 |
| `apps/term/mixviu.term.desktop` | `/opt/mix/share/applications/mixviu.term.desktop` | 0644 |
| `apps/term/` | `/opt/mix/share/mixviu/apps/term/` | files 0644 |
| `libraries/viu/mix/` | `/opt/mix/share/mixviu/libraries/viu/mix/` | files 0644 |

Keep the relative-import layout. The topbar selects `mixviu.term`; the app
registry and `mix --gui` already select the prefix's `bin/term`.
No compatibility alias or migration shim is shipped. At upgrade, remove the
previous comparison launcher and desktop entry, and update private launcher
customisations. Preserve the previous release and receipt for rollback.
No installation is performed by this change.

The original Rust terminal is called **term-rs** in MixViu. Its source and
binary belong to frozen MixOS: their on-disk names, immutable provenance,
schema names and historical receipts are retained. This repository does not
install or start that binary on a MixViu desk, and ships no Rust terminal
launcher. Copying or symlinking the frozen binary under the new executable
name would not rename its hard-coded Bus registration. Do not start it on
the same broker as Term: it still registers legacy `term`. A future term-rs
adapter must isolate or translate that registration before coexistence.

## Clash check and validation

The repository's production service registrations, desk scripts, agents,
unit templates and shared scenes have no other Bus `term` registrant.
The generic Viu host registers the selected bundle service, not an extra
terminal service. Compositor key bindings call `term` without owning it;
the app registry owns `apps`, and agents own their own service names.
This is a source/configuration check; no running desktop was accessed.
Private desk configuration and an already-running frozen term-rs must be
checked at installation.

Use scoped CBC fmt, clippy and `lib-viu` tests, plus the term app's Mix suites.
`tools/term-smoke.mix` explicitly runs the headless `term_host` test: default
`term` and isolated `term.test`, HELP descriptors, real shell output and
last-tab reply delivery before exit. The override exercises the generic
host facility; it is not a legacy alias.
