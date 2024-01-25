// Rendering output to PNG files
// (c) 2024 Ross Younger

use super::Renderer;
use crate::colouring::{OutputsRgb8, PaletteInstance};
use crate::fractal::Tile;
use crate::util::filename::Filename;

use anyhow::{Context, Result};
use std::io::BufWriter;

#[derive(Clone, Debug)]
pub struct Png {
    filename: Filename,
    colourer: PaletteInstance,
}

impl Default for Png {
    fn default() -> Self {
        Self {
            filename: Filename::new(""),
            colourer: PaletteInstance::LinearRainbow(crate::colouring::huecycles::LinearRainbow {}),
        }
    }
}

impl Png {
    pub(crate) fn new(filename: &str, colourer: PaletteInstance) -> Self {
        Png {
            filename: Filename::new(filename),
            colourer,
        }
    }
}

impl Png {
    fn render_inner(
        tile: &Tile,
        colourer: PaletteInstance,
        writer: Box<dyn std::io::Write>,
    ) -> Result<()> {
        let mut encoder = png::Encoder::new(writer, tile.spec.width(), tile.spec.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
        encoder.add_text_chunk("comment".to_string(), tile.info_string())?;

        // MAYBE: allow user to specify gamma of their monitor?
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        // MAYBE: set source chromaticities?

        let mut png_writer = encoder.write_header()?;
        let mut image_data =
            Vec::<u8>::with_capacity(4 * tile.spec.width() as usize * tile.spec.height() as usize);
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
                let col = colourer.colour_rgb8(iters);
                image_data.push(col.red);
                image_data.push(col.green);
                image_data.push(col.blue);
                image_data.push(255);
            });
        png_writer.write_image_data(&image_data)?;
        Ok(())
    }
}

impl Renderer for Png {
    fn render(&self, tile: &Tile) -> Result<()> {
        let handle = self.filename.write_handle()?;
        let bw = Box::new(BufWriter::new(handle));
        Png::render_inner(tile, self.colourer, bw).with_context(|| "Failed to render PNG")?;
        // You can test this error pathway by trying to write to /dev/full
        Ok(())
    }
}
