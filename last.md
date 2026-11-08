# Mix Term integration — 2026-11-08

PR: https://github.com/mixviu/mixviu/pull/24

Accepted lab/parity-term (tested 9ed90beb; receipt f5d7e1fa) is integrated with merge commits. Current main security migration 2ec0dca9 is included. Main child-window, raster, retained-reply and clipboard behaviour is preserved; authored no-window admission and native selection are opt-in. apps/term remains the shared bundle. The comparison launcher selects service term-mix, app id mixviu.term-mix and title Term (Mix). Rust Term is untouched.

Tested code checkpoint: 1cdff08b3fd7ad9d54ca330d81dac3186f6a1d25. This status-only receipt follows that checkpoint; no runtime source changes follow it.

- CBC term-mix-ci-4-a1 rc 0: tools/ci.mix all, including fmt, env, clippy, viud egui-free, all tests and Mix lint (0 regressions). All 13 Term Mix suites ran, as did Term bundle and the eight apps bundle/contract tests. Complete saved log loaded and checked: 602 Rust result groups, 10820 passed executions, 0 failures, 96 ignored. Warnings and ignored-test limits remain; the explicit smoke below runs separately. Log SHA256 159f84c08f2d9ab7526802fad81cfe34bc4bc5e509e31f90fbe7e4219c29c595.
- CBC term-mix-smoke-5-a1 rc 0 at the same code: simultaneous term-mix and default term, HELP descriptors, native shell attachment and real output, last-tab reply delivery and both normal process exits. Full smoke log reviewed against its preceding complete log; SHA256 d46ee32acae0141393419003c9587a433c816cb0419fe1c81a1e11052cea38ec.
- CBC term-mix-foreign-4-a1 rc 0: 16 foreign crates listed, strict lib-mix/mix-shell/mix/mixd clean, shell ELF checked. Existing allocator debt follows the main package rename; retired debt was removed.
- CBC regenerated Cargo.lock and supplied formatting patches. After the latest main merge, term-mix-prepare-6-a1 rc 0 returned an empty lock/fmt patch.

The default CI interpreter is now rebuilt incrementally on reused CBC targets, rather than trusting an executable left by another commit. Earlier superseded full gates were cancelled and are not claimed as passes. Final receipt-head checks are recorded in the PR.

Install/undo recipe (files, prefix layout, launcher and desktop entry): docs/ports/term-mix.md and PR body. No installation, desk4/VT access or desktop restart performed. The receipt proves headless integration; the desk4 visual comparison remains an operator step.

branch feat/term-mix; head 1cdff08b3fd7ad9d54ca330d81dac3186f6a1d25; PR https://github.com/mixviu/mixviu/pull/24; CBC term-mix-ci-4-a1 rc 0; smoke term-mix-smoke-5-a1 rc 0
