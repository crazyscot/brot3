// Fractal plotting
// (c) 2024 Ross Younger

mod plotspec;
mod pointdata;
mod tile;
pub mod userplotspec;

use num_complex::Complex;

pub use plotspec::PlotSpec;
pub use pointdata::PointData;
pub use tile::Tile;
pub use userplotspec::UserPlotSpec;

pub type Scalar = f64;
const SCALAR_LN_2: Scalar = std::f64::consts::LN_2;

pub type Point = Complex<Scalar>;

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
