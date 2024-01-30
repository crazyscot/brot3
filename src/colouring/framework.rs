// Palette selection & dispatch framework
// (c) 2024 Ross Younger

use enum_delegate;
use strum_macros::{
    Display, EnumDiscriminants, EnumMessage, EnumString, FromRepr, VariantArray, VariantNames,
};

use super::direct_rgb::Mandy;
use super::huecycles::LinearRainbow;
use super::types::White;
use super::Rgb8;

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
    /// A continuous cycle around the HSV cone with fixed saturation and lightness
    LinearRainbow(LinearRainbow),

    /// The colouring algorithm from ``mandy`` by rjk
    Mandy(Mandy),

    /// Test algorithm that always outputs white pixels
    White(White),
}

/// A colouring algorithm that outputs Rgb8 directly.
#[enum_delegate::register]
pub trait OutputsRgb8 {
    /// Colouring function
    fn colour_rgb8(&self, iters: f64) -> Rgb8;
}

/// Factory method
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn factory(selection: Selection) -> Instance {
    Instance::from_repr(selection as usize).unwrap_or_else(|| {
        panic!("Failed to convert enum discriminant {selection} into instance (can't happen)")
    })
}
