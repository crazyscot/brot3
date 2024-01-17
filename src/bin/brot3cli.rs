/// brot3 command line interface
/// (c) 2024 Ross Younger
use brot3::{
    fractal::{PlotData, Point, Tile},
    render::WhichRenderer,
};
use clap::{ArgAction, Parser};
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_flag = true)]
struct Cli {
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

    /// Enable debug output (may be repeated)
    #[arg(short, long, action = clap::ArgAction::Count, help = "Enables debug output (may be repeated)")]
    debug: u8,

    /// This help message
    #[arg(long="help", action = ArgAction::Help)]
    help: Option<bool>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Default plot size & params for now
    // Single tile, single thread for now
    let mut t = Tile::new(cli.height, cli.width, cli.debug);
    let p = PlotData {
        origin: Point { re: -3.0, im: -3.0 },
        axes: Point { re: 6.0, im: 6.0 },
    };
    t.prepare(&p);
    t.plot(cli.max_iter);
    let r = brot3::render::factory(
        cli.renderer.unwrap_or(brot3::render::DEFAULT),
        &cli.output_filename,
    );
    r.render(&t).map_err(|op| {
        println!("Failed to render: {}", op);
        std::process::exit(1);
    })
}
