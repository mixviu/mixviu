# paths

Dependency-free MixViu roots and `mix:` paths. `Root::Lib` follows the prefix,
like `Share`. Existing constructors retain the legacy system tier;
`Dirs::derived(prefix, env)` and `Dirs::derived_with_layout(prefix, env,
source)` expose the future prefix-relative system tier: for `etc`, `var`,
`run` and `cache`, own variable > `MIX_ROOT` > layout > under the prefix;
`share` and `lib` are installed data, so `MIX_ROOT` does not move them. A
relative or `..`-bearing value for `etc`, `var`, `run`, `cache` or the prefix
is refused and named (`root etc: ...`); a relative `MIX_SHARE` or `MIX_LIB` is
ignored, as it always was. A late or conflicting snapshot registration is
reported in debug builds only.
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
provenance and registers the binary's package name and full SHA with
`env::register_member_snapshot`
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

## `DEV.json`, schema 1 (dev snapshot capability)

This is the marker contract for installers, including `dev_ship`. It grants
only a named development member permission to read its base gold's `env.mx`.
It does not grant integrity or authorise an install.

```json
{"schema":1,"gold":"0123456789abcdef0123456789abcdef01234567","members":{"viu":"fedcba9876543210fedcba9876543210fedcba98"}}
```

The marker is `<executable-prefix>/DEV.json`, alongside `env.mx`. An
environment variable or flag cannot select another marker. If `MIX_PREFIX`
redirects defaults away from the executable's prefix, the exception is
refused before any marker access, with the original strict error and no marker
diagnostic. Executable symlinks resolve to the real binary before prefix lookup:
launching an in-prefix binary through either an inside or foreign symlink uses
the real binary's prefix. Symlink location does not impose a launch-path policy.

The file is a restricted JSON object, at most 64 KiB of UTF-8. Exactly three
fields are required, in any order: numeric `schema: 1`, string `gold`, and
non-empty object `members`. `gold` and every member value are full, lower-case
40-hex commits. Member keys match `[a-z][a-z0-9-]*` and identify the binary's
compile-time package name (the release table's binary/package names currently
agree). Each value is that member's own build commit. List only replaced
binaries; app archives do not belong here. Strings use unescaped printable
ASCII only. JSON whitespace is allowed; duplicate keys, unknown fields,
comments, escapes, trailing commas/data, a BOM and other schema versions are
refused. Keep timestamps, receipts and app selection in the private receipt,
not this marker. Write atomically with the same ownership and trust as env.mx.

The normal stamp-equals-snapshot path never opens the marker. Only a mismatch
reads it. Acceptance requires both `gold == env.mx.commit` and
`members[registered_member] == registered_stamp`. Old `register_snapshot`
callers have no member identity and remain strict and silent, without opening
the marker. Missing markers preserve
the existing `SnapshotMismatch` result and diagnostic. A malformed, unreadable
or non-matching marker returns the same error and adds a stderr diagnostic
naming the marker. Accepted exceptions emit one stderr startup diagnostic at
the first installed-default read, cached once per process, naming member,
build commit, base commit and marker path. Service stderr belongs in its log.

`build_info!` retains the exact ASCII token `mixviu-dev-snapshot:1` in the
artifact. `dev_ship` must scan each built binary for that token (for example,
`rg -a -F -q mixviu-dev-snapshot:1 ARTIFACT`) and verify the matching embedded
buildinfo full SHA before installing it. This requires no engine execution,
display or desk access. Absence refuses engine replacement across snapshots;
older binaries cannot use this contract. Rebase branch builds onto a commit
containing this capability. A source-file check alone is insufficient.

The paths integration test compiles separate gold/dev and strict-only binaries
against the real reader, starts gold/dev under one temporary prefix, and checks
acceptance, strict gold behaviour, logging, copied member identity, symlink
resolution, parser boundaries and refusals. FIFO probes have a deadline and
prove redirected and strict-only marker paths are not opened.
Run it with the focused paths tests on a build worker.
