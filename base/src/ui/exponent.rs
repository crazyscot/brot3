//! Exponent representation for fractal computation
//! Supports integer, real, and complex exponents with range validation

#![cfg(not(target_arch = "spirv"))]

use std::fmt;

use num_traits::AsPrimitive as _;
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, MapAccess, Visitor},
    ser::Serializer,
};

use crate::{NumericType, PushExponent};

/// A fractal exponent that can be an integer, real, or complex number.
///
/// This is the UI-facing representation of the exponent, serializing to a discriminated union:
/// - `{"integer": 2}` for integer exponents
/// - `{"real": 2.5}` for real exponents
/// - `{"complex": {"real": 2.5, "imag": 1.0}}` for complex exponents
///
/// All components are bounded to the range [-20, +20] by default, though bounds are configurable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Exponent {
    /// Integer exponent
    Integer(i32),
    /// Real exponent (when only the real part is needed)
    Real(f32),
    /// Complex exponent with both real and imaginary parts
    Complex {
        /// Real component of the complex exponent
        real: f32,
        /// Imaginary component of the complex exponent
        imag: f32,
    },
}

impl Serialize for Exponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Exponent::Integer(i) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("integer", i)?;
                map.end()
            }
            Exponent::Real(r) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("real", r)?;
                map.end()
            }
            Exponent::Complex { real, imag } => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("real", real)?;
                map.serialize_entry("imag", imag)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Exponent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExponentVisitor;

        impl<'de> Visitor<'de> for ExponentVisitor {
            type Value = Exponent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(
                    "a map containing either 'integer', 'real', or both 'real' and 'imag' keys",
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut integer_value: Option<i32> = None;
                let mut real_value: Option<f32> = None;
                let mut imag_value: Option<f32> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "integer" => {
                            integer_value = Some(map.next_value()?);
                        }
                        "real" => {
                            real_value = Some(map.next_value()?);
                        }
                        "imag" => {
                            imag_value = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                match (integer_value, real_value, imag_value) {
                    (Some(i), None, None) => Ok(Exponent::Integer(i)),
                    (None, Some(r), None) => Ok(Exponent::Real(r)),
                    (None, Some(real), Some(imag)) => Ok(Exponent::Complex { real, imag }),
                    _ => Err(de::Error::custom(
                        "Expected either 'integer', 'real', or both 'real' and 'imag' keys",
                    )),
                }
            }
        }

        deserializer.deserialize_map(ExponentVisitor)
    }
}

impl Exponent {
    /// Default exponent value (integer 2)
    #[must_use]
    pub fn default_integer() -> Self {
        Exponent::Integer(2)
    }

    /// Check if this exponent is within the given bounds (inclusive)
    #[must_use]
    pub fn is_valid(&self, min: i32, max: i32) -> bool {
        match self {
            Exponent::Integer(i) => i >= &min && i <= &max,
            Exponent::Real(r) => *r >= min.as_() && *r <= max.as_(),
            Exponent::Complex { real, imag } => {
                *real >= min.as_() && *real <= max.as_() && *imag >= min.as_() && *imag <= max.as_()
            }
        }
    }

    /// Convert to the GPU-facing `PushExponent` structure
    #[must_use]
    pub fn to_push_exponent(&self) -> PushExponent {
        match self {
            Exponent::Integer(i) => PushExponent {
                typ: NumericType::Integer,
                int: *i,
                real: 0.0,
                imag: 0.0,
            },
            Exponent::Real(r) => PushExponent {
                typ: NumericType::Float,
                int: 0,
                real: *r,
                imag: 0.0,
            },
            Exponent::Complex { real, imag } => PushExponent {
                typ: NumericType::Complex,
                int: 0,
                real: *real,
                imag: *imag,
            },
        }
    }

    /// Convert from a `PushExponent` structure
    #[must_use]
    pub fn from_push_exponent(push: PushExponent) -> Self {
        match push.typ {
            NumericType::Integer => Exponent::Integer(push.int),
            NumericType::Float => Exponent::Real(push.real),
            NumericType::Complex => Exponent::Complex {
                real: push.real,
                imag: push.imag,
            },
        }
    }
}

impl Default for Exponent {
    fn default() -> Self {
        Self::default_integer()
    }
}

impl From<PushExponent> for Exponent {
    fn from(push: PushExponent) -> Self {
        Self::from_push_exponent(push)
    }
}

impl From<Exponent> for PushExponent {
    fn from(exp: Exponent) -> Self {
        exp.to_push_exponent()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_integer_exponent() {
        let exp = Exponent::Integer(2);
        let push = exp.to_push_exponent();
        assert_eq!(push.typ, NumericType::Integer);
        assert_eq!(push.int, 2);
    }

    #[test]
    fn test_real_exponent() {
        let exp = Exponent::Real(2.5);
        let push = exp.to_push_exponent();
        assert_eq!(push.typ, NumericType::Float);
        assert!((push.real - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_complex_exponent() {
        let exp = Exponent::Complex {
            real: 2.5,
            imag: 1.0,
        };
        let push = exp.to_push_exponent();
        assert_eq!(push.typ, NumericType::Complex);
        assert!((push.real - 2.5).abs() < f32::EPSILON);
        assert!((push.imag - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = Exponent::Complex {
            real: 2.5,
            imag: 1.0,
        };
        let push = PushExponent::from(original);
        let reconstructed = Exponent::from(push);
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_validation() {
        let exp = Exponent::Integer(15);
        assert!(exp.is_valid(-20, 20));
        assert!(!exp.is_valid(-20, 10));
        assert!(!exp.is_valid(20, 20));

        let exp_real = Exponent::Real(2.5);
        assert!(exp_real.is_valid(-20, 20));
        assert!(!exp_real.is_valid(3, 20));

        let exp_complex = Exponent::Complex {
            real: 2.5,
            imag: 1.0,
        };
        assert!(exp_complex.is_valid(-20, 20));
        assert!(!exp_complex.is_valid(-20, 0));
    }

    #[test]
    fn test_integer_serialization() {
        let exp = Exponent::Integer(2);
        let json = serde_json::to_string(&exp).unwrap();
        assert_eq!(json, "{\"integer\":2}");
        let deserialized: Exponent = serde_json::from_str(&json).unwrap();
        assert_eq!(exp, deserialized);
    }

    #[test]
    fn test_real_serialization() {
        let exp = Exponent::Real(2.5);
        let json = serde_json::to_string(&exp).unwrap();
        assert_eq!(json, "{\"real\":2.5}");
        let deserialized: Exponent = serde_json::from_str(&json).unwrap();
        assert_eq!(exp, deserialized);
    }

    #[test]
    fn test_complex_serialization() {
        let exp = Exponent::Complex {
            real: 2.5,
            imag: 1.0,
        };
        let json = serde_json::to_string(&exp).unwrap();
        // Order of fields in JSON may vary, so parse and check
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["real"], 2.5);
        assert_eq!(parsed["imag"], 1.0);
        let deserialized: Exponent = serde_json::from_str(&json).unwrap();
        assert_eq!(exp, deserialized);
    }
}
