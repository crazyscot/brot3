// (c) 2024 Ross Younger

use super::{Algorithm, Point, PointData, TileSpec};

use num_complex::ComplexFloat;
use std::fmt;

/// A section of a fractal plot
#[derive(Debug, Clone, Default)]
pub struct Tile {
    /// Debug output level
    debug: u8,
    /// Working data. Address as [(row,column)] aka (y,x).
    /// <div class="warning">CAUTION: This array is TOP LEFT oriented. The first row is the top row, not the Origin row.</div>
    point_data: Vec<PointData>,
    /// Max iterations we plotted to
    pub max_iter_plotted: u32,
    /// Specification of this plot
    pub spec: TileSpec,
    /// If present, this tile is a strip of a larger plot; this is its Y offset in pixels, relative to the TOP LEFT of the plot.
    pub y_offset: Option<u32>,
}

impl Tile {
    /// Standard constructor. Also initialises the data for this tile.
    #[must_use]
    pub fn new(spec: &TileSpec, debug: u8) -> Self {
        let mut new1 = Tile::new_internal(spec, debug);
        new1.prepare(debug);
        new1
    }

    /// Internal constructor
    fn new_internal(spec: &TileSpec, debug: u8) -> Self {
        Self {
            debug,
            // Data for this tile.
            point_data: Vec::with_capacity((spec.height() * spec.width()) as usize),
            max_iter_plotted: 0,
            spec: spec.clone(),
            y_offset: spec.y_offset(),
        }
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

        for y in 0..self.spec.height() {
            let im = f64::from(y) * -step.im; // note '-' as we are stepping down the imaginary dimension

            // For each pixel in width...
            let points = (0..self.spec.width())
                // compute real coordinate
                .map(|x| f64::from(x) * step.re)
                // compute pixel
                .map(|re| origin_pixel + Point { re, im })
                // assemble and prepare point data
                .map(|or| {
                    let mut pd = PointData::new(or);
                    self.spec.algorithm().prepare(&mut pd);
                    pd
                });
            self.point_data.extend(points);
        }
    }

    /// Runs the fractal iteration for all points in this tile
    pub fn plot(&mut self) {
        let max_iter = self.spec.max_iter_requested();
        for p in &mut self.point_data {
            if p.result.is_none() {
                self.spec.algorithm().pixel(p, max_iter);
            }
        }
        self.max_iter_plotted = max_iter;
        // TODO live pixel count
    }

    /// Result accessor
    /// <div class="warning">CAUTION: This array is TOP LEFT oriented. The first row is the top row, not the Origin (bottom) row!</div>
    #[must_use]
    pub fn result(&self) -> &Vec<PointData> {
        &self.point_data
    }
}

/// CSV format output
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.spec.width() as usize;
        for y in 0..self.spec.height() as usize {
            // SOMEDAY: Rewrite this as a map operation. But it may not be worth it.
            self.point_data[y * width].fmt(f, self.debug)?;
            for x in 1..width {
                write!(f, ",")?;
                self.point_data[y * width + x].fmt(f, self.debug)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
