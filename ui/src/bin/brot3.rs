#![windows_subsystem = "windows"]

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = brot3_ui::main() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
