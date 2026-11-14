# Mix lint ratchet

The ratchet uses the analyser selected by `--mix`. CI and the full CBC
recipe use `target/debug/mix` from the branch. Locally run
`mix tools/ci.mix mix-lint --mix INSTALLED_MIX`
to use the installed analyser without building anything. There is no version
pin or dependency on a worker’s installed shell.

`tools/ci/lint_context_test.mix ANALYSER` checks acceptance of a registered
host call and rejection of an unknown call, a host call without a host
declaration, and a host call inside a remote program.
`tools/ci/lint_ratchet_test.mix ANALYSER` temporarily adds an unknown call
to a tracked gallery file, requires the real ratchet to exit 1, and restores
the file. Run it separately from the gate in an isolated checkout.

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
the 98 regressions. Existing baseline debt remains separate. The scene IO test also called the
retired `mixos_path`, which installed Mix knows but branch Mix does not; it
now calls `mix_path`.

The adapter reads viu's actual `eval.register` sites, as the repository's
analyser build already does. Only files declaring `-- host: viu` receive
that vocabulary. Native compatibility names come from runtime contract
declarations, minus names the selected analyser already knows. Mixd services
use that native contract vocabulary; they do not inherit viu functions.
No manually maintained function list is added to the ratchet.

Installed 0.119.0 supports `--allow-function`, but lacks the repository's
host-header vocabulary mechanism and a host-manifest input. The adapter
uses invocation allowances temporarily, partitioning files by host. Since
allowances propagate into remote bodies, it checks those bodies again
without local host allowances. Name resolution works across this bridge;
arity/type contracts for newer native primitives still require the newer
analyser. A future manifest primitive should carry those contracts too.
Compatibility is based on builtin metadata, not version strings. Analysers
must support the baseline’s lint codes and JSON/allow-function interface;
new or changed lint rules can still require an intentional baseline review.

Only after applying the source-derived context was the baseline regenerated;
no new undefined-function allowance was added to it. Regenerate it with
`mix tools/ci.mix mix-lint-baseline` only after fixing diagnostics, then
review the diff for increases.
