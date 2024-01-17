/// brot3 command line interface
/// (c) 2024 Ross Younger
use brot3::{
    fractal::{PlotData, Point, Tile},
    render::Renderer,
};
use clap::{ArgAction, Parser};
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_flag = true)]
struct Cli {
    // fractal: Option<String>, - defaulted
    // palette: Option<String>, - defaulted
    // optional one of origin, centre <complex float> - default from fractal
    // optional one of axis-length, pixel-length, zoom-factor <float OR complex float> - default from fractal - a float f => (f,f)
    /// Enable debug output (may be repeated)
    #[arg(short, long, action = clap::ArgAction::Count, help = "Enables debug output (may be repeated)")]
    debug: u8,
    #[arg(short = 'o', long = "output")]
    /// Output filename (or '-' for stdout)
    output_filename: String,
    /// Plot width
    #[arg(short, long, default_value = "300")]
    width: usize,
    /// Plot height
    #[arg(short, long, default_value = "300")]
    height: usize,
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
    t.plot(512);
    let r = brot3::render::ascii::AsciiArt::new(&cli.output_filename);
    r.render(&t).map_err(|op| {
        println!("Failed to render: {}", op);
        std::process::exit(1);
    })
}
