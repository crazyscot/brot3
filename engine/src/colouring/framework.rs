// Palette selection & dispatch framework
// (c) 2024 Ross Younger

use palette::{Hsv, Srgb, convert::FromColorUnclamped};
use spire_enum::prelude::{delegate_impl, delegated_enum};

use super::direct_rgb::{
    BlackFade, Mandy, Monochrome, MonochromeInverted, OneLoneCoder, WhiteFade,
};
use super::huecycles::{HsvGradient, LchGradient, LinearRainbow, LogRainbow, SqrtRainbow};
use super::testing::White;

/// Type sugar: Standard RGB, u8 storage
pub type Rgb8 = palette::Srgb<u8>;

/// Framework for all available colourers.
/// see [`IColourer`] and [`HsvfColourer`]
#[delegated_enum(impl_conversions)]
#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum_macros::Display,
    strum_macros::EnumDiscriminants,
    strum_macros::EnumIter,
    strum_macros::EnumMessage,
    strum_macros::EnumProperty,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "kebab_case")]
pub enum Colourer {
    /// Cyclic rainbow
    LinearRainbow(LinearRainbow),
    /// Cyclic rainbow (log-smoothed)
    LogRainbow(LogRainbow),
    /// Cyclic rainbow (sqrt-smoothed)
    SqrtRainbow(SqrtRainbow),

    /// The colouring algorithm from ``mandy`` by rjk
    Mandy(Mandy),

    /// fanf's White Fade algorithm
    WhiteFade(WhiteFade),
    /// fanf's Black Fade algorithm
    BlackFade(BlackFade),
    /// fanf's Monochrome Shade algorithm
    #[strum(serialize = "mono", serialize = "monochrome")]
    Monochrome(Monochrome),
    /// fanf's Monochrome Shade algorithm, inverted
    #[strum(serialize = "mono-inv", serialize = "monochrome-inverted")]
    MonochromeInverted(MonochromeInverted),

    /// OneLoneCoder's algorithm
    #[allow(clippy::doc_markdown)] // false positive
    #[strum(
        serialize = "one-lone-coder",
        serialize = "onelonecoder",
        serialize = "olc"
    )]
    OneLoneCoder(OneLoneCoder),

    /// A gradient in the HSV colour space
    HsvGradient(HsvGradient),
    /// A gradient in the LCH colour space which strives for perceptual uniformity
    LchGradient(LchGradient),

    /// Test algorithm that always outputs white pixels
    #[strum(props(hide_from_list = "1"))]
    White(White),
}

impl crate::util::Listable for Colourer {}

impl Default for Colourer {
    fn default() -> Self {
        Colourer::LinearRainbow(LinearRainbow {})
    }
}
/// A colouring algorithm that outputs Rgb8 directly.
pub trait IColourer {
    /// Colouring function
    fn colour_rgb8(&self, iters: f32, max_iter: u32) -> Rgb8;
}

#[delegate_impl]
impl IColourer for Colourer {
    fn colour_rgb8(&self, iters: f32, max_iter: u32) -> Rgb8;
}

/// A colouring algorithm that outputs HSV colours
pub trait HsvfColourer {
    /// Colouring function
    fn colour_hsvf(&self, iters: f32, max_iters: u32) -> Hsv<palette::encoding::Srgb, f32>;
}

/// Auto conversion helper
impl<T: HsvfColourer> IColourer for T {
    #[inline]
    fn colour_rgb8(&self, iters: f32, max_iters: u32) -> Rgb8 {
        let hsv = self.colour_hsvf(iters, max_iters);
        Srgb::<f32>::from_color_unclamped(hsv).into_format::<u8>()
    }
}

#[cfg(test)]
mod tests {
    use super::Colourer;
    use crate::{
        colouring::{huecycles::LinearRainbow, testing::White},
        util::Listable as _,
    };

    #[test]
    fn iter_works() {
        let it = LinearRainbow {}.into();
        assert!(Colourer::elements().any(|s| s == it));
    }

    #[test]
    fn test_algorithms_should_not_be_listed() {
        let it = White {}.into();
        assert!(Colourer::elements().all(|s| s != it));
    }

    #[test]
    fn discriminant_naming() {
        // We use kebab case in the CLI, so ensure that the helper output is in kebab case.
        let colourers: Vec<_> = super::Colourer::list_kebab_case().collect();
        assert!(colourers.iter().any(|it| it.name == "linear-rainbow"));
        assert!(!colourers.iter().any(|it| it.name == "LinearRainbow"));
    }
}
