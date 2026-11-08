# Mix Term integration — 2026-11-08

PR: https://github.com/mixviu/mixviu/pull/24 (draft while gates run).

Accepted Term lab merged with a merge commit. Main child-window, raster, retained-reply and clipboard paths are preserved; new no-window admission and native selection are opt-in. apps/term remains the bundle. term-mix uses its own Bus service, mixviu.term-mix app id and Term (Mix) title. The Rust terminal is unchanged.

CBC regenerated Cargo.lock without upgrading existing packages and supplied formatting patches. All 13 Term Mix suites are included in tools/ci.mix. Initial Viu run passed the eight apps; Term fixture-prefix/title-default failures were repaired. Current exact source a57b285e1c74 is under the full CBC gate and smoke repeat.

CBC term-mix-smoke-1-a1 rc 0 at 45b663d1: simultaneous default term and term-mix, HELP, native shell attachment/output and last-tab exit. Current gate term-mix-ci-1 and smoke term-mix-smoke-2 pending.

Install/undo recipe: docs/ports/term-mix.md and PR body. No install, desk4/VT access or desktop restart. This is headless integration, not a new visual acceptance claim.

The head below names the tested code checkpoint; this status-only commit follows it.

branch feat/term-mix; head a57b285e1c74; PR https://github.com/mixviu/mixviu/pull/24; CBC term-mix-ci-1 rc pending; smoke term-mix-smoke-2 rc pending
