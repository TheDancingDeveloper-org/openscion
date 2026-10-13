//! Runs every scenario end to end, as `openscion-bench` does.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use openscion_benchmarks::{Error, Fixture, FixtureShape, Scenario, git};

const SHAPE: FixtureShape = FixtureShape {
    directories: 2,
    files_per_directory: 2,
    file_bytes: 32,
};

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
fn a_fixture_that_cannot_be_written_is_an_io_error() {
    let blocker = workdir("io-blocker");
    std::fs::create_dir_all(blocker.parent().unwrap()).unwrap();
    std::fs::write(&blocker, b"a file, not a directory").unwrap();

    let error = Fixture::generate(SHAPE, 3).write_to(&blocker).unwrap_err();
    assert!(matches!(error, Error::Io { .. }));
}
