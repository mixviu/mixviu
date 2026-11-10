# viud agent seat opt-in — 2026-11-10

PR: https://github.com/mixviu/mixviu/pull/56 (against main, stacked on #54; unmerged).
Branch: feat/viud-agent-seat-optin. Base 496f627b is preserved.
Tested source head: 44dc62b2bf93b4c2f55fca735f874b2e067f50ca.
The final receipt commit changes only last.md; the PR exposes the current head.

## Design choices

- Keys: agent_seat.default=false, agent_seat.allow=["viu"], agent_seat.deny=[], applied by viud, page applications/agent-seat. Deny wins, then allow, then default. Lists use the registry's directory-name grammar for executable basenames (255 bytes, up to 256 entries).
- Current policy lives in comp-model::agent_seat behind a shared RwLock. The existing settingsd consumer sends it through the generation-fenced chrome Update. The shell change comparison includes agent_seat.*; settingsd publishes the new prefix topic and Prefs subscribes to it.
- Per-client decision lives in protocols::seat::state::Seat.agent_clients: Arc<Mutex<HashMap<ClientId,bool>>> shared with the seat filter. The first decision stays fixed for the connection; subsequent filter calls prune dead clients. Input checks and window snapshots read that decision. Credentials identify the executable through /proc/<pid>/exe; app IDs and directories are not policy inputs.
- Headless tests have ONE injection: MIX_TEST_VIUD_AGENT_SEAT_ALLOW, comma-separated executable basenames, read exclusively by the headless backend and registered in etc/env.conf.mix. A settingsd snapshot supersedes it. Test support allows the probe/app binaries and its own executable. In-process testkit clients always receive the seat with no override. Real sessions use settingsd only.
- Windows report agent_seat; named targets and output hit tests refuse seat_unavailable with the seat, window and setting to change. No fallback to seat0. Docs explain that policy changes apply the next time the app starts.

## Validation

- 20261110-144808-b0fca6c2-a1: **BUILD_RC=0**, full tools/ci.mix all at the tested source head. Saved log audited: 620 summaries, 10,688 passed, zero failed, 101 ignored; 198 environment names with zero problems; 153 Mix lint files with zero regressions. Rustfmt, clippy and the viud dependency check passed.
- 20261110-151842-e9270fac-a1: **BUILD_RC=0**, real render-node headless capture (9/9) and input (7/7), without settingsd. This includes the one-seat Firefox regression and seat_unavailable replies for named key/pointer targets and output hit tests. Built mix/mixd and set MIX_TEST_MIX_EXE to the source-built interpreter.
- The renderer-free external-client test passed at the same SHA: denied sees only seat0, allowed sees both. Existing testkit clipboard and interactive suites passed in the full gate.
- Rustfmt was run on the worker and its printed diffs applied locally. Cargo.lock's dispatcher dependency change also came from the worker.

Earlier CBC attempts, all resolved or superseded:
- 20261110-141334-80bc4248-a1: wrapper BUILD_RC=0; nested check=101 (missing map type annotation), fixed.
- 20261110-141541-819a736c-a1: BUILD_RC=101 (missing WaylandFocus import), fixed.
- 20261110-141725-0718304c-a1: BUILD_RC=1 (nested default 30-second timeout during compilation). Corrected subsequent wrappers to use explicit timeouts.
- 20261110-141941-b7a0ec7f-a1 and 20261110-142450-906ee564-a1: BUILD_RC=143, explicitly cancelled as superseded. The latter exposed the missing settings topic route, fixed before the green full gate.
- 20261110-150848-f8642d43-a1: BUILD_RC=101, private broker startup failed with the installed interpreter; corrected invocation passed as above.
- Unstarted headless jobs 20261110-141952-456c8a60, 20261110-142501-3890a14a, 20261110-144745-0b9827a4 and 20261110-150813-5d3fe0d9 were explicitly cancelled as their invocations were superseded.

## Left undone

- Prefs GUI deferred under item 6: the current list controls select fixed catalogue entries; executable allow/deny editing needs a new editor. No partial page was added. All keys remain writable through settings.get/settings.apply; docs include the Bus shape and restart behaviour. The page and its restart note need a second PR.
- Pre-existing app-test fixture is absent: headless/tests/app.rs requires examples/ced-headless-probe and a ced Cargo package, neither present in this checkout. That ignored test could not be run against exact-source artefacts. No stale worker binary or alternate app was substituted.

No installation, deployment, desktop/VT changes or merge performed.

---

Previous hand-backs follow unchanged.

# Dependency bundle — 2026-11-10

PR: https://github.com/mixviu/mixviu/pull/53
Branch: `deps/2026-11-10`.
Base: `a0a5bef3a9d7b6b34aa33cf5a9209f9fe1469587`.
Pushed source head SHA tested by the full gates:
`5948680add0ab1fd53142f9fd3c89fc9a69b33a3`.
The subsequent receipt commit changes only this file; the PR exposes its current head SHA.

Supersedes #18 #19 #20 #25. All four remain open; no merge or closure performed.
PR author verified as `app/mixviu-agent` (`is_bot: true`).

## Versions

The table lists resolved versions; tungstenite and SCTK already had both
versions on main, so their direct dependency requirements changed while the
older transitive lines remain. PR #25's current lockfile resolves arc-swap
1.9.4, beyond the 1.9.3 in its description. SQLite stays on main's versions.

