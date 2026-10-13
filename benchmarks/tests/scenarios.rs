//! Runs every scenario end to end, as `openscion-bench` does.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use openscion_benchmarks::{Error, Fixture, FixtureShape, Scenario, git, openscion};

const SHAPE: FixtureShape = FixtureShape {
    directories: 2,
    files_per_directory: 2,
    file_bytes: 32,
};

type Backend = fn(Scenario, &Fixture, &Path) -> openscion_benchmarks::Result<Duration>;

fn workdir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn every_scenario_runs_against_git() {
    let fixture = Fixture::generate(SHAPE, 3);
    for scenario in Scenario::ALL {
        git::run(scenario, &fixture, &workdir(&format!("git-{scenario}"))).unwrap();
    }
    assert!(
        workdir("git-full-clone")
            .join("clone/dir0000/file0000.txt")
            .exists()
    );
}

#[test]
fn every_scenario_runs_against_openscion() {
    let fixture = Fixture::generate(SHAPE, 3);
    for scenario in Scenario::ALL {
        openscion::run(
            scenario,
            &fixture,
            &workdir(&format!("openscion-{scenario}")),
        )
        .unwrap();
    }
}

#[test]
fn both_backends_clone_the_fixture_byte_for_byte() {
    let fixture = Fixture::generate(SHAPE, 3);
    for (backend, run) in [("git", git::run as Backend), ("openscion", openscion::run)] {
        let dir = workdir(&format!("{backend}-identical"));
        run(Scenario::FullClone, &fixture, &dir).unwrap();
        for file in fixture.files() {
            assert_eq!(
                std::fs::read(dir.join("clone").join(&file.path)).unwrap(),
                file.contents,
                "{backend}: {}",
                file.path
            );
        }
    }
}

#[test]
fn both_backends_sparse_clone_only_the_first_directory() {
    let fixture = Fixture::generate(SHAPE, 3);
    for (backend, run) in [("git", git::run as Backend), ("openscion", openscion::run)] {
        let dir = workdir(&format!("{backend}-sparse"));
        run(Scenario::SparseClone, &fixture, &dir).unwrap();
        assert!(dir.join("clone/dir0000/file0000.txt").exists(), "{backend}");
        assert!(!dir.join("clone/dir0001").exists(), "{backend}");
    }
}

#[test]
fn a_fixture_without_directories_cannot_be_cloned_sparsely() {
    let empty = Fixture::generate(
        FixtureShape {
            directories: 0,
            ..SHAPE
        },
        3,
    );
    assert!(empty.first_directory().is_none());
    let error = openscion::run(Scenario::SparseClone, &empty, &workdir("openscion-no-dir"));
    assert!(matches!(error, Err(Error::NoDirectory)));
    let error = git::run(Scenario::SparseClone, &empty, &workdir("git-no-dir"));
    assert!(matches!(error, Err(Error::NoDirectory)));
}

#[test]
fn a_fixture_that_cannot_be_written_is_an_io_error() {
    let blocker = workdir("io-blocker");
    std::fs::create_dir_all(blocker.parent().unwrap()).unwrap();
    std::fs::write(&blocker, b"a file, not a directory").unwrap();

    let error = Fixture::generate(SHAPE, 3).write_to(&blocker).unwrap_err();
    assert!(matches!(error, Error::Io { .. }));
}
