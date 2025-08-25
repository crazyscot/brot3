// Plot subcommand
// (c) 2024 Ross Younger

use std::time::SystemTime;

use brot3_engine::colouring::Colourer;
use brot3_engine::fractal::{self, Algorithm, Point, Scalar, Size, Tile, TileSpec};
use brot3_engine::render::{IRenderer, Renderer};
use brot3_engine::util::Rect;

use anyhow::{Context, ensure};
use clap::ArgAction;
use rayon::prelude::*;

/// Arguments for the 'plot' subcommand
#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)]
//#[command(flatten_help = true)] // doesn't do much for this subcommand
pub struct Args {
    #[arg(
        long, hide(true), action = ArgAction::Help, required(false)
    )]
    help: Option<bool>,

    /// The fractal algorithm to use (see also the `list fractals` command)
    #[arg(
        short = 'f',
        long,
        value_name = "NAME",
        default_value = "original",
        hide_possible_values = true,
        help_heading("Plot control"),
        display_order(5)
    )]
    pub(crate) fractal: fractal::Selection,

    /// Maximum number of iterations before assuming a pixel has escaped
    #[arg(
        short,
        long,
        value_name = "N",
        default_value = "512",
        help_heading("Plot control"),
        display_order(6)
    )]
    pub(crate) max_iter: u32,

    /// The colouring algorithm to use. (see also the `list colourers` command)
    #[arg(
        short = 'c',
        long,
        alias = "colour",
        alias = "color",
        alias = "colorer",
        value_name = "NAME",
        default_value = "linear-rainbow",
        help_heading("Plot control"),
        hide_possible_values = true,
        display_order(7)
    )]
    pub(crate) colourer: Colourer,

    /// The origin (bottom-left) point of the plot, e.g. -3-3i.
    #[arg(
        short = 'O',
        long,
        value_name = "COMPLEX",
        group = "location",
        allow_hyphen_values(true),
        help_heading("Plot location (specify once)"),
        display_order(10)
    )]
    pub(crate) origin: Option<Point>,

    /// The centre point of the plot, e.g. -1-1i.
    #[arg(
        short = 'C',
        long,
        value_name = "COMPLEX",
        group = "location",
        allow_hyphen_values(true),
        help_heading("Plot location (specify once)"),
        display_order(10)
    )]
    pub(crate) centre: Option<Point>,

    /// The length of the axes, e.g. 3+3i. If the imaginary dimension is not specified it will be assumed to be the same as the real.
    #[arg(
        short = 'A',
        long,
        value_name = "COMPLEX",
        group = "size",
        help_heading("Plot size (specify once)"),
        display_order(20)
    )]
    pub(crate) axes_length: Option<Point>,
    /// The size of a pixel, e.g. 0.003+0.003i. If the imaginary dimension is not specified it will be assumed to be the same as the real.
    #[arg(
        short = 'P',
        long,
        value_name = "COMPLEX",
        group = "size",
        help_heading("Plot size (specify once)"),
        display_order(20)
    )]
    pub(crate) pixel_size: Option<Point>,
    /// The zoom factor, relative to the plot default.
    #[arg(
        short = 'Z',
        long,
        value_name = "INT",
        group = "size",
        help_heading("Plot size (specify once)"),
        display_order(20)
    )]
    pub(crate) zoom: Option<Scalar>,

    /// Suppresses auto-aspect-adjustment. (By default we automatically grow the axes to make the pixels square, which is usually what you wanted.)
    #[arg(long, display_order(21), help_heading("Plot size (specify once)"))]
    pub(crate) no_auto_aspect: bool,

    /// Plot width
    #[arg(
        short,
        long,
        value_name = "PIXELS",
        default_value = "300",
        help_heading("Output"),
        display_order(100)
    )]
    pub(crate) width: u32,
    /// Plot height
    #[arg(
        short,
        long,
        value_name = "PIXELS",
        default_value = "300",
        help_heading("Output"),
        display_order(100)
    )]
    pub(crate) height: u32,

    /// Where to send the output (required; for stdout, use '-' and specify the --output-type)
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILENAME",
        help_heading("Output"),
        display_order(110)
    )]
    pub(crate) output_filename: String,

    /// Explicitly specifies the output file type (default: autodetect from filename). See also the `list output-types` command.
    #[arg(
        short = 't',
        long,
        value_name = "NAME",
        hide_possible_values = true,
        help_heading("Output"),
        display_order(120)
    )]
    pub(crate) output_type: Option<Renderer>,

    /// Prints the plot info string to stdout
    #[arg(long, display_order(130), help_heading("Output"))]
    pub(crate) info: bool,

    /// Prevents the internal processing of the plot as a series of strips.
    /// This disables parallelisation and may lead to slightly different numerical output as the plot co-ordinates shift subtly.
    #[arg(long, display_order(900), help_heading("Developer options"))]
    pub(crate) no_split: bool,

    /// Measures and outputs the time to complete various parts of the process.
    #[arg(long, display_order(900), help_heading("Developer options"))]
    pub(crate) show_timing: bool,

    /// The number of rows per render strip
    #[arg(
        long,
        display_order(900),
        help_heading("Developer options"),
        default_value = "10",
        value_name = "PIXELS"
    )]
    pub(crate) strip_size: u32,
}

