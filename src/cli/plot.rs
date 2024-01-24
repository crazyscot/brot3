// Plot subcommand
// (c) 2024 Ross Younger

use crate::fractal::{
    self, Algorithm, Location, PlotSpec, Point, Scalar, Size, SplitMethod, Tile, TileSpec,
};
use crate::render::{self, Renderer};
use crate::util::Rect;

use anyhow::ensure;

/// Arguments for the 'plot' subcommand
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Where to send the output (required; use '-' for stdout)
    #[arg(short = 'o', long = "output", value_name = "FILENAME")]
    pub output_filename: String,

    /// The fractal algorithm to use
    #[arg(short = 'f', long, value_name = "NAME", default_value = "original")]
    pub fractal: fractal::Selection,

    /// Rendering type
    #[arg(short, long, value_name = "NAME", default_value = "png")]
    pub renderer: render::Selection,

    /// The origin (bottom-left) point of the plot, e.g. -3-3i. Conflicts with --centre.
    #[arg(
        short = 'O',
        long,
        value_name = "COMPLEX",
        group = "location",
        allow_hyphen_values(true)
    )]
    pub origin: Option<Point>,

    /// The centre point of the plot, e.g. -1-1i. Conflicts with --origin.
    #[arg(
        short = 'C',
        long,
        value_name = "COMPLEX",
        group = "location",
        allow_hyphen_values(true)
    )]
    pub centre: Option<Point>,

    /// The length of the axes, e.g. 3+3i. If the imaginary dimension is not specified it will be assumed to be the same as the real. Conflicts with pixel_size and zoom.
    #[arg(short = 'A', long, value_name = "COMPLEX", group = "size")]
    pub axes_length: Option<Point>,
    /// The size of a pixel, e.g. 0.003+0.003i. If the imaginary dimension is not specified it will be assumed to be the same as the real. Conflicts with axes_length and zoom.
    #[arg(short = 'P', long, value_name = "COMPLEX", group = "size")]
    pub pixel_size: Option<Point>,
    /// The zoom factor, relative to the plot default. Conflicts with axes_length and pixel_size.
    #[arg(short = 'Z', long, value_name = "INT", group = "size")]
    pub zoom: Option<Scalar>,

    /// Maximum number of iterations
    #[arg(short, long, value_name = "N", default_value = "512")]
    pub max_iter: u32,

    /// Plot width
    #[arg(short, long, value_name = "PIXELS", default_value = "300")]
    pub width: u32,
    /// Plot height
    #[arg(short, long, value_name = "PIXELS", default_value = "300")]
    pub height: u32,

    /// For debugging. Prevents the internal processing of the plot as a series of strips.
    /// This disables parallelisation and may lead to slightly different numerical output as the plot co-ordinates shift subtly.
    #[arg(long)]
    pub no_split: bool,
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
pub fn plot(args: &Args, debug: u8) -> anyhow::Result<()> {
    let algorithm = fractal::factory(args.fractal);

    let user_plot_data = PlotSpec {
        location: {
            if let Some(o) = args.origin {
                Location::Origin(o)
            } else {
                Location::Centre(args.centre.unwrap_or(algorithm.default_centre()))
            }
        },
        axes: {
            if let Some(size) = args.pixel_size {
                Size::PixelSize(check_fix_axes(size)?)
            } else if let Some(zoom) = args.zoom {
                Size::ZoomFactor(check_zoom(zoom)?)
            } else {
                Size::AxesLength(check_fix_axes(
                    args.axes_length.unwrap_or(algorithm.default_axes()),
                )?)
            }
        },
        size_in_pixels: Rect::new(args.width, args.height),
        algorithm,
    };
    if debug > 0 {
        println!("Entered plot data: {user_plot_data:#?}");
    }

    let spec = TileSpec::from(&user_plot_data);
    if debug > 0 {
        println!("Computed plot data: {spec:#?}");
    }
    if args.no_split {
        let mut t = Tile::new(&spec, debug);
        t.plot(args.max_iter);
        render::factory(args.renderer, &args.output_filename).render(&t)
    } else {
        let splits = spec.split(SplitMethod::RowsOfHeight(10), debug)?;
        let mut tiles: Vec<Tile> = splits.iter().map(|ts| Tile::new(ts, debug)).collect();
        for t in &mut tiles {
            t.plot(args.max_iter);
        }
        let result = Tile::join(&spec, &tiles)?;
        render::factory(args.renderer, &args.output_filename).render(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::check_fix_axes;
    use crate::fractal::Point;

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
        assert!(check_fix_axes(Point {
            re: f64::INFINITY,
            im: 2.0
        })
        .is_err());
        assert!(check_fix_axes(Point {
            re: 2.0,
            im: f64::INFINITY
        })
        .is_err());
    }
    #[test]
    fn axes_nan_error() {
        assert!(check_fix_axes(Point {
            re: f64::NAN,
            im: 2.0
        })
        .is_err());
        assert!(check_fix_axes(Point {
            re: 2.0,
            im: f64::NAN
        })
        .is_err());
    }
}
