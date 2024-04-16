// Rendering output in various ASCII-based formats
// (c) 2024 Ross Younger
use super::Renderer;
use crate::colouring::Instance;
use crate::fractal::{PointData, Tile, TileSpec};
use crate::util::filename::Filename;

/// CSV format, fractal points
#[derive(Clone, Copy, Debug, Default)]
pub struct Csv {}

impl Renderer for Csv {
    fn render_file(
        &self,
        filename: &str,
        _: &TileSpec,
        tiles: &[Tile],
        _: Instance,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(self.check_ordering(tiles), "Tiles out of order");
        let mut output = Filename::open_for_writing(filename)?;
        for t in tiles {
            output.write_all(t.to_string().as_bytes())?;
        }
        Ok(())
    }
}

/// Rough and ready ASCII art renderer
#[derive(Clone, Copy, Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct AsciiArt {}

const DEFAULT_ASCII_ART_CHARSET: &[u8] = " .,:obOB%#".as_bytes();

impl Renderer for AsciiArt {
    fn render_file(
        &self,
        filename: &str,
        _: &TileSpec,
        tiles: &[Tile],
        _: Instance,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(self.check_ordering(tiles), "Tiles out of order");
        let mut output = Filename::open_for_writing(filename)?;
        // Preprocess: Find the range of output levels, discounting INF.
        let iter = tiles
            .iter()
            .flat_map(Tile::result)
            .map(PointData::iterations)
            .filter(|iters| iters.is_finite());
        let most = iter.clone().reduce(f32::max).unwrap();
        let least = iter.reduce(f32::min).unwrap();

        for tile in tiles {
            AsciiArt::render_tile(&mut output, tile, most, least)?;
        }
        Ok(())
    }
}

impl AsciiArt {
    fn render_tile(
        output: &mut Box<dyn std::io::Write>,
        tile: &Tile,
        most: f32,
        least: f32,
    ) -> anyhow::Result<()> {
        // Map the output levels to a set of characters
        let n_levels = DEFAULT_ASCII_ART_CHARSET.len();
        let range = most - least;
        // Infinity will take the last character, map the rest linearly for now
        #[allow(clippy::cast_precision_loss)] // this is a quick & dirty output module
        let step = range / (n_levels - 1) as f32;

        for y in 0..tile.spec.height() as usize {
            let start = y * tile.spec.width() as usize;
            let end = (y + 1) * tile.spec.width() as usize;
            let row = tile.result().get(start..end).expect("failed to slice row");
            //assert_eq!(row.len(), tile.spec.width() as usize);
            let mut rowstr = row
                .iter()
                .map(PointData::iterations)
                .map(|it| {
                    if it.is_infinite() {
                        *DEFAULT_ASCII_ART_CHARSET.last().unwrap()
                    } else {
                        // this is a quick & dirty output module
                        #[allow(clippy::cast_possible_truncation)]
                        #[allow(clippy::cast_sign_loss)]
                        DEFAULT_ASCII_ART_CHARSET[((it - least) / step) as usize]
                    }
                })
                .collect::<Vec<_>>();
            rowstr.push(b'\n');
            output.write_all(&rowstr)?;
        }
        Ok(())
    }
}
