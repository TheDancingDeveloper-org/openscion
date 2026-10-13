//! The `openscion-bench` binary: generate a fixture, run every scenario, and
//! print a Markdown table of median times.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use openscion_benchmarks::{
    Error, Fixture, FixtureShape, Measurement, Report, Result, Scenario, git, openscion,
};

/// Benchmark OpenScion against git on a synthetic repository.
#[derive(Debug, Parser)]
#[command(name = "openscion-bench", version)]
struct Args {
    /// Number of top-level directories in the fixture.
    #[arg(long, default_value_t = 10)]
    directories: usize,
    /// Number of files in each directory.
    #[arg(long, default_value_t = 100)]
    files_per_directory: usize,
    /// Size of every file, in bytes.
    #[arg(long, default_value_t = 4096)]
    file_bytes: usize,
    /// Fixture seed; the same seed always produces the same files.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Runs per scenario; the report shows the median.
    #[arg(long, default_value_t = 5)]
    runs: usize,
    /// Scratch directory, emptied before every run.
    #[arg(long, default_value = "target/openscion-bench")]
    workdir: PathBuf,
}

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(table) => {
            print!("{table}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("openscion-bench: {error}");
            ExitCode::FAILURE
        }
    }
}

/// One backend's entry point: run a scenario once and time it.
type Backend = fn(Scenario, &Fixture, &std::path::Path) -> Result<std::time::Duration>;

fn run(args: &Args) -> Result<String> {
    if args.runs == 0 {
        return Err(Error::ZeroRuns);
    }
    let shape = FixtureShape {
        directories: args.directories,
        files_per_directory: args.files_per_directory,
        file_bytes: args.file_bytes,
    };
    let fixture = Fixture::generate(shape, args.seed);
    let mut report = Report {
        fixture: format!(
            "{} files of {} bytes in {} directories (seed {})",
            fixture.files().len(),
            args.file_bytes,
            args.directories,
            args.seed
        ),
        measurements: Vec::new(),
    };
    let backends: [(&'static str, Backend); 2] = [("git", git::run), ("openscion", openscion::run)];
    for scenario in Scenario::ALL {
        for (backend, run) in backends {
            let workdir = args.workdir.join(format!("{backend}-{scenario}"));
            let runs = (0..args.runs)
                .map(|_| run(scenario, &fixture, &workdir))
                .collect::<Result<Vec<_>>>()?;
            report.measurements.push(Measurement {
                backend,
                scenario,
                runs,
            });
        }
    }
    report.to_markdown()
}
