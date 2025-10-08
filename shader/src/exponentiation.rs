//! Exponentation strategies for fractals, as a trait to allow monomorphisation and inlining

use super::Complex;

pub trait Exponentiator: Copy + Clone {
    fn apply_to(self, z: Complex) -> Complex;
}
#[derive(Copy, Clone, Debug)]
pub struct Exp2;
#[derive(Copy, Clone, Debug)]
pub struct ExpIntN(pub i32);
#[derive(Copy, Clone, Debug)]
pub struct ExpFloat(pub f32);

// special case for exponent 2, which is the most common and can be optimised to a simple complex multiplication
impl Exponentiator for Exp2 {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z * z
    }
}

impl Exponentiator for ExpIntN {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z.powi(self.0).to_rectangular()
    }
}

impl Exponentiator for ExpFloat {
    #[inline(always)]
    fn apply_to(self, z: Complex) -> Complex {
        z.powf(self.0).to_rectangular()
    }
}
