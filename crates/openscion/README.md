# openscion

`scn`, the OpenScion command-line client. In the full design it lays out a
sparse working copy, watches it for edits, uploads blobs, and wraps the server's
history and landing APIs (see
[`docs/specs/openscion-architecture.md`](../../docs/specs/openscion-architecture.md)).
Today it implements import, commit, and clone against an `openscion-server`.

```sh
scn version
scn server-info                                   # default http://127.0.0.1:7878
scn --server http://host:7878 import ./tree       # first changeset, creates `main`
scn commit -m "edit the readme" --dir ./tree      # record changes, move `main` by CAS
scn clone ./copy                                  # the whole snapshot
scn clone ./slice --path services/api             # one directory of it
```

| Command | What it does |
| --- | --- |
| `scn version` | prints the client and protocol versions |
| `scn server-info` | asks the server for its version and checks that the protocol binds |
| `scn import [DIR]` | records a directory as the repository's first changeset and creates the ref |
| `scn commit -m MSG [--dir DIR]` | records every change since the base changeset and moves the ref by compare-and-swap |
| `scn clone DEST [--path P]` | fetches a snapshot, or only the directory `P`, into a new working copy |

`--server` (or `OPENSCION_SERVER`) picks the server; `--ref` picks the ref
(default `main`); `--author` (or `OPENSCION_AUTHOR`) sets the author. A working
copy keeps its state in `.scn/state.json`, and `commit` talks to the server
the working copy came from. If someone else moved the ref first, `commit`
fails with `RefConflict`: there is no rebase yet.

The client speaks plain HTTP only for now. TLS will be added once a deployment
needs it.
