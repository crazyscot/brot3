// Palette selection & dispatch framework
// (c) 2024 Ross Younger

use enum_dispatch::enum_dispatch;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString};

use super::direct_rgb::Mandy;
use super::huecycles::{self, LinearRainbow};
use super::types::White;
use super::{direct_rgb, Rgb8};

/// Selector for available Palettes
#[enum_dispatch]
#[derive(Clone, Copy, Debug, Display, EnumIter, EnumMessage, PartialEq)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(name(Selection), derive(clap::ValueEnum, EnumIter, EnumString))] // ... and specifies what it derives from
#[allow(clippy::module_name_repetitions)] // enum_dispatch doesn't support structs with the same name but different paths
pub enum ColourerInstance {
    /// A continuous cycle around the HSV cone with fixed saturation and lightness
    LinearRainbow,

    /// The colouring algorithm from ``mandy`` by rjk
    Mandy,

    /// Test algorithm that always outputs white pixels
    #[strum(disabled)]
    White,
}

/// A colouring algorithm that outputs Rgb8 directly.
#[enum_dispatch(ColourerInstance)]
pub trait OutputsRgb8 {
    /// Colouring function
    fn colour_rgb8(&self, iters: f64) -> Rgb8;
}

/// Factory method
#[must_use]
pub fn factory(selection: Selection) -> ColourerInstance {
    match selection {
        Selection::LinearRainbow => huecycles::LinearRainbow {}.into(),
        Selection::Mandy => direct_rgb::Mandy {}.into(),
        Selection::White => White {}.into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{colouring::framework::ColourerInstance, util::listable::list_vec};

    #[test]
    fn list() {
        assert_ne!(list_vec::<ColourerInstance>().len(), 0);
    }
}
