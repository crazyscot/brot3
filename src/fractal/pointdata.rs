// (c) 2024 Ross Younger

use std::fmt;

use crate::fractal::{Point, Scalar};

/// Everything we know about a plotted point
#[derive(Clone, Copy, Default)]
pub struct PointData {
    /// Current number of iterations this point has seen
    pub iter: u32,
    /// Original value of this point
    pub origin: Point,
    /// Current value of this point (may not be valid if result is Some)
    pub value: Point,
    /// When we are finished with this point, this holds the smooth iteration count
    pub result: Option<f64>,
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
