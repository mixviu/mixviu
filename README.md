<p align="center">
  <img src="share/brand/logos/alien-cyclops.svg" alt="The MixViu cyclops" width="160">
</p>

<h1 align="center">MixViu</h1>

<p align="center">
  <strong>One computer, shared by people and AI agents.</strong><br>
  A shell, a service host, a compositor and a widget toolkit, all connected by one message Bus.
</p>

<p align="center">
  <a href="https://mixviu.ai">mixviu.ai</a> ·
  <a href="docs/architecture.md">Architecture</a> ·
  <a href="docs/mix/README.md">The Mix language</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

MixViu is an agent-first operating environment for Linux. Anything you can do
with a click, an agent can do with a request, because both go through the same
channel. Every app lists its actions as typed data and runs them through one
command. Menus, keyboard shortcuts, the command line and AI agents all end in
that same call, so an agent never has to scrape a screen or guess where a
button is.

It is built in the open by one human maintainer working with several AI
agents. Application behaviour is written in Mix, MixViu's own shell language.
Rust holds the native foundations.

## The four engines

| Engine | What it is |
|---|---|
| **[mix](engines/mix)** | The shell and the language. It is a login shell, REPL, script runner and manual. Values are structured, not text streams. External programs run by argv, so there are no quoting traps. Mix is also the language for automation and app behaviour. |
| **[mixd](engines/mixd)** | The headless host. It is one multi-call binary for servers, with no display code. `mixd serve` runs Mix services as Bus citizens. The daemons are personalities chosen by argv[0], linked to the one binary: `noded` (the Bus), `maild`, `webd`, `dnsd`, `indexd`, `settingsd`, `hostd` and others. `mixd --personalities` lists them all. |
| **[viud](engines/viud)** | The Wayland compositor and nothing more: scanout, rendering, input, capture, and window and output authority. It is exposed as the `comp` Bus service. |
| **[viu](engines/viu)** | The widget toolkit and scene host. Scene documents describe the layout and bindings, and Mix behaviours make the decisions. The same widgets serve the desktop shell and ordinary app windows. |

The engines share only the Mix library and the Bus crate. No engine links
another; every other boundary is the Bus.

### One Bus, two commands

Commands are named `owner.verb` and routed by the node daemon, which also
checks who may ask. Every app answers the same two:

```mix
send monitor monitor.actions                           -- what can you do? (typed, as data)
send monitor monitor.execute body={id: "file.refresh"}  -- do one of them
```

The command palette, the menus, Ctrl+R and an agent all reach `file.refresh`
the same way. A disabled action says why, so an agent can explain it to you.
MCP is offered only as an edge adapter ([`mix-mcp`](services/mix-mcp)). It
exposes a few generic tools and never bypasses the Bus.

## Where it is today

MixViu is young and moves fast, with several merges a day. It runs daily, but
it is not a finished product. There is no packaged release yet.

- **Daily use.** The maintainer's desktops run viud and viu every day. Each
  build of `main` passes a full gate before it is installed on a staging desk,
  and only then on the main one.
