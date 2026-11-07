
## Acceptance attempt — 2026-11-07, round 4

Overall verdict: **OPEN**. This record distinguishes completed contract checks
from remaining integration and visual work; no unapproved difference is accepted.

The oracle is MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, executable
SHA256 `21303df6158fbc7c856aa39525a9bacff31a258f05d03f1b6602de6aeb8e52e6`.
Candidate code is `cc43ae6825511b907db43f932ba12b349fc479d4`; documentation
commits after it contain receipts only. Captures, scripts, fixtures, calls,
metrics, original PNGs, comparison figures and closure receipts are retained in
the private `workshop-parity-20261107` archive.

The isolated recipe uses owned native noded/settingsd/compd, separate runtime
and application state, canonical `workshop` sequentially, 1040×680 client
pixels at scale 1, Studio Dark, CSD/right captions, bar menus and row actions.
Baseline typography is Inter 400 at 14.667. No live desktop settings, VT or
installed executable changed. Every native run retains its failed evidence
and a `finally` cleanup receipt.

### Functional and Bus evidence

`bus-cases.GCVOOt` matched **99/99** structured cases against gold, including
complete HELP/descriptors, payload/null/type refusals, Format, raw return codes,
errors and whitespace, Unicode byte bounds, observation provenance and quit.
Generated IDs, sequence numbers, timestamps, elapsed time and wire sizes remain
in the raw receipts and are normalised only for comparison. Filtered Traffic
retains payloads, outcomes and request/reply correlation.

`macro-flow.3TO8DF` matched **25/25** macro protocol cases. The independent
target is the pinned native Dopus, running on the owned broker. Checks cover
global and app builtin copy, user-tier shadowing, moving to a running app,
execution and exact UTF-8 output, stop/exit 143, offline catalogue/get/put,
move back to global, removal and rc10 UNAVAILABLE for offline execution.
Only private root path names and run-start timestamps are normalised.
Duplicate headless registration now exits 1 without a texture-delta panic;
the original primary PID survives. CLI malformed arguments exit 1.

Native panel evidence includes form selection/typing/Start, nine identical
agent lifecycle/configuration/permission backend calls, automated permission
refusal without dispatch and rc17 USE_AGENT for an attested MCP caller.
Opaque tool JSON retains numeric u64 digits and numeric-looking strings.
The native clipboard contains identical reply bytes. Menu Clear and
Ctrl+Shift+F dispatch the gold operations. In-process policy remains in Mix.

The 2,024-row installed-Mix benchmark records 27.7–28.4 ms tree processing and
80.1–81.3 ms full projection. This is pure projection timing, not embedded Viu
end-to-end latency. The seed scene hundred-row timing assertion is unchanged.

### Persistence and remaining integration

Native restart checks preserve selected command and expanded parents,
collapsed Details, panel shares, Pause, per-window theme and native placement.
Bodies and replies remain passive. Selection visibility takes priority over a
saved raw scroll position, using the gold forget-scroll operation. A separate
seeded-offset probe returned zero on gold; it is retained as diagnostic evidence
and does not establish a port difference.

The accepted shared native_document editor and New macro field support have
not been merged into this branch. The standing job assigns checkpoint merges
to Opus. The consumer and editor height reservation are present, but complete
macro UI/New-dialog and Ced/Dopus window effects remain open until integration
and native repetition. No duplicate editor was introduced.

### Visual and live Prefs evidence

Debug pair `pair-debug.mcrgH4` has whole-image and all-region SSIM/RMSE
distortion 0 for Command, About, Shortcuts and Agents. Mesh static details,
header/tree/Traffic/status are also 0. Its scrollbar follows truthful live
counter rows below the viewport (S2): details SSIM distortion 0.000285207,
whole 0.000157695. No gold readings were fabricated.

Native menu recapture after repair has exact File/Edit/View/Bus/Macros images.
The Help difference in the earlier capture is the blinking search caret;
the release recipe resets its input phase before capturing.

