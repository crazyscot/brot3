// Fractal maths type selection and helpers
// (c) 2024 Ross Younger

use num_complex::Complex;

/// One dimension of a fractal
pub type Scalar = f64;
/// Complex type for fractals
pub type Point = Complex<Scalar>;

/// Euler's number aka e or exp(1)
pub const E: f64 = std::f64::consts::E;

// For INFINITY: use Scalar::INFINITY
// For NAN: use Scalar::NAN

// /////////////////////////////////////////////////////////////////////
// f32 helpers

/// Natural logarithm of 3
#[inline]
#[must_use]
pub fn ln_3_f32() -> f32 {
    const THREE: f32 = 3.0;
    THREE.ln()
}
