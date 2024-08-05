// Rendering output to PNG files
// (c) 2024 Ross Younger

use super::Renderer;
use crate::colouring::{Instance, OutputsRgb8};
use crate::fractal::{Tile, TileSpec};
use crate::util::filename::Filename;

use anyhow::{Context, Result};
use palette::Srgb;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::io::BufWriter;

#[derive(Clone, Copy, Debug, Default)]
/// Render output to a PNG file (or, if you call it directly, to pixels in PNG format)
pub struct Png {}

impl Png {
    /// Renders a tile as a low-level array of RGBA values.
    /// These are returned in the obvious left to right, top to bottom order.
    #[must_use]
    pub fn render_rgba(tile: &Tile, colourer: Instance) -> Vec<u8> {
        let mut image_data: Vec<u8> =
            vec![0; 4 * tile.spec.width() as usize * tile.spec.height() as usize];
        Png::render_rgba_into(tile, colourer, image_data.as_mut_slice());
        image_data
    }
    /// Renders a tile as a low-level array of RGBA values, into a caller-provided buffer.
    /// Pixels are rendered in the obvious left to right, top to bottom order.
    /// # Panics
    /// `image_data` must be precisely 4 x tile height x tile width.
    pub fn render_rgba_into(tile: &Tile, colourer: Instance, image_data: &mut [u8]) {
        let tile_data = tile.result();
        let max_iter = tile.max_iter_plotted;
        let expected_size = 4 * tile.spec.width() as usize * tile.spec.height() as usize;
        assert_eq!(image_data.len(), expected_size);
        let mut output_index = 0;
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
                image_data[output_index] = pixel.red;
                image_data[output_index + 1] = pixel.green;
                image_data[output_index + 2] = pixel.blue;
                image_data[output_index + 3] = 255;
                output_index += 4;
            });
    }

    fn render_png(
        spec: &TileSpec,
        tiles: &[Tile],
        colourer: Instance,
        writer: Box<dyn std::io::Write>,
    ) -> Result<()> {
        let mut encoder = png::Encoder::new(writer, spec.width(), spec.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
        let info = spec.to_string();
        encoder.add_text_chunk("comment".to_string(), info)?;

        // MAYBE: allow user to specify gamma of their monitor?
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        // MAYBE: set source chromaticities?

        let mut png_writer = encoder.write_header()?;
        let image_data = tiles
            .par_iter()
            .flat_map(|t| Self::render_rgba(t, colourer))
            .collect::<Vec<u8>>();
        png_writer.write_image_data(&image_data)?;
        Ok(())
    }
}

impl Renderer for Png {
    fn render_file(
        &self,
        filename: &str,
        spec: &TileSpec,
        tiles: &[Tile],
        colourer: Instance,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(self.check_ordering(tiles), "Tiles out of order");
        let handle = Filename::open_for_writing(filename)?;
        let bw = Box::new(BufWriter::new(handle));
        Png::render_png(spec, tiles, colourer, bw).with_context(|| "Failed to render PNG")?;
        // You can test this error pathway by trying to write to /dev/full
        Ok(())
    }
}
