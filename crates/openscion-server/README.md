# openscion-server

The OpenScion server: the authoritative tier. In the full design it owns the
changeset log, the refs, and the derived-data indexes, and it is the only
component that creates changesets or moves refs (see
[`docs/specs/openscion-architecture.md`](../../docs/specs/openscion-architecture.md)).

Today it serves the discovery endpoints every client calls first. API
operations are added as the engine implements them.

```sh
cargo run -p openscion-server                       # listens on 127.0.0.1:7878
cargo run -p openscion-server -- --listen 0.0.0.0:7878
OPENSCION_LISTEN=127.0.0.1:9000 cargo run -p openscion-server
```

| Method | Path | Response |
| --- | --- | --- |
| `GET` | `/healthz` | `ok` while the process is serving |
| `GET` | `/v1/version` | `{ "server_version": "…", "protocol_version": [major, minor] }` |

The server shuts down gracefully on Ctrl-C or SIGTERM: it stops accepting
connections and finishes in-flight requests before exiting.