fn check_fix_axes(input: Point) -> anyhow::Result<Point> {
    let mut out = input;
    if out.im == 0.0 {
        out.im = out.re;
    }
    ensure!(out.re.is_finite(), "Real axis must be finite");
    ensure!(out.re != 0.0, "Real axis cannot be zero");
    ensure!(out.im.is_finite(), "Imaginary axis must be finite");
    Ok(out)
}

fn check_zoom(input: Scalar) -> anyhow::Result<Scalar> {
    ensure!(input > 0.0, "Zoom must be positive");
    Ok(input)
}

/// Implementation of 'plot'
pub(crate) fn plot(args: &Args, debug: u8) -> anyhow::Result<()> {
    let algorithm = fractal::factory(args.fractal);

    let mut spec = TileSpec::new(
        args_location(args, algorithm),
        args_axes(args, algorithm)?,
        Rect::new(args.width, args.height),
        algorithm,
        args.max_iter,
        args.colourer,
    );
    if !args.no_auto_aspect
        && let Ok(Some(new_axes)) = spec.auto_adjust_aspect_ratio()
    {
        println!(
            "Auto adjusted aspect ratio. Axes are now {new_axes} (you can suppress this behaviour with `--no-auto-aspect')"
        );
    }
    if debug > 0 {
        println!("Computed plot spec: {spec:#?}");
    }

    // If they didn't specify an output file type, attempt to autodetect
    let renderer: Renderer = if let Some(s) = args.output_type {
        s
    } else {
        Renderer::try_from_file_extension(&args.output_filename)
            .context("Could not autodetect desired output type from filename (try `--type ...')")?
    };

    if args.info {
        println!("{spec}");
    }

    let time0 = SystemTime::now();
    let splits: Vec<TileSpec> = if args.no_split {
        vec![spec.clone()]
    } else {
        spec.split(args.strip_size, debug)?
    };
    let mut tiles: Vec<Tile> = Vec::new();
    splits
        .par_iter()
        .map(|ts| Tile::new(ts, debug))
        .collect_into_vec(&mut tiles);
    let time1 = SystemTime::now();
    tiles
        .par_iter_mut()
        .for_each(brot3_engine::fractal::Tile::plot);
    let time2 = SystemTime::now();

    let result = renderer.render_file(&args.output_filename, &spec, &tiles, args.colourer);
    let time3 = SystemTime::now();
    if args.show_timing {
        println!(
            "times: prepare {:?}, plot {:?} render {:?}",
            time1.duration_since(time0).unwrap_or_default(),
            time2.duration_since(time1).unwrap_or_default(),
            time3.duration_since(time2).unwrap_or_default(),
        );
    }
    result
}

/// Unpick the possible user specifications for the plot location
fn args_location(args: &Args, algorithm: fractal::Instance) -> fractal::Location {
    if let Some(o) = args.origin {
        fractal::Location::Origin(o)
    } else {
        fractal::Location::Centre(args.centre.unwrap_or(algorithm.default_centre()))
    }
}

/// Unpick the possible user specifications for the plot axes
fn args_axes(args: &Args, algorithm: fractal::Instance) -> anyhow::Result<fractal::Size> {
    if let Some(size) = args.pixel_size {
        Ok(Size::PixelSize(check_fix_axes(size)?))
    } else if let Some(zoom) = args.zoom {
        Ok(Size::ZoomFactor(check_zoom(zoom)?))
    } else {
        Ok(Size::AxesLength(check_fix_axes(
            args.axes_length.unwrap_or(algorithm.default_axes()),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::check_fix_axes;
    use brot3_engine::fractal::{Point, Scalar};

    #[test]
    fn axes_fixup_nonzero() {
        let result = check_fix_axes(Point { re: 1.0, im: 0.0 }).unwrap();
        assert_eq!(result, Point { re: 1.0, im: 1.0 });
    }
    #[test]
    fn axes_zero_error() {
        assert!(check_fix_axes(Point { re: 0.0, im: 0.0 }).is_err());
    }
    #[test]
    fn axes_inf_error() {
        assert!(
            check_fix_axes(Point {
                re: Scalar::INFINITY,
                im: 2.0
            })
            .is_err()
        );
        assert!(
            check_fix_axes(Point {
                re: 2.0,
                im: Scalar::INFINITY
            })
            .is_err()
        );
    }
    #[test]
    fn axes_nan_error() {
        assert!(
            check_fix_axes(Point {
                re: Scalar::NAN,
                im: 2.0
            })
            .is_err()
        );
        assert!(
            check_fix_axes(Point {
                re: 2.0,
                im: Scalar::NAN
            })
            .is_err()
        );
    }
}
