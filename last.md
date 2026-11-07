# Email parity round 3 — OPEN

Branch lab/parity-email; code checkpoint 3140dbebeb93cb62aee1c7dbad10eb243c7cb557. No installation or desktop changes. Gold pinned 8779f33284b7d3f1178c3d57a699efb9b0867b22.

Release CBC 20261107-111355-394fcb63-a1 on cbc2 passed at source 27738f785b1e1ac5dfe3997b32c1d4fbb3017d27. Verified release SHA256 4a655372a221a7aad903e5218c2f372bc694cf1c61615cbeb9c5ee747f0473d9. Actual-host 2,000-distinct-date bulk projection, private round2/native.A7CLGn: three warm-ups, 21 retained samples, median 13.7523 ms, p95 19.1767 ms, p99 21.4132 ms. ACK timings retained separately. Fixture SHA256 1ca642d313c13d9a6eb58c086f51336650076ed2897d2a18f0e94b43e36cd780. All owned processes stopped. This measurement attests that source/byte cohort, not later changes.

Real-maild native round2/native.3vSdbG: 85 requests passed at a38cd5af0eaf177c52242fe78bdb95e3da771aec, debug SHA256 29b2e67b9c78e15f52d38970dafc62b569a333dfa624fabcde1dc5e999a75d76, CBC 20261107-114349-8a1c199d. Includes account/mail/PIM CRUD, MIME send and delivery/body readback, attachment bytes/collision saves, two same-PID composers, native edits and Keep/Discard, reply/forward creation, attachment add/remove and pristine close. All five owned PIDs stopped. Intra-app traffic stays in one runtime with local intents.

Matched round2/pair.js7Buv: calendar and contacts header/places/list/reader regions pixel-exact; status retains one short right-pane border difference (SSIM distortion 0.000348970, RMSE 0.00121496). Originals, montages and diffs inspected. New captures exposed initial main-table column-width instability, still open; earlier exact inbox/dark-reader/light-reader round2/pair.Fndyun remains historical evidence at 0433f98.

CBC 20261107-115343-0e2890be passed focused checks/debug at 2a4a7e59ccbbab9d134c06a114410c73789dfab3; formatting applied. Matched round2/pair.OB5gwe composer header and form regions are 0/0 SSIM/RMSE. Body/full remain 0.00456722/0.0261336 and 0.00439362/0.0226108. Originals, montage and diff inspected: an extra six-pixel spacer was identified and repaired next. Every owned process stopped.

Current code adds native PIM/triage forms, contact completions, draft restore/debounce/quit gates, document chooser/save/replace flows and reusable native command search. Native contact form input/submission and contact completions passed before Show exposed an unavailable embedded uuid() call; fixed to use the existing identity module. Expanded witness (restore/restart included) awaits the current build. CBC for 3140dbeb is queued behind the shared light worker. These repair checks do not replace the final full exact-source release gate.

Required open work: identity editing/stale reconciliation; account cache/migration/authority recovery; attachment previews/open/save-all and print; complete bulk selection/search/narrow/saved layouts; all Bus replies/refusals/stale/topic cases; live Prefs matrix; every-main-state release captures and final acceptance. No omission is approved, no deviation is proposed, no Fable question is pending.

lab/parity-email | code head 3140dbebeb93cb62aee1c7dbad10eb243c7cb557 | OPEN: inventory, Bus, Prefs and final visual/release acceptance; release projection 13.7523 ms median | CBC 20261107-111355-394fcb63-a1, 20261107-114349-8a1c199d, 20261107-115343-0e2890be; 3140dbeb repair queued
