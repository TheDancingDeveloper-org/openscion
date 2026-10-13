# openscion-cli

`scn`, the OpenScion command-line client. In the full design it lays out a
sparse working copy, watches it for edits, uploads blobs, and wraps the server's
history and landing APIs (see
[`docs/specs/openscion-architecture.md`](../../docs/specs/openscion-architecture.md)).
Today it implements the commands the server can already answer.

```sh
cargo run -p openscion-cli -- version
cargo run -p openscion-cli -- server-info                       # default http://127.0.0.1:7878
cargo run -p openscion-cli -- server-info --server http://host:7878
OPENSCION_SERVER=http://host:7878 cargo run -p openscion-cli -- server-info
```

| Command | What it does |
| --- | --- |
| `scn version` | prints the client and protocol versions |
| `scn server-info` | asks the server for its version and checks that the protocol binds |

The client speaks plain HTTP only for now. TLS will be added once a deployment
needs it.
