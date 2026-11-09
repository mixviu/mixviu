# Embedded panel artwork

The scene renderer embeds the canonical `share/brand/logos/alien-cyclops.svg`
directly using `include_bytes!`; there is no copied brand original or runtime
filesystem lookup. It retains full colour (`symbolic: false`). The source at
this change has SHA-256
`e83ab64cd4cfb160c680bcc3261a447812f84ba88565a09d5dee4c0e6d6b7c4e`
(the 4 px outline and `#FF0000` eye from PR #51, which replaced #23).
The SVG carries Mark Constable's 2026 copyright and MIT OR Apache-2.0 licence.
Future changes to the canonical brand source are picked up at build time;
the test in `engines/viu/src/images.rs` makes this pin move with them.

Four catalogue additions are byte-identical copies of existing Lucide 1.54.0
assets in `share/icons/lucide/`, with source hashes already pinned in
`share/icons/manifest.conf.mix`. They retain the existing Lucide licence in
`libraries/icons/assets/LICENSE-lucide.txt`:

| Asset | SHA-256 |
| --- | --- |
| bell.svg | 394889e9631fe3a9a63be9a479a95b451b708e432d895978d21d9130b5ee5dbe |
| bell-ring.svg | 57d5f12e0f4f89dfeb909ccbac79e6a822cf9d4bcff5111d74a00c3565828f70 |
| volume.svg | d2062d9512891894f25ad9f7be9ec8546ab45125814aebacdc6122f6f56ae65c |
| volume-1.svg | e8520e1c57f8bf84579ecacf5463cd3ac55ab1597e62b2844ecd95d753233bf2 |

Low/medium/high volume use volume/volume-1/volume-2; muted uses volume-x.
Inactive/active notifications use bell/bell-ring. There is no dedicated plain
wired-port variant in the catalogue: plain wired uses plug, activated wired
uses ethernet-port. bell-dot is not shipped; bell-ring preserves the active
notification distinction. The optional installed start-here-kde mark remains
available through the panel's cached apps.icon lookup.

Search inventories retain at most 4,096 tracked paths. Unstamped SVG/raster
files do not consume the traversal work budget, and directories are charged
only on their recursive visit. A partial inventory (limit, external directory
alias or read error) retains lookup and decoded-asset caches for up to one
second, then resnapshots and advances the generation once. This bounds
freshness for very large trees at about one second without repeating failed
lookups on every frame. Complete inventories validate tracked metadata at
each preparation batch. Explicit absolute/file:// paths are rechecked on
every preparation, independently of these search-root generations.
