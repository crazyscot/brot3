// Palette selection & dispatch framework
// (c) 2024 Ross Younger

use std::str::FromStr;

use enum_delegate;
use palette::{convert::FromColorUnclamped, Hsv, Srgb};
use strum::IntoStaticStr;
use strum_macros::{
    self, Display, EnumDiscriminants, EnumMessage, EnumProperty, EnumString, FromRepr, VariantArray,
};

use super::direct_rgb::{
    BlackFade, Mandy, Monochrome, MonochromeInverted, OneLoneCoder, WhiteFade,
};
use super::huecycles::{HsvGradient, LchGradient, LinearRainbow, LogRainbow, SqrtRainbow};
use super::testing::White;

/// Type sugar: Standard RGB, u8 storage
pub type Rgb8 = palette::Srgb<u8>;

/// Selector for available Palettes
#[enum_delegate::implement(OutputsRgb8)]
#[derive(
    Clone, Copy, Debug, Display, FromRepr, PartialEq, Eq, Hash, PartialOrd, Ord, IntoStaticStr,
)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(
    name(Selection),
    derive(
        clap::ValueEnum,
        Display,
        EnumMessage,
        EnumProperty,
        EnumString,
        VariantArray,
        PartialOrd,
        Ord,
    )
)] // ... and specifies what it derives from
pub enum Instance {
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
    #[strum_discriminants(value(alias = "mono"))]
    Monochrome(Monochrome),
    /// fanf's Monochrome Shade algorithm, inverted
    #[strum_discriminants(value(alias = "mono-inv"))]
    MonochromeInverted(MonochromeInverted),

    /// OneLoneCoder's algorithm
    #[allow(clippy::doc_markdown)] // false positive
    #[strum_discriminants(value(alias = "onelonecoder", alias = "olc"))]
    OneLoneCoder(OneLoneCoder),

    /// A gradient in the HSV colour space
    HsvGradient(HsvGradient),
    /// A gradient in the LCH colour space which strives for perceptual uniformity
    LchGradient(LchGradient),

    /// Test algorithm that always outputs white pixels
    #[strum_discriminants(strum(props(hide_from_list = "1")))]
    White(White),
}

impl Default for Selection {
    fn default() -> Self {
        Self::LinearRainbow
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::LinearRainbow(LinearRainbow {})
    }
}

impl crate::util::listable::Listable for Selection {}

/// A colouring algorithm that outputs Rgb8 directly.
#[enum_delegate::register]
pub trait OutputsRgb8 {
    /// Colouring function
    fn colour_rgb8(&self, iters: f32, max_iter: u32) -> Rgb8;
}

/// A colouring algorithm that outputs HSV colours
pub trait OutputsHsvf {
    /// Colouring function
    fn colour_hsvf(&self, iters: f32, max_iters: u32) -> Hsv<palette::encoding::Srgb, f32>;
}

/// Auto conversion helper
impl<T: OutputsHsvf> OutputsRgb8 for T {
    #[inline]
    fn colour_rgb8(&self, iters: f32, max_iters: u32) -> Rgb8 {
        let hsv = self.colour_hsvf(iters, max_iters);
        Srgb::<f32>::from_color_unclamped(hsv).into_format::<u8>()
    }
}

/// Factory method
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn factory(selection: Selection) -> Instance {
    Instance::from_repr(selection as usize).unwrap_or_else(|| {
        panic!("Failed to convert enum discriminant {selection} into instance (can't happen)")
    })
}

impl FromStr for Instance {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match Selection::from_str(s) {
            Ok(s) => Ok(factory(s)),
            Err(_) => anyhow::bail!("unknown colourer name"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;
    use crate::util::listable::{self, list_kebab_case};

    #[test]
    fn iter_works() {
        assert!(listable::elements::<Selection>(false).any(|s| s == &Selection::LinearRainbow));
    }

    #[test]
    fn test_algorithms_should_not_be_listed() {
        assert!(listable::elements::<Selection>(false).all(|s| s != &Selection::White));
    }

    #[test]
    fn discriminant_naming() {
        // We use kebab case in the CLI, so ensure that the helper output is in kebab case.
        let colourers: Vec<_> = list_kebab_case::<super::Selection>().collect();
        assert!(colourers.iter().any(|it| it.name == "linear-rainbow"));
        assert!(!colourers.iter().any(|it| it.name == "LinearRainbow"));
    }
}
