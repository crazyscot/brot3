// brot3 shared ui types
// (c) 2024 Ross Younger

use std::cell::RefCell;

use brot3_engine::{
    colouring,
    fractal::{self, Algorithm, AlgorithmSpec, Point, Scalar, TileSpec},
    util::Rect,
};

pub(crate) const UI_TILE_SIZE_LOG2: isize = 8;
pub(crate) const UI_TILE_SIZE: i32 = 1 << UI_TILE_SIZE_LOG2;

pub(crate) type TileIndex = i64; // TECHDEBT Precision limit: This will be converted to engine::Scalar [f64 at present]
pub(crate) type PixelIndex = i64;
pub(crate) type ZoomLevel = i32;

/// 2D coordinates of a pixel
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelCoordinate {
    pub x: PixelIndex,
    pub y: PixelIndex,
}

/// Default algorithm for the UI
pub(crate) fn default_algorithm() -> AlgorithmSpec {
    AlgorithmSpec::new(
        fractal::factory(fractal::Selection::Original),
        256,
        colouring::factory(colouring::Selection::LinearRainbow),
    )
}

// TILE SPECIFICATION ======================================================================

/// 3D coordinates ("address") of a tile within the universe
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TileCoordinate {
    /// zoom level
    pub z: ZoomLevel,
    /// Index of this tile within the universe, X dimension
    pub x: TileIndex,
    /// Index of this tile within the universe, Y dimension
    pub y: TileIndex,
    /// Algorithm details
    pub algspec: AlgorithmSpec,
}

impl TileCoordinate {
    /// Computes the pixel index of the top left pixel of this tile
    pub fn top_left_pixel_address(&self) -> PixelCoordinate {
        PixelCoordinate {
            x: self.x << UI_TILE_SIZE_LOG2,
            y: self.y << UI_TILE_SIZE_LOG2,
        }
    }
}

impl TryFrom<&TileCoordinate> for TileSpec {
    type Error = anyhow::Error;

    fn try_from(spec: &TileCoordinate) -> anyhow::Result<Self> {
        let algorithm = spec.algspec.algorithm;
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
            spec.algspec.max_iter,
            spec.algspec.colourer,
        ))
    }
}

// TILE DATA ===============================================================================

/// Data about tiles we've worked on which we may persist
/// N.B. This struct must implement `Send` (which should be automatically derived)
#[derive(Clone)]
pub(crate) struct PlottedTile {
    pub(crate) tile: RefCell<fractal::Tile>,
    pub(crate) image: slint::SharedPixelBuffer<slint::Rgba8Pixel>, // this cannot be slint::Image, which is not Send
}

/// A tile that is "done for now" and renderable, though we might come back to it later.
/// This struct is intended to be cached.
#[derive(Clone)]
pub(crate) struct FinishedTile {
    pub(crate) tile: PlottedTile,
    pub(crate) image: slint::Image,
}

impl From<PlottedTile> for FinishedTile {
    fn from(tile: PlottedTile) -> Self {
        let image = slint::Image::from_rgba8(tile.image.clone());
        Self { tile, image }
    }
}
