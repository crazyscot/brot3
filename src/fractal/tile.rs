// (c) 2024 Ross Younger

use super::{Algorithm, FractalInstance, Point, PointData, TileSpec};
use crate::util::Rect;

use anyhow::{anyhow, ensure, Context};
use array2d::Array2D;
use std::{cmp::max, fmt};

/// A section of a fractal plot
#[derive(Debug)]
pub struct Tile {
    /// Debug output level
    debug: u8,
    /// Working data. Address as [(x,y)] aka (re,im).
    point_data: Array2D<PointData>,
    /// Max iterations we plotted to
    pub max_iter_plotted: u32,
    /// Specification of this plot
    pub spec: TileSpec,
    /// The algorithm to use
    algorithm: FractalInstance,
    /// If present, this tile is part of a larger plot; this is its location offset (X,Y) in pixels, from the origin
    offset_within_plot: Option<Rect<u32>>,
}

impl Tile {
    /// Standard constructor. Also initialises the data for this tile.
    #[must_use]
    pub fn new(spec: &TileSpec, debug: u8) -> Self {
        let mut new1 = Tile::new_internal(spec, debug);
        new1.prepare();
        new1
    }

    /// Internal constructor used by new() and join()
    fn new_internal(spec: &TileSpec, debug: u8) -> Self {
        Self {
            debug,
            // Data for this tile. @warning Array2D square bracket syntax is (row,column) i.e. (y,x) !
            point_data: Array2D::filled_with(
                PointData::default(),
                spec.height() as usize,
                spec.width() as usize,
            ),
            max_iter_plotted: 0,
            spec: *spec,
            algorithm: spec.algorithm(),
            offset_within_plot: spec.offset_within_plot(),
        }
    }

    /// Quasi-constructor: Reassembles the tiles of a split plot into a single plot
    pub fn join(spec: &TileSpec, tiles: &Vec<Tile>) -> anyhow::Result<Tile> {
        ensure!(!tiles.is_empty(), "No tiles given to join");
        // TODO: Might be nice if we could ensure that all data points were covered i.e. no tiles are missing...

        let mut result = Tile::new_internal(spec, 0);
        result.max_iter_plotted = tiles.iter().fold(0, |b, t| max(b, t.max_iter_plotted));

        for t in tiles {
            let offset = t
                .offset_within_plot
                .ok_or_else(|| anyhow!("joining subtitle did not contain offset"))
                .with_context(|| format!("{t:?}"))?;

            // map source (0,0) => dest OFFSET
            let dest_address = |s: Rect<u32>| s + offset;

            // there's probably a nicer way to do this
            for y in 0..t.spec.height() {
                for x in 0..t.spec.width() {
                    let a = dest_address(Rect::new(x, y));

                    // Caution! PointData coordinates are in order (y,x)
                    let val = t.point_data[(y as usize, x as usize)];
                    result.point_data[(a.height as usize, a.width as usize)] = val;
                }
            }
        }
        Ok(result)
    }

    /// Initialises the data for this tile
    fn prepare(&mut self) {
        let step = self.spec.pixel_size();
        // TRAP: Plot origin != first pixel origin !
        // The plotted point of each pixel should be the CENTRE of the region, i.e. offset by half a pixel from plot origin.
        let origin_pixel = self.spec.origin() + 0.5 * step;

        let mut imag = origin_pixel.im;
        for y in 0..self.spec.height() {
            let mut real = origin_pixel.re;
            for x in 0..self.spec.width() {
                let real_y = self.spec.height() - y - 1;
                // curveball: origin is bottom left of the plot, but we want to output the top row first.
                let point = &mut self.point_data[(real_y as usize, x as usize)];
                point.origin = Point { re: real, im: imag };
                self.algorithm.prepare(point);
                real += step.re;
            }
            imag += step.im;
        }
        // TODO: live_pixel count
    }

    /// Runs the fractal iteration for all points in this tile
    pub fn plot(&mut self, max_iter: u32) {
        for y in 0..self.spec.height() as usize {
            for x in 0..self.spec.width() as usize {
                let point = &mut self.point_data[(y, x)];
                if point.result.is_none() {
                    self.algorithm.pixel(point, max_iter);
                }
            }
        }
        self.max_iter_plotted = max_iter;
        // TODO live pixel count
    }

    /// Result accessor
    #[must_use]
    pub fn result(&self) -> &Array2D<PointData> {
        &self.point_data
    }

    /// Accessor
    #[must_use]
    pub fn offset_within_plot(&self) -> Option<Rect<u32>> {
        self.offset_within_plot
    }

    /// Info string quasi-accessor
    #[must_use]
    pub fn info_string(&self) -> String {
        self.spec.to_string()
    }
}

/// CSV format output
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.spec.height() as usize {
            self.point_data[(y, 0)].fmt(f, self.debug)?;
            for x in 1..self.spec.width() as usize {
                write!(f, ",")?;
                self.point_data[(y, x)].fmt(f, self.debug)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        fractal::{
            tilespec::SplitMethod, FractalInstance, Location, PlotSpec, Point, Size, TileSpec, Zero,
        },
        util::Rect,
    };

    use super::Tile;

    const ZERO_ALG: FractalInstance = FractalInstance::Zero(Zero {});
    const ZERO: Point = Point { re: 0.0, im: 0.0 };
    const ONE: Point = Point { re: 1.0, im: 1.0 };

    const TD_TILE: PlotSpec = PlotSpec {
        location: Location::Origin(ZERO),
        axes: Size::AxesLength(ONE),
        size_in_pixels: Rect::<u32> {
            width: 100,
            height: 101, // not dividable by 10
        },
        algorithm: ZERO_ALG,
    };
    #[test]
    fn rejoin() {
        let spec = TileSpec::from(&TD_TILE);
        let split = spec.split(SplitMethod::RowsOfHeight(10));
        let mut tiles: Vec<Tile> = split.iter().map(|ts| Tile::new(ts, 0)).collect();
        for t in &mut tiles {
            t.plot(1);
        }
        let result = Tile::join(&spec, &tiles).unwrap();
        let data = result.result();
        assert_eq!(data.num_rows(), spec.height() as usize);
        assert_eq!(data.num_columns(), spec.width() as usize);
    }
}
