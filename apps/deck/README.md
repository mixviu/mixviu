# Deck

Deck text uses the measured HTML faces: Liberation Sans (400/500 Regular,
600/700 Bold), Noto Sans Mono (400 Regular, 500 Medium, 600/700 Bold),
and Noto Sans Regular for missing glyphs. No synthetic weight is used.
Gold ships six unmodified files and OFL licences in `share/fonts`.
Toolkit and Viu register these optional named families; desktop defaults
remain Inter and JetBrains Mono. A design typography role or app
can select them by family name. Content settings retain Inter/JetBrains Mono.
Scene text accepts `font` and CSS `weight`;
deck supplies its sans/mono families for every text node, including spans.
The host binds scenes against the context's loaded families before replacing
the current scene. Model patches use that bound set, with no filesystem stats.
Missing or unknown families refuse there rather than substituting; the painter
also checks binding before drawing. Structural parsing alone does not bind fonts.

`cargo test -p lib-viu --test deck deck_font_identity -- --nocapture` (CBC only) paints
the roles and compares their glyph atlas coordinates with isolated faces,
reporting the files' internal PostScript names. Its renamed-file control checks
the production scene refusal against the context's bound families. This is the
font acceptance check; no SSIM claim is made. `tools/deck_font_probe.mix` prints
the source episode's 16 sections and adjacent sting to PDF using supplied
templates, then records pdffonts output and hashes. See
`provenance/deck-font-measurement.json` for the 17-slide measurement.

Run `viu run apps/deck`. The ordinary window starts full-screen. The host fits
the 1600 × 900 logical canvas into the output, centred with letterboxing;
scene dimensions never depend on output resolution. Application behaviour is
Mix. Scene widgets and motion sampling use the native Viu facilities.

## Data format: deck.v1

`deck.open` accepts an absolute path to JSON (`.json`) or Mix strict data.
No executable source or converter is involved. The top-level object is:

```json
{
  "schema": "deck.v1",
  "slides": [{
    "id": "intro",
    "kind": "slide",
    "steps": 2,
    "scene": {
      "root": {"widget":"column", "children":["title","answer"]},
      "title": {"widget":"text", "text":"Before", "size":64},
      "answer": {"widget":"text", "text":"After", "size":64}
    },
    "build": {
      "title": {"b":0, "until":0, "anim":"left"},
      "answer": {"b":1, "delay":0.2, "anim":"up"}
    }
  }]
}
```

Each `scene` is the node map from a scene document's strict-data fence.
The app supplies the envelope. Ordinary scene schema validation still applies.
Scene data cannot contain bindings or `on_*` handlers, including nested data.
Image nodes accept known `icon:` names or PNG/JPEG paths, relative to the deck
directory or absolute within it (including `assets/`). Open checks the opened
file's resolved path; outside paths and escaping symlinks refuse. Files must
be regular and at most 32 MiB, dimensions at most 16,384 per side and 32 Mi
pixels. The raster decoder downsizes to 3,840 on the long side. Pixels are
pinned at open, so source replacement/removal cannot change a later goto.
The shared store caps handles at 64 and resident pixels at 512 MiB; active
and candidate decks share those budgets. Refused opens release candidate
handles and preserve the active deck. Use `w` and `h` for the display size.
The static `brand:cyclops` uses the approved intro geometry; draw, blink and
spin remain separate motion work.

Grow rows allocate each cell its weighted share of the remaining width.
Nested content wraps where supported and clips at its cell edge when it
cannot wrap; it cannot move the next card outside the canvas.
`kind` defaults to `slide`; `sting` accepts a composed brand slide with the same
timing contract. A sting needs no HTML or separate renderer.

`steps` counts build steps starting at zero. A build entry identifies a scene
node. `b` defaults to zero; `until` is inclusive. Without `until`, an element
keeps its layout space before its build. With `until`, it leaves layout outside
its step range. Unlisted nodes are always visible. Manual navigation samples
the settled step and ignores entrance delays. Timed playback samples the
entrance at `scene_time - beats[b] - delay`; it never restarts an entrance on
each publication. Final playback freezes the last scene's end sample.

For the reference title (`left`, beat 0.9, delay 0.15), both the native
0.7-second preset and `oracle/reference/deck.js`/`anim.css` start at 1.05:
opacity is 0 at 1.0/1.05, 0.287865 at 1.1 and 0.978681 at 1.5 seconds.
The core/native-port test pins these samples. There is no extra lead in the
entrance sampler. Live playback defines zero at its first `viu.presentation`
timestamp and paints the resulting sample on the following draw. A recorder
marker placed before `deck.play` is a different origin; its RPC/admission,
first-frame and draw latency must be measured before treating a recording's
1.1-second frame as app scene time 1.1. The historical recording did not log
that correlation, so its precise wall-clock offset remains unproven.

`anim` and `child_anim` support `fade`, `up`, `down`, `left`, `right`.
`delay` and `stagger` are seconds, each in 0..60. A stagger container gives
unconfigured direct children its beat and `delay + child_index * stagger`.
Explicit child build entries take precedence. Unsupported transforms must be
documented outside slide artwork; the loader refuses unsupported animation names.

