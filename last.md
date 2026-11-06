# Workshop parity round 3 — in progress

Pinned gold: 8779f33284b7d3f1178c3d57a699efb9b0867b22. W1 follows gold; no proposed deviation. Shared frame, full Mix feature implementation, native panels, bounded traffic/Mesh, agents/macros and passive saved arrangement are implemented. Final native acceptance remains open.

CBC 20261107-085318-997cbc39-a1 passed at 442801a88f445642254804f92b45c45c6e2b8f58: Workshop and ports integration tests, native bridge/launch tests, all 72 scene tests and debug binary. Complete log: ~/.cache/mixos/cbc-build/20261107-085318-cbc2-442801a88f44-20261107-085318-997cbc39-a1-36e86fde.log. Returned formatting applied in this checkpoint. Earlier 20261107-084148-2f27ec84-a1 hit the known 100-row parse wall-clock flake; unchanged rerun 20261107-084321-bf2eb64e-a1 passed. No test relaxed.

Private evidence: ~/.viu/_var/workshop-parity-20261107/. Five paired native 1040×680 captures completed in pair-debug.lvTAIy (command/About/Shortcuts/Mesh/Agents), exact debug source 442801a. Header and status were already pixel-identical in the previous five-state run; content inspection found extra selection scrolling and a short collapsed Traffic dock, repaired in this checkpoint. These captures are diagnostic, not release acceptance.

Native Bus run bus-cases.KJVSHu records 99 paired cases: 97 structured matches, action ordering repaired here; remaining traffic comparison includes true broker IDs and wall-clock timestamps requiring explicit field comparison. Includes null/type refusals, Format, UTF-8 limits, raw nonzero replies, observation provenance and quit. Full HELP descriptors match. All owned native processes stopped; closure receipts retained.

Open: regenerate/inspect all five content differences, native persistence restart checks, agent/macro panel interactions and full Bus cases, live Prefs matrix and authority recovery, exact release/full gate, final acceptance record. Shared native_document editor/new-macro field support awaits Opus merge of accepted shared editor checkpoint d665443 from lab/gold-parity; request sent while independent work continues. No desktop/VT/install changes.

Branch lab/parity-workshop; head SHA 442801a88f445642254804f92b45c45c6e2b8f58 (tested code; current receipt commit contains pending repairs); OPEN content regions, native persistence/panels, full Bus/Prefs and release acceptance; CBC jobs 20261107-082936-25ef644a-a1 PASS, 20261107-084148-2f27ec84-a1 FLAKE, 20261107-084321-bf2eb64e-a1 PASS, 20261107-085318-997cbc39-a1 PASS.
