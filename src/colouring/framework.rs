// Palette selection & dispatch framework
// (c) 2024 Ross Younger

use enum_delegate;
use strum_macros::{
    Display, EnumDiscriminants, EnumMessage, EnumString, FromRepr, VariantArray, VariantNames,
};

use super::direct_rgb::{
    BlackFade, Mandy, Monochrome, MonochromeInverted, OneLoneCoder, WhiteFade,
};
use super::huecycles::{HsvGradient, LchGradient, LinearRainbow, LogRainbow};
use super::testing::White;

use palette::{FromColor, Hsv, Srgb};

/// Type sugar: Standard RGB, u8 storage
pub type Rgb8 = palette::Srgb<u8>;

/// Selector for available Palettes
#[enum_delegate::implement(OutputsRgb8)]
#[derive(Clone, Copy, Debug, Display, FromRepr, PartialEq)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(
    name(Selection),
    derive(
        clap::ValueEnum,
        Display,
        EnumMessage,
        EnumString,
        VariantArray,
        VariantNames
    )
)] // ... and specifies what it derives from

pub enum Instance {
    /// Cyclic rainbow
    LinearRainbow(LinearRainbow),
    /// Cyclic rainbow (log-smoothed)
    LogRainbow(LogRainbow),

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
    #[strum_discriminants(value(alias = "onelonecoder", alias = "olc"))]
    OneLoneCoder(OneLoneCoder),

    /// A gradient in the HSV colour space
    HsvGradient(HsvGradient),
    /// A gradient in the LCH colour space which strives for perceptual uniformity
    LchGradient(LchGradient),

    /// Test algorithm that always outputs white pixels
    White(White),
}

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
        Srgb::<f32>::from_color(hsv).into_format::<u8>()
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
