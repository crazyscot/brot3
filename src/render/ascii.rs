/// Rendering output in various ASCII-based formats
/// (c) 2024 Ross Younger
use crate::fractal::Tile;
use crate::render::Renderer;
use std::error::Error;
use std::fs;

/// CSV format, fractal points
pub struct Csv {
    filename: String,
}

impl Csv {
    pub fn new(filename: &str) -> Self {
        Csv {
            filename: String::from(filename),
        }
    }
}

impl Renderer for Csv {
    fn render(&self, tile: &Tile) -> Result<(), Box<dyn Error>> {
        if self.filename == "-" {
            // Tile fmt finishes with a newline
            print!("{}", tile);
            Ok(())
        } else {
            Ok(fs::write(&self.filename, tile.to_string())?)
        }
    }
}
