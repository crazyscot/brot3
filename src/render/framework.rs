// Rendering selection & dispatch
// (c) 2024 Ross Younger

use crate::colouring::ColourerInstance;
use crate::fractal::Tile;

use anyhow;
use enum_dispatch::enum_dispatch;
use strum::IntoStaticStr;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumProperty, EnumString};

use super::ascii::{AsciiArt, Csv};
use super::png::Png;

#[enum_dispatch]
#[derive(Clone, Copy, Debug, Display, EnumIter, EnumMessage, EnumProperty)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(
    name(Selection),
    derive(clap::ValueEnum, EnumIter, EnumString, EnumProperty, IntoStaticStr)
)] // ... and specifies what it derives from
/// Selector for available Renderers
#[allow(clippy::module_name_repetitions)]
pub enum RenderInstance {
    // Note we set the file_extension and alias properties on the members of the discriminant enum,
    // not on RenderInstance itself.
    //
    /// Comma Separated Values, one line per line of plot (.csv)
    Csv,
    /// Good old ASCII art (.txt) [short-form: "aa"]
    #[strum_discriminants(value(alias = "aa"))]
    #[strum_discriminants(strum(props(file_extension = "txt")))]
    AsciiArt,
    /// Portable Network Graphics (.png) file
    Png,
}

/// A Renderer accepts ``PointData`` and deals with it completely.
/// This is distinct from a Palette, which accepts ``PointData`` and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
#[enum_dispatch(RenderInstance)]
pub trait Renderer {
    /// Renders fractal data and sends it to its output
    fn render_file(
        &self,
        filename: &str,
        data: &Tile,
        colourer: ColourerInstance,
    ) -> anyhow::Result<()>;
}

/// Factory method for renderers
#[must_use]
pub fn factory(selection: Selection) -> RenderInstance {
    match selection {
        Selection::Csv => Csv::default().into(),
        Selection::AsciiArt => AsciiArt::default().into(),
        Selection::Png => Png::default().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::RenderInstance;
    use crate::util::listable::list_vec;

    #[test]
    fn renderers_list() {
        assert_ne!(list_vec::<RenderInstance>().len(), 0);
    }
}
