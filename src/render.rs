// Conversion of fractal PointData into its output format
// (c) 2024 Ross Younger
mod ascii;
mod png;

use super::fractal::Tile;

use anyhow;
use enum_dispatch::enum_dispatch;
use strum::{EnumMessage, IntoEnumIterator};
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
pub enum Selection {
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
#[enum_dispatch(Selection)]
pub trait Renderer {
    /// Renders fractal data and sends it to its output
    fn render(&self, data: &Tile) -> anyhow::Result<()>;
}

/// Lists all available renderers
#[must_use]
pub fn list_vec() -> Vec<String> {
    Selection::iter().map(|a| a.to_string()).collect()
}

/// Implementation of 'list renderers'
pub fn list(machine_parseable: bool) {
    if machine_parseable {
        println!("{:?}", list_vec());
        return;
    }

    println!("Available renderers:");
    let longest = Selection::iter()
        .map(|r| r.to_string().len())
        .max()
        .unwrap_or(1);

    let _ = Selection::iter()
        .map(|r| {
            println!(
                "  {:width$}  {}",
                r.to_string(),
                r.get_documentation().unwrap_or_default(),
                width = longest
            );
        })
        .collect::<Vec<_>>();
}

/// Factory method for renderers
#[must_use]
pub fn factory(selection: SelectionDiscriminants, filename: &str) -> Selection {
    match selection {
        SelectionDiscriminants::Csv => Selection::Csv(ascii::Csv::new(filename)),
        SelectionDiscriminants::AsciiArt => Selection::AsciiArt(ascii::AsciiArt::new(filename)),
        SelectionDiscriminants::Png => Selection::Png(png::Png::new(filename)),
    }
}

#[cfg(test)]
mod tests {
    use super::list_vec;

    #[test]
    fn renderers_list() {
        assert_ne!(list_vec().len(), 0);
    }
}
