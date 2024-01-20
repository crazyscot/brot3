// Conversion of fractal PointData into its output format
// (c) 2024 Ross Younger
mod ascii;
mod png;

use crate::fractal::Tile;

use anyhow;
use enum_dispatch::enum_dispatch;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString};

pub use png::colour_temp;

use self::ascii::{AsciiArt, Csv};
use self::png::Png;

#[enum_dispatch]
#[derive(Clone, Debug, Display, EnumIter, EnumMessage)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum RenderBehaviourEnumDiscriminants ...
#[strum_discriminants(derive(clap::ValueEnum, EnumIter, EnumString))] // ... and specifies what it derives from
/// Selector for available Renderers
pub enum SelectionR {
    /// Comma Separated Values, one line per line of plot
    Csv,
    /// Good old ASCII art (can be abbreviated to "aa")
    #[strum_discriminants(value(alias = "aa"))]
    AsciiArt,
    /// Portable Network Graphics file
    Png,
}

/// A Renderer accepts ``PointData`` and deals with it completely.
/// This is distinct from a Palette, which accepts ``PointData`` and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
#[enum_dispatch(SelectionR)]
pub trait Renderer {
    /// Renders fractal data and sends it to its output
    fn render(&self, data: &Tile) -> anyhow::Result<()>;
}

/// Factory method for renderers
#[must_use]
pub fn factory(selection: SelectionRDiscriminants, filename: &str) -> SelectionR {
    match selection {
        SelectionRDiscriminants::Csv => SelectionR::Csv(ascii::Csv::new(filename)),
        SelectionRDiscriminants::AsciiArt => SelectionR::AsciiArt(ascii::AsciiArt::new(filename)),
        SelectionRDiscriminants::Png => SelectionR::Png(png::Png::new(filename)),
    }
}

#[cfg(test)]
mod tests {
    use super::SelectionR;
    use crate::util::listable::list_vec;

    #[test]
    fn renderers_list() {
        assert_ne!(list_vec::<SelectionR>().len(), 0);
    }
}
