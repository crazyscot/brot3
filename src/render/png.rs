// Rendering output to PNG files
// (c) 2024 Ross Younger

use crate::fractal::Tile;
use crate::render::Renderer;
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

impl Renderer for Png {
    fn render(&self, tile: &Tile) -> Result<(), Box<dyn Error>> {
        let path = Path::new(self.filename.get());
        let file = File::create(path)?;
        let mut w = &mut BufWriter::new(file);
        {
            let mut encoder = png::Encoder::new(&mut w, tile.width as u32, tile.height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
            // TODO ... tile info string ?
            encoder.add_text_chunk("comment".to_string(), "image info goes here".to_string())?;

            // MAYBE: allow user to specify gamma of their monitor?
            encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
            // MAYBE: set source chromaticities?

            let mut writer = encoder.write_header()?;
            let mut image_data = Vec::<u8>::with_capacity(4 * tile.width * tile.height);
            let tile_data = tile.result();
            tile_data
                .elements_row_major_iter()
                .map(|pd| pd.iterations())
                .for_each(|iters| {
                    // TODO: if it's maxiter, set inf
                    let col = colour(iters);
                    for b in col {
                        image_data.push(b);
                    }
                });
            writer.write_image_data(&image_data)?;
        }

        w.flush()?;
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
