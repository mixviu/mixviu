## Summary

<!-- What changes and why, in two or three sentences. Link the issue. -->

## Risk tier

<!-- 0: Mix apps, docs. 1: viu widgets/runtime, mix builtins, viud non-input paths, CI.
     2: mixd network personalities, auth/session, Bus protocol, mesh, viud input/focus,
        mix bootstrap/package path, release/signing. See REVIEW.md. -->
Tier: <!-- 0 | 1 | 2 --> because <!-- the paths it touches -->

## How tested

<!-- Commands run and what you observed (CBC job ids, `mix tools/ci.mix all`,
     smoke runs, captures). Then what was NOT tested and why. -->
- Run:
- Observed:
- Not tested:

## How to undo

<!-- Revert this PR? A config flip? A package downgrade on hosts? -->

## Authorship and review

- Author: <!-- human, or agent + model family (e.g. Claude Opus 5.5, Sol gpt-6.1) -->
- Agent reviewers: <!-- families other than the author's, each with blocking yes/no -->
- Explainer: <!-- Tier 1+: link or inline "what changed, why, where the risk is" -->

<!-- Review rules (REVIEW.md): at most 3 review rounds, then rethink the approach;
     re-review only the new diff; never repeat a finding; nits never force a commit;
     if the diff grows past 3x its first-round size, stop and ask Mark. -->
