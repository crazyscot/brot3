// Plot subcommand
// (c) 2024 Ross Younger

use crate::fractal::{PlotSpec, Point, Scalar, Tile, UserPlotLocation, UserPlotSize, UserPlotSpec};
use crate::render::{self, WhichRenderer};

use anyhow::ensure;

const DEFAULT_CENTRE: Point = Point { re: -1.0, im: 0.0 };
const DEFAULT_ZOOM: f64 = 1.0;

/// Arguments for the 'plot' subcommand
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Where to send the output (required; use '-' for stdout)
    #[arg(short = 'o', long = "output", value_name = "FILENAME")]
    pub output_filename: String,

    // TODO: fractal: Option<String>, - defaulted
    /// Rendering type
    #[arg(short, long, value_name = "NAME", default_value = "png")]
    pub renderer: WhichRenderer,

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

/// Implementation of 'plot'
pub fn plot(args: &Args, debug: u8) -> anyhow::Result<()> {
    // Single tile, single thread for now
    let user_plot_data = UserPlotSpec {
        location: {
            if let Some(o) = args.origin {
                UserPlotLocation::Origin(o)
            } else {
                UserPlotLocation::Centre(args.centre.unwrap_or(DEFAULT_CENTRE))
            }
        },
        axes: {
            if let Some(axes) = args.axes_length {
                UserPlotSize::AxesLength(check_fix_axes(axes)?)
            } else if let Some(size) = args.pixel_size {
                UserPlotSize::PixelSize(check_fix_axes(size)?)
            } else {
                UserPlotSize::ZoomFactor(args.zoom.unwrap_or(DEFAULT_ZOOM))
            }
        },
        height: args.height,
        width: args.width,
    };
    if debug > 0 {
        println!("Entered plot data: {user_plot_data:#?}");
    }

    let pd = PlotSpec::from(&user_plot_data);
    if debug > 0 {
        println!("Computed plot data: {pd:#?}");
    }

    let mut t = Tile::new(&pd, debug);
    t.prepare();
    t.plot(args.max_iter);
    let r = render::factory(args.renderer, &args.output_filename);
    r.render(&t)
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
