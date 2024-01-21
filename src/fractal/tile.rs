// (c) 2024 Ross Younger

use super::{Algorithm, FractalInstance, Point, PointData, TileSpec};
use array2d::Array2D;
use std::fmt;

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
}

impl Tile {
    /// Standard constructor. Also initialises the data for this tile.
    #[must_use]
    pub fn new(spec: &TileSpec, debug: u8) -> Self {
        let mut new1 = Self {
            debug,
            // Data for this tile. @warning Array2D square bracket syntax is (row,column) i.e. (y,x) !
            point_data: Array2D::filled_with(
                PointData::default(),
                spec.height as usize,
                spec.width as usize,
            ),
            max_iter_plotted: 0,
            spec: *spec,
            algorithm: spec.algorithm,
        };
        new1.prepare();
        new1
    }

    /// Initialises the data for this tile
    fn prepare(&mut self) {
        let step = self.spec.pixel_size();
        // TRAP: Plot origin != first pixel origin !
        // The plotted point of each pixel should be the CENTRE of the region, i.e. offset by half a pixel from plot origin.
        let origin_pixel = self.spec.origin + 0.5 * step;

        let mut imag = origin_pixel.im;
        for y in 0..self.spec.height as usize {
            let mut real = origin_pixel.re;
            for x in 0..self.spec.width as usize {
                let real_y = self.spec.height as usize - y - 1;
                // curveball: origin is bottom left of the plot, but we want to output the top row first.
                let point = &mut self.point_data[(real_y, x)];
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
        for y in 0..self.spec.height as usize {
            for x in 0..self.spec.width as usize {
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

    /// Info string quasi-accessor
    #[must_use]
    pub fn info_string(&self) -> String {
        self.spec.to_string()
    }
}

/// CSV format output
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.spec.height as usize {
            self.point_data[(y, 0)].fmt(f, self.debug)?;
            for x in 1..self.spec.width as usize {
                write!(f, ",")?;
                self.point_data[(y, x)].fmt(f, self.debug)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
