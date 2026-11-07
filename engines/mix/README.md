# mix

The `mix` engine: the Mix shell profile, the login shell and script runner.
The service profile and the headless daemons are the `mixd` engine; the shell
itself is the `mix-shell` library (`libraries/mix-shell`).

Build with `cargo build -p mix`, never as part of a workspace-unified build.
