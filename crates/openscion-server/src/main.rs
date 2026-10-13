//! The `openscion-server` binary.
//!
//! Parses [`ServerConfig`] from the command line and environment, binds, and
//! serves until Ctrl-C or SIGTERM.

use std::process::ExitCode;

use clap::Parser;
use openscion_server::{Server, ServerConfig, shutdown_signal};

#[tokio::main]
async fn main() -> ExitCode {
    let config = ServerConfig::parse();
    let server = match Server::bind(&config).await {
        Ok(server) => server,
        Err(error) => {
            eprintln!("openscion-server: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("openscion-server listening on http://{}", server.local_addr());
    match server.serve(shutdown_signal()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("openscion-server: {error}");
            ExitCode::FAILURE
        }
    }
}
