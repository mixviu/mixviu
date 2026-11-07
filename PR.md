## Summary

`mix lint` now recognises viu's native vocabulary only for files declaring
`-- host: viu`. A build-time table comes from the real runtime registration
sites, without adding GUI dependencies to Mix. Unknown names still fail;
host extensions also participate in collision checks and do not leak into
remote `ssh_mix` bodies.

Mix apps declare their host. Ced's sibling policy functions use explicit,
literal `require` bindings and namespaced calls; lazy imports preserve its
cyclic policy dependencies. Reusable macro policy is separate from the
standalone macro handler. Backdrop and Dopus handlers stay in their app
entries because `require` deliberately refuses handler-owning modules.
Dopus's comparison harness loads the same entry with automatic startup
disabled, then sends its existing initialise event.

Newly exposed defect fixed: Workshop's open-in-Dopus job used the removed
`mixos_path()` builtin. It now resolves the installed executable with
`which("dopus")`, returning `UNAVAILABLE` if it is absent.

Existing diagnostics outside
the app migration, including the webd examples/widgets and intentional
negative interpreter tests, remain in the ratchet.

Baseline before: 828 diagnostics (728 E1102, 36 E1101, 50 W2401, 14 others).
The final CBC-generated counts and test evidence are recorded below when
the gate completes.

## Risk tier

Tier 1: Mix lint and viu runtime/app integration. Module isolation is the
main behavioural risk; the native Ced, Backdrop and Dopus tests exercise it.

## How tested

- Run: local Mix lint for changed scripts; CBC formatting and focused native
  regression recipe (`tools/lint-viu-check.mix`). Final receipts pending.
- Observed: initial host vocabulary and collision tests pass on CBC.
- Not tested: interactive desktop, VT acceptance or production installation.

## How to undo

Revert this branch's commits, including the regenerated baseline. No runtime
installation or external data migration is involved.

## Authorship and review

Author: OpenAI Codex, committed and pushed through the mixviu-agent bot.
Independent reviewer and human approval are pending. No PR has been opened,
as required by the pre-cut-over brief.
