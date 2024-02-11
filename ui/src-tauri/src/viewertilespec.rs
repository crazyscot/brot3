// Tile spec from the OpenSeadragon viewer's point of view
// (c) 2024 Ross Younger

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ViewerTileSpec {
    /// Zoom level (OpenSeadragon spec; level X means a square image is represented by 2^X pixels in either dimension)
    pub level: usize,
    /// Column indicator for the tile (0-based)
    pub dx: usize,
    /// Row indicator for the tile (0-based)
    pub dy: usize,
    /// Tile width
    pub width: usize,
    /// Tile height
    pub height: usize,
}
