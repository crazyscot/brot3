// Fractal plotting
// (c) 2024 Ross Younger
use array2d::Array2D;
use num_complex::Complex;
use std::fmt::{self, Display, Formatter};

type Scalar = f64;
const SCALAR_LN_2: Scalar = std::f64::consts::LN_2;

pub type Point = Complex<Scalar>;

#[derive(Debug, Clone)]
pub struct PlotData {
    pub origin: Point,
    pub axes: Point,
}

impl PlotData {
    // TODO allow creation by origin/centre, axes length/pixel size
    pub fn pixel_size(&self, tile: &Tile) -> Point {
        Point {
            re: self.axes.re / tile.width as Scalar,
            im: self.axes.im / tile.height as Scalar,
        }
    }
}

impl Display for PlotData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "@{} axes={}", self.origin, self.axes)
    }
}

#[derive(Clone, Copy, Default)]
pub struct PointData {
    /// Current number of iterations this point has seen
    pub iter: u32,
    /// Original value of this point
    origin: Point,
    /// Current value of this point (may not be valid if result is Some)
    value: Point,
    /// When we are finished with this point, this holds the smooth iteration count
    result: Option<f64>,
}

impl PointData {
    pub fn mark_infinite(&mut self) {
        self.result = Some(Scalar::INFINITY);
    }
    /// The result, if we have it, or the _working result_ (current iteration count) if not.
    pub fn iterations(&self) -> f64 {
        self.result.unwrap_or(self.iter as f64)
    }
    pub fn fmt(&self, f: &mut std::fmt::Formatter<'_>, debug: u8) -> std::fmt::Result {
        match debug {
            0 => write!(f, "{}", self.iterations()),
            1 => write!(f, "[{}, {:?}]", self.origin, self.result),
            _ => write!(
                f,
                "[{}, {}, {}, {:?}]",
                self.origin, self.value, self.iter, self.result
            ),
        }
    }
}

impl fmt::Display for PointData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iterations())
    }
}

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
    /// What have we plotted?
    plot_data: Option<PlotData>,
}

impl Tile {
    pub fn new(height: usize, width: usize, debug: u8) -> Self {
        Self {
            height,
            width,
            debug,
            // Data for this tile. @warning Array2D square bracket syntax is (row,column) i.e. (y,x) !
            point_data: Array2D::filled_with(PointData::default(), height, width),
            max_iter_plotted: 0,
            plot_data: None,
        }
        // TODO should this merge with prepare?
    }

    /// Initialises the data for this tile
    pub fn prepare(&mut self, spec: &PlotData) {
        self.plot_data = Some(spec.clone());
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
                mandelbrot_prepare(point);
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
                    mandelbrot_pixel(point, max_iter);
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
        match &self.plot_data {
            Some(pd) => pd.to_string(),
            None => String::new(),
        }
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

fn mandelbrot_prepare(point: &mut PointData) {
    // Cardioid and period-2 bulb checks
    let c1 = point.origin.re - 0.25;
    let y2 = point.origin.im * point.origin.im;
    let q = c1 * c1 + y2;
    let p1 = point.origin.re + 1.0;
    //println!("prep {}: c1={} y2={} q={} p1={}", point.origin, c1, y2, q, p1);
    if (q * (q + (c1)) <= 0.25 * y2) || (p1 * p1 + y2 <= 0.0625) {
        //println!("INF");
        point.mark_infinite();
    }
}

fn mandelbrot_iterate(point: &mut PointData) {
    point.value = point.value * point.value + point.origin;
    point.iter += 1;
}
fn mandelbrot_pixel(point: &mut PointData, max_iter: u32) {
    for _ in point.iter..max_iter {
        mandelbrot_iterate(point);
        if point.value.norm_sqr() >= 4.0 {
            // TODO make escape radius configurable
            // It escaped
            // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
            /*let temp = point.value;*/
            // A couple of extra iterations helps decrease the size of the error term
            mandelbrot_iterate(point);
            mandelbrot_iterate(point);
            point.result =
                Some((point.iter as Scalar) - point.value.norm().ln().ln() / SCALAR_LN_2);

            /* TODO debug tidyup
            let norm = temp.norm();
            let norm2 = temp.norm_sqr();
            let pushnorm2 = point.value.norm_sqr();
            let log1 = norm2.ln();
            let log2 = log1.ln();
            println!(
                "addr={:?} value={} iter={} norm={} norm2={} pushnorm2={} log1={} log2={} final={}",
                point.origin,
                temp,
                point.iter,
                norm,
                norm2,
                pushnorm2,
                log1,
                log2,
                point.result.unwrap()
            );
            */
            return;
        }
    }
}
