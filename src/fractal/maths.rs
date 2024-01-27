// Fractal maths type selection
// (c) 2024 Ross Younger

use num_complex::Complex;

/// One dimension of a fractal
pub type Scalar = f64;
/// Complex type for fractals
pub type Point = Complex<Scalar>;

/// Natural logarithm of 2, as a Scalar
pub const LN_2: f64 = std::f64::consts::LN_2;

/// Natural logarithm of 3, as a Scalar
#[inline]
#[must_use]
pub fn ln_3() -> f64 {
    const THREE: f64 = 3.0;
    THREE.ln()
}

// For INFINITY: use Scalar::INFINITY
// For NAN: use Scalar::NAN
