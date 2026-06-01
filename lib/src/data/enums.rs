//! enum definitions for the brot3 shader
//!
//! (c) 2025 Ross Younger

use bytemuck::NoUninit;
use const_default::ConstDefault;

macro_rules! enumdef {
    (
        $(#[$attr:meta])*
        $ident:ident
        $(#[$first_attr:meta])*
        $first:ident,
        $(
            $(#[$var_attr:meta])*
            $variant:ident
        ), +
    ) => {
        #[derive(Copy, Clone, Debug, Default, PartialEq, NoUninit)]
        #[cfg_attr(
            not(spirv),
            derive(
                clap::ValueEnum,
                serde::Serialize,
                serde::Deserialize,
                strum::Display,
                strum::EnumIter,
                strum::EnumMessage,
                strum::IntoStaticStr,
                strum::VariantArray,
                num_derive::FromPrimitive,
                num_derive::ToPrimitive,
            )
        )]
        #[repr(u32)]
        $(#[$attr])*
        pub enum $ident {
            #[default]
            $(#[$first_attr])*
            $first,
            $(
                $(#[$var_attr])*
                $variant,
            )+
        }
        impl ConstDefault for $ident {
            const DEFAULT: Self = Self::$first;
        }
    };
}

enumdef!(
    /// Fractal algorithm selection
    Algorithm
    /// The original Mandelbrot set, `z := z^2+c`
    Mandelbrot,
    /// The inverted set, `z:=z^2+c` using `1/z0` as the value of `c`
    Mandeldrop,
    /// Mandelbar aka Tricorn: `z:=(z*)^2+c`
    Mandelbar,
    /// `z:=(|Re(z)|+i|Im(z)|)^2+c`
    BurningShip,
    /// Generalised Celtic `z:= (|Re(z^2)| + i.Im(z^2) + c)`
    Celtic,
     /// `z:=z^2+c with Re(z):=|Re(z)|` on odd iterations
    Variant,
    /// `z:=(Re(z)+i|Im(z)|)^2+c`
    BirdOfPrey
);

enumdef!(
    /// Colouring algorithm selection
    Colourer
    /// Fades in from black, then a bright colorful gradient.
    ///
    /// Based on Tony Finch's "Black Fade" colourer
    /// <https://dotat.at/prog/mandelbrot/>
    BlackFade,
    /// A cool appearance with shades of blue.
    ///
    /// Inspired by the `iceblue` theme by David Bau <https://github.com/davidbau/mandelbrot/blob/main/index.html>
    IcyBlue,
    /// Gradient from white, through pulsing deep hues
    ///
    /// Based on Richard Kettlewell's "mandy". <http://www.greenend.org.uk/rjk/mandy/>
    Mandy,
    /// Colourless, with a gradient from white to black to white.
    Monochrome2,
    /// Bright colours with high contrast and saturation
    Neon2,
    /// Slightly muted gradient
    ///
    /// Based on the colouring algorithm by `OneLoneCoder.com`
    /// <https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp>
    OneLoneCoder,
    /// Fades in from white, then a bright colorful gradient.
    ///
    /// Based on Tony Finch's "White Fade" colourer
    /// <https://dotat.at/prog/mandelbrot/>
    WhiteFade,
    /// No colouration; all pixels are white. This is useful in conjunction with a non-standard brightness modifier.
    None
);

enumdef!(
    /// Colouring style
    ColourStyle
    /// A smooth gradient that computes a fractional escape count, based on the double logarithm of the escape time.
    ///
    /// See <http://linas.org/art-gallery/escape/escape.html>
    Continuous,
    /// Each point is coloured according to the number of iterations before escape, rounded down.
    Discrete
);

enumdef!(
    /// Style modifier for brightness and saturation
    Modifier
    /// No modification
    Standard,
    /// Modifies the pixel using proximity to the edge of the fractal
    Filaments,
    /// Modifies the pixel using the final angle of the point
    FinalAngle,
    /// Modifies the pixel using the final radius of the point
    FinalRadius
);

enumdef!(
    /// Rendering mode for saving images
    RenderMode
    /// Use the GPU to render the image. This is usually the fastest option.
    Gpu,
    /// Use the CPU to render the image, using multiple threads.
    CpuParallel,
    /// Use the CPU to render the image, using a single thread. This is slow, but may be useful for benchmarking or debugging.
    Cpu
);

macro_rules! incrementable {
    ($enum:ty) => {
        #[cfg(not(spirv))]
        impl core::ops::Add<i32> for $enum {
            type Output = Self;

            fn add(self, delta: i32) -> Self::Output {
                use num_traits::{FromPrimitive as _, ToPrimitive as _};
                use strum::VariantArray as _;
                use $crate::easy_cast::Cast as _;
                let n = Self::VARIANTS.len().cast();
                let mut i = self.to_i32().unwrap_or_default() + delta;
                i = i.rem_euclid(n);
                Self::from_i32(i).unwrap()
            }
        }
        #[cfg(not(spirv))]
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

#[derive(Clone, Copy, Default, Debug, PartialEq, NoUninit)]
#[cfg_attr(not(spirv), derive(strum::Display))]
#[repr(u32)]
/// How close is this point to the edge of the fractal?
pub enum BoundaryClass {
    #[default]
    Indeterminate,
    Inside,
    VeryClose,
    Close,
    NotClose,
    /// We don't care about boundary classes right now
    Ignored,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use pretty_assertions::assert_eq;
    #[test]
    fn increment() {
        use super::Colourer;
        let mut c = Colourer::IcyBlue;
        c += 1;
        assert_eq!(c, Colourer::Mandy);
    }
}
