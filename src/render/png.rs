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
pub struct Png {}

impl Png {
    fn render_inner(
        tile: &Tile,
        colourer: Instance,
        writer: Box<dyn std::io::Write>,
    ) -> Result<()> {
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
        let mut image_data =
            Vec::<u8>::with_capacity(4 * tile.spec.width() as usize * tile.spec.height() as usize);
        let tile_data = tile.result();
        let max_iter = tile.max_iter_plotted;
        tile_data
            .elements_row_major_iter()
            .map(|pd| {
                #[allow(clippy::cast_possible_truncation)]
                // Apply colouring function
                if pd.iter == max_iter {
                    std::f32::INFINITY
                } else {
                    pd.iterations()
                }
            })
            .for_each(|iters| {
                // PNG wants four u8s, in RGBA order
                #[allow(clippy::cast_lossless)]
                let (r, g, b) =
                    Srgb::<u8>::from(colourer.colour_rgb8(iters, max_iter)).into_components();
                image_data.push(r);
                image_data.push(g);
                image_data.push(b);
                image_data.push(255);
            });
        png_writer.write_image_data(&image_data)?;
        Ok(())
    }
}

impl Renderer for Png {
    fn render_file(&self, filename: &str, tile: &Tile, colourer: Instance) -> anyhow::Result<()> {
        let handle = Filename::open_for_writing(filename)?;
        let bw = Box::new(BufWriter::new(handle));
        Png::render_inner(tile, colourer, bw).with_context(|| "Failed to render PNG")?;
        // You can test this error pathway by trying to write to /dev/full
        Ok(())
    }
}
