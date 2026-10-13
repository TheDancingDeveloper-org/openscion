# Benchmarks

`openscion-bench` times the operations developers and CI jobs actually wait for
on a deterministic synthetic repository, and prints a Markdown table of median
wall-clock times per scenario and backend.

```sh
cargo run -p openscion-benchmarks --release
cargo run -p openscion-benchmarks --release -- --directories 100 --files-per-directory 200 --runs 7
```

| Flag | Default | Meaning |
| --- | --- | --- |
| `--directories` | 10 | top-level directories in the fixture |
| `--files-per-directory` | 100 | files in each directory |
| `--file-bytes` | 4096 | size of every file |
| `--seed` | 1 | fixture seed; the same seed produces byte-identical files |
| `--runs` | 5 | runs per scenario; the report shows the median |
| `--workdir` | `target/openscion-bench` | scratch directory, emptied before every run |

## Scenarios

| Scenario | git | OpenScion |
| --- | --- | --- |
| `import` | `init` + `add -A` + `commit` of the whole tree | `PutBlobs` + `CreateChangeset` (phase 1) |
| `commit-one-file` | `commit -a` after changing one file | `PutBlobs` + `CreateChangeset` with one change (phase 1) |
| `full-clone` | `clone --no-local` | fetch of the root tree and every object (phase 1) |

Setup that a scenario does not measure, such as writing the fixture or creating
the history a commit starts from, runs before the clock starts. Git runs with
an isolated configuration (no system or global config, no hooks, no signing),
so a developer's setup cannot change the numbers.

## Status

Only the git backend exists. The OpenScion column is filled in once the engine
can create changesets and serve objects (phase 1 of
[`docs/specs/openscion-architecture.md`](../docs/specs/openscion-architecture.md)).
Until then the harness records the baselines OpenScion will be measured against.

These are coarse, whole-operation timings meant to show trends between versions
and against git. Run them on a quiet machine with `--release`; CI runs a tiny
fixture only to check that the harness works.