- **Production servers.** mixd's `maild` and `webd` serve real mail and web
  traffic. [mixviu.ai](https://mixviu.ai) itself is served by `webd`.
- **Apps written in Mix.** A twin-pane file manager ([DOpus](apps/dopus)), a
  Mix [terminal](apps/term), [email](apps/email), a
  [system monitor](apps/monitor), [screenshot](apps/screenshot),
  [preferences](apps/prefs), a [text editor](apps/ced),
  [slide decks](apps/deck), a live [Bus tap](apps/bustap), the agent
  [workshop](apps/workshop) and the [widget gallery](apps/gallery).
- **Pure-Rust media.** The [`av`](services/av) service renders timelines to
  AV1 and Opus WebM with no ffmpeg.
- **Self-disclosure.** The node daemon keeps a rotating history of Bus
  traffic, so the system can show you what it did. This is the first step of
  a wider plan.
- **Next.** Agent-generated UI: an agent composes a scene from the existing
  widget catalogue, binds it only to declared actions, and viu shows it as a
  clearly marked agent surface.

## Install from source

You need Linux on x86-64 and [rustup](https://rustup.rs). The toolchain is
pinned in [`rust-toolchain.toml`](rust-toolchain.toml), and rustup fetches it
on the first build. Every MixViu binary lives in one prefix, `/opt/mix`.

**1. Build each engine with its own `-p`.** A workspace-wide build mixes the
two Mix profiles' features and is refused.

```bash
git clone https://github.com/mixviu/mixviu && cd mixviu
cargo build --release --locked -p mix  --bin mix
cargo build --release --locked -p mixd --bin mixd
cargo build --release --locked -p viu  --bin viu
cargo build --release --locked -p viud --bin viud
```

For a server, `mix` and `mixd` are enough. The release set also includes
`av`, `inputd`, `portald` and `mix-mcp`; see
[`tools/release-set.conf.mix`](tools/release-set.conf.mix).

**2. Install the binaries and the Mix shell.**

```bash
sudo install -Dm755 -t /opt/mix/bin target/release/{mix,mixd,viu,viud}
sudo ln -sf /opt/mix/bin/mix /usr/local/bin/mix
mix --version
```

**3. Link the daemons you want to run** (each one is a personality of mixd):

```bash
/opt/mix/bin/mixd --personalities     # what this build carries
cd /opt/mix/bin && sudo ln -sf mixd noded && sudo ln -sf mixd webd
```

**4. Add the apps and shared data** (Mix apps, the Viu Mix library, scenes,
icons, fonts, wallpapers and unit templates). The bundler needs `viu` built
next to `mix`. It downloads the pinned fonts and wallpapers and checks every
byte against its hash.

```bash
target/release/mix tools/apps_bundle.mix apps.tar \
    --mix target/release/mix --assets ~/.cache/mixviu-assets
sudo tar -xf apps.tar -C /opt/mix
```

The systemd unit templates land in `/opt/mix/share/mixviu/units/`. Start with
`noded`: everything else joins its Bus.

**5. Try it.** `mix` opens the Mix shell (a REPL, not the desktop),
`mix man overview` explains the model, and `mix builtins` lists all ~400
built-in functions.

**The desktop** runs one session per virtual terminal, as systemd units:
`viud@N` (the compositor on VT N, which needs `seatd`) plus `apps@N`,
`scenes@N` and `files@N`. The templates are in [`engines/viud/units`](engines/viud/units).
Setting up a desktop by hand is not documented for newcomers yet, so start
with the shell and the services, and open an issue if you want to try it.

## Contributing

Contributions are welcome from people and from AI agents. The full rules are
in [CONTRIBUTING.md](CONTRIBUTING.md); in short:

- **Talk first.** Open an issue for anything bigger than a fix, so direction
  is settled before code is written.
- **Small PRs from a branch** (`fix/…`, `feat/…`, `docs/…`, `infra/…`), one
  change each, with the template filled in: risk tier, how it was tested and
  how to undo it.
- **Check what you touched.** Run formatting, clippy and the tests of the
  crates you changed (`mix tools/ci.mix fmt`, `mix tools/ci.mix clippy`,
  or `cargo test -p <crate>`). The full `mix tools/ci.mix all` runs once per PR,
  in the merge gate.
- **Mix, not shell or Python.** Scripts, tools and automation in this
  repository are `.mix` files. `mix lint` diagnostics may only go down
  ([ratchet](docs/development/mix-lint-ratchet.md)).
- **Rust for primitives.** New behaviour starts in Mix. Code moves to Rust
  only after a measured, diagnosed performance problem.
- **Foreign code has a high bar.** New crates must pass `cargo deny`
  (permissive licences, known sources) and the
  [foreign-code gate](tools/foreign_gate.mix). C or C++ inside a MixViu
  binary is a short, justified allowlist.
- **Review and landing.** `main` is protected. Every PR is reviewed (Tier 2
  paths by their code owner, see [REVIEW.md](REVIEW.md)) and lands through a
  serial merge queue that runs the full gate.

Good starting points: [architecture](docs/architecture.md),
[app control](docs/app-control.md),
[scene documents](docs/scene-documents.md), the
[Mix manual](docs/mix/README.md) and the [engines](engines/README.md).

## Lineage

MixViu draws on decades of the same idea: programs that talk to each other
through a shared, scriptable port. ARexx on the Amiga (1987) showed the way.
This project grew out of earlier work by the same author, from SPE and markweb
through CosMix and MixOS, and the story is told on
[mixviu.ai/history](https://mixviu.ai/history/).

## Licence

MIT OR Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE), [NOTICE](NOTICE) and
[REUSE.toml](REUSE.toml). Imported source keeps its attribution
([provenance](provenance/README.md)). The cyclops mascot and other brand art
are in [`share/brand`](share/brand/README.md).
