# Shared assets: icons, brand, layout, daylight, fonts, wallpapers

The desk installs these from the release (`apps.tar`, `share/...` under
`/opt/mix`), never from `/opt/mixos`.

## Sources imported from MixOS

MixOS (github.com/markc/mixos) at `8779f33284b7d3f1178c3d57a699efb9b0867b22`, imported verbatim into
`share/` in one commit, then renamed (MIXOS_* to MIX_*, mixos_path to mix_path,
/opt/mix, MixOS to MixViu in prose). The icon theme directory name `mixos`, the
app ids `dev.mixos*` and the brand file names are wire and data names and stay.

| Path | Git blob | Bytes | Licence |
|---|---|---|---|
| `share/assets/core.conf.mix` | `c2ad796635cb84e398c963c7baf7a130a2ea19e3` | 11022 | MIT OR Apache-2.0 (manifest and installer); each pinned file carries its own licence in core.conf.mix |
| `share/assets/install.mix` | `0af57cd7bd775c5429790c574d78ae75d14af0de` | 12224 | MIT OR Apache-2.0 (manifest and installer); each pinned file carries its own licence in core.conf.mix |
| `share/brand/README.md` | `1f827909558a28f3891348240d2bd6910c0e88c7` | 4291 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/apple-touch-icon-152x152.png` | `0ea50380b1fc1aed1c5f82871ee7da463941d21a` | 7270 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/apple-touch-icon-167x167.png` | `0b7eae036ac91c91ed15902b0417de063067d9a1` | 8118 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/apple-touch-icon.png` | `64ffa5d7a6cb0cde55282863482f4d1c3837ba4d` | 8719 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon-128x128.png` | `bcaefb29135c7779a68d2854379581dddcfad815` | 6128 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon-16x16.png` | `e15280aa2ec9e00b3c94b3bed6175ae59e9a55dd` | 552 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon-256x256.png` | `0909428419c2ccc53c600c04337810838919e1da` | 13003 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon-32x32.png` | `38d16c80a94e56d7a1ecc6b90e04a3a55b8c5f71` | 1274 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon-48x48.png` | `a4cb99bf2a55f0d5e053501a9f29b466225eb02c` | 1985 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon-64x64.png` | `60d4f83073ecdac40c9e85ba225c3e8007ad95a0` | 2812 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon.ico` | `8662d0215bdaf714e51975345470ded70c451fd1` | 26033 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/favicon.svg` | `690ffc30ac379028ac0532e9a81feebf3387d804` | 4909 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/black-on-white/icon-512.png` | `ee703dc4956cc3afcaed6f843214b915d33aa254` | 28705 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/apple-touch-icon.png` | `438c9b437b0216dde575c7284616793bfea5579b` | 13595 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon-128x128.png` | `2e6b6556e6cee9db8053c98a543b2d3bbe2a81fc` | 7962 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon-16x16.png` | `f2ca12062eecde3f457093a76be3e8580850dbe2` | 693 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon-256x256.png` | `ccf38b1ddb1844c641792819af7c2e439b1e6630` | 18232 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon-32x32.png` | `ed714dcf289236809070986865412b5b59167407` | 1286 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon-48x48.png` | `771c6ca6a5f73fa0d0cc63729468f6e3314b1ba8` | 1851 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon-64x64.png` | `0df29e7a360d75f53ff2106679a6dda2b3c27001` | 3475 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/cyclops/favicon.ico` | `58f2c5ed5edd7750f1336d8c75b12ea8a0283997` | 370070 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/apple-touch-icon-152x152.png` | `e77fc70d24d0d42ee4a533283fc6c4e7409ad037` | 7019 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/apple-touch-icon-167x167.png` | `1571f8e474f18f045ad5f53bd6fd45ee06e1191a` | 7946 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/apple-touch-icon.png` | `cf0b089058196d294c847f26854007c0c25ad689` | 8423 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon-128x128.png` | `a3d0fcec7662395ac79e56b081ab96d7145374dc` | 5966 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon-16x16.png` | `942fbfee9ed4a0047c0c0c809582a4a15d75d0c6` | 517 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon-256x256.png` | `861960c3c7fd0b8cefd964367e9cd79749bb02ed` | 12570 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon-32x32.png` | `7b625667d6a439885f13617d48b2e48e3ad48741` | 1215 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon-48x48.png` | `f4654b56ee909998f5d8e88d216e4b36a80b11b2` | 1951 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon-64x64.png` | `92bdc493cdd74731c0790d3e9b0da276259420c7` | 2747 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon.ico` | `76aaedcc3b3cf3d1933587790c83e083528d3df1` | 25234 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/favicon.svg` | `565152bab6b1d1e90b91b2b0c0b38599e403d539` | 4909 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/favicons/white-on-black/icon-512.png` | `e5fdf7ea93bccfcb8b4b1cbfc2d86b4e2235775b` | 27013 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/hicolor-index.theme` | `53924de3314efc3bc510ea9bb074dda32991f567` | 936 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/install.mix` | `e75f88012f354dd66a3923e30ccba2d384d544a2` | 6143 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/logos/alien-cyclops-favicon.svg` | `b6d5fc561b14b855eb977bd853b5fd53d00e8b2d` | 861 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/logos/alien-cyclops.svg` | `9d712f25ab9cf56621528649faf99857f95258cb` | 834 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/logos/mixos-black-transparent.svg` | `7c57f5ee95c0afa6c8d2bfca5a305890fb9170ad` | 4842 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/logos/mixos-white-transparent.svg` | `0c63ded5f615bf8080c43d21ec0e1503ae492d68` | 4842 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/brand/preview.png` | `23202baf7e49f36fde4333af874b00a582e04b23` | 58882 | MIT OR Apache-2.0 (brand art by Mark Constable; README) |
| `share/daylight/LICENSE-tzdb.txt` | `8ba4399c622d3a1a3175865c39c3469e82539795` | 252 | zones.mx derived from tzdb, public domain (LICENSE-tzdb.txt) |
| `share/daylight/zones.mx` | `08207ec82837225c2f95d34ce4a93b115772ec2b` | 22909 | zones.mx derived from tzdb, public domain (LICENSE-tzdb.txt) |
| `share/icons/LICENSE-lucide.txt` | `718bb3f0e44153809972abed31839375804bf652` | 3208 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/README.md` | `5783827af9fa7062bbd13a5d678a184b6f34259a` | 8739 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/index.theme` | `783888d53ad42faa3f1c07ab0f05175b88db71c4` | 491 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/install.mix` | `2e12fe471670214394fbf6c745df43def213e961` | 18148 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/app-window.svg` | `815548bce9df1e721b38413caa69cff700052042` | 329 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/arrow-left.svg` | `b40bff851c85eff44f678eb7984fc6070d7ea721` | 262 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/arrow-right.svg` | `433ded3fcc5f7bd2b6a59356c6ffe8eceac6b7e3` | 261 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/bell-ring.svg` | `2c93f7573a8d2ac4ef0e8ca8de8fa31d0b99a20a` | 469 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/bell.svg` | `0345ca35bc8cead87cf05857545e24de031e7779` | 393 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/camera.svg` | `1e82bf45f3c522db0e22f16e0a5fc8fd00b79b7e` | 437 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/chevron-left.svg` | `056d5abc06e44940302c8b2a1912a382ea26b36d` | 238 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/chevron-right.svg` | `db62a0c20181edd6173722b2a483e0b85e30f240` | 237 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/ethernet-port.svg` | `c8a39a5933ce68584e4417a305169795b91dd175` | 471 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/film.svg` | `fae756862406a2e36566334e77b77c1fbc4ebb92` | 432 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/folder-open.svg` | `b01f60fcb51848a9b3a7380d4566c4de7534745c` | 401 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/layout-grid.svg` | `f8eca3a721201b652dbbc3b76c15e92fd271c9f9` | 416 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/layout-template.svg` | `ec9481ddd5eb9a7f540a7f5af9c5a3602624f9cd` | 365 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/mail.svg` | `4cff729d97c4b4daacb4d86220bf45b8e5c199fe` | 316 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/minus.svg` | `75093e27c20965c9e37fb9dcef9f939746c67578` | 232 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/monitor.svg` | `e869f2f27acef115bc121a0bf62e1d01834c6b17` | 346 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/notebook-pen.svg` | `b90c54bf82fd3a0c3992c3597230c2599bd33be6` | 519 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/plug.svg` | `f28b6f0abef77a6e843dfa99f81731332c72db64` | 363 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/plus.svg` | `41f92bb93eb6626368d71c9884127bbcf7d3ccdf` | 256 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/power.svg` | `bacd9d5337b68211662eef2b22733dc7b99d663e` | 276 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/rotate-cw.svg` | `6795f15767d8ae1b56f669aa7a3fe2e99adae6f1` | 299 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/settings.svg` | `4c6263af909841a3911f41a385021778eacba736` | 586 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/speaker.svg` | `387baab0e8c35241e374df71caf42742e3366a28` | 347 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/terminal.svg` | `c09409f42887ce20aeac641fe4da55969b704be5` | 261 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/volume-1.svg` | `65e6a6656ef6a95935ee66a84f5d872dde95cc7f` | 427 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/volume-2.svg` | `519e6cbd4d2693a38f7db0e396354cc07b9f4e26` | 476 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/volume-x.svg` | `3e2600946f38313e8cfb617e35bbc3c95159d172` | 443 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/volume.svg` | `726e059dd4b6f77ab5e2e04f3e6390e1a79670dd` | 392 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/wifi-off.svg` | `6bb8ddfa6c94f9b15b797f484c9921b7e839f0ad` | 494 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/lucide/wifi.svg` | `e46d6d32cc10ba91fddfe0fe130109fa3a0c44f0` | 357 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/manifest.conf.mix` | `f0a74c4a73ba2b0a0e7ccad82e0f151d00090cee` | 7750 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/scalable/actions/window-close-symbolic.svg` | `6decd3febbb93d8094ce361720057d3dc3f2fd4b` | 329 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/scalable/actions/window-maximize-symbolic.svg` | `95c2f88359bda454539400ba5d9dee94a260f745` | 341 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/scalable/actions/window-minimize-symbolic.svg` | `8d077898172baafd403939373d9a8d6cfbb25710` | 318 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/icons/scalable/actions/window-restore-symbolic.svg` | `d55b25d8df1e5936d969f1068c0880d88e978f92` | 374 | Lucide SVGs: ISC (LICENSE-lucide.txt); caption glyphs and scripts: MIT OR Apache-2.0 |
| `share/layout/apply.mix` | `5d666e918f473ab4b642f6f25606419b6d95739c` | 852 | MIT OR Apache-2.0 |
| `share/layout/files.mix` | `f1742081389de6e15290fda81d99ec899c5a0d55` | 8409 | MIT OR Apache-2.0 |
| `share/layout/render.mix` | `36ed4fbaae2eed44f58e37930916cab7e300a73f` | 2636 | MIT OR Apache-2.0 |
| `share/layout/templates.mx` | `e74c86d4aa66375dfde7797ec0d2c20de26b45d3` | 5724 | MIT OR Apache-2.0 |

## Pinned font and symbol set `2026-10-14-core-5`

Listed in `share/assets/core.conf.mix` with upstream URL, revision, size, SHA-256
and BLAKE3. The bundle step copies the set from an asset store (the gold install
has it at `share/assets/sets/2026-10-14-core-5`) and checks every file's
SHA-256 and BLAKE3 before packing; `share/assets/install.mix` can instead fetch
each pinned URL.

| File | Bytes | Licence | Upstream | SHA-256 |
|---|---|---|---|---|
| `fonts/NotoSans.ttf` | 2049096 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d` |
| `fonts/NotoSans-Italic.ttf` | 2322640 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `58e6e0ebd1931b29a365aa2d3e2ee9a9e831a3af7cf3ad1462d4e72154f0b291` |
| `licences/NotoSans-OFL.txt` | 4396 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a` |
| `fonts/InterVariable.ttf` | 879708 | OFL-1.1 | https://github.com/rsms/inter @ `353b61b9f4430d5f420d56605a6e7993e0941470` | `4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf` |
| `web/InterVariable.woff2` | 352240 | OFL-1.1 | https://github.com/rsms/inter @ `353b61b9f4430d5f420d56605a6e7993e0941470` | `693b77d4f32ee9b8bfc995589b5fad5e99adf2832738661f5402f9978429a8e3` |
| `licences/Inter-OFL.txt` | 4380 | OFL-1.1 | https://github.com/rsms/inter @ `353b61b9f4430d5f420d56605a6e7993e0941470` | `262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a` |
| `fonts/JetBrainsMono.ttf` | 303144 | OFL-1.1 | https://github.com/JetBrains/JetBrainsMono @ `cd5227bd1f61dff3bbd6c814ceaf7ffd95e947d9` | `662a196d58f1183bf2d77428b6d5283fe3f45161ab021bea4036bc98e5cac016` |
| `fonts/JetBrainsMono-Italic.ttf` | 308888 | OFL-1.1 | https://github.com/JetBrains/JetBrainsMono @ `cd5227bd1f61dff3bbd6c814ceaf7ffd95e947d9` | `f115aaa12113718c02ce72864fe6823b87241bc23d3e44cf1220155f861063f2` |
| `licences/JetBrainsMono-OFL.txt` | 4399 | OFL-1.1 | https://github.com/JetBrains/JetBrainsMono @ `cd5227bd1f61dff3bbd6c814ceaf7ffd95e947d9` | `30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad` |
| `fonts/NotoSerif.ttf` | 1887192 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `4d8e6761424656867019081a1a01336f3cb086982682698714054fc33f782713` |
| `fonts/NotoSerif-Italic.ttf` | 2448496 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `e87acbc6c0efd0d9a20d6a8cbbda2b266c14be3a3a6f5af8ec9d7b2460570ad1` |
| `licences/NotoSerif-OFL.txt` | 4396 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a` |
| `emoji/NotoColorEmoji.ttf` | 10673480 | OFL-1.1 | https://github.com/googlefonts/noto-emoji @ `8998f5dd683424a73e2314a8c1f1e359c19e8742` | `72a635cb3d2f3524c51620cdde406b217204e8a6a06c6a096ff8ed4b5fd6e27b` |
| `licences/NotoColorEmoji-OFL.txt` | 4301 | OFL-1.1 | https://github.com/googlefonts/noto-emoji @ `8998f5dd683424a73e2314a8c1f1e359c19e8742` | `6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2` |
| `fonts/Quicksand.ttf` | 124824 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `39c9b64223561f56aaff6062a6f04063c4fc86809ad6768722c06614d977e1cc` |
| `licences/Quicksand-OFL.txt` | 4445 | OFL-1.1 | https://github.com/google/fonts @ `9710da1eacb3be272583c3224dcb70f9da6eadbb` | `14d28541780d90f6577bb7abcc6f005bf6969614294e75c82abb8758a696ad46` |
| `web/NotoColorEmoji-COLRv1.ttf` | 4991984 | OFL-1.1 | https://github.com/googlefonts/noto-emoji @ `8998f5dd683424a73e2314a8c1f1e359c19e8742` | `0ae57fe58645638523ba35f388d93739d292539a9acb84df5700c81b1e1a28d2` |

