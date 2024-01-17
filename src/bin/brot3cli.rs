/// brot3 command line interface
/// (c) 2024 Ross Younger
use brot3::{
    fractal::{PlotData, Point, Tile},
    render::WhichRenderer,
};
use clap::{ArgAction, Args, Parser, Subcommand};
use std::error::Error;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_flag = true)]
#[command(styles=get_styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug output (may be repeated)
    #[arg(short, long, action = ArgAction::Count, help = "Enables debug output (may be repeated)")]
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
    #[arg(short, long, value_name = "NAME")]
    renderer: Option<WhichRenderer>,

    // TODO: plot params
    // optional one of origin, centre <complex float> - default from fractal
    // optional one of axis-length, pixel-length, zoom-factor <float OR complex float> - default from fractal - a float f => (f,f)
    /// Maximum number of iterations
    #[arg(short, long, value_name = "N", default_value = "512")]
    max_iter: u32,

    /// Plot width
    #[arg(short, long, value_name = "PIXELS", default_value = "300")]
    width: usize,
    /// Plot height
    #[arg(short, long, value_name = "PIXELS", default_value = "300")]
    height: usize,
}

#[derive(Debug, Subcommand)]
enum ListableThings {
    /// Lists all available renderers
    Renderers,
    /// Lists available wombats
    Wombats,
}

#[derive(Debug, Args)]
#[command(flatten_help = true)]
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

fn plot(args: PlotArgs, debug: u8) -> Result<(), Box<dyn Error>> {
    // Default plot size & params for now
    // Single tile, single thread for now
    let mut t = Tile::new(args.height, args.width, debug);
    let p = PlotData {
        origin: Point { re: -3.0, im: -3.0 },
        axes: Point { re: 6.0, im: 6.0 },
    };
    t.prepare(&p);
    t.plot(args.max_iter);
    let r = brot3::render::factory(
        args.renderer.unwrap_or(brot3::render::DEFAULT),
        &args.output_filename,
    );
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
