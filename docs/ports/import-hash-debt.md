# Inherited Mix import-hash drift

Audit at the Prefs/Screenshot integration freeze. All 22 files below are byte-identical to accepted integration source `4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0`; neither incoming app branch nor this reconciliation changes them. `tools/verify_runtime.mix` currently fails on the first one. This is outstanding provenance debt, not a passing provenance gate.

`runtime-imports.json` declares MixOS `884fdd8ef2c548cc42017b9170593ec67ed30e57`. Track A imported the shell closure from `8779f33284b7d3f1178c3d57a699efb9b0867b22` at `715c3931` (2026-11-06 17:12:32 +10), with separate per-file records in `engine-split-mix-imports.json`. The runtime inventory entered this branch at `fc4d90ee` (2026-11-06 21:17:09 +10), without reconciling these two source pins or subsequent Track A adaptations. Twenty current mismatches trace to A1's replacement; script_version adds A5's host-header parser; builtins_hof adds the later bulk change. The latter first appears on this branch at `0dd1634c` (2026-11-07 05:56:35 +10), merging `4c1d715`. Thus 21 mismatches were already detectable when the runtime inventory entered at fc4d90ee, and the 22nd arrived with bulk integration.

The table gives the first content divergence in this branch's first-parent history (which can precede the stale inventory), plus subsequent adaptations. Full expected/current SHA256 values follow.

| File under libraries/mix/ | First content divergence | Subsequent changes / reason |
| --- | --- | --- |
| CHANGELOG.md | `715c3931` | A1 source-pin replacement; no subsequent content change |
| Cargo.toml | `715c3931` | A1 source-pin replacement; no subsequent content change |
| src/analyzer.rs | `715c3931` | a8bdc049 Lab A5: -- host: header, lint D3025-D3027, mix -> mixd handoff |
| src/ast.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| src/builtins.rs | `715c3931` | e3b8dfc1 Lab A2/A3: mixd engine, shell/service profiles, refusals naming mixd; a8bdc049 Lab A5: -- host: header, lint D3025-D3027, mix -> mixd handoff; b91ff80a Lab A7 review fixes: handoff safety, honest has_builtin, workspace guard; 0dd1634c Merge bulk primitives into the Monitor parity integration branch; 9c5264cd Implement filesystem creation timestamps required by the retained stat contract |
| src/builtins_hof.rs | `0dd1634c` | Bulk indexed-row operations imported by 0dd1634c |
| src/evaluator.rs | `715c3931` | 0dd1634c Merge bulk primitives into the Monitor parity integration branch |
| src/lexer.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| src/lib.rs | `715c3931` | a8bdc049 Lab A5: -- host: header, lint D3025-D3027, mix -> mixd handoff; 0dd1634c Merge bulk primitives into the Monitor parity integration branch |
| src/lint_docs.rs | `715c3931` | a8bdc049 Lab A5: -- host: header, lint D3025-D3027, mix -> mixd handoff |
| src/native_events.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| src/parser.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| src/scope.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| src/script_version.rs | `a8bdc049` | A5 host-header parser and supported engine names |
| src/token.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/audit_interp_corpus.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/contract_args.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/handler_event_scope.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/integration.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/lint_analyzer.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/require_modules.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |
| tests/tcp_listen.rs | `715c3931` | A1 source-pin replacement; no subsequent content change |

A5's `a8bdc049` host-header change is dated 2026-11-06 17:21:50 +10.
The later `9c5264cd` creation-time repair is dated 2026-11-07 06:16:41 +10.
These content dates are distinct from the date the stale runtime inventory
became present on this integration branch.

The two upstream pins are dated 2026-10-31 (`884fdd8`) and 2026-11-05
(`8779f33`). Their Mix difference includes generation-owned UDP sources
(`6d7d2b0cc`), bounded offset reads (`e89605ae3`), global declarations and
scope routing (`37ed10f95` and its review repairs), hyphenated Bus commands
(`26ccb3ff6`) and monotonic timers (`15bdfb63f`). Those changes explain the
newer grammar, analyser, evaluator, scope, event-pump and contract-test
hashes; the version/changelog advance accompanies that source import.
They are recorded upstream changes, rather than unexplained local edits.

## Recorded hashes

