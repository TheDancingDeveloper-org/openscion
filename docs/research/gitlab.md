# Background: GitLab

Notes on how GitLab hosts git repositories, from the storage service up to the
merge request workflow. This is source material, not a spec: what OpenScion
actually does is defined in
[`../specs/openscion-architecture.md`](../specs/openscion-architecture.md).
GitLab is useful prior art because it already runs history, merges, rebases,
and commit creation on the server, exposed as RPCs, on top of stock git.

Facts below were checked against GitLab and Gitaly documentation, design
documents, and source as of October 2026. Where a source was unclear, the text
says so.

## Components

| Component | Role |
| --- | --- |
| NGINX / load balancer | Passes HTTP through untransformed |
| GitLab Workhorse | Go reverse proxy in front of Rails (Puma); handles git over HTTP, uploads, archives |
| GitLab Rails | Application, REST and GraphQL APIs, authorization, merge request logic |
| Sidekiq | Background jobs (rebases, merges, housekeeping scheduling) |
| GitLab Shell | Handles git over SSH after the SSH server authenticates the key |
| Gitaly | gRPC service that owns every git repository on local disk |
| Praefect | Optional router and transaction manager for replicated Gitaly ("Gitaly Cluster") |
| PostgreSQL | Rails metadata; separately, Praefect's cluster metadata |
| Object storage | LFS objects, uploads, optionally external merge request diffs |

