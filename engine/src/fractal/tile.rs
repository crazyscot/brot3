// (c) 2024 Ross Younger

use super::{Algorithm, Point, PointData, Scalar, TileSpec};
use crate::{colouring, util::Rect};

use anyhow::{anyhow, ensure, Context};
use ndarray::Array2;
use num_complex::ComplexFloat;
use std::{cmp::max, fmt};

/// A section of a fractal plot
#[derive(Debug)]
pub struct Tile {
    /// Debug output level
    debug: u8,
    /// Working data. Address as [(row,column)] aka (y,x).
    /// <div class="warning">CAUTION: This array is TOP LEFT oriented. The first row is the top row, not the Origin row.</div>
    point_data: ndarray::Array2<PointData>,
    /// Max iterations we plotted to
    pub max_iter_plotted: u32,
    /// Specification of this plot
    pub spec: TileSpec,
    /// The algorithm to use
    algorithm: super::Instance,
    /// If present, this tile is part of a larger plot; this is its location offset (X,Y) in pixels, relative to the TOP LEFT of the plot.
    offset_within_plot: Option<Rect<u32>>,
}

impl Tile {
    /// Standard constructor. Also initialises the data for this tile.
    #[must_use]
    pub fn new(spec: &TileSpec, debug: u8) -> Self {
        let mut new1 = Tile::new_internal(spec, debug);
        new1.prepare(debug);
        new1
    }

    /// Internal constructor used by `new()` and `join()`
    fn new_internal(spec: &TileSpec, debug: u8) -> Self {
        Self {
            debug,
            // Data for this tile.
            point_data: Array2::default((spec.height() as usize, spec.width() as usize)),
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
                .ok_or_else(|| anyhow!("joining subtile did not contain offset"))
                .with_context(|| format!("{t:?}"))?;

            let mut dest = result.point_data.slice_mut(ndarray::s![
                offset.height as usize..(offset.height + t.spec.height()) as usize,
                offset.width as usize..(offset.width + t.spec.width()) as usize
            ]);
            dest.assign(&t.point_data);
        }
        Ok(result)
    }

    /// Initialises the data for this tile
    fn prepare(&mut self, debug: u8) {
        #![allow(clippy::cast_precision_loss)]
        // This is a compound step in both dimensions. We will step the dimensions separately (see `real` and `imag`).
        let step = self.spec.pixel_size();

        // Start plotting from the top-left
        let origin_pixel = Point {
            re: self.spec.origin().re,
            im: self.spec.origin().im + self.spec.axes().im(), // Top row
        };
        // TODO: Consider offsetting each pixel by half a step. However, it's only material on very small plots so maybe not worth it?

        if debug > 0 {
            println!(
                "Plot origin {}, TL pixel {origin_pixel}, axes {}, step {step}",
                self.spec.origin(),
                self.spec.axes()
            );
        }

        for ((y, x), point) in self.point_data.indexed_iter_mut() {
            point.origin = origin_pixel
                + Point {
                    re: x as Scalar * step.re,
                    im: y as Scalar * -step.im, // note '-' as we are stepping down the imaginary dimension
                };
            if debug > 1 {
                println!("point {x},{y} => {}", point.origin);
            }
            self.algorithm.prepare(point);
        }
        // TODO: live_pixel count
    }

    /// Runs the fractal iteration for all points in this tile
    pub fn plot(&mut self) {
        let max_iter = self.spec.max_iter_requested();
        for p in &mut self.point_data {
            if p.result.is_none() {
                self.algorithm.pixel(p, max_iter);
            }
        }
        self.max_iter_plotted = max_iter;
        // TODO live pixel count
    }

    /// Result accessor
    /// <div class="warning">CAUTION: This array is TOP LEFT oriented. The first row is the top row, not the Origin (bottom) row!</div>
    #[must_use]
    pub fn result(&self) -> &Array2<PointData> {
        &self.point_data
    }

    /// Accessor
    #[must_use]
    pub fn offset_within_plot(&self) -> Option<Rect<u32>> {
        self.offset_within_plot
    }

    /// Info string quasi-accessor
    #[must_use]
    pub fn info_string(&self, colourer: &colouring::Instance) -> String {
        format!(
            "{} maxiter={} colourer={}",
            self.spec, self.max_iter_plotted, colourer
        )
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
            framework::Zero, tilespec::SplitMethod, Instance, Location, PlotSpec, Point, Size,
            TileSpec,
        },
        util::Rect,
    };

    use super::Tile;

    const ZERO_ALG: Instance = Instance::Zero(Zero {});
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
        max_iter: 256,
    };
    #[test]
    fn rejoin() {
        let spec = TileSpec::from(&TD_TILE);
        let split = spec.split(SplitMethod::RowsOfHeight(10), 0);
        let mut tiles: Vec<Tile> = split.unwrap().iter().map(|ts| Tile::new(ts, 0)).collect();
        for t in &mut tiles {
            t.plot();
        }
        let result = Tile::join(&spec, &tiles).unwrap();
        let data = result.result();
        assert_eq!(data.nrows(), spec.height() as usize);
        assert_eq!(data.ncols(), spec.width() as usize);
    }
}
