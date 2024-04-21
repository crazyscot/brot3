// Tile spec from the OpenSeadragon viewer's point of view
// (c) 2024 Ross Younger

use std::fmt;

use brot3_engine::{
    colouring,
    fractal::{self, Algorithm, Point, Scalar, TileSpec},
    util::Rect,
};
use serde::Deserialize;

/// Twin of TS TileSpec class
#[derive(Clone, Deserialize)]
pub struct ViewerTileSpec {
    /// Request serial number from viewer
    pub serial: u64,
    /// Zoom level (OpenSeadragon spec; level X means a square image is represented by 2^X pixels in either dimension)
    pub level: u32,
    /// Column indicator for the tile (0-based)
    pub dx: u64,
    /// Row indicator for the tile (0-based)
    pub dy: u64,
    /// Tile width (pixels)
    pub width: u32,
    /// Tile height (pixels)
    pub height: u32,
    /// Iteration limit
    pub max_iter: u32,
    /// Selected fractal
    pub algorithm: String,
}

impl fmt::Display for ViewerTileSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}-{}", self.level, self.dx, self.dy)
    }
}

impl TryFrom<&ViewerTileSpec> for TileSpec {
    type Error = anyhow::Error;

    fn try_from(spec: &ViewerTileSpec) -> anyhow::Result<Self> {
        let algorithm = fractal::decode(&spec.algorithm)?;
        let col_requested = "LogRainbow"; // TODO this will come from spec
        let colourer = colouring::decode(col_requested)?;
        anyhow::ensure!(spec.level < 64, "zoom too deep");
        anyhow::ensure!(spec.width == spec.height, "only square tiles supported");
        // Map the total number of pixels across the zoom level into the algorithm's full axes
        let total_dimension: u64 = 1 << spec.level;
        let tile_count: u64 = total_dimension / spec.width as u64;
        // now we know where we are in integer co-ordinates, map into reals.
        // axes are easy :-)
        let tile_axes = algorithm.default_axes() / tile_count as Scalar;

        // origin is a little more work... we again face the top-left vs bottom-left war.
        let total_bottom_left = algorithm.default_centre() - 0.5 * algorithm.default_axes();
        let total_top_left = Point {
            re: total_bottom_left.re,
            im: total_bottom_left.im + algorithm.default_axes().im,
        };

        let tile_top_left = Point {
            re: total_top_left.re + spec.dx as Scalar * tile_axes.re,
            im: total_top_left.im - spec.dy as Scalar * tile_axes.im,
        };
        let tile_bottom_left = Point {
            re: tile_top_left.re,
            im: tile_top_left.im - tile_axes.im,
        };

        let output_size = Rect::new(spec.width, spec.height);
        let origin = fractal::Location::Origin(tile_bottom_left);
        let axes = fractal::Size::AxesLength(tile_axes);
        Ok(TileSpec::new(
            origin,
            axes,
            output_size,
            algorithm,
            spec.max_iter,
            colourer,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::ViewerTileSpec;
    use approx::assert_relative_eq;
    use brot3_engine::fractal::{self, Algorithm, Point, TileSpec};

    static DEFAULT_SPEC_STORAGE: OnceLock<ViewerTileSpec> = OnceLock::new();

    fn default_spec() -> ViewerTileSpec {
        DEFAULT_SPEC_STORAGE
            .get_or_init(|| ViewerTileSpec {
                serial: 42,
                level: 9,
                dx: 0,
                dy: 0,
                width: 256,
                height: 256,
                max_iter: 1024,
                algorithm: "Original".to_string(),
            })
            .clone()
    }

    #[test]
    fn convert_level_9() {
        // at level 9, a 256x256 tile is exactly a quarter of the total image
        // therefore, tile(0,0) top-left should match the overall top-left and end at the centre
        // and tile (1,1) should start at the centre and end bottom-right
        let alg = fractal::factory(fractal::Selection::Original);
        let alg_centre = alg.default_centre();
        let alg_origin = alg_centre - 0.5 * alg.default_axes();
        let alg_top_left = alg_origin
            + Point {
                re: 0.0,
                im: alg.default_axes().im,
            };
        let alg_end = alg_origin + alg.default_axes();
        let mut td = default_spec();
        {
            // tile(0,0) top-left (NOT ORIGIN) should match the overall top-left; its bottom-right should be the centre
            let ts = TileSpec::try_from(&td).unwrap();
            let top_left = ts.top_left();
            assert_relative_eq!(top_left.re, alg_top_left.re);
            assert_relative_eq!(top_left.im, alg_top_left.im);
            let bottom_right = ts.bottom_right();
            assert_relative_eq!(bottom_right.re, alg.default_centre().re);
            assert_relative_eq!(bottom_right.im, alg.default_centre().im);
        }
        {
            // tile (1,1) should start (top-left) at the centre and end bottom-right
            td.dx = 1;
            td.dy = 1;
            let ts = TileSpec::try_from(&td).unwrap();
            assert_relative_eq!(ts.top_left().re, alg_centre.re);
            assert_relative_eq!(ts.top_left().im, alg_centre.im);
            assert_relative_eq!(ts.bottom_right().re, alg_end.re);
            assert_relative_eq!(ts.bottom_right().im, -alg_end.im);
        }
    }

    #[test]
    fn convert_level_13() {
        // at level 13, the total image is 2^13 pixels across
        // therefore there are 32 256x256 tiles in either dimension
        let alg = fractal::factory(fractal::Selection::Original);
        let alg_centre = alg.default_centre();
        let alg_origin = alg_centre - 0.5 * alg.default_axes();
        let alg_end = alg_origin + alg.default_axes();

        let mut td = default_spec();
        td.level = 13;
        // 1. check tile (0,0) has the same top left as overall
        {
            let ts = TileSpec::try_from(&td).unwrap();
            assert_relative_eq!(ts.top_left().re, alg_origin.re);
            assert_relative_eq!(ts.top_left().im, alg_end.im);
        }
        // 2. tile(15,15) should bottom-right at the centre
        {
            td.dx = 15;
            td.dy = 15;
            let ts = TileSpec::try_from(&td).unwrap();
            assert_relative_eq!(ts.bottom_right().re, alg_centre.re);
            assert_relative_eq!(ts.bottom_right().im, alg_centre.im);
        }
        // 3. tile (16,16) should top-left at the centre
        {
            td.dx = 16;
            td.dy = 16;
            let ts = TileSpec::try_from(&td).unwrap();
            assert_relative_eq!(ts.top_left().re, alg_centre.re);
            assert_relative_eq!(ts.top_left().im, alg_centre.im);
        }
        // 4. tile (31,0) is the top right tile, so the real axis ends where the overall plot's real axis does
        {
            td.dx = 31;
            td.dy = 0;
            let ts = TileSpec::try_from(&td).unwrap();
            assert_relative_eq!(ts.bottom_right().re, alg_end.re);
        }
        // 5. tile (0,31) is the bottom left, so the imaginary axis ends where the overall plot's imag axis starts
        {
            td.dx = 0;
            td.dy = 31;
            let ts = TileSpec::try_from(&td).unwrap();
            assert_relative_eq!(ts.bottom_right().im, alg_origin.im);
        }
    }

    #[test]
    fn invalid_conversions() {
        let mut td = ViewerTileSpec {
            serial: 42,
            level: 65,
            dx: 0,
            dy: 0,
            width: 256,
            height: 256,
            max_iter: 1024,
            algorithm: "Original".into(),
        };
        TileSpec::try_from(&td).expect_err("should have failed");
        td.level = 10;
        TileSpec::try_from(&td).expect("should have succeeded");
        td.width += 1;
        TileSpec::try_from(&td).expect_err("should have failed");
    }
}