Sources: [architecture overview](https://docs.gitlab.com/development/architecture/),
[Workhorse](https://docs.gitlab.com/development/workhorse/),
[Gitaly administration](https://docs.gitlab.com/administration/gitaly/).

Every Gitaly client (Rails, Shell, Workhorse, the Elasticsearch and Zoekt
indexers, the Kubernetes agent) talks to Gitaly over gRPC. Nothing else touches
repository data. Only local storage is supported for repositories; NFS and
cloud file systems are explicitly unsupported.

## Gitaly

### Why it exists

Before Gitaly, GitLab.com mounted repositories over NFS on every application
server and background worker. The
[Gitaly design document](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/DESIGN.md)
records the evidence: P99 time to open a `Rugged::Repository` spiked to 30
seconds of wall time against about 15 ms of CPU, and one broken NFS server could
take down every application server. Its stated first design reason was "to
avoid having the Git binary operate on remote storage." The team tried Ceph
first and found its cloud performance inadequate
([*The road to Gitaly v1.0*](https://about.gitlab.com/blog/the-road-to-gitaly-1-0/),
September 2018). Migration went RPC by RPC, each behind its own feature flag,
until NFS could be unmounted from production without customer impact.

Design decisions from that document that still shape the API:

- **gRPC over HTTP+JSON**, chosen "not so much for performance reasons" but
  because gRPC is a framework with conventions, so the team did not have to
  invent one.
- **High-level operations, not object lookups.** A diff needs an unbounded
  number of object reads, so making each one an RPC was judged infeasible.
  "Defining new gRPC calls is cheap"; moving Rails logic into Gitaly is fine
  when it saves round trips.
- **A daemon, not a library**, so resources and caches are shared across all
  callers and no application host needs the storage mounted.
- **Go**, after a Ruby sidecar (`gitaly-ruby`) used 20 GB RSS for 5 requests per
  second on one server while the Go process used under 3 GB for 90.

### How it runs git

Gitaly shells out to stock `git(1)`, preferring plumbing commands, through a
command factory that guards against injection and fixes the git version and
environment ([server-side git usage](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/serverside_git_usage.md)).
Small, stable pieces such as pkt-line parsing are reimplemented in Go. Gitaly
ignores on-server git config and injects its own.

- **Process reuse.** Object reads go through long-lived `git cat-file --batch`
  and `--batch-check` processes kept in a per-repository
  [cat-file cache](https://docs.gitlab.com/administration/gitaly/configure_gitaly/#cat-file-cache)
  (default 100 entries, 10 s TTL in source).
- **Merges and rebases without a worktree.** `UserMergeToRef`, `UserMergeBranch`
  and friends use `git merge-tree --write-tree`; rebase is implemented as a
  `rev-list` todo list replayed with merge-tree based cherry-picks
  ([`merge.go`](https://gitlab.com/gitlab-org/gitaly/-/blob/master/internal/git/localrepo/merge.go),
  [`rebase.go`](https://gitlab.com/gitlab-org/gitaly/-/blob/master/internal/git/localrepo/rebase.go)).
- **Hooks.** Gitaly sets `core.hooksPath` to symlinks for a `gitaly-hooks`
  binary, which calls back into the Gitaly process over gRPC. `pre-receive`
  posts the ref updates to Rails' `/internal/allowed` endpoint for the
  authorization decision; `reference-transaction` drives Praefect voting
  ([hooks](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/hooks.md)).
- **Quarantine.** Pushed objects land in git's quarantine directory until
  `pre-receive` approves them; Gitaly passes the quarantine path to Rails so its
  access checks can read objects that are not yet in the repository
  ([object quarantine](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/object_quarantine.md)).
- **Layout.** Repositories live under hashed paths
  (`@hashed/xx/yy/<sha256>.git`), so a project rename does not move data.

### gRPC surface

Services are defined in [`proto/`](https://gitlab.com/gitlab-org/gitaly/-/tree/master/proto).
Counts are RPCs in each service on `master` at the time of writing.

| Service | RPCs | Representative calls |
| --- | --- | --- |
| `RepositoryService` | 42 | `CreateRepository`, `CreateFork`, `OptimizeRepository`, `FindMergeBase`, `GetArchive`, `SearchFilesByContent`, `ReplicateRepository`, `BackupRepository` |
| `RefService` | 16 | `ListRefs`, `FindBranch`, `UpdateReferences`, `DeleteRefs`, `ListBranchNamesContainingCommit` |
| `CommitService` | 22 | `ListCommits`, `FindCommit`, `CommitIsAncestor`, `TreeEntry`, `GetTreeEntries`, `RawBlame`, `LastCommitForPath`, `ListLastCommitsForTree`, `CheckObjectsExist` |
| `OperationService` | 16 | `UserCommitFiles`, `UserMergeBranch`, `UserMergeToRef`, `UserFFBranch`, `UserRebaseConfirmable`, `UserRebaseToRef`, `UserSquash`, `UserCherryPick`, `UserRevert`, `UserCreateBranch`, `UserApplyPatch` |
| `DiffService` | 10 | `CommitDiff`, `CommitDelta`, `RawDiff`, `DiffStats`, `FindChangedPaths`, `RangeDiff`, `DiffBlobs` |
| `BlobService` | 7 | `GetBlob`, `GetBlobs`, `ListBlobs`, LFS pointer listing |
| `ConflictsService` | 2 | `ListConflictFiles`, `ResolveConflicts` |
| `ObjectPoolService` | 6 | `CreateObjectPool`, `LinkRepositoryToObjectPool`, `FetchIntoObjectPool`, `DisconnectGitAlternates` |
| `SmartHTTPService` | 5 | `InfoRefsUploadPack`, `PostUploadPackWithSidechannel`, `PostReceivePack`, `AdvertiseBundleURI` |
| `SSHService` | 3 | `SSHUploadPackWithSidechannel`, `SSHReceivePack`, `SSHUploadArchive` |

`OperationService` is the server-side write path. Its RPCs take a `User`, run
the access-check hooks as that user, and update a ref. Details worth noting:

- **`UserCommitFiles`** is a client stream: a header (branch, message, author,
  optional `start_sha` or `start_branch_name`, optional `start_repository` for
  cross-fork commits) followed by actions with content. It builds a commit and
  moves the branch without any clone.
- **`expected_old_oid`** on `UserCommitFiles` and `UserMergeBranch` is a
  compare-and-swap guard: if set, the branch must still point there or the RPC
  fails; the zero OID means "create". If unset, the branch is overwritten.
- **Two-phase confirmation.** `UserMergeBranch` and `UserRebaseConfirmable`
  first return the commit they computed; only a second message with
  `apply = true` updates the ref. Rails uses the gap to record the merge commit
  ID before it becomes visible.
- **`UserMergeToRef` and `UserRebaseToRef`** write a merge or rebase result to
  an arbitrary ref (overwriting it) without touching the target branch. They
  exist for internal refs such as `refs/merge-requests/*`; `UserRebaseToRef`
  does not run hooks.
- **`UserFFBranch`** fast-forwards a branch to a given commit and returns an
  empty response if the ref update loses a race.

### Bulk transfer and caching

Clone and fetch run `git upload-pack`; push runs `git receive-pack`. Two
mechanisms keep that path affordable:

- **Sidechannels.** gRPC message framing limited how much fetch traffic a
  server could serve, so since GitLab 14.4 upload-pack bytes travel on a
  separate stream multiplexed over the same TCP connection, with the gRPC call
  used only for control ([sidechannel](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/sidechannel.md)).
- **Pack-objects cache.** Git's `uploadpack.packObjectsHook` routes
  `git pack-objects` through `gitaly-hooks` back into Gitaly, which keys the
  request and streams a cached response to concurrent identical fetches. It was
  built for one problem, CPU load from massively parallel CI fetches, and it
  streams unfinished entries with backpressure so misses are not slower. Data
  lives in local files (relying on the page cache), metadata in process memory;
  object storage was rejected as a new failure point without a page cache. It is
  off by default because it can sharply increase disk writes, and it does
  nothing for unique fetches
  ([design notes](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/design_pack_objects_cache.md),
  [admin docs](https://docs.gitlab.com/administration/gitaly/configure_gitaly/#pack-objects-cache)).
- **Bundle URIs.** Gitaly can advertise a pre-built bundle (manually created
  or generated automatically when clones of a repository are frequent) so the
  client downloads most objects from a CDN and fetches only the remainder
  ([bundle URI](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/bundle-uri.md)).
- **Concurrency limits.** Per-RPC, per-repository limits and per-IP
  pack-objects limits queue or reject work; an optional adaptive mode adjusts
  them with AIMD. GitLab.com runs stricter limits than the self-managed default
  ([concurrency limiting](https://docs.gitlab.com/administration/gitaly/concurrency_limiting/)).

### Housekeeping

Gitaly runs its own maintenance instead of `git gc --auto`, which the team
found gave too little control and too little observability
([*Scaling repository maintenance*](https://about.gitlab.com/blog/scaling-repository-maintenance/)).
Rails used to call fine-grained RPCs (`RepackFull`, `PackRefs`,
`GarbageCollect`) every few pushes; now it calls one black-box
`OptimizeRepository`, which inspects on-disk state and does only what is
needed:

- **Heuristic strategy.** Thresholds on loose objects, packfile count (allowed
  count grows with total size, so large repositories are fully repacked less
  often), loose-to-packed ref ratio, and stale commit-graphs. The rollout more
  than halved CPU spent on optimization on GitLab.com.
- **Geometric repacking** (git 2.32, enabled globally in GitLab 16.0) merges
  only the small end of a geometric sequence of packs instead of rewriting all
  of them. **Cruft packs** (git 2.37) hold unreachable objects with per-object
  mtimes so the 14-day grace period no longer needs loose objects. Repacking
  time on the largest `gitlab-org` and `gitlab-com` repositories fell by about
  80% ([*Rearchitecting object database maintenance*](https://about.gitlab.com/blog/rearchitecting-git-object-database-mainentance-for-scale/)).
- **Triggers.** After a configurable number of pushes, on manual request, and a
  scheduled daily window (default 10 minutes, repositories in random order) for
  repositories that receive no pushes
  ([housekeeping](https://docs.gitlab.com/administration/housekeeping/)).

The old all-into-one repack could allocate large amounts of memory and take
hours on a monorepo. Housekeeping is the cost Scion's "no packfiles" principle
removes.

### Object pools for forks

A fork shares most objects with its upstream, so Gitaly creates an **object
pool**: a hidden repository, seeded by a `--local` clone of the source, that
members reference through `objects/info/alternates`
([object pools](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/object_pools.md)).

- Linking deletes the member's pack bitmaps, because git can use only one
  bitmap.
- `FetchIntoObjectPool` updates the pool from the primary member with a
  non-pruning fetch, packs refs, and repacks into a single pack with a bitmap
  and delta islands on `refs/heads` and `refs/tags`.
- **Objects in a pool can never be pruned.** No moment exists when every
  member's refs can be read race-free, so each newly dangling object gets a
  `refs/dangling/<oid>` ref. On busy repositories these refs pile up and make
  pool fetches and ref repacking expensive.
- Leaving a pool hard-links all pool objects into the member, removes the
  alternates file, and runs `git fsck`; on failure the alternate is restored.

## Gitaly Cluster (Praefect)

### Architecture

Praefect sits between clients and a set of Gitaly nodes forming a **virtual
storage**, and is described as "a router and transaction manager for Gitaly"
([Praefect docs](https://docs.gitlab.com/administration/gitaly/praefect/)).
Several Praefect instances run behind a load balancer and share a PostgreSQL
database holding repository metadata, replica assignments, and the replication
queue. By default every repository is replicated to every storage in the
virtual storage; a lower replication factor can be configured.

- **Writes** to RPCs that support transactions go to all healthy, up-to-date
  replicas synchronously. Replicas that fail, or RPCs that do not support
  transactions, get asynchronous **replication jobs** from the primary.
- **Reads** for RPCs marked `ACCESSOR` go to any up-to-date healthy replica,
  meaning one with no pending replication jobs. Otherwise the primary serves
  them.
- **Metadata atomicity** comes from PostgreSQL, not the disks. Creation
  reserves an ID from a sequence and writes replicas under a path derived from
  it; deletion removes the metadata row and cleans disks later; moves only
  rewrite the relative path. Leftover on-disk state is expected.
- **Failover** promotes the healthy secondary a majority of Praefects agree on,
  preferring the one with the fewest failed replication jobs. A repository whose
  primary is outdated goes read-only until reconciled, so it cannot accept
  writes that conflict with unreplicated ones.

### Voting on ref updates

Strong consistency is scoped to ref updates, on the argument that every
user-visible mutation ends in one
([HA design](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/design_ha.md)):

1. Praefect proxies the mutating RPC to every replica.
2. Each Gitaly runs git; git's `reference-transaction` hook fires before refs
   are committed.
3. The hook sends Praefect a **vote**: a hash of the old and new values of the
   refs it is about to update.
4. Praefect commits when quorum agrees and aborts otherwise. Strategies are
   `strong` (all nodes), `primary-wins`, and `majority-wins` (primary plus at
   least half the secondaries).
5. Outvoted or failed nodes are repaired by replication jobs.

An RPC can cast several votes in order (subtransactions); all must pass.

The voting protocol deadlocked under concurrency. At the `prepared` phase, git
already holds on-disk ref locks, so two transactions locking overlapping refs
in different orders on different nodes wait on each other forever. The fix
([serialized writes](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/serialized_writes.md))
needed a new upstream git hook phase, `preparing`, first released in git
2.54.0, and a PostgreSQL-backed lock per repository taken at that phase and
held until `committed`. An in-process mutex would not work across Praefect
instances. The documented cost: at least one extra database round trip per
write, and single-repository write throughput "upper-bounded by
`1 / lock_hold_duration`". Writes to different repositories do not contend.

### Known problems

From the [Praefect docs](https://docs.gitlab.com/administration/gitaly/praefect/#known-issues)
and design notes:

- A subset of operations still use replication jobs, so some writes are only
  eventually consistent; unreplicated writes can be lost on failover. Stated
  objectives for a single-node failure are RPO under one minute and RTO under 10
  seconds.
- `PostReceiveHook` can run before the write is replicated, so CI jobs routed
  to a lagging replica fail with `couldn't find remote ref
  refs/merge-requests/$iid/{head,merge}`.
- On a heavily written monorepo, writes outpace replication and the primary
  ends up serving most reads, so distribution stops helping exactly where it is
  needed.
- Restoring one node from a snapshot breaks reconciliation; snapshot backups
  are unsupported. Kubernetes support is beta; ECS is unsupported. The on-disk
  layout is internal and may change.

### The abandoned Raft and write-ahead log direction

GitLab spent several years on a replacement. The
[transaction management blueprint](https://handbook.gitlab.com/handbook/engineering/architecture/design-documents/gitaly_transaction_management/)
(created 2023) listed ways Gitaly violated ACID: refs updated one by one and
left half-done on interruption, quarantine migration ignoring object
dependencies, backups capturing states that never existed, missing fsyncs. It
proposed per-partition write-ahead logs with snapshot isolation, as the first
stage of Raft replication that would remove Praefect and PostgreSQL
([epic 8903](https://gitlab.com/groups/gitlab-org/-/epics/8903)).

The implementation ([transactions](https://gitlab.com/gitlab-org/gitaly/-/blob/master/doc/transactions.md))
worked around git rather than inside it:

- **Partitions.** Each repository is its own partition, except that an object
  pool and all its members share one so a fork's refs cannot be logged before
  the pool objects they need. Transactions never span partitions.
- **Snapshots** are made by copying the directory tree and hard-linking files,
  so their cost "scales by the number of files and directories" in the
  repository. Read-only transactions at the same log position share one.
- **Log entries** are a manifest of hard-link and remove operations plus the
  new files (packs, ref files) git wrote. A single goroutine per partition
  applies the log.
- **Conflict checks** happen at commit: file-level (two transactions touching
  `packed-refs` conflict even when their refs differ), ref-level, and object
  dependency checks. Write skew is not prevented.

As of September 2026 the blueprint is marked **rejected**: "the
Gitaly-Cluster-without-Praefect direction has been dropped," transactions are
disabled in staging, and the WAL, transaction, and Raft code is being removed.
Praefect, with serialized writes, remains the replication design.

A related rollout moves repositories to git's **reftable** backend (git 2.45),
because deleting a ref from `packed-refs` rewrites the whole file: `DeleteRefs`
averaged about 20 times the latency of `WriteRef`
([reftable blueprint](https://handbook.gitlab.com/handbook/engineering/architecture/design-documents/gitaly_reftable_rollout/);
it reports a staging rollout, and production status was not clear from it).

## Request flows

**HTTP fetch or clone**
([architecture](https://docs.gitlab.com/development/architecture/)):

1. `GET /info/refs?service=git-upload-pack` reaches Workhorse, which asks Rails
   to authorize it.
2. Workhorse calls `SmartHTTPService.InfoRefsUploadPack`; Gitaly runs
   `git upload-pack --stateless-rpc --advertise-refs`.
3. `POST /git-upload-pack` is authorized by Rails again, then streamed to
   `PostUploadPackWithSidechannel`. Push uses `git-receive-pack` the same way.

**SSH fetch.** The SSH server's `AuthorizedKeysCommand` asks Rails which key ID
owns the key; GitLab Shell calls Rails' `/internal/api/allowed` with the action
and key ID, receives Gitaly connection details, and proxies the session to
`SSHUploadPackWithSidechannel` (or `SSHReceivePack`).

**Push, inside Gitaly.** `git receive-pack` writes objects to quarantine,
`pre-receive` calls Rails `/internal/allowed` (which may call back into Gitaly,
for example `CommitIsAncestor`, against the quarantine), custom hooks run, refs
update (voted on under Praefect), and `post-receive` tells Rails to enqueue
pipelines, merge request refreshes, and housekeeping counters.

**UI or API write.** Rails calls an `OperationService` RPC directly. The same
hooks run, so access checks apply identically to pushes and server-side
commits.

## Merge requests

### Refs

| Ref | Contents | Written by |
| --- | --- | --- |
| `refs/merge-requests/<iid>/head` | Source branch tip, fetched into the target project even from forks | On push to the source branch; deleted 14 days after close or merge |
| `refs/merge-requests/<iid>/merge` | Result of merging source into target, as the target would look after a merge | `UserMergeToRef`, refreshed by mergeability checks and the [`merge_ref` API](https://docs.gitlab.com/api/merge_requests/#merge-to-default-merge-ref-path) |
| `refs/merge-requests/<iid>/train` | Merge train result with all earlier cars | Merge train ([source](https://gitlab.com/gitlab-org/gitlab/-/blob/master/app/models/merge_request.rb)) |

Every open merge request therefore costs the target repository at least two
refs, and CI pipelines run against them. Clients can fetch them with the refspec
`+refs/merge-requests/*/head:refs/remotes/origin/merge-requests/*`
([troubleshooting](https://docs.gitlab.com/user/project/merge_requests/merge_request_troubleshooting/)).
Merge request diffs shown in the UI are size-limited copies stored in the
`merge_request_diff_files` database table by default, optionally in object
storage ([diff storage](https://docs.gitlab.com/administration/merge_request_diffs/)).

### Merge methods

From [merge methods](https://docs.gitlab.com/user/project/merge_requests/methods/):

| Method | History | Requires up-to-date source |
| --- | --- | --- |
| Merge commit | Always a merge commit | No |
| Merge commit with semi-linear history | Merge commit, but only when a fast-forward would be possible | Yes; rebase offered |
| Fast-forward | Linear, no merge commits | Yes; rebase offered |

- **Squash** first builds one commit with `UserSquash`; with a merge commit
  method the result is two commits on the target, with fast-forward just one.
  Projects choose do-not-allow, allow, encourage, or require.
- **Server-side rebase** (UI button, `/rebase`, or
  `PUT /merge_requests/:iid/rebase`) is queued to Sidekiq and returns `202` with
  `rebase_in_progress`; callers poll. It works only when conflict-free and drops
  GPG signatures. An automatic rebase at merge time does not re-run CI on the
  rebased result.
- **The merge call** (`PUT /merge_requests/:iid/merge`) accepts `sha`, which must
  equal the source head or the call fails with `409`; groups can make it
  mandatory so only reviewed commits merge. It also takes `squash` and
  `auto_merge`.

### Mergeability, approvals, and code owners

The API exposes `detailed_merge_status` with values including `checking`,
`conflict`, `need_rebase`, `ci_must_pass`, `not_approved`,
`discussions_not_resolved`, `requested_changes`, `status_checks_must_pass`,
`locked_paths`, and `mergeable`
([merge status](https://docs.gitlab.com/api/merge_requests/#merge-status)).
Each check is a class run by `RunChecksService`, optionally cached; the team
targets 10 to 20 ms per check because checks run very frequently, and the
approval check is singled out as the slow one
([mergeability framework](https://docs.gitlab.com/development/merge_request_concepts/mergeability_framework/)).

- **Approval rules** (Premium) name users or groups and a required count, as
  project defaults or per merge request. Fork merge requests use the target
  project's rules
  ([rules](https://docs.gitlab.com/user/project/merge_requests/approvals/rules/)).
- **Settings** can forbid author approval and approval by committers, require
  re-authentication, reset all approvals on new commits, or reset only code
  owner approvals whose files changed
  ([settings](https://docs.gitlab.com/user/project/merge_requests/approvals/settings/)).
- **CODEOWNERS** maps path patterns to owners, read from the target branch.
  Owner approval becomes required only when the target is a protected branch
  with code-owner approval enabled; unmatched files need none
  ([Code Owners](https://docs.gitlab.com/user/project/codeowners/)).

Rules are path- and membership-based. Nothing documented here classifies a
change by its semantics, which is what Scion's Antique does.

### Merge trains

A merged results pipeline tests source plus current target, but two merge
requests can pass separately and break the target together. Merge trains
(Premium) fix this
([merge trains](https://docs.gitlab.com/ci/pipelines/merge_trains/)):

1. Adding a merge request creates a car whose train ref is the merge of its
   source onto the previous car's train ref (or the target for the first car),
   built with `first_parent_ref` set to that previous ref.
2. A pipeline runs on each car; car *n* tests the target plus cars 1 to *n*.
   Pipelines run in parallel, up to 20 per train by default (configurable per
   project, minimum 1 for strictly sequential). Extra cars queue without limit.
3. A car merges only after its pipeline passes and every car ahead of it has
   merged.
4. If a car fails, it is removed and every pipeline behind it is cancelled and
   recreated without it. "Merge immediately" also restarts the whole train.

The landing step is a fast-forward
([`FromTrainRef`](https://gitlab.com/gitlab-org/gitlab/-/blob/master/ee/app/services/merge_requests/merge_strategies/from_train_ref.rb)):
Rails re-runs mergeability checks, verifies the train ref still points at the
SHA the pipeline tested, then fast-forwards the target branch to it. The tested
commit is exactly the commit that lands. The code comments warn the train ref
"must be constructed to be exactly like the desired target branch."

The cost is speculative CI. One failure near the head of a 20-car train
discards up to 19 in-flight pipelines.

## Server-side changes over the API

| Endpoint | What it does |
| --- | --- |
| [`POST /projects/:id/repository/commits`](https://docs.gitlab.com/api/commits/#create-a-commit) | One commit from a batch of `actions` (`create`, `update`, `delete`, `move`, `chmod`), content as text or base64, no clone. `start_branch` or `start_sha` (optionally from `start_project`) picks the parent; `force` replaces history; per-action `last_commit_id` rejects edits to a file changed since. Requests over 300 MB are rejected; over 20 MB are rate limited to 3 per 30 s. |
| GraphQL [`commitCreate`](https://docs.gitlab.com/api/graphql/reference/#mutationcommitcreate) | The same, with `CommitAction` inputs |
| [Repository files](https://docs.gitlab.com/api/repository_files/) | Read, raw read, blame (with line ranges), and single-file create, update, delete, each a commit |
| [Compare](https://docs.gitlab.com/api/repositories/#compare-branches-tags-or-commits) | Commits and diffs between two refs; merge-base (`...`) by default, `straight` for direct. Diffs may be truncated with `compare_timeout` |
| [Cherry-pick, revert](https://docs.gitlab.com/api/commits/) | Server-side, onto a named branch |

All of these reach Gitaly through `UserCommitFiles` and its siblings. The REST
API does not expose `expected_old_oid` on commit creation; the closest guards
are `start_sha` and per-file `last_commit_id`.

## Large repositories

GitLab's [monorepo guidance](https://docs.gitlab.com/user/project/repository/monorepos/)
is mostly about reducing what git asks the server to do:

- **Server side:** enable the pack-objects cache, publish bundle URIs, raise
  negotiation timeouts, size hardware to a reference architecture, prune old
  branches and hide refs with `transfer.hideRefs` (set in Gitaly config, since
  on-server git config is ignored), and schedule housekeeping.
- **CI:** runners shallow-clone by default and use the `fetch` strategy;
  `GIT_DEPTH`, `GIT_CLONE_EXTRA_FLAGS=--filter=blob:limit=1m`, `--no-tags`,
  disabling `git clean`, staggering scheduled pipelines.
- **Partial clone** (git 2.22+) works with `blob:limit`, `blob:none` plus
  sparse checkout, and an experimental `sparse:oid` filter that GitLab warns may
  be slow and costly on the server
  ([reduce clone size](https://docs.gitlab.com/topics/git/clone/)).
- **LFS** stores large binaries in object storage behind a pointer file
  ([Git LFS](https://docs.gitlab.com/topics/git/lfs/)).
- Profile with `git-sizer` and avoid shallow clones for development work.

## Where cost scales

| Cost driver | Scales with | GitLab mitigation |
| --- | --- | --- |
| Clone, full fetch | Reachable objects; `git pack-objects` is "the most resource intensive operation" | Bitmaps, pack-objects cache (identical concurrent fetches only), bundle URIs, shallow and partial clone |
| Ref advertisement and walk | Ref count, on every fetch | Pack refs, hide refs, reftable |
| Ref deletion | Size of `packed-refs` (whole-file rewrite) | Reftable |
| Repacking | Total objects; all-into-one took hours on monorepos | Geometric repacking, cruft packs, heuristics |
| Object pool upkeep | Dangling refs that can never be dropped | Delta islands; none for growth |
| Replication | Write rate per repository | Primary absorbs reads when replicas lag |
| Serialized writes | One lock per repository | Throughput capped at `1 / lock_hold_duration` |
| WAL snapshots (abandoned) | Files and directories in the repository | Shared read snapshots |
| Merge request refs | Open and recent merge requests, two or three refs each | Deletion 14 days after close |
| Mergeability checks | Merge requests times checks, run frequently | Per-check caching |
| Merge trains | Cars behind a failure | Parallelism cap |

Write cost on the server is mostly proportional to the change: merge-tree and
`UserCommitFiles` do not need a worktree. Everything around the write (ref
storage, replication, housekeeping, transfer) grows with the repository.

## What this means for OpenScion

- **Server-side changeset creation has strong precedent.** `UserCommitFiles`
  and the Commits API `actions` array are the same idea as OpenScion's
  changeset writes: a client names file-level changes against a parent and the
  server builds the commit. GitLab's action set (`create`, `update`, `delete`,
  `move`, `chmod`, base64 content, per-file `last_commit_id`) is a tested shape
  worth matching, and matching it eases porting tools. Unlike GitLab, OpenScion
  separates writing a changeset from moving a ref.
- **Single-record CAS versus Praefect voting.** Praefect needed votes from every
  replica through a git hook, a new upstream git hook phase, and a PostgreSQL
  lock per repository to stop deadlocks, ending with writes serialized per
  repository. Its remaining failure modes (lagging replicas, CI fetching refs
  that are not there yet, read-only repositories after failover) all come from
  replicating mutable file-system state. OpenScion's split of immutable,
  content-addressed objects from a small set of refs moved by single-record
  compare-and-swap removes the need for votes: an object is either present or
  fetchable, and only the ref record needs consistency. `expected_old_oid` shows
  that even Gitaly exposes the CAS primitive at the RPC level.
- **Do not retrofit transactions onto git's file layout.** GitLab's WAL and
  snapshot work was abandoned after three years. Snapshot cost grew with
  repository file count, and conflicts arose at the file level (`packed-refs`)
  as well as the logical level. That is evidence for OpenScion's choice of a
  storage model designed for atomic single-record updates over wrapping git.
- **Merge trains versus lock-free `Land`.** A merge train lands exactly the
  tested commit by fast-forward, at the price of speculative CI that is thrown
  away on any failure and a queue whose throughput is bounded by pipeline
  length times parallelism. OpenScion's `Land` rebases onto whatever the tip is
  and retries the swap, and leaves breakage to post-merge testing, as Scion
  does. A review or CI system that wants train semantics can build them on top:
  speculatively compose changesets server-side (cheap, no refs needed), test
  them, then `Land` with the tested tip as the expected value.
- **Merge request refs are a scaling trap.** GitLab pays two or three refs per
  merge request, and its ref-count problems (advertisement, `packed-refs`
  rewrites, reftable migration) follow. OpenScion's rule that work in progress
  is a changeset, not a ref, avoids this. Review tools should address
  changesets by ID.
- **gRPC is a reasonable transport, with a caveat.** Gitaly chose gRPC for its
  conventions and has run it at GitLab.com scale for years. But per-message
  framing limited fetch throughput enough that Gitaly added sidechannels for
  bulk bytes. If OpenScion picks gRPC, object and blob streaming should either
  use large messages, a side stream, or a plain HTTP path that tiered caches
  can serve. Gitaly's rule of preferring high-level RPCs to chained low-level
  calls also applies to history, blame, and diff in OpenScion.
- **Caching lessons.** The pack-objects cache only helps identical concurrent
  fetches and is off by default because of write amplification. OpenScion's
  content-addressed objects make every response cacheable, which is the
  property the cache tries to recover. Gitaly's choice of local files plus the
  page cache over object storage for a hot cache is relevant to `scion-cache`.
- **Interop.** Two concrete options, neither evaluated in depth:
  1. Offer a GitLab-compatible subset of the Commits, Repository files, and
     Compare REST endpoints over OpenScion, so existing bots and CI scripts work
     unchanged.
  2. Implement a subset of Gitaly's protos (`CommitService`, `BlobService`,
     `DiffService`, parts of `OperationService`) as a bridge. This is much
     larger, since Gitaly RPCs assume git object IDs, branches, and hooks that
     call back into Rails; it is noted as a possibility, not a plan.
- **Housekeeping is an ongoing cost GitLab cannot remove.** Years of tuning
  (heuristics, geometric repacks, cruft packs, pool dangling refs) cut repack
  time by 80% but did not remove it. Avoiding packfiles in the authoritative
  store, as Scion does, is the larger win, and any git-ingestion path in
  OpenScion should not reintroduce them on the serving side.
