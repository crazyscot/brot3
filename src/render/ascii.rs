// Rendering output in various ASCII-based formats
// (c) 2024 Ross Younger
use super::Renderer;
use crate::fractal::{PointData, Tile};
use crate::util::filename::Filename;

use anyhow::Result;

/// CSV format, fractal points
pub(crate) struct Csv {
    filename: Filename,
}

impl Csv {
    pub(crate) fn new(filename: &str) -> Self {
        Csv {
            filename: Filename::new(filename),
        }
    }
}

impl Renderer for Csv {
    fn render(&self, tile: &Tile) -> Result<()> {
        self.filename
            .write_handle()?
            .write_all(tile.to_string().as_bytes())?;
        Ok(())
    }
}

/// Rough and ready ASCII art renderer
pub(crate) struct AsciiArt {
    filename: Filename,
}

const DEFAULT_ASCII_ART_CHARSET: &[u8] = " .,:obOB%#".as_bytes();

impl AsciiArt {
    pub(crate) fn new(filename: &str) -> Self {
        AsciiArt {
            filename: Filename::new(filename),
        }
    }
}

impl Renderer for AsciiArt {
    fn render(&self, tile: &Tile) -> Result<()> {
        let mut output = self.filename.write_handle()?;

        // Find the range of output levels, discounting INF.
        let data = tile.result();
        let iter = data
            .elements_column_major_iter()
            .map(PointData::iterations)
            .filter(|iters| iters.is_finite());
        let most = iter
            .clone()
            .reduce(f64::max) // this syntax needed as f64 doesn't implement Ord
            .unwrap();
        let least = iter.reduce(f64::min).unwrap();

        // Map the output levels to a set of characters
        let n_levels = DEFAULT_ASCII_ART_CHARSET.len();
        let range = most - least;
        // Infinity will take the last character, map the rest linearly for now
        let step = range / (n_levels - 1) as f64;

        for row in data.as_rows() {
            let mut rowstr = row
                .iter()
                .map(PointData::iterations)
                .map(|it| {
                    if it.is_infinite() {
                        *DEFAULT_ASCII_ART_CHARSET.last().unwrap()
                    } else {
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
