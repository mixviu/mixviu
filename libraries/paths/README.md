# paths

Dependency-free MixViu roots and `mix:` paths. `Root::Lib` follows the prefix,
like `Share`. Existing constructors retain the legacy system tier;
`Dirs::derived(prefix, env)` exposes the future prefix-relative system tier.
`prefix_from_exe_strict` requires a `bin` parent without changing the legacy
executable resolver.

`env` owns the registered variable names (`env::test` for test names).
`env::raw` treats empty overrides as unset. `env::setting` reads an override
first, otherwise a lazy process-wide `env.mx` snapshot under the prefix.
The bounded flat map requires `schema: 1`, `commit: '<sha>'`, and quoted
string settings. `mix:` values resolve through the process roots.
Missing defaults, unreadable/malformed files, unknown schemas and snapshot
mismatches have distinct errors. Paths has no build script or compiled commit.
The binary registers its full SHA with `env::register_snapshot` before reading
settings; the first registration wins and conflicting later values are ignored.
With no registration, the commit check is skipped. Registration after the
first installed setting read does not revalidate that cached result.
`stamp!` expands at the caller's compile site; buildinfo uses it for consumer
provenance and registers known full SHAs when `build_info!` runs.

Process, user and system layouts resolve in separate lazy caches. Valid
layout overrides retain their precedence; a broken system layout cannot
block user roots. The existing no-HOME user fallback to `/root` remains
until the remove PR.
