/// Rendering output in various ASCII-based formats
/// (c) 2024 Ross Younger
use crate::fractal::Tile;
use crate::render::Renderer;
use std::error::Error;
use std::fs::File;

#[derive(PartialEq)]
struct Filename {
    filename: String,
}

impl Filename {
    pub fn new(filename: &str) -> Self {
        Filename {
            filename: String::from(filename),
        }
    }
    pub fn write_handle(&self) -> std::io::Result<Box<dyn std::io::Write>> {
        if self.filename == "-" {
            Ok(Box::new(std::io::stdout()))
        } else {
            Ok(Box::new(File::create(&self.filename)?))
        }
    }
}

/// CSV format, fractal points
pub struct Csv {
    filename: Filename,
}

impl Csv {
    pub fn new(filename: &str) -> Self {
        Csv {
            filename: Filename::new(filename),
        }
    }
}

impl Renderer for Csv {
    fn render(&self, tile: &Tile) -> Result<(), Box<dyn Error>> {
        self.filename
            .write_handle()?
            .write_all(tile.to_string().as_bytes())?;
        Ok(())
    }
}
