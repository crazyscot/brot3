//! enum definitions for the brot3 shader
//!
//! (c) 2025 Ross Younger

use core::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator};
use std::marker::PhantomData;

use bytemuck::NoUninit;
use const_default::ConstDefault;
use my_wgsl::{WgslCompatible, WgslKind};
use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

/// A zero-cost wrapper forcing Rust type-safety on WGSL structs using these enums
#[derive(Debug, Copy, Clone, PartialEq, Eq, NoUninit, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct WgslEnum<E>(pub E);

// Manually implement WgslCompatible to trick the macro into seeing a standard u32 layout.
impl<E> WgslCompatible for WgslEnum<E> {
    const WGSL_ALIGN: usize = 4;
    const WGSL_ARRAY_LENGTH: usize = 0;
    const WGSL_DEFINE: &'static str = "";
    const WGSL_IDENT: &'static str = "u32";
    const WGSL_KIND: WgslKind = WgslKind::Primitive;
    const WGSL_SIZE: usize = 4;
}

impl<E> Default for WgslEnum<E>
where
    E: Default,
{
    fn default() -> Self {
        WgslEnum(E::default())
    }
}

#[derive(Debug, Clone)]
pub struct EnumIter<E> {
    idx: usize,
    back_idx: usize,
    marker: PhantomData<E>,
}

impl<E> Iterator for EnumIter<E>
where
    E: strum::VariantArray + Copy,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.back_idx {
            let item = <E as strum::VariantArray>::VARIANTS[self.idx];
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<E> DoubleEndedIterator for EnumIter<E>
where
    E: strum::VariantArray + Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_idx > self.idx {
            self.back_idx -= 1;
            Some(<E as strum::VariantArray>::VARIANTS[self.back_idx])
        } else {
            None
        }
    }
}
impl<E> FusedIterator for EnumIter<E> where E: strum::VariantArray + Copy {}
impl<E> ExactSizeIterator for EnumIter<E>
where
    E: strum::VariantArray + Copy,
{
    fn len(&self) -> usize {
        self.back_idx - self.idx
    }
}

/// Helper trait for enums allowing cycling through the variants.
/// This only works where the number of enums can be converted between `usize` and `isize` without
/// overflow.
pub trait EnumCycle: strum::VariantArray + Copy + PartialEq + 'static {
    /// Adds the given delta to the enum, cycling around the end if necessary
    #[must_use]
    fn step(self, delta: isize) -> Self {
        let all = <Self as strum::VariantArray>::VARIANTS;
        let i: isize = all.iter().position(|v| *v == self).unwrap_or(0).as_();
        let len = all.len().as_();
        all[(i + delta).rem_euclid(len) as usize]
    }
    /// Steps to the next enum variant, cycling around to the start if necessary.
    #[must_use]
    fn next(self) -> Self {
        self.step(1)
    }
    /// Steps to the previous enum variant, cycling around to the end if necessary.
    #[must_use]
    fn prev(self) -> Self {
        self.step(-1)
    }
}

