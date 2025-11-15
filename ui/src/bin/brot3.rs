#![windows_subsystem = "windows"]

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = brot3_ui::main() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
