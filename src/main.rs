//! Payments engine CLI

use std::process::ExitCode;

use payments_engine::EngineError;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the payments engine with the given path
fn run() -> Result<(), EngineError> {
    let path = std::env::args().nth(1).ok_or(EngineError::MissingPath)?;
    payments_engine::process_csv(path, std::io::stdout().lock())
}
