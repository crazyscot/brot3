//! Floating-point utilities
//!
//! (c) 2026 Ross Younger

pub(crate) trait FloatIsNear {
    #[allow(clippy::wrong_self_convention)]
    fn is_near(self, other: Self) -> bool;
}

impl FloatIsNear for f64 {
    fn is_near(self, other: Self) -> bool {
        float_eq::float_eq!(self, other, ulps <= 4, abs <= 1e-10)
    }
}

impl FloatIsNear for f32 {
    fn is_near(self, other: Self) -> bool {
        float_eq::float_eq!(self, other, ulps <= 4, abs <= 1e-5)
    }
}