## Wallpapers

Pinned in `share/wallpapers/manifest.conf.mix`; credits in
`share/wallpapers/{photos,generated}/CREDITS.md`. The JPEGs are not committed:
the bundle step copies them from an asset store and checks each SHA-256. CC BY-SA
files must stay with their licence text; the photos CREDITS notes the Nunivak
author should be named from the source page before a public release.

| File | Bytes | Licence | Author | Source |
|---|---|---|---|---|
| `photos/etang-de-thau.jpg` | 1470760 | CC-BY-4.0 | Christian Ferrer | https://commons.wikimedia.org/wiki/File:Étang_de_Thau_and_Sète.jpg |
| `photos/ijen-banyuwangi.jpg` | 1007118 | CC-BY-SA-4.0 | Gardian Novel | https://commons.wikimedia.org/wiki/File:Ijen_Banyuwangi.jpg |
| `photos/nepal-chola-lake.jpg` | 2797605 | CC-BY-4.0 | Vyacheslav Argenberg | https://commons.wikimedia.org/wiki/File:Mountains_in_snow,_Mountain_lake,_Chola_Valley,_Nepal,_Himalayas.jpg |
| `photos/nunivak-dunes.jpg` | 1337475 | CC0-1.0 | Wikimedia Commons contributor (author on the source page) | https://commons.wikimedia.org/wiki/File:Sand_Dunes_on_the_Northern_Coast_of_Nunivak_Island.jpg |
| `photos/yosemite-mirror-lake.jpg` | 3935344 | CC-BY-SA-4.0 | Dietmar Rabich | https://commons.wikimedia.org/wiki/File:Yosemite_National_Park_(CA,_USA),_Yosemite_Valley,_Mirror_Lake_--_2022_--_2814.jpg |
| `photos/zanskar-shila-tsarap.jpg` | 4661329 | CC-BY-SA-4.0 | Timothy A. Gonsalves | https://commons.wikimedia.org/wiki/File:Shila_Tsarap_Mountains_Zanskar_Oct22_A7C_04113.jpg |
| `generated/gen-01-deep-ocean.jpg` | 364987 | MIT OR Apache-2.0 | Mark Constable (procedural, ImageMagick plasma) | generated 2026-10-13 |
| `generated/gen-02-blush.jpg` | 365855 | MIT OR Apache-2.0 | Mark Constable (procedural, ImageMagick plasma) | generated 2026-10-13 |
| `generated/gen-03-nebula.jpg` | 387374 | MIT OR Apache-2.0 | Mark Constable (procedural, ImageMagick plasma) | generated 2026-10-13 |
| `generated/gen-04-sunset.jpg` | 371943 | MIT OR Apache-2.0 | Mark Constable (procedural, ImageMagick plasma) | generated 2026-10-13 |
| `generated/gen-05-aqua.jpg` | 359369 | MIT OR Apache-2.0 | Mark Constable (procedural, ImageMagick plasma) | generated 2026-10-13 |
| `generated/gen-06-meadow.jpg` | 354505 | MIT OR Apache-2.0 | Mark Constable (procedural, ImageMagick plasma) | generated 2026-10-13 |
