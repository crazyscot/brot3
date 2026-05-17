//! GPU-friendly representation of the exponent
// (c) 2025-26 Ross Younger

use bytemuck::NoUninit;
#[cfg(not(spirv))]
use serde::{Deserialize, Serialize};

#[cfg(not(spirv))]
use crate::ui::Exponent;

/// The exponent type for the fractal
#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[repr(u32)]
#[allow(missing_docs)]
#[cfg_attr(not(spirv), derive(Serialize, Deserialize))]
pub enum NumericType {
    #[default]
    Integer,
    Float,
}

/// The exponent for the fractal, in a GPU-friendly format. It can be an integer or a real number.
#[derive(Copy, Clone, Debug, PartialEq, NoUninit)]
#[repr(C)]
#[cfg_attr(not(spirv), derive(Serialize, Deserialize))]
#[cfg_attr(not(spirv), serde(into = "Exponent", from = "Exponent"))]
pub struct PushExponent {
    /// The type of the exponent, which determines which of the other fields are used.
    pub typ: NumericType,
    /// Only used when `typ` is Integer
    pub int: i32,
    /// Used when `typ` is Float
    pub real: f32,
}

impl Default for PushExponent {
    fn default() -> Self {
        Self {
            typ: NumericType::Integer,
            int: 2,
            real: 0.,
        }
    }
}

impl From<i32> for PushExponent {
    fn from(i: i32) -> Self {
        Self {
            typ: NumericType::Integer,
            int: i,
            ..Default::default()
        }
    }
}

impl From<f32> for PushExponent {
    fn from(f: f32) -> Self {
        Self {
            typ: NumericType::Float,
            real: f,
            ..Default::default()
        }
    }
}

impl PushExponent {
    /// Check if the exponent is 2 (or 2+0i), which is a common special case.
    #[must_use]
    #[allow(clippy::float_cmp)]
    pub fn is_two(&self) -> bool {
        match self.typ {
            NumericType::Integer => self.int == 2,
            NumericType::Float => self.real == 2.0,
        }
    }

    /// Get the step size for UI adjustments based on the type of exponent.
    #[must_use]
    pub fn ui_step(&self) -> f32 {
        if self.typ == NumericType::Integer {
            1.
        } else {
            0.1
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use assert_matches::assert_matches;

    use super::{NumericType, PushExponent};
    #[test]
    fn construct_exponent() {
        let pf = PushExponent::from(31.2);
        assert_matches!(
            pf,
            PushExponent {
                typ: NumericType::Float,
                real: 31.2,
                ..
            }
        );
    }
}
