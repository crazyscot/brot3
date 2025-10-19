//! Command line argument definitions
// (c) 2025 Ross Younger

#[cfg(all(
    any(feature = "hot-reload-shader", feature = "runtime-compilation"),
    not(target_arch = "wasm32")
))]
use std::path::PathBuf;

#[derive(Debug, clap::Parser, Clone, Default)]
pub(crate) struct Args {
    #[cfg(all(
        any(feature = "hot-reload-shader", feature = "runtime-compilation"),
        not(target_arch = "wasm32")
    ))]
    #[arg(long)]
    /// Specifies the path to the shader directory.
    ///
    /// This is only allowed when run standalone (not via `cargo run`).
    pub shader: Option<String>,

    #[cfg(all(
        any(feature = "hot-reload-shader", feature = "runtime-compilation"),
        not(target_arch = "wasm32")
    ))]
    #[arg(long)]
    /// Specifies the path to the the SPIRV tools library, if needed
    /// (librustc_codegen_spirv.so, librustc_codegen_spirv.dylib, rustc_codegen_spirv.dll)
    ///
    /// This is only required when the tools library is not on your shared library/DLL search path.
    /// It works best with absolute paths.
    pub spirv_tools: Option<PathBuf>,
}
