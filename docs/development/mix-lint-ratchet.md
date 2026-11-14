# Mix lint ratchet

The ratchet uses the clean Mix 0.119.0 analyser at commit
`8779f33284b7d3f1178c3d57a699efb9b0867b22`. Its executable and source pin
live in `tools/ci/lint.conf.mix`. Hosted CI builds that revision separately;
local and CBC runs verify the installed binary against the same pin.
`--mix` still selects the shell under test, but cannot change this analyser.

Run `mix tools/ci.mix mix-lint` locally. This stage builds nothing.
`tools/ci/lint_context_test.mix` checks acceptance of a registered host call
and rejection of an unknown call, a host call without a host declaration,
and a host call inside a remote program.

The baseline is `tools/ci/mix-lint-baseline.json`, with error/warning counts
per file and diagnostic code. Before this repair it last changed on
2026-11-10 in `b31733d4` (#6), using the repository analyser with generated
viu vocabulary. CI used `target/debug/mix lint --json`; local `--mix` could
instead select an older installed analyser. On main `b1cbf48e`, installed
0.119.0 reproduced the reported 98 regressing file/code pairs.

Those regressions consist of registered `viu_*` functions and three native
functions absent from the old analyser: `mix_path` (the prefix rename),
`stat_many` and `rows_project`. All three have runtime implementations and
contracts in this tree. There are no genuinely undefined functions among
the 98 regressions. Existing baseline debt remains separate.

The adapter reads viu's actual `eval.register` sites, as the repository's
analyser build already does. Only files declaring `-- host: viu` receive
that vocabulary. Native compatibility names come from runtime contract
declarations, minus names the pinned analyser already knows. Mixd services
use that native contract vocabulary; they do not inherit viu functions.
No manually maintained function list is added to the ratchet.

Installed 0.119.0 supports `--allow-function`, but lacks the repository's
host-header vocabulary mechanism and a host-manifest input. The adapter
uses invocation allowances temporarily, partitioning files by host. Since
allowances propagate into remote bodies, it checks those bodies again
without local host allowances. Name resolution works across this bridge;
arity/type contracts for newer native primitives still require the newer
analyser. A future manifest primitive should carry those contracts too.

Only after applying the source-derived context was the baseline regenerated;
no new undefined-function allowance was added to it. Regenerate it with
`mix tools/ci.mix mix-lint-baseline` only after fixing diagnostics, then
review the diff for increases.
