# Deck composition specimens

`deck.mix` composes ordinary scene nodes. `deck-pages.mix` regenerates the
twelve standalone review scenes in `deck-pages/`; each uses a 1600 × 900
logical canvas. The named source tokens live in
`libraries/design/deck.tokens.mix`. These are port specimens, not a finished
deck app or a declaration of HTML parity.

```text
mix /home/user/Projects/mixviu/apps/gallery/deck-pages.mix
```

The constructors cover typography, cards, chips/groups, code, terminal rows,
flow nodes/arrows, trees/badges, count-up stats, mock panels, brand/topic cards
and a rail sample. The generic `text.spans` port keeps inline runs in one
text layout, including monospace, colour, strike-through and highlighting.
No new scene kind is introduced. Explicit span colours come from the Mix
theme; omitted colours resolve semantic design roles in the host.

The terminal sample is presentation data, not a PTY. Numeric stats use the
generic `text.number` scalar track; captions remain ordinary text nodes.
Completed and constant tracks request no frames. Card and flow outlines use
the generic box outline ports. Cursor backgrounds, full-width code line
highlights, exact type metrics and the rail's fixed placement remain work.
Raster shots/callouts, other motion effects beyond the existing entrances,
and Gallery navigation integration remain work.

The rendering tests exercise each specimen through `SceneView`. The ignored
`render_review_slide` test accepts a private scene staged by a CBC job at
`temp_dir()/mixviu-deck-review/slide.scene.mix` and writes a 3840 × 2160 PNG
beside it. Real episode scenes and captures stay outside this public tree.