| Package | Before | After |
| --- | --- | --- |
| hyper | 1.11.1 | 1.12.0 |
| unicode-normalization | 0.1.25 | 0.1.26 |
| arc-swap | 1.9.2 | 1.9.4 |
| blake3 | 1.8.6 | 1.8.7 |
| serde_json | 1.0.151 | 1.0.154 |
| thiserror | 1.0.69, 2.0.20 | 1.0.69, 2.0.21 |
| tokio | 1.53.3 | 1.54.1 |
| uuid | 1.24.1 | 1.29.0 |
| bitflags | 1.3.2, 2.13.1 | 1.3.2, 2.13.2 |
| cc | 1.4.3 | 1.11.0 |
| crossbeam-channel | 0.5.16 | 0.5.17 |
| proptest | 1.11.0 | 1.12.0 |
| crc32fast | 1.5.1 | 1.5.4 |
| indexmap | 2.14.1 | 2.14.2 |
| jaq-core | 3.1.1 | 3.1.2 |
| jaq-std | 3.0.3 | 3.0.4 |
| unicode-width | 0.2.2 | 0.2.3 |
| log | 0.4.34 | 0.4.35 |
| zeroize | 1.9.0 | 1.9.1 |
| tokio-util | 0.7.19 | 0.7.20 |
| candle-core | 0.10.2 | 0.11.1 |
| candle-nn | 0.10.2 | 0.11.1 |
| candle-transformers | 0.10.2 | 0.11.1 |
| tokenizers | 0.22.2 | 0.23.2 |
| tungstenite | 0.26.2, 0.29.0 | 0.26.2, 0.29.0 |
| smithay-client-toolkit | 0.19.2, 0.20.0 | 0.19.2, 0.20.0 |
| rusqlite | 0.41.1 | 0.41.1 |
| libsqlite3-sys | 0.39.1 | 0.39.1 |

## Repairs and foreign-code inspection

SCTK 0.20 requires RawModifiers in update_modifiers and a repeat_key callback.
Both toolkit and Viu layer handlers now accept the new argument and forward
compositor repeats as repeated presses, stopping the local repeat timer first.

candle-nn alone failed compilation: Device/DType came from candle-core 0.10.2,
while VarBuilder came from 0.11.1, and candle-transformers expected the old
builder. Align core/nn/transformers to 0.11.1 and tokenizers to 0.23.2.
No update was dropped.

Base has no tools/foreign_gate.mix. Inspected changed registry crates through
locked metadata and their actual build-script sources. No newly introduced
crate compiles C/C++, and no new -sys crate was introduced. Existing foreign
debt remains, including blake3's cc build edge and onig. Tokenizers defaults
stay disabled with esaxx_fast off; its existing onig feature stays enabled.
SCTK 0.20 was already locked on main. Workspace cargo tree -i confirms desktop
consumers; scoped inverse trees for mix and mix-shell report no matching
package. Their full normal/build trees contain no desktop crate.

cargo deny check passes without policy changes. It reports wildcard and
yanked-dependency warnings, unused licence allowances and now-unused zip
advisory exceptions. The new Candle, tokenizers and fancy-regex versions were
included in the final foreign audit.

## CBC receipts

All jobs used --repo mixviu --worker any --lock-wait 7200 at pushed source
commits. No rc75 occurred. Resolution jobs printed their full diff, restored
their own worker tree, and the printed patches were applied locally.

| Job | SHA | BUILD_RC | Result |
| --- | --- | --- | --- |
| 20261110-075257-3d6b46c2-a1 | a0a5bef3 | 1 | Resolver harness used reserved function name; no edits ran. |
| 20261110-075313-b4ead392-a1 | a0a5bef3 | 0 | Resolve requested bundle, print and restore diff. |
| 20261110-075438-d1b9024f-a1 | fa8adaac | 1 | Audit harness treated absent SCTK in Mix as failure; fixed harness. |
| 20261110-075446-52822086-a1 | fa8adaac | 1 | Full gate caught SCTK keyboard API breaks. |
| 20261110-075516-0be8e8c3-a1 | fa8adaac | 0 | Foreign audit and cargo deny check. |
| 20261110-075656-dbfba9a3-a1 | 17c06497 | 1 | Full gate caught mismatched Candle tensor and builder types. |
| 20261110-075745-73d0af8d-a1 | 17c06497 | 0 | Align embedding stack, print and restore diff. |
| 20261110-075833-ad85bf35-a1 | 5948680a | 0 | Full ci.mix all passed: 10,640 Rust tests passed, 100 ignored; app checks and lint ratchet passed. |
| 20261110-075835-7a306a11-a1 | 5948680a | 0 | Final foreign audit and cargo deny check passed. |
| 20261110-080720-35129d8b-a1 | 5948680a | 0 | Full ci.mix all with explicit CARGO_BUILD_JOBS=2 passed: 10640 Rust tests passed, 100 ignored; app checks and lint ratchet passed. |

Earlier full gates used the CBC default, which caps jobs automatically only
for gold builds. A final explicit CARGO_BUILD_JOBS=2 full-gate rerun supplies
the required cap within both heavy workers' release_jobs limits.

Live Wayland/compositor repeat behaviour and real model inference were not
tested. Existing hardware tests remain ignored. No deployment, installed
prefix, desktop or VT changes. Code-owner and independent agent reviews are
pending; no review approval is claimed.

Final saved logs read:
- `20261110-075834-cbc3-5948680add0a-20261110-075833-ad85bf35-a1-fe1707d8.log`
- `20261110-082608-cbc3-5948680add0a-20261110-075835-7a306a11-a1-74ceae22.log`
- `20261110-082616-cbc3-5948680add0a-20261110-080720-35129d8b-a1-dd628ada.log`
