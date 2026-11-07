# Email round 2 — work in progress

Branch lab/parity-email. Tested code 05aabb63c7332f84e9651decee790f16a0970c11; latest feature checkpoint 8e284d97b7e4 (CBC running). No install or desktop change.

Real mixd maild account: seven gold query replies match (accounts, mailboxes, list, search, get, calendars, books). Reader info matches except pane proportions. The 1040×680 inbox header, places, list and empty reader are pixel-exact; whole SSIM distortion 0.0000151023 / RMSE 0.000255193. Open-reader width/wrapping and a small status border still differ. Reader originals/montage/diff inspected; other image inspection remains. All six owned PIDs stopped. Private evidence: email-parity-20261107/round2/pair.GvbPD8.

Settings accounts/secrets, token session, live Email Prefs, hierarchy/native table, asynchronous read/triage/folder writes and PIM CRUD policy implemented. Full feature/Bus/Prefs/visual parity remains open. See docs/ports/email.md for current scope, precise hashes, regions and historical evidence. No Fable deviation proposed.

CBC 20261107-103429-36f6485a-a1 passed focused checks/debug at 05aabb6; saved log read, fmt patch applied. Modern 2000 distinct-date projection: 201.99 ms debug, above budget; bulk date decoding repair is now under test. Attachment save uses native binary transport and non-replacing final links, also under test.

Remaining: native composers/MIME send/identities/quoting/suggestions, drafts/documents/close gates, PIM views/forms, attachments/previews, account cache/migration/conflict recovery, remaining UI/actions/search/narrow/saved state, complete Bus and Prefs matrix, matched main states and final exact-source release acceptance.

lab/parity-email | tested head 05aabb63c7332f84e9651decee790f16a0970c11; implementation 8e284d97b7e4 | OPEN: full features, Bus, Prefs and reader/all-state visuals; inbox main regions 0 SSIM/RMSE | CBC 20261107-103429-36f6485a-a1; running 20261107-103855-abc251bf-a1
