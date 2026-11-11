# inputd derivation

Source: frozen markc/mixos origin/main at `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Licences and copyright headers retained. Read only with git objects.

Imported services/inputd (excluding crates, already shared) and docs/inputd.
Rust sources in the two shared libraries are byte-identical to the source's
services/inputd/crates copies; only Cargo manifests differ (workspace paths).
No shared code imported or changed.

Adaptations: workspace dependencies; etc-root default keymap and fixture paths;
installed log-filter default through paths::env; relocatable unit and manifest;
documentation links and historical state paths. Service/injection/grab/reader/
device code and all original tests otherwise retained. See docs/inputd/port.md
for protocol, reserved-key mismatch, migration and coverage limits.

Frozen input tree (git blob object ids; includes compared shared crates):

```text
100644 blob e9544bdd28f75e0158f43cb1fec92e4cb15fb7e2	docs/inputd/README.md
100644 blob c1211770193ac862ea08cf3b351662d827b46224	docs/inputd/changelog.md
100644 blob 82a8bb72c0ed88b57f866cef48aa3af4d1fe79a0	docs/inputd/changelog/index.html
100644 blob 0cb27e660db8fc98e440ba80e7fb524a4cffa927	docs/inputd/index.html
100644 blob 18e269c1967a3491dc840c4aa969ec1256140bbc	services/inputd/CHANGELOG.md
100644 blob 1ffc5e8321506d489cc7ce7ea76aff37f68fca5f	services/inputd/Cargo.toml
100644 blob 1edd7daa36ccf811e288e12c1bdf0f80a4f5beb7	services/inputd/README.md
100644 blob 08025b90ee675e17b4faeb189c5d62733ba8bd5b	services/inputd/build.rs
100644 blob 922d0824cd4dd08b71aef70996074b0672defa08	services/inputd/crates/inputd-core/Cargo.toml
100644 blob 6eeef6b3b9cce354222d2054be1c1de016204b31	services/inputd/crates/inputd-core/src/interceptor.rs
100644 blob fe115b6bfaf9274adc175d00854854e9de8f2737	services/inputd/crates/inputd-core/src/lib.rs
100644 blob 773a4c380fb0693c3ce1ff72c175a908d844d9b9	services/inputd/crates/inputd-core/src/tests.rs
100644 blob fd6923c7c416fbc98bb40678a409f15dc1d08d5a	services/inputd/crates/inputd-schema/Cargo.toml
100644 blob 45db3a0e4ec2b8903894d03d5281295f44ffae35	services/inputd/crates/inputd-schema/src/action.rs
100644 blob a380eb968147b0eef3236dd81b1b0da50c263e27	services/inputd/crates/inputd-schema/src/lib.rs
100644 blob 62484540abf83fd35a7e06eb2079ef008229ce0c	services/inputd/crates/inputd-schema/src/scope.rs
100644 blob 361b2791cd569273387f688709ae09fbb1fa4331	services/inputd/examples/inputd-grab.conf.example
100644 blob f41a3b043548876d87a4a46850cb9cce80e17ef2	services/inputd/src/devices.rs
100644 blob 06705be3a8f9f413e0141bcc94a9c3bbb8baad8c	services/inputd/src/grab.rs
100644 blob 8ba9af9f89f49ceade38c7ee81ecd4982a022c9a	services/inputd/src/inject.rs
100644 blob 5a36ec3ecee47900df6301ec7052f095de61e5d6	services/inputd/src/keymap_file.rs
100644 blob af2f9a2f4fac82ba92c7a718b1cfad1a39c447ef	services/inputd/src/main.rs
100644 blob 653a8a9596f6743ffc90fba890023dcdf1a3ed78	services/inputd/src/reader.rs
100644 blob 0d9084889c77e8e1e8f70d77ae55dbc953afde37	services/inputd/src/service.rs
100644 blob 0f6450cb8f939d73801093450187dbd3623da8a9	services/inputd/tests/fixtures/legacy-keymap.json
100644 blob e4ae8ce53d461a94163b25cc945e87921f82135e	services/inputd/units/inputd.service
```
