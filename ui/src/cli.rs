//! Command line argument definitions
// (c) 2025 Ross Younger

#[cfg(we_compile)]
use std::path::PathBuf;
use std::str::FromStr;

use shader::enums::{Algorithm, Colourer};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Parser, Clone, Default)]
pub(crate) struct Args {
    #[arg(short = 'V', long, help = "Print version")]
    pub version: bool,

    #[cfg(we_compile)]
    #[arg(long)]
    /// Specifies the path to the shader directory.
    ///
    /// This is only allowed when run standalone (not via `cargo run`).
    pub shader: Option<PathBuf>,

    #[cfg(we_compile)]
    #[arg(long)]
    /// Specifies the path to the the SPIRV tools library, if needed
    /// (`librustc_codegen_spirv.so`, `librustc_codegen_spirv.dylib`, `rustc_codegen_spirv.dll`)
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
        default_value = "neon"
    )]
    pub colourer: Colourer,

    /// Overrides the point cache size autodetection, in the format "x,y" (e.g. "1920,1080")
    ///
    /// The cache is normally set up to suit the largest available monitor detected.
    /// If the detection fails or you have an unusual setup, it may be necessary to override.
    ///
    /// The maximum available cache size is determined at runtime by the GPU driver.
    #[arg(long, value_name = "WIDTH,HEIGHT")]
    pub cache_size: Option<LocalUVec2>,
}

// A simple tuple struct to represent a 2D u32 vector.
// You can also use a custom struct if you prefer named fields.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct LocalUVec2(pub u32, pub u32);

// Implement FromStr for UVec2 to use the standard parse() method.
// This allows clap's built-in value_parser! macro to work seamlessly.
impl FromStr for LocalUVec2 {
    // Use a String for a simple error type
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("'{s}' is not in the format 'u32,u32'"));
        }

        let x = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid x value: {e}"))?;
        let y = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid y value: {e}"))?;

        Ok(LocalUVec2(x, y))
    }
}

impl From<LocalUVec2> for glam::UVec2 {
    fn from(value: LocalUVec2) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
