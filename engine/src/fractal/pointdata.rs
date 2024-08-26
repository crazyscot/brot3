// (c) 2024 Ross Younger

use std::fmt;

use super::Point;

/// Everything we know about a plotted point
#[derive(Clone, Copy, Default, Debug)]
pub struct PointData {
    /// Current number of iterations this point has seen
    pub iter: u32,
    /// Original value of this point
    pub origin: Point,
    /// Current value of this point (may not be valid if result is Some)
    pub value: Point,
    /// When we are finished with this point, this holds the smooth iteration count
    pub result: Option<f32>,
}

impl PointData {
    /// Standard constructor. This knows nothing about fractal-specific optimisations.
    #[must_use]
    pub fn new(origin: Point) -> Self {
        PointData {
            iter: 0,
            origin,
            value: 0.0.into(),
            result: None,
        }
    }

    /// Set this point as infinite
    pub fn mark_infinite(&mut self) {
        self.result = Some(f32::INFINITY);
    }
    /// The result, if we have it, or the _working result_ (current iteration count) if not.
    #[must_use]
    pub fn iterations(&self) -> f32 {
        #![allow(clippy::cast_precision_loss)]
        self.result.unwrap_or(self.iter as f32)
    }
    /// Standard output format
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
