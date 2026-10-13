# openscion-proto

Every payload type that crosses an OpenScion process boundary, and the protocol
version that governs them.

OpenScion's engine, node daemon, regional cache, server, and `scn` client all
speak one API (see [`docs/specs/openscion-api.md`](../../docs/specs/openscion-api.md)).
A tool that only makes calls — a CI job, a review bot, a bisect pipeline —
needs the vocabulary, not the engine. This crate is the vocabulary.

| module     | what it holds                                                  |
| ---------- | -------------------------------------------------------------- |
| `object`, `path`, `tree`, `changeset`, `refs`, `transfer`, `failure` | the phase-1 data model payloads |
| `version`  | `PROTOCOL_VERSION` and the bind rule a client applies to it    |

Two dependencies, both pure Rust: `serde` and `serde_json`.

## This crate sits underneath `openscion-core`

`openscion-core` **depends on this crate and re-exports all of it**. A client needs
the payload types and nothing else; making it depend on the engine to spell a
request type would be the wrong shape. A parallel set of payload types for
clients would be worse: two definitions of one request are two distinct types,
with a conversion at every call site that nothing checks.

## What is deliberately absent

**No behavior.** Hashing, validation, and every operation live in
`crates/openscion-core`. A payload type describes what a request carries, not what
the server does with it.

**No transport.** No connection, client, codec, async runtime, HTTP client, or
native library. The transport is still an open question in the architecture
spec, and whatever it turns out to be, it carries these types. CI asserts the
dependency tree stays this small.

## Versioning

`PROTOCOL_VERSION` describes *this vocabulary*, not the package. Bump its major
component when a payload's wire form changes incompatibly or an operation is
removed or renamed, and its minor component when an operation or an optional
field is added. It is independent of the package version the release workflow
owns.

The payload tests pin the serde representation, because that representation is
the wire form: a client and a server that disagree about a field name fail at
runtime with a decode error, so the shape is asserted rather than assumed.
