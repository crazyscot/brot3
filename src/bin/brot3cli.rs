// brot3 command line interface
// (c) 2024 Ross Younger
use brot3::{
    fractal::{
        userplotdata::{UserPlotLocation, UserPlotSize},
        PlotData, Point, Scalar, Tile, UserPlotData,
    },
    render::WhichRenderer,
};
use clap::{ArgAction, Args, Parser, Subcommand};
use std::error::Error;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_flag = true)]
#[command(styles=get_styles())]
#[command(infer_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug output (may be repeated)
    #[arg(short, long, global(true), action = ArgAction::Count, help = "Enables debug output (may be repeated)")]
    debug: u8,

    #[arg(long, hide(true))]
    debug_cli: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Plots fractals
    Plot(PlotArgs),
    /// Lists things known to this interface
    List(ListArgs),
}

#[derive(Debug, Args)]
struct PlotArgs {
    /// Where to send the output (required; use '-' for stdout)
    #[arg(short = 'o', long = "output", value_name = "FILENAME")]
    output_filename: String,

    // TODO: fractal: Option<String>, - defaulted
    /// Rendering type
    #[arg(short, long, value_name = "NAME", default_value = "png")]
    renderer: WhichRenderer,

    /// The origin (top-left) point of the plot, e.g. -3-3i. Conflicts with --centre.
    #[arg(
        short = 'O',
        long,
        value_name = "COMPLEX",
        group = "location",
        allow_hyphen_values(true)
    )]
    origin: Option<Point>,

    /// The centre point of the plot, e.g. -1-1i. Conflicts with --origin.
    #[arg(
        short = 'C',
        long,
        value_name = "COMPLEX",
        group = "location",
        allow_hyphen_values(true)
    )]
    centre: Option<Point>,

    /// The length of the axes, e.g. 3+3i. If the imaginary dimension is not specified it will be assumed to be the same as the real. Conflicts with pixel_size and zoom.
    #[arg(short = 'A', long, value_name = "COMPLEX", group = "size")]
    axes_length: Option<Point>,
    /// The size of a pixel, e.g. 0.003+0.003i. If the imaginary dimension is not specified it will be assumed to be the same as the real. Conflicts with axes_length and zoom.
    #[arg(short = 'P', long, value_name = "COMPLEX", group = "size")]
    pixel_size: Option<Point>,
    /// The zoom factor, relative to the plot default. Conflicts with axes_length and pixel_size.
    #[arg(short = 'Z', long, value_name = "INT", group = "size")]
    zoom: Option<Scalar>,

    /// Maximum number of iterations
    #[arg(short, long, value_name = "N", default_value = "512")]
    max_iter: u32,

    /// Plot width
    #[arg(short, long, value_name = "PIXELS", default_value = "300")]
    width: u32,
    /// Plot height
    #[arg(short, long, value_name = "PIXELS", default_value = "300")]
    height: u32,
}

#[derive(Debug, Subcommand)]
enum ListableThings {
    /// Lists all available renderers
    Renderers,
    /// Lists available wombats
    Wombats,
}

#[derive(Debug, Args)]
//#[command(flatten_help = true)]
struct ListArgs {
    /// Machine-parseable output
    #[arg(short, long)]
    machine_parseable: bool,

    #[command(subcommand)]
    thing: ListableThings,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    if cli.debug_cli {
        println!("{:#?}", cli);
        return Ok(());
    }

    match cli.command {
        Commands::Plot(args) => plot(args, cli.debug),
        Commands::List(what) => list(what),
    }
}

const DEFAULT_CENTRE: Point = Point { re: -1.0, im: 0.0 };
const DEFAULT_ZOOM: f64 = 1.0;

fn check_fix_axes(input: Point) -> Result<Point, Box<dyn Error>> {
    let mut out = input;
    if out.im == 0.0 {
        out.im = out.re;
    }
    if out.re == 0.0 {
        return Err("Real axis cannot be zero".into());
    }
    Ok(out)
}

fn plot(args: PlotArgs, debug: u8) -> Result<(), Box<dyn Error>> {
    // Single tile, single thread for now
    let mut t = Tile::new(args.height as usize, args.width as usize, debug);
    let user_plot_data = UserPlotData {
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
        println!("Entered plot data: {:#?}", user_plot_data);
    }

    let p = PlotData::from(&user_plot_data);
    if debug > 0 {
        println!("Computed plot data: {:#?}", p);
    }

    t.prepare(&p);
    t.plot(args.max_iter);
    let r = brot3::render::factory(args.renderer, &args.output_filename);
    r.render(&t).map_err(|op| {
        println!("Failed to render: {}", op);
        std::process::exit(1);
    })
}

fn list(args: ListArgs) -> Result<(), Box<dyn Error>> {
    match args.thing {
        ListableThings::Renderers => brot3::render::list(args.machine_parseable),
        ListableThings::Wombats => {
            if args.machine_parseable {
                println!("[\"fred\",\"barney\"]");
            } else {
                println!("wombats!")
            };
        }
    }
    Ok(())
}

pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}
