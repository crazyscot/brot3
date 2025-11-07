//! enum definitions for the brot3 shader

use bytemuck::NoUninit;
use const_default::ConstDefault;

macro_rules! incrementable {
    ($enum:ty) => {
        #[cfg(not(target_arch = "spirv"))]
        impl core::ops::Add<i32> for $enum {
            type Output = Self;

            fn add(self, delta: i32) -> Self::Output {
                use num_traits::FromPrimitive as _;
                use num_traits::ToPrimitive as _;
                use strum::VariantArray as _;
                let n = Self::VARIANTS.len() as i32;
                let mut i = self.to_i32().unwrap_or_default() + delta;
                i = i.rem_euclid(n);
                Self::from_i32(i).unwrap()
            }
        }
        #[cfg(not(target_arch = "spirv"))]
        impl core::ops::AddAssign<i32> for $enum {
            fn add_assign(&mut self, delta: i32) {
                let t = *self + delta;
                *self = t;
            }
        }
    };
}
incrementable!(Colourer);
incrementable!(Algorithm);

/// Fractal algorithm selection
#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
#[non_exhaustive]
pub enum Algorithm {
    #[default]
    Mandelbrot,
    Mandeldrop,
    Mandelbar,
    BurningShip,
    Celtic,
    Variant,
    BirdOfPrey,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
#[non_exhaustive]
/// Colouring algorithm selection
pub enum Colourer {
    #[default]
    LogRainbow,
    SqrtRainbow,
    WhiteFade,
    BlackFade,
    OneLoneCoder,
    LchGradient,
    Monochrome,
}

impl ConstDefault for Colourer {
    const DEFAULT: Self = Self::LogRainbow;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
#[cfg_attr(
    not(target_arch = "spirv"),
    derive(
        clap::ValueEnum,
        strum::EnumIter,
        strum::IntoStaticStr,
        strum::VariantArray,
        num_derive::FromPrimitive,
        num_derive::ToPrimitive,
    )
)]
#[repr(u32)]
#[non_exhaustive]
/// Colour style selection
pub enum ColourStyle {
    #[default]
    ContinuousDwell,
    EscapeTime,
}

impl ConstDefault for ColourStyle {
    const DEFAULT: Self = Self::ContinuousDwell;
}

#[cfg(all(test, not(target_arch = "spirv")))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use pretty_assertions::assert_eq;
    #[test]
    fn increment() {
        use super::Colourer;
        let mut c = Colourer::LogRainbow;
        c += 1;
        assert_eq!(c, Colourer::SqrtRainbow);
    }
}
