// Rendering output to PNG files
// (c) 2024 Ross Younger

use super::Renderer;
use crate::colouring::{Instance, OutputsRgb8};
use crate::fractal::Tile;
use crate::util::filename::Filename;

use anyhow::{Context, Result};
use palette::Srgb;
use std::io::BufWriter;

#[derive(Clone, Copy, Debug, Default)]
/// Render output to a PNG file (or, if you call it directly, to pixels in PNG format)
pub struct Png {}

impl Png {
    /// Renders a tile as a low-level array of RGBA values.
    /// These are returned in the obvious left to right, top to bottom order.
    #[must_use]
    pub fn render_rgba(tile: &Tile, colourer: Instance) -> Vec<u8> {
        let mut image_data =
            Vec::<u8>::with_capacity(4 * tile.spec.width() as usize * tile.spec.height() as usize);
        let tile_data = tile.result();
        let max_iter = tile.max_iter_plotted;
        tile_data
            .iter()
            .map(|pd| {
                // if it's still alive, assume infinity
                if pd.iter == max_iter {
                    std::f32::INFINITY
                } else {
                    pd.iterations()
                }
            })
            .for_each(|iters| {
                // Colour and output
                let pixel = Srgb::<u8>::from(colourer.colour_rgb8(iters, max_iter));
                image_data.push(pixel.red);
                image_data.push(pixel.green);
                image_data.push(pixel.blue);
                image_data.push(255);
            });
        image_data
    }

    fn render_png(tile: &Tile, colourer: Instance, writer: Box<dyn std::io::Write>) -> Result<()> {
        let mut encoder = png::Encoder::new(writer, tile.spec.width(), tile.spec.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
        let info = tile.info_string(&colourer);
        encoder.add_text_chunk("comment".to_string(), info)?;

        // MAYBE: allow user to specify gamma of their monitor?
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        // MAYBE: set source chromaticities?

        let mut png_writer = encoder.write_header()?;
        let image_data = Self::render_rgba(tile, colourer);
        png_writer.write_image_data(&image_data)?;
        Ok(())
    }
}

impl Renderer for Png {
    fn render_file(&self, filename: &str, tile: &Tile, colourer: Instance) -> anyhow::Result<()> {
        let handle = Filename::open_for_writing(filename)?;
        let bw = Box::new(BufWriter::new(handle));
        Png::render_png(tile, colourer, bw).with_context(|| "Failed to render PNG")?;
        // You can test this error pathway by trying to write to /dev/full
        Ok(())
    }
}
