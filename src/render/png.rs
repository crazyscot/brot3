// Rendering output to PNG files
// (c) 2024 Ross Younger

use super::Renderer;
use crate::fractal::Tile;
use crate::util::filename::Filename;

use anyhow::{Context, Result};
use std::io::BufWriter;

pub(crate) struct Png {
    filename: Filename,
}

impl Png {
    pub(crate) fn new(filename: &str) -> Self {
        Png {
            filename: Filename::new(filename),
        }
    }
}

impl Png {
    fn render_inner(&self, tile: &Tile, writer: Box<dyn std::io::Write>) -> Result<()> {
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
    fn render(&self, tile: &Tile) -> Result<()> {
        let handle = self.filename.write_handle()?;
        let bw = Box::new(BufWriter::new(handle));
        self.render_inner(tile, bw)
            .with_context(|| "Failed to render PNG")?;
        // You can test this error pathway by trying to write to /dev/full
        Ok(())
    }
}

/// An 8-bit RGBA quadruplet.
type Rgba = [u8; 4];

// temporary
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn colour(iters: f64) -> Rgba {
    // This is the colourer from mandy, impl here because it's quick
    // inf -> black, that's all good with us.
    let c = 2.0 * std::f64::consts::PI * iters.sqrt();
    [
        (((0.2 * c).cos() + 1.0) * 127.0) as u8,
        (((0.14285 * c).cos() + 1.0) * 127.0) as u8,
        (((0.090_909 * c).cos() + 1.0) * 127.0) as u8,
        255,
    ]
}
