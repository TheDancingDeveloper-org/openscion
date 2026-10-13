# Roadmap

The full sequence, with what each phase delivers, is in
[`docs/specs/openscion-architecture.md`](docs/specs/openscion-architecture.md#delivery-phases).
This page tracks where things stand.

## Done

- Workspace, lints, CI (format, clippy, build, test, per-file coverage,
  rustdoc, MSRV, supply chain), and a tag-and-release workflow.
- Draft specs: architecture, data model, and API.

## Next

1. **Core data model.** Object ids and hashing, paths, blobs, trees,
   changesets, the in-memory log and refs, root-tree derivation, and the basic
   read and write operations. Replaces the template `greeting` module.
2. **Derived data and queries.** Generation numbers, skip lists, path history,
   `Log`, `Diff`, `MergeBase`, `IsAncestor`, `ListTree`, `Stat`.
3. **Server-side rebase and landing.** `Rebase`, `Land`, `Revert`, file-level
   conflicts.

## Later

Persistent stores, the `scn` client and node daemon, git ingestion, network
tiers, line-level blame and merge, virtual checkouts, read access control.

## Out Of Scope

- Replacing git for ordinary repositories.
- Offline-first operation and branch-heavy workflows.
- Code review, code search, and CI themselves. OpenScion provides the APIs
  they need.
