use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = kyle::cli::run() {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
