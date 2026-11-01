//! GPU-friendly representation of the exponent
// (c) 2025-26 Ross Younger

#[cfg(not(spirv))]
use std::str::FromStr;

use bytemuck::NoUninit;
#[cfg(not(spirv))]
use easy_cast::CastFloat;
use my_wgsl::WgslCompatible;
#[cfg(not(spirv))]
use serde::{Deserialize, Serialize};

use super::enums::NumericType;
#[cfg(not(spirv))]
use crate::Exponent;
use crate::enums::WgslEnum;

/// The exponent for the fractal, in a GPU-friendly format. It can be an integer or a real number.
#[derive(Copy, Clone, Debug, PartialEq, NoUninit, WgslCompatible)]
#[repr(C)]
#[cfg_attr(not(spirv), derive(Serialize, Deserialize))]
#[cfg_attr(not(spirv), serde(into = "Exponent", from = "Exponent"))]
pub struct PushExponent {
    /// The type of the exponent, which determines which of the other fields are used.
    pub typ: WgslEnum<NumericType>,
    /// Only used when `typ` is Integer
    pub int: i32,
    /// Used when `typ` is Float
    pub real: f32,
}

impl Default for PushExponent {
    fn default() -> Self {
        Self {
            typ: NumericType::Integer.into(),
            int: 2,
            real: 0.,
        }
    }
}

#[cfg(not(spirv))]
impl TryFrom<i32> for PushExponent {
    type Error = String;

    fn try_from(i: i32) -> Result<Self, Self::Error> {
        if !(Self::MIN_INT..=Self::MAX_INT).contains(&i) {
            return Err(format!("Exponent must be between 2 and 20, got {i}"));
        }
        Ok(Self {
            typ: NumericType::Integer.into(),
            int: i,
            ..Default::default()
        })
    }
}

#[cfg(not(spirv))]
impl TryFrom<f32> for PushExponent {
    type Error = String;

    fn try_from(f: f32) -> Result<Self, Self::Error> {
        if !(Self::MIN..=Self::MAX).contains(&f) {
            return Err(format!("Exponent must be between 2 and 20, got {f}"));
        }
        Ok(match f.fract() {
            0. => Self {
                typ: NumericType::Integer.into(),
                int: f.cast_floor(),
                ..Default::default()
            },
            _ => Self {
                typ: NumericType::Float.into(),
                real: f,
                ..Default::default()
            },
        })
    }
}

#[cfg(not(spirv))]
impl FromStr for PushExponent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f32>()
            .map_err(|e| e.to_string())
            .and_then(Self::try_from)
    }
}

#[cfg(not(spirv))]
impl PushExponent {
    /// Maximum exponent value
    pub const MAX: f32 = 20.0;
    /// Maximum exponent as integer
    #[allow(clippy::cast_possible_truncation)]
    pub const MAX_INT: i32 = Self::MAX as i32;
    /// Minimum exponent value
    pub const MIN: f32 = 2.0;
    /// Minimum exponent as integer
    #[allow(clippy::cast_possible_truncation)]
    pub const MIN_INT: i32 = Self::MIN as i32;
}

impl PushExponent {
    /// Check if the exponent is 2 (or 2+0i), which is a common special case.
    #[must_use]
    #[allow(clippy::float_cmp)]
    pub fn is_two(&self) -> bool {
        match self.typ {
            WgslEnum(NumericType::Integer) => self.int == 2,
            WgslEnum(NumericType::Float) => self.real == 2.0,
        }
    }

    /// Get the step size for UI adjustments based on the type of exponent.
    #[must_use]
    pub fn ui_step(&self) -> f32 {
        if self.typ == WgslEnum(NumericType::Integer) {
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
    use my_wgsl::WgslCompatible;

    use super::{NumericType, PushExponent};
    use crate::enums::WgslEnum;
    #[test]
    fn construct_exponent() {
        let pf = PushExponent::try_from(11.2).unwrap();
        assert_matches!(
            pf,
            PushExponent {
                typ: WgslEnum(NumericType::Float),
                real: 11.2,
                ..
            }
        );
    }

    #[test]
    fn test_wgsl_struct() {
        println!("{}", PushExponent::WGSL_DEFINE);
        assert!(PushExponent::WGSL_DEFINE.contains("struct PushExponent"));
        assert!(PushExponent::WGSL_DEFINE.contains("typ:u32"));
        assert!(PushExponent::WGSL_DEFINE.contains("int:i32"));
        assert!(PushExponent::WGSL_DEFINE.contains("real:f32"));
    }
}