Limits: 128 slides, 256 steps per slide, 4096 nodes per slide, 3600 seconds per
scene. IDs must be unique and 1..128 characters. Unknown deck/build fields
refuse. Errors have `error_code`, `message`, and optional `details.field`.
Messages identify the slide index, ID and offending field, for example
`DECK_RANGE: slides[0](intro).build.answer.b`. Native schema refusals are
wrapped in `DECK_SCENE` with the slide ID and native diagnostic.

## Bus

All commands target service `deck`. Parameters are objects.

| Command | Body | Result |
|---|---|---|
| `deck.open` | `{path}` | Validated deck at its first slide, step zero |
| `deck.goto` | `{slide, step?}` | Slide ID, optional step (default zero) |
| `deck.next` | `{}` | Next step, then next slide |
| `deck.prev` | `{}` | Previous step, then previous slide |
| `deck.play` | `{slides?, beats}` | Immediate state with a `play_id` |
| `deck.state` | `{}` | `{path, slide, step, playing, elapsed, finished, play_id}` |
| `deck.actions` | `{}` | Typed action catalogue |
| `deck.execute` | `{id, params?}` | Same command path as the direct verbs |

`slides` is an ordered list of slide IDs; omitted means all slides. `beats` is
one list per selected scene. Each list contains exactly `steps + 1` times:
every build's start and an explicit scene end. Times strictly increase and
are nonnegative. For a two-step slide, `[[0, 3.2, 8.0]]` shows step zero at zero,
step one at 3.2 seconds and ends at eight seconds. No narration duration is
guessed. `[[0.9,3.2,8.0]]` leaves configured builds unrevealed until 0.9
seconds; unlisted background/structure nodes remain visible. No empty build
step is needed. Poll `deck.state` until the returned `play_id` finishes; a
different ID or a nonfinished stopped state means another command replaced
the play. A valid open/navigation/play interrupts playback; refused commands
leave it alone. Caller cancellation does not stop an accepted play. The host's
180-second reply limit only covers command admission, so a play can contain
multiple scenes of up to 3,600 seconds each.

The generic host's `presentation.clock` drives `viu.presentation {time,serial}`
from the render clock. Mix samples elapsed time from that clock, never from
timers or a separate video clock. Playing or awaiting the final drawn serial
requests another frame. `finished` waits for that serial's client draw
acknowledgement; compositor recording and presentation acceptance are separate
evidence. Caller cancellation cannot disarm this final draw.

The generic presentation model also accepts `presentation.scene` (a complete
scene document) and `presentation.size` (two logical dimensions). This mounts
an unframed scene directly; it does not create or configure another window.
Local native ports are `scene.validate {source}` and
`motion.seek {preset,time}`. The latter samples the shared finite opacity and
translation presets at a supplied time, returning `{opacity,offset}`.

## Reference and checks

`examples/vfl-self-recording.json` is a hand-authored native adaptation of the
20 November 2026 “The desk records itself” episode, including all eleven source
sections and an opening sting. It preserves the historical story and measured
numbers. Infrastructure names are generalised. Paragraphs are condensed for
the available widget layout. The screenshot/callout/Ken Burns, count-up strip,
cyclops drawing/spin and underline effects are omitted and listed in the PR.
It is a reference adaptation, with visual differences from the HTML original.

`core-test.mix` checks format refusals, boundaries, navigation and deterministic
time sampling. `libraries/viu/tests/deck.rs` loads the real Mix behaviour and
paints every reference step headlessly. Its explicit offscreen evidence test
saves two 4K PNGs. Rust checks run on CBC only.

`mix apps/deck/check.mix check` is the scoped CBC driver: all lib-viu/viu
test targets, all-target clippy, fmt, core policy and gallery regeneration.
Its `fmt` mode returns a worker diff and restores the owned checkout.

The reference source is `examples/author.mix`; run it with Mix to regenerate
`vfl-self-recording.json`. It uses the 3.1 composition constructors from
`apps/gallery/deck.mix`, including cards, stats and chapter rail/footer, and
a record-strip compound composition of cells and a legend. No deck format
extension is needed. Unsupported shot/callout and extra motion are omitted
from the artwork and listed in the PR, never labelled on a slide.

Review captures use scene time **10 seconds**, beats **[0,3,15]**, for both
`before` and `recorder`. Chrome uses animated seeking, not still build mode.
HTML count-up has settled; the reference uses settled numeric values.

Gallery-reference API audit: this PR changes `app_host` presentation handling,
`bundle::Manifest.fullscreen`, and adds two `runtime::Local` command arms
(`scene.validate`, `motion.seek`). The retired standalone Gallery-reference
uses none of these: it constructs its own eframe host, imports only
`runtime::{Program,Runtime}`, `view::SceneView`, widgets and icons, and sends
neither new local command. Those imported APIs and implementations are
unchanged against main; no changed file is path-included by that app.
Its stale manifest therefore needs no dependent tests for this scoped change.
It is not reactivated or brought into the workspace. The owning lib-viu and
viu test targets, including the runtime tests, remain in the CBC scope.
