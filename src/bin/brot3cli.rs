use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // fractal: Option<String>, - defaulted
    // palette: Option<String>, - defaulted
    // optional one of origin, centre <complex float> - default from fractal
    // optional one of axis-length, pixel-length, zoom-factor <float OR complex float> - default from fractal - a float f => (f,f)
    #[arg(short, long, action = clap::ArgAction::Count, help = "Enables debug output (may be repeated)")]
    debug: u8,
}

fn main() {
    let cli = Cli::parse();

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    println!("Hello, world!");
}
