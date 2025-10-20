//! Command line argument definitions
// (c) 2025 Ross Younger

#[cfg(we_compile)]
use std::path::PathBuf;

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
}
