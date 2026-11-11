# paths

Dependency-free MixViu roots and `mix:` paths. `Root::Lib` follows the prefix,
like `Share`. Existing constructors retain the legacy system tier;
`Dirs::derived(prefix, env)` and `Dirs::derived_with_layout(prefix, env,
source)` expose the future prefix-relative system tier: for `etc`, `var`,
`run` and `cache`, own variable > `MIX_ROOT` > layout > under the prefix;
`share` and `lib` are installed data, so `MIX_ROOT` does not move them. Both
refuse a relative or `..`-bearing root and name it (`root etc: ...`).
`prefix_from_exe_strict` requires a `bin` parent without changing the legacy
executable resolver; `/bin/mix` gives the prefix `/` (a rootfs install).

`env` owns the registered variable names (`env::test` for test names).
`env::raw` treats empty overrides as unset. `env::setting` reads an override
first, otherwise the installed default. `env::setting_or(name, middle)` puts
the caller's own step (a config file, manifest or probe) between the two.

Installed defaults come from `<prefix>/env.mx`, read lazily once per process.
The prefix is `MIX_PREFIX`, else the strict rule above. A binary that is not
in a `bin` directory (a cargo tree) has no installed defaults, so a test
binary never reads a host install's file.

`stamp!` expands at the caller's compile site; buildinfo uses it for consumer
provenance and registers the binary's full SHA with `env::register_snapshot`
when `build_info!` runs. Every engine and service does that first thing in
`main`. The first registration wins. A registration after the first
installed-default read panics in debug builds (that read skipped the commit
check); release builds keep the first, unchecked result. A conflicting second
value is ignored and, in debug builds, reported once on stderr. With no
registration, the commit check is skipped.

Process, user and system layouts resolve in separate lazy caches from one read
of `layout.mx`. Valid layout overrides retain their precedence. A layout that
does not parse cannot block user roots; a layout that cannot be read
(permissions, I/O, size, non-UTF-8) fails every tier. The existing no-HOME
user fallback to `/root` remains until the remove PR.

## `env.mx`, schema 1

Written only by the installer (and `tools/ci.mix stage-install`) from the
registry entries in `etc/env.conf.mix` that are runtime scope and carry a
`default`. The reader accepts any well-formed `MIX_` name, so a newer file
still reads.

- A flat map, braced (`{a: 'x', b: 'y'}`, comma separated) or not (one entry
  per line). `#` and `--` comments. At most 64 KiB of UTF-8, no byte order
  mark.
- `schema: 1`, an unquoted number, exactly once. `commit: '<40 lower-case
  hex>'`, the full commit of the snapshot the binaries were built from.
- Every other key is a `MIX_` name of upper-case letters, digits and `_`,
  once each. Keys may be quoted.
- Values are quoted strings. Single quotes are raw; double quotes take the
  escapes `\\`, `\"`, `\$`, `\~`, `\n`, `\t`, `\r`, `\uXXXX` and `\u{X..}`.
  A value may not contain a
  newline, raw or escaped.
- A value starting `mix:` is a root reference (`mix:run/hostd/hostd.sock`),
  expanded through the process roots when read, so the file survives a move.
  It cannot climb out of its root.

Errors (`env::SettingError`): `NotSet` (no override, no middle answer, no
default or no file), `NotUtf8` (the override is not UTF-8), `NoPrefix`
(`MIX_PREFIX` is not an absolute path without `..`), `Unreadable` (the file
exists but cannot be read), `Malformed` (not schema-1 data as above),
`UnknownSchema`, `SnapshotMismatch` (the file's commit is not the binary's
registered one), `Layout` (a root reference cannot be expanded).

Security: `env.mx` is trusted like `layout.mx`. It must be installed
root-owned, mode 0644, in a root-owned prefix. The commit check is a
consistency check (binaries and defaults from one snapshot), not integrity:
anyone who can write the file can write a matching commit.
