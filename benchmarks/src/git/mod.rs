//! The git backend: each scenario performed with the `git` command-line tool.
//!
//! Git runs with an isolated configuration — no system or global config, no
//! hooks, no signing — so a developer's setup cannot change the numbers.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::{Error, Fixture, Result, Scenario};

/// Runs `scenario` once against `fixture` inside `workdir` and returns how
/// long the measured part took. `workdir` is emptied first.
///
/// # Errors
///
/// Returns [`Error::Io`] when the work directory cannot be prepared,
/// [`Error::Spawn`] when `git` cannot be started, and
/// [`Error::CommandFailed`] when a git command exits unsuccessfully.
pub fn run(scenario: Scenario, fixture: &Fixture, workdir: &Path) -> Result<Duration> {
    reset(workdir)?;
    let repo = workdir.join("repo");
    fixture.write_to(&repo)?;
    match scenario {
        Scenario::Import => timed(|| import(&repo)),
        Scenario::CommitOneFile => {
            import(&repo)?;
            let changed = repo.join(&fixture.files()[0].path);
            std::fs::write(&changed, b"changed\n").map_err(|source| Error::Io {
                path: changed,
                source,
            })?;
            timed(|| git(&repo, &["commit", "-q", "-a", "-m", "change one file"]))
        }
        Scenario::FullClone => {
            import(&repo)?;
            // Both paths are relative to `workdir`, where git runs, so a
            // relative `workdir` resolves the same way for every command.
            timed(|| git(workdir, &["clone", "-q", "--no-local", "repo", "clone"]))
        }
    }
}

fn import(repo: &Path) -> Result<()> {
    git(repo, &["init", "-q"])?;
    git(repo, &["add", "-A"])?;
    git(repo, &["commit", "-q", "-m", "import"])
}

fn timed(operation: impl FnOnce() -> Result<()>) -> Result<Duration> {
    let start = Instant::now();
    operation()?;
    Ok(start.elapsed())
}

fn reset(workdir: &Path) -> Result<()> {
    if workdir.exists() {
        std::fs::remove_dir_all(workdir).map_err(|source| Error::Io {
            path: workdir.to_path_buf(),
            source,
        })?;
    }
    std::fs::create_dir_all(workdir).map_err(|source| Error::Io {
        path: workdir.to_path_buf(),
        source,
    })
}

fn git(dir: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", null_device())
        .args([
            "-c",
            "user.name=openscion-bench",
            "-c",
            "user.email=bench@openscion.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
            "-c",
            "core.hooksPath=",
        ])
        .args(args)
        .output()
        .map_err(|source| Error::Spawn {
            program: "git".to_owned(),
            source,
        })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed {
            command: format!("git {}", args.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }
}

fn null_device() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from("NUL")
    } else {
        PathBuf::from("/dev/null")
    }
}

#[cfg(test)]
mod test;
