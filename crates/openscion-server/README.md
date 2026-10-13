# openscion-server

The OpenScion server: the authoritative tier. In the full design it owns the
changeset log, the refs, and the derived-data indexes, and it is the only
component that creates changesets or moves refs (see
[`docs/specs/openscion-architecture.md`](../../docs/specs/openscion-architecture.md)).

It serves the object, changeset, and ref operations over an in-memory
repository. A restart starts from an empty repository.

```sh
cargo run -p openscion-server                       # listens on 127.0.0.1:7878
cargo run -p openscion-server -- --listen 0.0.0.0:7878
OPENSCION_LISTEN=127.0.0.1:9000 cargo run -p openscion-server
```

| Method | Path | Response |
| --- | --- | --- |
| `GET` | `/healthz` | `ok` while the process is serving |
| `GET` | `/v1/version` | `{ "server_version": "…", "protocol_version": [major, minor] }` |
| `POST` | `/v1/blobs` | upload blob frames (`id ‖ u64 len ‖ bytes`); answers `{ "ids": [...] }` |
| `POST` | `/v1/objects` | `{ "ids": [...] }` in; tree and blob frames out, in request order |
| `POST` | `/v1/changesets` | `CreateChangeset` in; `{ "id", "lsn", "root_tree" }` out |
| `GET` | `/v1/changesets/{id}` | the changeset and its summary |
| `GET` | `/v1/refs/{name}` | `{ "name", "target", "version" }` |
| `POST` | `/v1/refs/{name}` | `{ "expected", "new" }` compare-and-swap; `409 RefConflict` if it lost |

Failures answer with `{ "code", "message" }`: `404 NotFound`, `409 RefConflict`,
or `400` for anything the caller sent wrong. Uploaded blobs are re-hashed and
rejected with `CorruptObject` if an id does not match.
The server shuts down gracefully on Ctrl-C or SIGTERM: it stops accepting
connections and finishes in-flight requests before exiting.
