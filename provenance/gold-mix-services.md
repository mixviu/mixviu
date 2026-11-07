# Gold Mix services: clipd, notifyd, exportd

Source: MixOS (github.com/markc/mixos) at `8779f33284b7d3f1178c3d57a699efb9b0867b22`, licence MIT OR
Apache-2.0 (the workspace licence; every file keeps its SPDX header). Imported
verbatim into `services/` in one commit, then renamed in the next: `MIXOS_*`
to `MIX_*` (registered in `etc/env.conf.mix`), `mixos_path` to `mix_path`,
`/opt/mixos` to `/opt/mix`, per-owner directories, compd to viud, MixOS to
MixViu in prose. desk4 runs them under the four engines instead of gold.

| Path at the source revision | Git blob | Disposition |
|---|---|---|
| `services/clipd/CHANGELOG.md` | `56c63eaee002bbd3c7bb69cbf97f4f0100fa44d4` | imported |
| `services/clipd/README.md` | `54719ff6d4bd3e3b11c5f8fed31114d6bbda0b20` | imported |
| `services/clipd/component.mx` | `90ffa83da6bacc75ecc1f8e786285d4e012b4b54` | imported |
| `services/clipd/scripts/clipd.mix` | `f0bbee3466c8c201db0d8ad377781ec0c74a73fa` | imported |
| `services/clipd/scripts/encode_stdin.mix` | `7eead172d9611cb00f071d6d34cf3d694434860a` | imported |
| `services/clipd/scripts/lib/clipboard.mix` | `786178082ca9dbb9f307f0c91aef4a44cc806412` | imported |
| `services/clipd/scripts/watch.mix` | `a4d076d895fd0a3fa04ec1e1ffb2510c81edc42f` | imported |
| `services/clipd/tests/cbc_checks.mix` | `d2967701b18a2dc3861e8718a9740930bf45b223` | not imported (MixOS-only CBC check of mixos crates) |
| `services/clipd/tests/clipboard_test.mix` | `b72170b382000c3c16829c4c93531651b2172239` | imported |
| `services/clipd/tests/clipd_test.mix` | `2e5d7a7492b96fa6b88cf9a906e133f3953c8f6f` | imported |
| `services/clipd/tests/fixtures/listener.mix` | `62267d8bed0b56905468f0dd733dc490839f9a6f` | imported |
| `services/clipd/tests/wayland_test.mix` | `a05f9fdcd0d669412d9886e7c5ea1ee9236da35c` | imported |
| `services/clipd/units/clipd.service` | `6c6829ca5e8eee6f559c183a7c1d51478b96a7cd` | imported |
| `services/clipd/units/clipd@.service` | `77c5748c00562648aba60232533be3a0a3844456` | imported |
| `services/exportd/CHANGELOG.md` | `dadefbf566253f05ae2cdebaa83aa74ecaa357f8` | imported |
| `services/exportd/README.md` | `25238bf960e347f88dc2389b5942dd380f24efb8` | imported |
| `services/exportd/component.mx` | `13d2758bd12915b4b64c4e06e64cd3186316d033` | imported |
| `services/exportd/scripts/cosmic.mix` | `eab32183561fd3a91fdc69ee6135b0908787faed` | imported |
| `services/exportd/scripts/gtkqt.mix` | `4b9518aadf59e7b4aa36603d11ee40975e43a615` | imported |
| `services/exportd/scripts/kde.mix` | `32d5945a6b180b0d256d5e1e6ab08aa53bf9c1d5` | imported |
| `services/exportd/scripts/lib/cosmic_map.mix` | `d937d51d774248eff438449173f82594db094958` | imported |
| `services/exportd/scripts/lib/cosmic_store.mix` | `afbcaac333bc74106e8939fdda3747f0797f284b` | imported |
| `services/exportd/scripts/lib/gtkqt_map.mix` | `5661e1eebbf4a0dc10cebeadab9cda8019ceb540` | imported |
| `services/exportd/scripts/lib/kconfig.mix` | `215227b76c38803f0e6f066584082d30c60ffa10` | imported |
| `services/exportd/scripts/lib/kde_apply.mix` | `833ed324968efec92a3d311ebc83562fe58c9f5b` | imported |
| `services/exportd/scripts/lib/kde_map.mix` | `51fb3ea78768be12d946e4f63232d11de60113e4` | imported |
| `services/exportd/scripts/lib/ron.mix` | `2cddf5564dc6e8029fe7be42f841bccdf0755858` | imported |
| `services/exportd/tests/busctl_test.mix` | `59f00e021e1c8829b387fcc2e0493e5412773e05` | imported |
| `services/exportd/tests/cosmic_export_test.mix` | `80ab3b7df9853433e56d4a239d8ffb1ed7623236` | imported |
| `services/exportd/tests/cosmic_test.mix` | `620248aed35084319f353fdd39fe310b850ec9a2` | imported |
| `services/exportd/tests/fixtures/busctl.mix` | `b121150c32b3adf6cb0fb40887082c7ca6fd4394` | imported |
| `services/exportd/tests/fixtures/fake_settingsd.mix` | `53723d76c034905e5231005c795b8ef3af40ea7b` | imported |
| `services/exportd/tests/fixtures/gtkqt_settingsd.mix` | `20b90ee7119cdd6b3e221e890be6240d5c9a7d60` | imported |
| `services/exportd/tests/fixtures/kwriteconfig6.mix` | `8dee64214f20fd6997fb3b74c26f440f028fdafe` | imported |
| `services/exportd/tests/gtkqt_export_test.mix` | `d791a0983c7b3ca0698ae53a6d1b63ddd35860b4` | imported |
| `services/exportd/tests/gtkqt_recovery_test.mix` | `36e8dcbe1a4582901b97110ed8711602d92fb433` | imported |
| `services/exportd/tests/gtkqt_test.mix` | `37cdf7cf1825305bb38bf0a54c297737018e9811` | imported |
| `services/exportd/tests/kde_test.mix` | `ead4715764abdef5b92799417ad5f85566872466` | imported |
| `services/exportd/units/exportd-cosmic@.service` | `f979cb344372fe33d8a55b4ae02463e435d21b78` | imported |
| `services/exportd/units/exportd-gtkqt@.service` | `75f2c2aedcaade9f52a65b5a7e120e7df303880c` | imported |
| `services/exportd/units/exportd-kde.service` | `be48a66a9d59bfd63b5a0d642efffe4ff15c233e` | imported |
| `services/notifyd/README.md` | `af8e407919172e4ac74beb7d4e6017a4e686d0c6` | imported |
| `services/notifyd/component.mx` | `7bc847c7cbe6f18f0dc2e9f77fc7f4ffa3135297` | imported |
| `services/notifyd/scripts/lib/notify.mix` | `3242c3a581eb9d06e675b9638f5ab101b80f21fb` | imported |
| `services/notifyd/scripts/notifyd.mix` | `544bb83d7ac639cdf9c4e5152a1965d61f1455dc` | imported |
| `services/notifyd/tests/fixtures/listener.mix` | `7beaf4abf1e4bac59c3521da2d9a016ced0cede7` | imported |
| `services/notifyd/tests/notify_test.mix` | `54fc4172a1864ec0f37155a56125ef772eed66ab` | imported |
| `services/notifyd/tests/notifyd_test.mix` | `cbc53f640fbec48e6cc85ff44b7f6159595023c4` | imported |
| `services/notifyd/units/notifyd@.service` | `cf3b72e36e976c57abf92fbf913ec532e102c3b4` | imported |
