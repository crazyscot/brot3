// Rendering selection & dispatch
// (c) 2024 Ross Younger

use std::ffi::OsStr;

use crate::colouring::Instance;
use crate::fractal::{Tile, TileSpec};

use anyhow;
use strum_macros::{Display, EnumDiscriminants, EnumMessage, FromRepr, IntoStaticStr};

use super::ascii::{AsciiArt, Csv};
use super::png::Png;

#[enum_delegate::implement(Renderer)]
#[derive(Clone, Copy, Debug, Display, FromRepr)]
#[strum(serialize_all = "kebab_case")]
#[derive(EnumDiscriminants)] // This creates the enum Selection ...
#[strum_discriminants(
    name(Selection),
    derive(
        clap::ValueEnum,
        Display,
        EnumMessage,
        strum_macros::EnumProperty,
        IntoStaticStr,
        strum_macros::VariantArray,
    )
)] // ... and specifies what it derives from
/// Selector for available Renderers
#[allow(clippy::module_name_repetitions)]
pub enum RenderInstance {
    // Note we set the file_extension and alias properties on the members of the discriminant enum,
    // not on RenderInstance itself.
    //
    /// Comma Separated Values, one line per line of plot (.csv)
    Csv(Csv),
    /// Good old ASCII art (.txt) [short-form: "aa"]
    #[strum_discriminants(value(alias = "aa"))]
    #[strum_discriminants(strum(props(file_extension = "txt")))]
    AsciiArt(AsciiArt),
    /// Portable Network Graphics (.png) file
    Png(Png),
}
impl crate::util::listable::Listable for Selection {}

/// A Renderer accepts ``PointData`` and deals with it completely.
/// This is distinct from a Palette, which accepts ``PointData`` and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
#[enum_delegate::register]
pub trait Renderer {
    /// Renders fractal data and sends it to its output.
    /// Data tiles must be in correct order for output (they are generated in order, so this should be no imposition)
    fn render_file(
        &self,
        filename: &str,
        spec: &TileSpec,
        data: &[Tile],
        colourer: Instance,
    ) -> anyhow::Result<()>;

    /// A sanity check that the input tiles are in the correct order
    fn check_ordering(&self, tiles: &[Tile]) -> bool {
        tiles
            .windows(2)
            .all(|w| w[0].y_offset.unwrap_or(0) < w[1].y_offset.unwrap_or(0))
    }
}

/// Factory method for renderers
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn factory(selection: Selection) -> RenderInstance {
    RenderInstance::from_repr(selection as usize)
        .expect("Failed to convert enum discriminant into instance (can't happen)")
}

/// Attempt to auto-match a file extension to a renderer
pub fn autodetect_extension(filename: &str) -> Option<&Selection> {
    use strum::{EnumProperty, VariantArray};

    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    Selection::VARIANTS
        .iter()
        //.flat_map(|name| render::Selection::from_str(name))
        .find(|sel| {
            let trial = sel.get_str("file_extension").map_or_else(
                || {
                    // No property? use the enum name
                    sel.to_string().to_ascii_lowercase()
                },
                // the property exists? convert &str to string
                std::string::ToString::to_string,
            );
            trial == extension
        })
}

#[cfg(test)]
mod tests {
    use super::Selection;
    use crate::util::listable;

    #[test]
    fn renderers_list() {
        assert_ne!(listable::elements::<Selection>(false).count(), 0);
    }
}
