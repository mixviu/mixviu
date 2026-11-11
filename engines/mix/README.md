# mix

The `mix` engine: the Mix shell profile, the login shell and script runner.
The service profile and the headless daemons are the `mixd` engine; the shell
itself is the `mix-shell` library (`libraries/mix-shell`).

Build with `cargo build -p mix`, never as part of a workspace-unified build.

`mix mesh` contains the derived inventory signer. Its `sign`, `verify`,
`pubkey`, `genesis`, `d2-gen`, `d2-pubkey`, `wg-gen` and `wg-pubkey`
commands retain the predecessor's arguments, output and refusal rules.
Verification JSON retains the frozen `cosmix-mesh-sign.verify.v1` format.

The genesis secret stays in the operator's SQLite secrets database. The
global `--secrets-db PATH` defaults to `~/.ns/_etc/secrets/secrets.db`, with
the leading `~/` expanded against `HOME`. The `password` column contains a
base64 32-byte Ed25519 seed, selected by all four fields: `vnode="mesh"`,
`domain=--mesh` (default `bus`), `service="mesh-genesis"` and
`username="genesis"`. `pubkey` derives and prints only its public key;
`genesis` refuses to replace an existing seed without `--force`.

`verify --genesis-pub PATH` or `--genesis-pub-base64 B64` uses a public
trust anchor without opening the secrets database. Verification alone has
no persisted epoch floor: callers retain their freshness checks. Tests
use throwaway databases and keys, never the operator database.
