// brot3 shared ui types
// (c) 2024 Ross Younger

use brot3_engine::{
    colouring,
    fractal::{self, Algorithm, Point, Scalar, TileSpec},
    util::Rect,
};

pub(crate) const UI_TILE_SIZE_LOG2: isize = 8;
pub(crate) const UI_TILE_SIZE: i32 = 1 << UI_TILE_SIZE_LOG2;

pub(crate) type TileIndex = i64; // TECHDEBT Precision limit: This will be converted to engine::Scalar [f64 at present]
pub(crate) type PixelIndex = i64;
pub(crate) type ZoomLevel = i32;

/// 3D coordinates ("address") of a tile within the universe
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TileCoordinate {
    /// zoom level
    pub z: ZoomLevel,
    /// Index of this tile within the universe, X dimension
    pub x: TileIndex,
    /// Index of this tile within the universe, Y dimension
    pub y: TileIndex,
}

impl TryFrom<&TileCoordinate> for TileSpec {
    type Error = anyhow::Error;

    fn try_from(spec: &TileCoordinate) -> anyhow::Result<Self> {
        let algorithm = fractal::decode("Original")?; // TODO comes from spec
        let colourer = colouring::decode("LogRainbow")?; // TODO comes from spec
        let tile_size = crate::UI_TILE_SIZE as u32;

        // TECHDEBT: Max resolution is 2^53 at present, limited by f64 mantissa size.
        anyhow::ensure!(spec.z < 54, "zoom too deep");

        // Map the total number of pixels across the zoom level into the algorithm's full axes
        // Zoom level 0 means "we can see the entire fractal in a single tile"
        let total_dimension: TileIndex = 1 << (8 + spec.z);
        let tile_count = total_dimension / TileIndex::from(tile_size);
        // now we know where we are in integer co-ordinates, map into reals.
        // axes are easy :-)
        #[allow(clippy::cast_precision_loss)]
        // Tile_count is a power of 2, so can safely be cast to f64.
        let tile_axes = algorithm.default_axes() / tile_count as Scalar;

        // origin is a little more work... we again face the top-left vs bottom-left war.
        let total_bottom_left = algorithm.default_centre() - 0.5 * algorithm.default_axes();
        let total_top_left = Point {
            re: total_bottom_left.re,
            im: total_bottom_left.im + algorithm.default_axes().im,
        };

        #[allow(clippy::cast_precision_loss)]
        // TECHDEBT: spec.x and spec.y cannot safely be case to f64 unless we can guarantee that they will
        // fit into the mantissa, so the effective limit is 2^53
        let tile_top_left = Point {
            re: total_top_left.re + spec.x as Scalar * tile_axes.re,
            im: total_top_left.im - spec.y as Scalar * tile_axes.im,
        };
        let tile_bottom_left = Point {
            re: tile_top_left.re,
            im: tile_top_left.im - tile_axes.im,
        };

        let output_size = Rect::new(tile_size, tile_size);
        let origin = fractal::Location::Origin(tile_bottom_left);
        let axes = fractal::Size::AxesLength(tile_axes);
        Ok(TileSpec::new(
            origin,
            axes,
            output_size,
            algorithm,
            256, /* TODO becomes spec.max_iter */
            colourer,
        ))
    }
}
