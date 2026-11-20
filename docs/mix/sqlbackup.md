# sqlbackup — online SQLite file backup

`sqlbackup(src_path, dst_path[, opts])` uses SQLite's online backup API to
copy all pages from a consistent read snapshot, including committed WAL
rows, while the writer remains open. It runs `PRAGMA integrity_check` on
the copy and publishes it only when the sole result is `ok`.

```mix
$copy = sqlbackup("/srv/app/state.db", "/srv/backups/state.db")
print($copy.integrity)
```

The result is `{ok: true, pages, bytes, integrity: "ok", duration_ms}`.
`pages` is the copied page count; `bytes` is the standalone destination
file size; `duration_ms` is elapsed wall time, including retries and checking.
Failures raise, rather than returning `ok: false`.

Options are a map. Wrong top-level argument types raise `TYPE_MISMATCH`;
unknown option keys and invalid option values raise `OPTION_INVALID`:

| Option | Default | Contract |
|---|---|---|
| `overwrite` | `false` | Boolean. Refuse an existing destination unless true. |
| `mode` | `0o600` | Integer Unix permission bits, `0..0o777`; applied exactly before publication. |
| `retry_ms` | `5000` | Integer `0..60000`; bounded busy/locked retry deadline, shared by snapshot acquisition and copying. Zero means no retry. |

The source must be an existing filesystem database, opened read-only.
Neither `:memory:` nor SQLite URI paths are supported. The copy is staged
in the destination directory with mode `0600`, checked, synced, closed
and published atomically. A failed copy preserves an existing destination.
The published copy uses DELETE journal mode, so it needs no WAL sidecars.
The source's journal mode stays unchanged. A pinned read transaction keeps
the snapshot stable during copying; a writer may continue committing, but
its WAL can grow until the copy completes.

Use overwrite only for an offline backup file. The destination must have
no open users; symlinks, the source itself and destinations with SQLite
sidecars are refused. Sidecar checks cannot detect every open user or
protect against another process opening the destination concurrently.
Use a directory trusted by the operator. Publication uses a same-directory
hard link when overwrite is false, and rename when true; the filesystem
must support that operation. Unix modes are enforced on Unix hosts.

| Error code | Meaning |
|---|---|
| `SQL_NO_DATABASE` | Source does not exist. |
| `SQL_DEST_EXISTS` | Destination exists and overwrite is false. |
| `SQL_BUSY`, `SQL_LOCKED` | SQLite could not acquire a lock within the retry budget. |
| `SQL_IO` | Filesystem, open, read-only, full-disk or SQLite I/O failure. |
| `SQL_INTEGRITY` | Destination integrity check did not return only `ok`. |
| `SQL_ERROR` | Other SQLite failure, including an invalid database. |
| `OPTION_INVALID` | Invalid paths/options or unsafe overwrite target. |
| `TYPE_MISMATCH` | A path is not a string or opts is not a map. |

This builtin is in the **shell profile**, behind the existing `sqlite`
feature, just like `sqlopen`/`sqlexec`/`sqlclose`. It requires `FsWrite`
authority; it is available to server `mix` scripts without `mixd run`.

There is no `sqlrestore` builtin. To restore: stop the writer and all readers,
close their connections, move the old database and any residual `-wal`,
`-shm` and `-journal` files aside as a set, then rename the backup into place.
Set the restored file's owner and mode for the service before restarting it.
Never rename a backup over an open SQLite database.
