//! Unit tests for the git command wrapper. They need `git` on `PATH`, as the
//! harness does.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use super::git;
use crate::Error;

#[test]
fn a_failing_git_command_reports_its_stderr() {
    let error = git(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &["no-such-subcommand"],
    )
    .unwrap_err();
    let Error::CommandFailed { command, stderr } = error else {
        panic!("expected CommandFailed, got {error:?}");
    };
    assert_eq!(command, "git no-such-subcommand");
    assert!(stderr.contains("no-such-subcommand"));
}

#[test]
fn git_that_cannot_start_is_a_spawn_error() {
    let error = git(Path::new("/nonexistent/openscion-bench"), &["status"]).unwrap_err();
    assert!(matches!(error, Error::Spawn { ref program, .. } if program == "git"));
    assert!(error.to_string().starts_with("cannot run git: "));
}
