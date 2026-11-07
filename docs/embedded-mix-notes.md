# Embedded Mix adaptations

The repository owns its embedded Mix source. Imports remain pinned in
provenance/runtime-imports.json, with original and adapted hashes retained.
The host development shell is separate from the client artifact.

The Dopus port needs file creation time. `stat` now adds `btime` and
`btime_nsec`, the filesystem's birth timestamp as epoch seconds and its
nonnegative nanosecond component. Both are nil if the filesystem does not
provide creation time. They follow the same stat/lstat choice as other
metadata. `ctime` retains its inode-change meaning. Existing fields and
call signatures remain compatible; no timestamp is fabricated from ctime.
