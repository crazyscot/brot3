//! enum definitions for the brot3 shader

use bytemuck::NoUninit;
use const_default::ConstDefault;

macro_rules! enumdef {
    ($attr: meta, $name:ident, $first:ident, $($variant:ident), +) => {
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
        #[$attr]
        pub enum $name {
            #[default]
            $first,
            $($variant,)*
        }
        impl ConstDefault for $name {
            const DEFAULT: Self = Self::$first;
        }
    };
}

enumdef!(
    doc = "Fractal algorithm selection",
    Algorithm,
    Mandelbrot,
    Mandeldrop,
    Mandelbar,
    BurningShip,
    Celtic,
    Variant,
    BirdOfPrey
);

enumdef!(
    doc = "Colouring algorithm selection",
    Colourer,
    LogRainbow,
    SqrtRainbow,
    WhiteFade,
    BlackFade,
    OneLoneCoder,
    LchGradient,
    Monochrome
);

enumdef!(
    doc = "Colouring style",
    ColourStyle,
    Continuous,
    Discrete,
    None
);

enumdef!(
    doc = "Style modifier",
    Modifier,
    Standard,
    Filaments1,
    Filaments2,
    FinalAngle,
    FinalRadius
);

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
