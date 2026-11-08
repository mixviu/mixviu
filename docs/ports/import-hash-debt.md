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

## Stale vendor retention flags found at final eight-app audit

The final source `e1bbea99071c2fa8793a7e0e52aed381707d536f` still has exactly the 22 hash mismatches above. A complete manifest audit also finds **49 entries marked `retained: false` whose tracked files are present**. These are separate stale absence declarations, not 49 additional hash mismatches: all 49 match their `imported_sha256` byte for byte. `verify_runtime.mix` would reject their presence after the earlier Mix mismatch is reconciled. Do not call the whole provenance inventory clean.

The two imported inventories were already inconsistent when the runtime manifest entered at `fc4d90ee` (2026-11-06). `vendor/calloop` was restored in that commit; `vendor/dirs-sys` came from Track A A1 `715c3931` (2026-11-06). Neither set has later content changes. The manifest's earlier client-only unused-import removal claims were copied without reconciling the four-engine closure. Before production derivation, assess the actual dependency/licence closure and reconcile retention/adaptation evidence; do not blindly delete required vendored sources or mark them retained without review. This integration changes none of these bytes or flags.

| Tracked file declared removed | Actual/imported SHA256 (equal) |
| --- | --- |
| vendor/calloop/.cargo_vcs_info.json | `8354fa2dd5ead67f6d148e87557c6b01c1cf55d84c1f3bba7c7978319920baab` |
| vendor/calloop/.cirrus.yml | `010e9963e1a6a826eced127070baf4ec3f40f1c08dcc4962e31e468262cb9bb3` |
| vendor/calloop/.github/dependabot.yml | `db76b8360b5460ae333e00659f7ff11dd62359cb6edf1b8bb493e21103e5f459` |
| vendor/calloop/.github/workflows/ci.yml | `d74c50b612d13ebcc115a3e8d5992bda493253d01ee2978a84fcf096d6a6ce60` |
| vendor/calloop/.gitignore | `5a49caaacf3fbf482c35d53f8085d71f0d6cfb6b824156fa3945346a84dcbca1` |
| vendor/calloop/CHANGELOG.md | `798a2909020e18e895197ae29cc25390cbc1d1b5f5478eb97562fb49744de3be` |
| vendor/calloop/Cargo.toml | `5b4b2bf814683166cdb306eed86871a64dba40d55bf7181b94a7c1d96ef6773e` |
| vendor/calloop/Cargo.toml.orig | `e09c5f7020f3556e364f397285add88b65da07aff3f6576f3afdc195b4399378` |
| vendor/calloop/LICENSE.txt | `4b71658d3410d813b6fc6ceb654ce2aa8c2d3094c944dc59c2041da6227810e3` |
| vendor/calloop/PATCHES.md | `f8be6eafd49452a9d36311d5370b92cc98499b11e8f3b999751533df7592dd73` |
| vendor/calloop/README.md | `9e1acb153bcfe11bb9e0eb4f00c3df14e81812333a10b134c23ad66bf9c94266` |
| vendor/calloop/README.tpl | `521393c5b72ea749489781d3f752a1797044f0c0b0707ec7a208299c3f3ee9db` |
| vendor/calloop/benches/timer.rs | `cebc54f18673676f3b65dff74a47740c03785742b69f140c22ecd2635350f384` |
| vendor/calloop/examples/high_precision.rs | `376ea8d83518e5c7b72b4037ea2967c145fc95af97d285c3f4c3d3020c6ec911` |
| vendor/calloop/examples/timer.rs | `20e4bd48f1ded3b0bec9cc850914f40a40e0bff2e5e051f1946b385355f2ad9e` |
| vendor/calloop/src/error.rs | `4eb81e064ed32954ac7216e27d174cbc1abd4b23b7bea7389ac88fd50a7962f7` |
| vendor/calloop/src/io.rs | `0003c1ea61b897d1dbf9d09b691fb49448006d3a3eb211b4c6b6fcf60c615401` |
| vendor/calloop/src/lib.rs | `8cd3484d19072ad493893ccfc89134d8a0d84a4f1e64ac36231bf374d07accae` |
| vendor/calloop/src/list.rs | `0a7d9f36224f221ac4a4c2abbcf1d6d16a8458dfa3a56ac0065b5cdf06d2c712` |
| vendor/calloop/src/loop_logic.rs | `2e4b995c8698412813b3364778263b387c02d92986980d67515e39e75ac07a1b` |
| vendor/calloop/src/macros.rs | `16635ffae0bccd8f24c232566407044aa0644648ec129c2d81c4fc70a5dc10a2` |
| vendor/calloop/src/sources/channel.rs | `5b43eef8febb7fd794a9692f9ea660753aae90a6bb72ac3483fa3579a469e9a4` |
| vendor/calloop/src/sources/futures.rs | `d5f1906c58110ba76d7e8a970a90c59e9e629e1ae55d2992d86ebf12689454f3` |
| vendor/calloop/src/sources/generic.rs | `6e2c5318272237dbc354c5f91a8dab1b60a0abed22bb28e7d03d236c9c4fe58b` |
| vendor/calloop/src/sources/mod.rs | `02786fc3ebbe8eada06859a77fdd0ba48d315f54b31cd381aad7f0b5422cf5ef` |
| vendor/calloop/src/sources/ping.rs | `794a811f0d4bcc316f0e3cf96596c0704e43d6e0a3c831ec2e60c47cb18ce76f` |
| vendor/calloop/src/sources/ping/eventfd.rs | `21e0b19837aab472d23d09a95d490f4e094be70123b7276a10c9279af945b3c9` |
| vendor/calloop/src/sources/ping/iocp.rs | `de47071d66265eed2de37821d4329a3f3467c0007a745121c82c074e4c112a36` |
| vendor/calloop/src/sources/ping/pipe.rs | `679454675dd1912957e28e5cdd56c22a8f397a48f6f293cd7990f168fcf1af34` |
| vendor/calloop/src/sources/signals.rs | `5fbfb89772410a9e52681dce4e4e7828378c5833a05656a30dee79b151d995ed` |
| vendor/calloop/src/sources/stream.rs | `7e4e0bcf95d6d5b115ad8dd72ec823397b8a5fd512d1d4f2de7aa5b8af5fb137` |
| vendor/calloop/src/sources/timer.rs | `1e106bf0e7daa67b3d386576a476d9d2f05831b53ec707bea135ce593b6d9b76` |
| vendor/calloop/src/sources/transient.rs | `65b71142308d9d8dbff0fe5c7380e943d1ba0ca1877a2773038ca13880efb3fd` |
| vendor/calloop/src/sys.rs | `8a34e0110eb35fc763bb4f54a0612d9b187a06c16a350f6ec018bbd43c57634d` |
| vendor/calloop/src/token.rs | `aab917dbe4eee5c8b4e6ba53388b446d7f2bda4d94e0504a2a55de57d53ee48c` |
| vendor/calloop/tests/signals.rs | `51adb8b33f39d65abe6dc691d3334f0bd7996ab63a2fd86d23dc038e05a9196e` |
| vendor/dirs-sys/.cargo-ok | `afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91` |
| vendor/dirs-sys/.cargo_vcs_info.json | `5e6863143ff981280627f680b6720b7649422480faed6a05c3e491d9594a32c2` |
| vendor/dirs-sys/.github/workflows/ci.yml | `905b81ce1ca8866fce44604c526d4356eb4390d2095051ef024194094bed8e0c` |
| vendor/dirs-sys/.gitignore | `e01ee9f9fd6d4ead8f4b718b127eca56023b4d771b6b48ca68ead2abe8f6f0ac` |
| vendor/dirs-sys/CONTRIBUTING.md | `f2492580980fa01e2e13b404950b3d70346c774d8e88723cb49b0c9824c64805` |
| vendor/dirs-sys/Cargo.toml | `7cd82d0d0b8d0cdb5bc4f412339c313de99095510a84406f8f3d519c0fb9b0dc` |
| vendor/dirs-sys/Cargo.toml.orig | `4b51207fe73535bf0727b972a75dc84767e9dc4bb49002df398821335a1c7817` |
| vendor/dirs-sys/LICENSE-APACHE | `d3174ad63e721d4c9dccb8ad4320848992d314369bc46319720b5802c9153fe9` |
| vendor/dirs-sys/LICENSE-MIT | `6a2e0ade09a7d5f816f11566fee2b151b32235a7fad52b41d49cce96f833c1a9` |
| vendor/dirs-sys/PATCHES.md | `9ca5f28089dd6c08e4608c687094a321ab9974459259cb00cd73e59db265ad20` |
| vendor/dirs-sys/README.md | `0b15d135ff8743513430d0861954a271c38c0ac3e3e025eb25b59cc7d0437278` |
| vendor/dirs-sys/src/lib.rs | `a35ed53af4bae60254e0c688978a7d4d79a019f80f4c50e3a86c50104c88423e` |
| vendor/dirs-sys/src/xdg_user_dirs.rs | `7a4da3467fb15dd1ce49bcecfc2b27898e915b824954e067a8d05f2c75c93fa4` |