macro_rules! enumdef {
    (
        $(#[$attr:meta])*
        $vis:vis
        $ident:ident
        $(#[$first_attr:meta])*
        $first:ident = $first_val:expr,
        $(
            $(#[$var_attr:meta])*
            $variant:ident = $val:expr
        ), +
    ) => {
        #[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
        #[cfg_attr(
            not(spirv),
            derive(
                serde::Serialize,
                serde::Deserialize,
                strum::EnumMessage,
                strum::FromRepr,
                strum::IntoStaticStr,
                strum::VariantArray,
            )
        )]
        #[repr(u32)]
        $(#[$attr])*
        $vis enum $ident {
            #[default]
            $(#[$first_attr])*
            #[allow(missing_docs)]
            $first,
            $(
                $(#[$var_attr])*
                #[allow(missing_docs)]
                $variant = $val,
            )+
        }
        impl ConstDefault for $ident {
            const DEFAULT: Self = Self::$first;
        }
        impl std::str::FromStr for $ident {
            type Err = strum::ParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <Self as strum::VariantArray>::VARIANTS.iter().copied()
                    .find(|v| <&'static str>::from(v) == s)
                    .ok_or(strum::ParseError::VariantNotFound)
            }
        }
        impl clap::ValueEnum for $ident {
            fn value_variants<'a>() -> &'a [Self] {
                <Self as strum::VariantArray>::VARIANTS
            }
            fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
                Some(clap::builder::PossibleValue::new(<&'static str>::from(self)))
            }
        }
        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", <&'static str>::from(self))
            }
        }

        impl strum::IntoEnumIterator for $ident {
            type Iterator = EnumIter<$ident>;

            fn iter() -> EnumIter<Self> {
                EnumIter {
                    idx: 0,
                    back_idx: <Self as strum::VariantArray>::VARIANTS.len(),
                    marker: PhantomData,
                }
            }
        }

        impl EnumCycle for $ident {}
        impl core::ops::Add<isize> for $ident {
            type Output = Self;
            fn add(self, delta: isize) -> Self {
                self.step(delta)
            }
        }
        impl core::ops::AddAssign<isize> for $ident {
            fn add_assign(&mut self, delta: isize) {
                *self = self.step(delta);
            }
        }

        // WGSL definition and helper type
        impl $ident {
            #[allow(missing_docs)]
            $vis const WGSL_DEFINE: &'static str = concat!(
                "const ",
                stringify!($ident),
                "_",
                stringify!($first),
                ": u32 = ",
                stringify!($first_val),
                ";\n",
                $(
                    "const ",
                    stringify!($ident),
                    "_",
                    stringify!($variant),
                    ": u32 = ",
                    stringify!($val),
                    ";\n"
                ),+
            );
        }
        impl From<$ident> for WgslEnum<$ident> {
            fn from(value: $ident) -> Self {
                WgslEnum(value)
            }
        }
        impl std::ops::Deref for WgslEnum<$ident> {
            type Target = $ident;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for WgslEnum<$ident> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

enumdef!(
    /// Fractal algorithm selection
    pub Algorithm
    /// The original Mandelbrot set, `z := z^2+c`
    Mandelbrot = 0,
    /// The inverted set, `z:=z^2+c` using `1/z0` as the value of `c`
    Mandeldrop = 1,
    /// Mandelbar aka Tricorn: `z:=(z*)^2+c`
    Mandelbar = 2,
    /// `z:=(|Re(z)|+i|Im(z)|)^2+c`
    BurningShip = 3,
    /// Generalised Celtic `z:= (|Re(z^2)| + i.Im(z^2) + c)`
    Celtic = 4,
     /// `z:=z^2+c with Re(z):=|Re(z)|` on odd iterations
    Variant = 5,
    /// `z:=(Re(z)+i|Im(z)|)^2+c`
    BirdOfPrey = 6
);

enumdef!(
    /// Colouring algorithm selection
    pub Colourer
    /// Fades in from black, then a bright colorful gradient.
    ///
    /// Based on Tony Finch's "Black Fade" colourer
    /// <https://dotat.at/prog/mandelbrot/>
    BlackFade = 0,
    /// A cool appearance with shades of blue.
    ///
    /// Inspired by the `iceblue` theme by David Bau <https://github.com/davidbau/mandelbrot/blob/main/index.html>
    IcyBlue = 1,
    /// Gradient from white, through pulsing deep hues
    ///
    /// Based on Richard Kettlewell's "mandy". <http://www.greenend.org.uk/rjk/mandy/>
    Mandy = 2,
    /// Colourless, with a gradient from white to black to white.
    Monochrome2 = 3,
    /// Bright colours with high contrast and saturation
    Neon2 = 4,
    /// Slightly muted gradient
    ///
    /// Based on the colouring algorithm by `OneLoneCoder.com`
    /// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
    OneLoneCoder = 5,
    /// Fades in from white, then a bright colorful gradient.
    ///
    /// Based on Tony Finch's "White Fade" colourer
    /// <https://dotat.at/prog/mandelbrot/>
    WhiteFade = 6,
    /// No colouration; all pixels are white. This is useful in conjunction with a non-standard brightness modifier.
    None = 7
);

enumdef!(
    /// Colouring style
    pub ColourStyle
    /// A smooth gradient that computes a fractional escape count, based on the double logarithm of the escape time.
    ///
    /// See <http://linas.org/art-gallery/escape/escape.html>
    Continuous = 0,
    /// Each point is coloured according to the number of iterations before escape, rounded down.
    Discrete = 1
);

enumdef!(
    /// Style modifier for brightness and saturation
    pub Modifier
    /// No modification
    Standard = 0,
    /// Modifies the pixel using proximity to the edge of the fractal
    Filaments = 1,
    /// Modifies the pixel using the final angle of the point
    FinalAngle = 2,
    /// Modifies the pixel using the final radius of the point
    FinalRadius = 3
);

enumdef!(
    /// Rendering mode for saving images
    pub RenderMode
    /// Use the GPU to render the image. This is usually the fastest option.
    Gpu = 0,
    /// Use the CPU to render the image, using multiple threads.
    CpuParallel = 1,
    /// Use the CPU to render the image, using a single thread. This is slow, but may be useful for benchmarking or debugging.
    Cpu = 2
);

enumdef!(
    /// The exponent type for the fractal
    pub NumericType
    Integer = 0,
    Float = 1
);

enumdef!(
    /// How close is this point to the edge of the fractal?
    pub BoundaryClass
    Indeterminate = 0,
    Inside = 1,
    VeryClose = 2,
    Close = 3,
    NotClose = 4,
    DontCare = 5
);

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Algorithm;
    use crate::EnumCycle as _;

    #[test]
    fn string_conversion() {
        let a = Algorithm::BirdOfPrey;
        let s = a.to_string();
        let s2: &'static str = a.into();
        let b = s.parse::<Algorithm>().unwrap();
        assert_eq!(a, b);
        assert_eq!(s, s2);
        assert_eq!(s, "BirdOfPrey");
    }

    #[test]
    fn clap_parse() {
        use clap::ValueEnum;
        let a = Algorithm::BirdOfPrey;
        let s = a.to_possible_value().unwrap();
        let s = s.get_name();
        let b = s.parse::<Algorithm>().unwrap();
        assert_eq!(a, b);
        assert_eq!(s, "BirdOfPrey");
    }

    #[test]
    fn increment_cycles() {
        use super::Colourer;
        let c0 = Colourer::from_repr(0).unwrap();

        let mut cc = c0;
        cc += 1;
        assert_eq!(cc, c0.next());
        cc += -2;
        cc += 1;
        assert_eq!(cc, c0);

        let c = Colourer::IcyBlue;
        assert_eq!(c.next(), Colourer::Mandy);
        assert_eq!(Colourer::Mandy.prev(), c);
    }

    #[test]
    fn test_wgsl_enums() {
        println!("{}", Algorithm::WGSL_DEFINE);
        assert!(Algorithm::WGSL_DEFINE.contains("Algorithm_Mandelbrot"));
        assert!(Algorithm::WGSL_DEFINE.contains("Algorithm_BirdOfPrey"));
    }
}