| File | Stale expected SHA256 | Current SHA256 (also 4f2c0fd) |
| --- | --- | --- |
| CHANGELOG.md | `6e47f1c8cd43e9dfff60a8783afc85cd28d63c363fc071f9e880a16367d5b56f` | `44f91aa80d5c88b4475f25b861dc1cfe625706102e9213c0eac132735551d6b4` |
| Cargo.toml | `af5afbdfee08e344b0296ca5acbe0441997244c42abe6d279dbdeec27872d573` | `1f54463e3e095cd5b7032fadef37b9b7cf0f662f1950a19771780d8a6c4011c4` |
| src/analyzer.rs | `85de636030df165549c47a408aeeacec83f4cb5a199158664a18727c45aa7418` | `16b5c396c1a40d23be43b69dae4a1788da95e28738758e640912909fc7ee843c` |
| src/ast.rs | `8c60dc807d86468b757db473b8cc2dc87e39a29a59e733859d4709203a5579ec` | `31828618807e47c9e537663235430615a40e92355259de03301a6c4ac9a182c9` |
| src/builtins.rs | `8082c2c5bc294daf8b447e9ffa1d9b906bd5ccf94e7c726c04489b4921512b15` | `1051a3387e2c44cb8a1af26b1bd49ce4afc36f6db95b0f783d599e682f92ba05` |
| src/builtins_hof.rs | `7bdcb9204c2d7af6de1ac1c54a35edadf83e94279187fbc072ba5d4eac4425b6` | `0a35a03d0cf488eecf4cf95a4e3d840f2804bffd9a8de525b507bba8a35c6891` |
| src/evaluator.rs | `e5d1d8a403ada7eca95680dbf47cd5513e4d85a9f58c5e524cc08d3b0620ffd0` | `e63816abe18eff67d220c67d7e83cf4aaef65686aa2682e7588d57ef2276516d` |
| src/lexer.rs | `cbe905c0568d749da3aceebe4bd165ebcb22f292f438bc1a4de917a87cb2256e` | `66679254cfeff2aae89c6e50d3586d16a9a19705015f87ef23d14ff394328ae3` |
| src/lib.rs | `62613722053d9be370338fb8e1125af670dfe86faaca26a29b8fb61ded49a67b` | `f1c8a5c9531f28e914d43a5b23fdaabf0064ec602274068560db3ddaf014726a` |
| src/lint_docs.rs | `ecf2b1bae666b453c268856c0f7b4e602fd703163dea1e1d67361f810640057e` | `d2e236f46a1c0c13e962bc81ec8890096b9a74854abcbc38ff44efddb86b5098` |
| src/native_events.rs | `1d201010d240df5998da9b80df41be9ea669d6f2746c03cb7c3b4262673679ab` | `3117e02f1728e5d03b518af443235bfd7971968caebb80bf1bdee18fd82de814` |
| src/parser.rs | `473b2219d91ace7653fe3088d0cd440986b890f968b398c34b2a1ac1ab013d28` | `f687be9ad24329a3cf13cf4f86983d89a3c873b978471d4899358f092aa53b63` |
| src/scope.rs | `531ff1f48bfcc5cc58fb255a22785e6c78bdf355428dcac79f53edf1b4ae02fd` | `82d6a1f6d27dbc274162ce58e08e69cbd3d0034e0970d6cfbfd1d6b242408cc3` |
| src/script_version.rs | `2d72a679f66796bc1450bceb786a93b670d3157f76439a7eec7a869678445341` | `a18fd8a3d454e4b75bec084d5aaa7e35c653d6a6eee3b8b19c1e34bb12e1bdf9` |
| src/token.rs | `2fe0c7979242cecd916207437b3b18f98afc293a41650436db897ed4d8c52c54` | `c56dff1c43d7ddb3ba856fe4e2784476ee8ae662dca1f1094115376c8a0ed311` |
| tests/audit_interp_corpus.rs | `4a97c9525c86c10527a3a0de187b9a3278693d711759aa49143cb15b2c3d759a` | `ed1c6a420dcc7466b5cbe67ce969a8d9e81667778d20bab5ace578a6c0382ad0` |
| tests/contract_args.rs | `814a12d4fbf3f3da8ff1d225c4a98eb19144a576cdd55487a64143ed1a8f2eae` | `138c3b736a0a8d3d694a58b1286220f744cf93cad26b04952a74183c81fa9696` |
| tests/handler_event_scope.rs | `9c364c676b0db4a243ef32d0d93f4fcd783ed8a983413706fa9630a3fd0e257e` | `1365c4e78b3cb8a87d2bdc421fa34dadac7b0337e1367da49414c2650b16cc9f` |
| tests/integration.rs | `3467226975196540df0d40a6404c46acb8dcdb4208f5642a5f37800cf08a20f0` | `2a45734d95130a6d741a1b79ab3a75c9861b83e0264fa3124a92f96325cdf64c` |
| tests/lint_analyzer.rs | `5e1c01bf0ab99df9aebcc5721793193ac0ba6b51cd1a9770b6e2343672122405` | `6aafd5aa508ed38cef0bd8071a855f4c472f80fac529578ba6c7b55ba982cf63` |
| tests/require_modules.rs | `804e1dbfd54ee850933c0ca46ac5b78643886288160c265ce712439371bf45ea` | `8f51cd8334004b901558fe14e55a3167009100d5eb7ab703e838407d42086234` |
| tests/tcp_listen.rs | `676b1279b0079d922e9ad9dfdde23ebf7ea1198234ab98fb260f6cf593e645c0` | `9b2cb7903aacbf03ac31df79ade27d5f32fe6bfed2adb2c47297ea7e60187ade` |

## Before production derivation

Reconcile the two inventories against their pinned source trees; retain both original hashes and the adaptation history. Review A2/A3 shell/service profiles, A5 host declarations and lint, A7 handoff safety, bulk indexed projection/evaluation and the 9c5264cd filesystem creation-time repair. Then update the authoritative current hashes with those reviewed reasons and make verify_runtime pass. Do not merely replace the expected values with today's hashes: that would erase the import/adaptation distinction. The lab's behavioural and visual gates do not discharge this provenance debt.
