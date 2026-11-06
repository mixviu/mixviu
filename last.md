# Workshop parity round 2 — in progress

W1 is resolved: pinned gold wins. Format matches gold JSON pretty-printing,
65,536-byte body refusal and 1,000,000-byte UTF-8 reply bound. Gold/inventory
disagreements are corrected without stopping. No deviation is proposed.

The shared frame checkpoint is committed (93c1578, receipt 47d91e1). The
feature pass now includes the node/service/command tree, asynchronous discovery,
raw native replies, deferred requests, macro storage and routing, agent session
and transcript policy, bounded traffic, Mesh tallies and observation/probe wiring.
Application policy remains in Mix; reusable native facilities share the existing
in-process runtime and Bus connection. Full panels and feature acceptance remain
in progress. Other jobs still own the shared editor and bulk primitives.

CBC 20261107-074633-c12acdca-a1 at 548c177 passed: five Viu integration tests,
four native bridge tests and 72 scene tests; debug binary SHA-256
0250265519b51d810aebaf3704b11be299d541171e86426d3128f2fe69d23cba.
Earlier jobs: 20261107-065902-b38ad05a-a1 passed;
20261107-071445-a62d3e27-a1 found compile errors subsequently fixed;
20261107-073743-b4f1a848-a1 found a test reply outside a handler, fixed.
The next gate includes every policy test in the embedded runtime and launch tests.

Private exploratory native smoke headless.sXZzWK passed on an owned noded:
discovery, selection, Format, native fixture call/reply, macro/agent reads and
clean shutdown. It used debug 548c177 with revised Mix sources and is diagnostic,
not acceptance evidence. It exposed an unavailable sum() helper, now replaced
with reduce(). Private artefacts: ~/.viu/_var/workshop-parity-20261107/.

Pending: complete Mesh/macro/agent presentation, saved arrangement, full Bus
negative/race checks, exact release gate, live Prefs and matched captures/diffs.
Pinned gold is 8779f33284b7d3f1178c3d57a699efb9b0867b22. No install/VT work.
Overall NOT ACCEPTED; work continues without a Fable question.

Branch lab/parity-workshop; head SHA 548c17772198257e73fb4ee7f071c31d49fffe5d (previous tested checkpoint; current checkpoint is the commit containing this record); features/Bus incomplete, headless diagnostic pass, Prefs pending, SSIM/regions unmeasured; CBC jobs 20261107-065902-b38ad05a-a1 PASS, 20261107-071445-a62d3e27-a1 fixed, 20261107-073743-b4f1a848-a1 fixed, 20261107-074633-c12acdca-a1 PASS.
