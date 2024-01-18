// (c) 2024 Ross Younger

use super::{PlotSpec, Point, PointData};
use array2d::Array2D;
use std::fmt;

/// A section of a fractal plot
pub struct Tile {
    /// Height in pixels
    pub height: usize,
    /// Width in pixels
    pub width: usize,
    /// Debug output level
    debug: u8,
    /// Working data. Address as [(x,y)] aka (re,im).
    point_data: Array2D<PointData>,
    /// Max iterations we plotted to
    pub max_iter_plotted: u32,
    /// Specification for this plot
    plot_data: PlotSpec,
}

impl Tile {
    pub fn new(data: &PlotSpec, debug: u8) -> Self {
        Self {
            height: data.height as usize, // TODO factor out
            width: data.width as usize,   // TODO factor out
            debug,
            // Data for this tile. @warning Array2D square bracket syntax is (row,column) i.e. (y,x) !
            point_data: Array2D::filled_with(
                PointData::default(),
                data.height as usize,
                data.width as usize,
            ),
            max_iter_plotted: 0,
            plot_data: data.clone(),
        }
        // TODO should this merge with prepare?
    }

    /// Initialises the data for this tile
    pub fn prepare(&mut self, spec: &PlotSpec) {
        let step = spec.pixel_size(self);
        // TRAP: Plot origin != first pixel origin !
        // The plotted point of each pixel should be the CENTRE of the region, i.e. offset by half a pixel from plot origin.
        let origin_pixel = spec.origin + 0.5 * step;

        let mut imag = origin_pixel.im;
        for y in 0..self.height {
            let mut real = origin_pixel.re;
            for x in 0..self.width {
                let point = &mut self.point_data[(y, x)];
                point.origin = Point { re: real, im: imag };
                // The first iteration is easy
                point.value = point.origin;
                point.iter = 1;
                crate::fractal::mandelbrot_prepare(point);
                real += step.re;
            }
            imag += step.im;
        }
        // TODO: live_pixel count
    }

    pub fn plot(&mut self, max_iter: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let point = &mut self.point_data[(y, x)];
                if point.result.is_none() {
                    crate::fractal::mandelbrot_pixel(point, max_iter);
                }
            }
        }
        self.max_iter_plotted = max_iter;
        // TODO live pixel count
    }

    pub fn result(&self) -> &Array2D<PointData> {
        &self.point_data
    }

    pub fn info_string(&self) -> String {
        self.plot_data.to_string()
    }
}

/// CSV format output
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            self.point_data[(y, 0)].fmt(f, self.debug)?;
            for x in 1..self.width {
                write!(f, ",")?;
                self.point_data[(y, x)].fmt(f, self.debug)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
