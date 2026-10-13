//! Unit tests for fixture generation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{Fixture, FixtureShape};

const SHAPE: FixtureShape = FixtureShape {
    directories: 3,
    files_per_directory: 4,
    file_bytes: 100,
};

#[test]
fn the_same_seed_generates_the_same_fixture() {
    assert_eq!(Fixture::generate(SHAPE, 1), Fixture::generate(SHAPE, 1));
}

#[test]
fn different_seeds_generate_different_contents() {
    assert_ne!(Fixture::generate(SHAPE, 1), Fixture::generate(SHAPE, 2));
}

#[test]
fn the_shape_sets_the_file_count_paths_and_sizes() {
    let fixture = Fixture::generate(SHAPE, 1);
    assert_eq!(fixture.files().len(), 12);
    assert_eq!(fixture.files()[0].path, "dir0000/file0000.txt");
    assert_eq!(fixture.files()[11].path, "dir0002/file0003.txt");
    assert!(fixture.files().iter().all(|file| file.contents.len() == 100));
    assert!(
        fixture
            .files()
            .iter()
            .flat_map(|file| &file.contents)
            .all(|byte| (b' '..=b'~').contains(byte))
    );
}
