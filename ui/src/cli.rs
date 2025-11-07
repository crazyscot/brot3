//! Command line argument definitions
// (c) 2025 Ross Younger

#[cfg(we_compile)]
use std::path::PathBuf;

use shader_common::enums::{Algorithm, Colourer};

#[derive(Debug, clap::Parser, Clone, Default)]
pub(crate) struct Args {
    #[cfg(we_compile)]
    #[arg(long)]
    /// Specifies the path to the shader directory.
    ///
    /// This is only allowed when run standalone (not via `cargo run`).
    pub shader: Option<PathBuf>,

    #[cfg(we_compile)]
    #[arg(long)]
    /// Specifies the path to the the SPIRV tools library, if needed
    /// (librustc_codegen_spirv.so, librustc_codegen_spirv.dylib, rustc_codegen_spirv.dll)
    ///
    /// This is only required when the tools library is not on your shared library/DLL search path.
    /// It works best with absolute paths.
    pub spirv_tools: Option<PathBuf>,

    #[cfg(we_compile)]
    #[arg(long)]
    /// Disables runtime shader compilation and uses the built-in shader.
    pub static_shader: bool,

    /// Causes the UI window to start up in fullscreen
    #[arg(long)]
    pub fullscreen: bool,

    /// Starts up with the UI hidden (press F2 to show it)
    #[arg(long)]
    pub no_ui: bool,

    /// Selects the initial fractal algorithm to use
    #[arg(
        short = 'F',
        long,
        alias = "fractal",
        value_name = "NAME",
        default_value = "mandelbrot"
    )]
    pub fractal: Algorithm,

    /// Selects the initial colouring algorithm to use
    #[arg(
        short = 'C',
        long,
        alias = "colorer",
        value_name = "NAME",
        default_value = "log-rainbow"
    )]
    pub colourer: Colourer,
}
