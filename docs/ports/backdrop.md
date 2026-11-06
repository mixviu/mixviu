# Backdrop port inventory

Oracle: MixOS `8779f3328`, `apps/backdrop/{src,crates/wallpaper}`, its README
and `docs/backdrop/README.md`. This is a bounded lab port, not parity acceptance.

| Original capability | Round 1 status |
|---|---|
| Background layer, all outputs, fractional scale, output hotplug | Ported through shared retained layer backend; native single-output headless capture verified |
| Solid RGB and fallback colour | Ported in Mix document projection |
| PNG/JPEG/WebP images; fill/fit/centre/tile/stretch | Ported through shared bounded image-fill; all five modes captured and inspected against original |
| Per-output document override with default fallback | Ported in Mix; actual headless-output override captured; simultaneous multi-output acceptance deferred (reference headless backend exposes one output) |
| `backdrop.ping`, `.state`, `.set` | Ported, model-ready replies; original wire envelope differs |
| Settings wallpaper image, colour, fit, same_on_all, outputs | Ported Mix follower; native Bus settings witness change captured and inspected |
| CLI config and default overrides | Ported config path as first bundle argument; original flags deferred to avoid a second parser before host contract settles |
| Strict schema, output/image limits | Ported Mix validation; unknown keys and unsupported sources refuse |
| Decode-before-commit and durable store on set | Deferred: generic image widget decodes asynchronously; this specimen reports model acceptance, not original transactional image acceptance |
| Theme colour / gradient first stop | Ported gradient first stop; exact live theme-token colour deferred until shared appearance integration |
| Slideshow folder/list, independent schedules, shuffle, crossfade, next | Deferred: requires Mix scheduling and explicit shared texture-budget/transition primitive; unsupported documents refuse |
| Live train views, queues/history/gate, alerts, 2D/3D, visibility leases | Deferred outside wallpaper specimen; no synthetic replacement |
| Action registry, macros, executed events and original probe CLI | Deferred until generic host exposes common app lifecycle and action contracts |
| Original 256 MiB resident/texture budgets and 3840 downscale | Deferred parity; current per-image limits are 32 MiB input, 8192 sides, 128 MiB decode, 16 MP output; retained decoder mailbox bounded |
| Idle zero polling, failed registration exits | Deferred parity: settings refresh currently one-second timer; Bus disconnect stays observable rather than terminating host |
| Presented revision/frame counts and original error/refusal envelopes | Deferred: model acceptance and native presentation are separate; generic tree/capture endpoints available |

Per-app Rust added: **0**. Shared native additions: retained layer backend,
generic bundle host, image-fill renderer/decoder, widget admission binding.
No installation or production acceptance follows.
