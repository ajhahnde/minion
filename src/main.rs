use std::process::ExitCode;

fn main() -> ExitCode {
    match minion::cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(minion::errors::MinionError::BrokenPipe) => ExitCode::from(1),
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::from(1)
        }
    }
}
