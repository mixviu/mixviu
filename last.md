# Workshop parity — Fable question W1 (2026-11-07)

Status: **STOPPED for the job brief's inventory-disagreement ruling**.
No frame, feature, Bus, Prefs or visual parity is claimed.

Inspected candidate: lab/parity-workshop at
22f4878d954e67c9141a44140aa324511eeb750a (the requested base).
Pinned gold: 8779f33284b7d3f1178c3d57a699efb9b0867b22.
Gold evidence was read with git show at that revision, independently of the
current MixOS checkout. This documentation-only receipt's commit is HEAD.

## W1: resolve the inventory's existing Format/reply deviations

Question for Fable via Opus: should this round replace the bounded specimen's
compact Format and shortened reply rendering with the pinned gold contract,
or retain either as an explicitly approved deviation?

Options:

1. **Recommended: match gold.** Pretty-print Format with gold's validation
   and size behaviour; render replies with gold's headers and JSON formatting,
   using its 1,000,000-byte UTF-8 boundary and truncation marker. Correct the
   inventory to distinguish historical specimen behaviour from the target
   contract, then resume shared-frame adoption and the full port.
2. Retain compact Format and/or the 900-character reply display. This requires
   explicit Fable approval and rationale, then paired captures and functional
   evidence. Neither difference is accepted.

Evidence:

- docs/ports/workshop.md calls “JSON body editing, clear and format” ported,
  but records compact JSON instead of original indentation. Its “Reply display
  bounds” row records a smaller 900-scalar budget and says raw whitespace/error
  headers are not preserved.
- Candidate apps/workshop/app.mix Format intent uses
  json_encode(json_parse($state.body)).
- Candidate apps/workshop/core.mix call_outcome serialises the parsed reply
  again and retains substr($answer,0,900), marking longer replies truncated.
  These are existing specimen differences, not proposed new limits.
- Pinned gold apps/workshop/crates/inspector/src/model.rs lines 13–14 defines
  BODY_LIMIT = 65_536 and REPLY_LIMIT = 1_000_000.
- Pinned gold model.rs lines 161–182: pretty uses
  serde_json::to_string_pretty and preserves non-JSON text; bounded cuts on a
  UTF-8 boundary and puts the truncation marker on its own line.
- Pinned gold apps/workshop/crates/inspector/src/engine.rs lines 622–639:
  format_body validates first, uses model::pretty and retains the original
  body if the formatted result exceeds BODY_LIMIT.
- Pinned gold engine.rs lines 1591–1609: reply rendering includes target,
  command, rc or transport-error headers before model::bounded.

The standing brief says to correct inventory disagreements and continue.
The explicit per-app job instead says “Write a Fable question in last.md
(options + evidence) whenever gold and the inventory disagree or a deviation
is needed, and stop there.” This handoff follows that explicit instruction.
Answer and rationale: **pending**; no deviation approved.

## Evidence and next checkpoint

No implementation changed. No CBC job submitted. No app, broker, settings
authority or compositor launched; there are no owned processes to clean up.
Workshop binary hashes, captures, SSIM/region metrics, Bus compatibility and
live Prefs witnesses have not been collected. Inherited Monitor acceptance
does not attest Workshop. Overall verdict: **NOT ACCEPTED**.

After the ruling, resume shared-frame adoption and its commit/push checkpoint,
then complete features/Bus/Prefs and isolated matched captures according to
docs/ports/parity-method.md. Feature and visual checkpoints remain outstanding.

Branch lab/parity-workshop; head SHA 22f4878d954e67c9141a44140aa324511eeb750a (inspected code; receipt commit is HEAD containing this file); frame not adopted; features/Bus/Prefs unverified; SSIM/regions not measured; CBC job ids none; Fable W1 pending.
