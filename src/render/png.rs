// Rendering output to PNG files
// (c) 2024 Ross Younger

use super::Renderer;
use crate::fractal::Tile;
use crate::util::filename::Filename;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Png {
    filename: Filename,
}

impl Png {
    pub fn new(filename: &str) -> Self {
        Png {
            filename: Filename::new(filename),
        }
    }
}

impl Png {
    fn render_inner(
        &self,
        tile: &Tile,
        writer: Box<dyn std::io::Write>,
    ) -> Result<(), Box<dyn Error>> {
        let mut encoder = png::Encoder::new(writer, tile.spec.width, tile.spec.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
        encoder.add_text_chunk("comment".to_string(), tile.info_string())?;

        // MAYBE: allow user to specify gamma of their monitor?
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        // MAYBE: set source chromaticities?

        let mut png_writer = encoder.write_header()?;
        let mut image_data =
            Vec::<u8>::with_capacity(4 * tile.spec.width as usize * tile.spec.height as usize);
        let tile_data = tile.result();
        let max_iter = tile.max_iter_plotted;
        tile_data
            .elements_row_major_iter()
            .map(|pd| {
                if pd.iter == max_iter {
                    f64::INFINITY
                } else {
                    pd.iterations()
                }
            })
            .for_each(|iters| {
                let col = colour(iters);
                for b in col {
                    image_data.push(b);
                }
            });
        png_writer.write_image_data(&image_data)?;
        Ok(())
    }
}

impl Renderer for Png {
    fn render(&self, tile: &Tile) -> Result<(), Box<dyn Error>> {
        let path = Path::new(self.filename.get());
        let file = File::create(path)?;
        let w = Box::new(BufWriter::new(file)) as Box<dyn Write>;
        self.render_inner(tile, w)?;
        Ok(())
    }
}

/// An 8-bit RGBA quadruplet.
type Rgba = [u8; 4];

// temporary
fn colour(iters: f64) -> Rgba {
    // This is the colourer from mandy, impl here because it's quick
    // inf -> black, that's all good with us.
    let c = 2.0 * std::f64::consts::PI * iters.sqrt();
    [
        (((0.2 * c).cos() + 1.0) * 127.0) as u8,
        (((0.14285 * c).cos() + 1.0) * 127.0) as u8,
        (((0.090909 * c).cos() + 1.0) * 127.0) as u8,
        255,
    ]
}
