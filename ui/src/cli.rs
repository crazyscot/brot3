//! Command line argument definitions
// (c) 2025 Ross Younger

#[cfg(runtime_compile)]
use std::path::PathBuf;
use std::{path::PathBuf, str::FromStr};

use brot3_lib::{
    data::{Algorithm, ColourStyle, Colourer, Modifier, Palette},
    ui::UiState,
    util::Size,
};
use clap::builder::{
    Styles,
    styling::{AnsiColor, Color, Style},
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Parser, Clone, Default)]
#[command(styles=CLAP_STYLES)]
/// Yet Another Fractal Plotter
pub(crate) struct Args {
    #[arg(short = 'V', long, help = "Print version")]
    pub version: bool,

    #[cfg(runtime_compile)]
    #[arg(long)]
    /// Specifies the path to the shader directory.
    ///
    /// This is only allowed when run standalone (not via `cargo run`).
    pub shader: Option<PathBuf>,

    #[cfg(runtime_compile)]
    #[arg(long)]
    /// Specifies the path to the the SPIRV tools library, if needed
    /// (`librustc_codegen_spirv.so`, `librustc_codegen_spirv.dylib`, `rustc_codegen_spirv.dll`)
    ///
    /// This is only required when the tools library is not on your shared library/DLL search path.
    /// It works best with absolute paths.
    pub spirv_tools: Option<PathBuf>,

    #[cfg(runtime_compile)]
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

    /// Selects the initial colour style to use
    #[arg(
        long,
        alias = "color-style",
        value_name = "NAME",
        default_value = "continuous"
    )]
    pub colour_style: ColourStyle,

    /// Selects the initial brightness rendering style to use
    #[arg(
        long,
        alias = "color-brightness-style",
        value_name = "NAME",
        default_value = "standard"
    )]
    pub brightness_style: Modifier,

    /// Overrides the point cache size autodetection, in the format "x,y" (e.g. "1920,1080")
    ///
    /// The cache is normally set up to suit the largest available monitor detected.
    /// If the detection fails or you have an unusual setup, it may be necessary to override.
    ///
    /// The maximum available cache size is determined at runtime by the GPU driver.
    #[arg(long, value_name = "WIDTH,HEIGHT")]
    pub cache_size: Option<LocalUVec2>,

    /// Uses parameters in the given file on startup, instead of the default.
    ///
    /// Using this option causes --fractal and styling options to be ignored.
    #[arg(short = 'i', long, value_name = "FILENAME")]
    pub input: Option<PathBuf>,

    /// Renders the fractal to an image file instead of showing the UI.
    /// This option specifies the output file.
    ///
    /// This currently requires --input to be set, and will render the same image that would be
    /// shown in the UI.
    #[arg(short = 'o', long, value_name = "FILENAME", requires = "input")]
    pub output: Option<PathBuf>,

    /// The size of the output image when using --output. This is in the format "WIDTH,HEIGHT"
    /// (e.g. "1920,1080").
    #[arg(long, value_name = "WIDTH,HEIGHT", requires = "output")]
    pub size: Option<Size>,

    /// Disables parallel rendering when using --output. This may be useful for benchmarking or
    /// other analytical runs.
    #[arg(long, default_value_t = false, requires = "output")]
    pub no_parallel_render: bool,
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

impl From<&Args> for UiState {
    fn from(args: &Args) -> Self {
        UiState {
            algorithm: args.fractal,
            palette: Palette::default()
                .with_colourer(args.colourer)
                .with_style(args.colour_style)
                .with_brightness(args.brightness_style),
            ..UiState::default()
        }
    }
}

// CLI styling for clap.
// We don't need to make this conditional, as clap already reads the CLICOLOR environment variables.
pub(crate) const CLAP_STYLES: Styles = Styles::styled()
    .usage(
        Style::new()
            .bold()
            .underline()
            .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
    )
    .header(
        Style::new()
            .bold()
            .underline()
            .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
    )
    .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .invalid(
        Style::new()
            .bold()
            .fg_color(Some(Color::Ansi(AnsiColor::Red))),
    )
    .error(
        Style::new()
            .bold()
            .fg_color(Some(Color::Ansi(AnsiColor::Red))),
    )
    .valid(
        Style::new()
            .bold()
            .underline()
            .fg_color(Some(Color::Ansi(AnsiColor::Green))),
    )
    .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))));