Live Prefs variants cover mode, scheme, style, accent, density, roundness,
font family/weight/size, mono family, text scale, stroke, application overrides,
auto daylight state and tune.title_control_spacing. Accepted revisions,
unchanged PIDs, invalid refusal and authority loss/recovery are recorded.
Release repetition and inspected region metrics are appended below.

Metrics use ImageMagick 7.1.2-35 **distortion**, lower is better and 0 means
identical. Regions are header [0,0,1040,38], tree [0,38,354,570], details
[354,38,686,570], Traffic [0,608,1040,42], status [0,650,1040,30]; Mesh static
details [354,38,670,570] exclude its data-dependent scrollbar. No arbitrary
numeric acceptance threshold is used.

### Exact release checkpoint

CBC `20261107-105013-9650fd5f-a1` passed at `cc43ae6`: fmt, all-target
check, tests, clippy, debug and release builds, with no worker diff. Its full
saved log was read: `20261107-105101-cbc3-cc43ae682551-20261107-105013-9650fd5f-a1-924dfcdb.log`
in the private CBC log directory. Release SHA256 is
`c696f723eb33d5517c6de48941c36147ecec12b34755c8d0950238c9a2b461b5`;
the source/profile/hash receipt under `/build/es-artifacts/<sha>/viu.json`
was fetched and verified before these runs.

| Release evidence | Result |
| --- | --- |
| release-bus.Qvag7k | 99/99 structured comparisons match |
| release-macro.7TtyCS | 25/25 macro comparisons match, including execution/cancellation/offline |
| release-pair.nCJfXw | Command/About/Shortcuts/Agents: whole and every region 0; Mesh static regions 0, S2 scrollbar as above |
| release-native.WX8KZr | File/Edit/View/Bus/Macros: whole and every region 0; native clipboard, menu Clear, shortcut, Pause and placement callbacks pass |
| release-help.6MpXkw | Gold/candidate phase 4 hashes match; the earlier Help difference is only caret phase |
| release-panels.rTqIoa | New/session/expanded u64 tool/refusal: whole and every region 0; windowed activation keeps the existing PID |
| release-prefs.OkkQg5 | Compact diagnostic absent; seven of nine captured states are exact in every region; invalid input and authority restart checks pass on unchanged PIDs |

The two live Prefs raster cases were inspected on release originals, montage,
diffs and native UI trees. Font-scale details SSIM/RMSE distortion is
0.0000456998/0.000657549 (full 0.000025268/0.000488941); the Clear body edge
is at 682.5 versus 682.46875 logical pixels. Auto overrides header distortion
is 0.00172234/0.0164557 (full 0.0000962486/0.00389003); the affected icon
rectangle differs by 1/32 pixel and caption height by 0.00625 pixel.
Other regions are exact. These are fractional raster-edge differences within
the method's anti-aliasing allowance, not omitted controls or an arbitrary
score waiver. Compact, base, auto-night, both caption sides and authority
absence/recovery are exact. No C1 deviation is claimed.

All six principal release suites confirmed all 44 owned PIDs stopped; the
additional Help run has its own cleanup receipt. Final scripts are copied into
their run directories. The agent fixture SHA256 is
`cc0b5f74c401dd7df96a5bed93a8568a9ca53edc66b126a2f8355152e344b211`;
the Bus fixture is `118fed3aec43fa0202a2db11e2575667ca0ac33fe2de5abdffbd3c56d397185a`.

The expanded-bottom Traffic capture exposed a separate sizing error: its
fraction used content height instead of gold's full-window height. The
reusable reference-extent repair is in `cfa94ff`, validated in `5a9acc5`;
native recapture and a fresh exact-source gate are in progress. This does not
invalidate the already recorded release evidence, but it is not yet closed.
Overall acceptance remains **OPEN**, also awaiting Opus's shared editor/New
field integration and the macro UI/window-effects repetition.
