# Trial: OpenScion against rustTorrent

- **Date:** 2026-10-22
- **OpenScion:** `00e92ab` (fork `TheDancingDeveloper-org/openscion`, `main`), v0.2.1, protocol 1.0
- **Subject:** a scratch `git clone --mirror` of `TheDancingDeveloper-org/rustTorrent`
  at `ae7fb6e` (the live checkout and the GitHub repository were not touched)
- **Scripts and raw numbers:** [`rusttorrent-2026-10-22/`](rusttorrent-2026-10-22/)

## Summary

OpenScion today is the **phase-1 core data model** and nothing more. It
round-trips rustTorrent's full first-parent history exactly, and the operations
it has are 5–14× faster than git on this repository. But the operations an agent
workflow depends on beyond clone and commit are missing: server Log, Diff,
Blame, Rebase and Land, durable storage, and git ingestion. **It is not usable
for our estate's agent workflow today.** It is a promising design with a correct
core; see [Verdict](#verdict) for what it would need.

## Setup

| Item | Value |
| --- | --- |
| Host | Node B (shared), i7-13700, 24 threads, 62 GiB, ext4 on NVMe; load average 3.5–5 during runs |
| Toolchain | rustc 1.99.1, release build, `CARGO_BUILD_JOBS=4` |
| git | 2.53.0; server side is a bare copy of the mirror over `file://` with `uploadpack.allowFilter` |
| OpenScion | one `openscion-server` on loopback (`127.0.0.1:7979`), `opnscn` client binary per call |
| Stores | **in-memory only.** The brief asked for local filesystem and SQLite stores; they do not exist yet (roadmap phase 4). The single-node in-memory server is the only configuration there is |
| Repository | rustTorrent: 296 commits (252 first-parent, 9 merges), 673 files and 20 MiB at HEAD, 23.7 MiB pack; 650 regular files, 23 executables, symlinks in history (768 symlink entries across first-parent snapshots) |

### Build and tests

- `cargo build --release --all-targets --all-features`: clean, 35 s.
- `cargo test --all-features`: **180 passed, 0 failed** (14 test binaries).
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: **fails on rustc
  1.99.1** with five lints that are new in this toolchain. They were noted, not
  fixed, per the operator:
  - `chunks_exact_to_as_chunks`: `crates/openscion-proto/src/object/types.rs:72`
    (the fix `as_chunks::<2>()` is within the 1.88 MSRV);
  - assert-on-emptiness lints at `crates/openscion-core/src/repo/test.rs:132`,
    `crates/openscion-core/src/encoding/test.rs:50`, `:51`, and
    `crates/openscion/src/commands/test.rs:253`.

### What is implemented vs roadmap

| Capability | State at `00e92ab` |
| --- | --- |
| BLAKE3 object ids, blobs, trees, changesets, root-tree derivation | implemented, in memory |
| Refs with compare-and-swap (`POST /v1/refs/{name}`) | implemented |
| HTTP routes | only `blobs`, `objects`, `changesets`, `changesets/{id}`, `refs/{name}`, `healthz`, `version` |
| `opnscn import / commit / clone [--path]` | implemented; `--path` takes **one** path |
| `Log`, `Diff`, `MergeBase`, `IsAncestor`, `ListTree`, `Stat`, `ReadFile` | roadmap phase 2, **not implemented** |
| `Rebase`, `Land`, `Revert`, conflict reporting | roadmap phase 3, **not implemented** |
| Filesystem `ObjectStore`, SQLite `MetadataStore` | roadmap phase 4, **not implemented** |
| Daemon, `opnscn status`/update/land | roadmap phase 5, **not implemented** |
| Git ingestion | roadmap phase 6, **not implemented** |
| Line-level blame | roadmap phase 8 ("later"), **not implemented** |
| `HasObjects`, auth, access control | not implemented |

## Import

There is no git importer, so the nearest supported path is to replay history
through the client. [`replay.sh`](rusttorrent-2026-10-22/replay.sh) walks the
first-parent chain oldest first. For each commit it checks the commit out,
mirrors the tree into an `opnscn` working copy with rsync, and runs
`opnscn import` (first commit) or `opnscn commit` with the git author and
message.

- 252 first-parent commits became **251 changesets in 18.1 s** (2,490 file
  changes); the server held them in **~55 MB RSS**.
- The one skipped commit, `0888e65`, is a merge whose tree is identical to its
  first parent. The data model allows a merge with no changes, but `opnscn`
  cannot express a second parent, so the CLI answers `nothing to commit`.
- Lost in import, by design of the replay: second parents of merges, author
  and commit dates (the server assigns `created_at`), committer identity, and
  git commit ids (kept in a side map, `map.tsv`, not in changeset metadata,
  because the CLI cannot set metadata).

Harness note: the first replay missed 7 same-size edits (version bumps in
`Cargo.toml`). That was rsync, not OpenScion: rsync's quick check compares
mtimes to the whole second by default, so it skipped files that git had
rewritten within the same second. With `--modify-window=-1` the replay is
exact. A separate test of 50 back-to-back same-size edits through
`opnscn commit` missed none, so the client's racily-clean check held up.

## Timings

Median of 5 runs, milliseconds, wall clock including process start. The
OpenScion server and git "server" were both local, so this measures CPU and
local IO, not network. Raw rows are in
[`results.tsv`](rusttorrent-2026-10-22/results.tsv).

| Scenario | git | opnscn | Notes |
| --- | ---: | ---: | --- |
| Full clone (full history) | 352.7 | **37.5** | git ships all history (23.7 MiB pack); opnscn ships only the tip snapshot |
| Full clone, `git --depth 1` | 227.0 | **37.5** | the fairer comparison: both move one snapshot |
| Sparse clone `crates/librtbit` (incl. web UI) | 155.7 (partial + sparse) | **16.7** | 294 files for opnscn; git cone mode also materializes top-level files (323) |
| Sparse clone, `git --depth 1` | 123.8 | **16.7** | |
| Commit one file | 47.1 (`commit -a`) | **7.4** | opnscn commit also publishes; git needs a push as well |
| Publish that commit | 22.3 (`push`) | (included above) | git commit + push: 69.4 |
| Log of a hot file (`crates/librtbit/src/session/mod.rs`) | **7.8** | 54.6 (client walk) | no server `Log`; walked 251 changesets over the API, results identical to `git log --first-parent` |
| Diff `main~20..main` (names) | **4.9** | 27.6 (client walk) | no server `Diff`; union of per-changeset changes, same 59 paths as git |
| Blame of the hot file | **15.7** | n/a | not implemented |
| Two concurrent changesets to `main` | 83.4 | **15.3** | both landed every run (3 attempts); opnscn only via the harness's re-clone-and-retry loop |
| 8 parallel agents (sparse clone, edit, commit, land) | 477.7 | **35.3** | all 8 landed every run; attempts git 36, opnscn 28–36 |

The log and diff walks are stand-ins written for this trial
([`history.py`](rusttorrent-2026-10-22/history.py)). They are slower than git
because they cost one HTTP round trip per changeset. A server-side `Log` with
path history would not have that cost.

### The concurrent-land path in detail

`opnscn commit` creates the changeset, then does **one** compare-and-swap of
the ref from the working copy's base. There is no rebase and no retry:

- The loser of a race gets `409 RefConflict`. Its changeset stays in the log,
  unreferenced, and its working copy is left on the old base with no command to
  move it forward. Running `commit` again makes another changeset on the same
  stale base and loses again.
- The only working recovery is the one [`agents.sh`](rusttorrent-2026-10-22/agents.sh)
  uses: delete the working copy, clone the slice again, re-apply the edit, and
  commit. That is cheap here (a slice clone is ~15 ms), but it is the agent
  doing the rebase by hand, with no file-level conflict check.
- Cost of the 8-agent runs: 28–36 attempts for 8 landings, so **20–28 orphan
  changesets per run**. Over the whole trial 246 orphans built up in the log next to
  the 356 changesets `main` moved through. Nothing collects or reports them.
- git's 36 attempts every run (1+2+…+8) show the same contention resolved by
  `pull --rebase`. git still lands each agent's own commit, without re-cloning.

## Correctness

- **History round-trip:** [`verify.sh`](rusttorrent-2026-10-22/verify.sh) pointed
  a ref at each of the 251 changesets, cloned it with `opnscn clone`, and
  compared it with `git checkout` of the same commit. **All 251 matched**:
  path set, file type, executable bit, symlink targets, every byte (`diff -r
  --no-dereference`), and the changeset's author and message.
- **Trees after the concurrent runs:** a full `opnscn clone` of the final `main`
  was compared with the git server's final `main`. All 673 files are identical
  except the eight agent-touched `lib.rs` files. Those carry exactly twice as
  many agent lines in OpenScion, because the first pass of agent runs only hit
  the OpenScion server (the git server was rebuilt between passes). No update
  was lost, and the sparse commits left every file outside their slice intact.
- **Blame:** cannot be checked. There is no blame, line-level or otherwise.
- **Log and diff:** the client-side walks agreed with `git log --first-parent`
  (27 changesets touching `crates/librtbit/src/http_api/handlers/qbit_compat.rs`) and `git diff --name-only`
  (59 paths).

## What broke and what is missing

Candidate upstream issues. These are listed for the operator, not filed.

1. **Total data loss on restart.** The server is in-memory only; a restart
   drops every object, changeset and ref. Existing working copies then fail
   with `/v1/changesets answered 404 NotFound: object <id> not found`. That is
   the generic `NotFound`, not the spec's `UnknownParent`, and the client has
   already uploaded the blobs by then.
2. **A lost compare-and-swap strands the working copy.** `commit` neither
   retries, rebases, nor offers an `update`. The orphan changeset is never
   reported or reclaimed (spec: "a changeset that no ref points at is still
   first-class", but nothing lists them).
3. **`GetObjects` fails the whole batch on one missing id** (`404 NotFound`).
   The spec says it returns what it has and lists the rest in `missing`, so
   lower tiers can forward misses.
4. **No `HasObjects`.** `import` and `commit` upload every changed blob even
   when the server already has it.
5. **`opnscn` cannot record merges or metadata.** It has no second parent, no
   `--metadata` and no way to carry git dates. A history import loses merge
   structure and empty merges are refused.
6. **One path per sparse clone.** `--path` takes a single directory, so "one
   crate plus the shared web UI" only works here because rustTorrent keeps the
   UI inside the crate.
7. **Single global `RwLock` around the repository.** Every write (blob upload,
   changeset, ref) serializes on it. That was invisible at 8 agents and could
   matter at the target scale.
8. **Clippy breaks on current stable (1.99.1)** with five new lints (listed
   above). CI will go red when its toolchain moves.

Missing for this trial: git ingestion; server `Log`, `Diff`, `Blame`,
`Rebase` and `Land`; filesystem and SQLite stores; a multi-path sparse clone;
`status` and `update` in the client.

## Verdict

**Not usable for the estate's agent workflow today.** The core is correct (an
exact round-trip of real history) and fast where it exists, and the agent shape
works: a slice clone is ~17 ms against git's ~125–155 ms, and 8 agents land in
~35 ms against ~480 ms. But we would lose everything on a restart, agents would
have to do their own rebase by re-cloning, and CI and review tools would get no
server history.

What it would need before a pilot, in roadmap order:

1. **Persistent stores** (phase 4), with crash-recovery tests. This is
   non-negotiable for any real use.
2. **Server `Rebase` and `Land`** (phase 3) with file-level conflicts, plus
   `opnscn land` so a lost race no longer strands the working copy.
3. **`Log`, `Diff`, `Stat` and `ReadFile`** (phase 2), so CI target selection
   and bisect need no checkout.
4. **Git ingestion** (phase 6) that keeps commit ids, dates and merge parents in
   metadata, plus a git export or mirror. Our forges, CI (Woodpecker, GitHub
   Actions) and review all speak git, so OpenScion would have to sit beside git,
   not replace it.
5. Auth on the HTTP API, multi-path sparse clones, and blame (phase 8) before
   any agent-facing rollout.

Our repositories are also small: rustTorrent is 20 MiB and 673 files, and git
already serves it in a quarter of a second. The scale problems OpenScion
targets (100 GB of history, millions of checkouts a day) are not ours yet. For
the estate, it is worth re-trialling when phases 3 and 4 land. Until then it is
not a reason to change our workflow.

## Reproducing

```sh
# from a scratch directory, with opnscn and openscion-server on PATH
git clone --mirror https://github.com/TheDancingDeveloper-org/rustTorrent.git rt.git
openscion-server --listen 127.0.0.1:7979 &
export OPENSCION_SERVER=http://127.0.0.1:7979
./replay.sh rt.git replay      # history -> changesets, writes replay/map.tsv
RUN=1 ./verify.sh replay       # round-trip every changeset against git
./bench.sh . 5                 # single-operation scenarios -> results.tsv
for r in 1 2 3 4 5; do for t in git opnscn; do
  ./agents.sh . $t 2 $r land-2-concurrent barrier; ./agents.sh . $t 8 $r agents-8
done; done
```
