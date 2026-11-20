# Deck

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
`kind` defaults to `slide`; `sting` accepts a composed brand slide with the same
timing contract. A sting needs no HTML or separate renderer.

`steps` counts build steps starting at zero. A build entry identifies a scene
node. `b` defaults to zero; `until` is inclusive. Without `until`, an element
keeps its layout space before its build. With `until`, it leaves layout outside
its step range. Unlisted nodes are always visible. Manual navigation samples
the settled step and ignores entrance delays. Timed playback samples the
entrance at `scene_time - beats[b] - delay`; it never restarts an entrance on
each publication. Final playback freezes the last scene's end sample.

`anim` and `child_anim` support `fade`, `up`, `down`, `left`, `right`.
`delay` and `stagger` are seconds, each in 0..60. A stagger container gives
unconfigured direct children its beat and `delay + child_index * stagger`.
Explicit child build entries take precedence. Unsupported transforms must be
labelled in the scene as stubs; the loader refuses unsupported animation names.

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
| `deck.play` | `{slides?, beats}` | Retained reply at the end of the last scene |
| `deck.state` | `{}` | `{path, slide, step, playing, elapsed, finished}` |
| `deck.actions` | `{}` | Typed action catalogue |
| `deck.execute` | `{id, params?}` | Same command path as the direct verbs |

`slides` is an ordered list of slide IDs; omitted means all slides. `beats` is
one list per selected scene. Each list contains exactly `steps + 1` times:
every build's start and an explicit scene end. Times strictly increase and
start at zero. For a two-step slide, `[[0, 3.2, 8.0]]` shows step zero at zero,
step one at 3.2 seconds and ends at eight seconds. No narration duration is
guessed. A new valid open/navigation/play interrupts the previous retained
reply with `DECK_INTERRUPTED`; a refused command leaves playback alone.

The generic host's `presentation.clock` drives `viu.presentation {time,serial}`
from the render clock. Mix samples elapsed time from that clock, never from
timers or a separate video clock. Only playing or awaiting the final drawn
serial requests another frame. Final serial acknowledgement is a client draw
acknowledgement; compositor recording/presentation acceptance is separate
evidence and must not be inferred from it.

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
cyclops drawing/spin and underline effects are visibly labelled 3.1b stubs.
It is a reference adaptation, with visual differences from the HTML original.

`core-test.mix` checks format refusals, boundaries, navigation and deterministic
time sampling. `libraries/viu/tests/deck.rs` loads the real Mix behaviour and
paints every reference step headlessly. Its explicit offscreen evidence test
saves two 4K PNGs. Rust checks run on CBC only.

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
