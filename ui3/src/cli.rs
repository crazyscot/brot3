//! Command line argument definitions
// (c) 2025 Ross Younger

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
}
