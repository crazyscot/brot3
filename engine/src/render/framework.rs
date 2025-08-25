// Rendering selection & dispatch
// (c) 2024 Ross Younger

use std::ffi::OsStr;

use anyhow;
use spire_enum::prelude::{delegate_impl, delegated_enum};
use strum_macros::{Display, EnumIter, EnumMessage, EnumProperty, EnumString};

use super::ascii::{AsciiArt, Csv};
use super::png::Png;
use crate::colouring::Colourer;
use crate::fractal::{Tile, TileSpec};

/// Framework for all available renderers.
/// see [`IRenderer`]
#[delegated_enum(impl_conversions)]
#[derive(Clone, Copy, Debug, Display, EnumIter, EnumMessage, EnumProperty, EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum Renderer {
    /// Comma Separated Values, one line per line of plot (.csv)
    Csv(Csv),
    /// Good old ASCII art (.txt) [short-form: "aa"]
    #[strum(
        props(file_extension = "txt"),
        serialize = "aa",
        serialize = "ascii-art"
    )]
    AsciiArt(AsciiArt),
    /// Portable Network Graphics (.png) file
    Png(Png),
}
impl crate::util::Listable for Renderer {}

/// A Renderer accepts ``PointData`` and deals with it completely.
/// This is distinct from a Palette, which accepts ``PointData`` and returns colour data.
/// The trait knows nothing about output or buffering; the implementation is responsible for setting that up.
pub trait IRenderer {
    /// Renders fractal data and sends it to its output.
    /// Data tiles must be in correct order for output (they are generated in order, so this should be no imposition)
    fn render_file(
        &self,
        filename: &str,
        spec: &TileSpec,
        data: &[Tile],
        colourer: Colourer,
    ) -> anyhow::Result<()>;

    /// A sanity check that the input tiles are in the correct order
    fn check_ordering(&self, tiles: &[Tile]) -> bool {
        tiles
            .windows(2)
            .all(|w| w[0].y_offset.unwrap_or(0) < w[1].y_offset.unwrap_or(0))
    }
}

#[delegate_impl]
impl IRenderer for Renderer {
    fn render_file(
        &self,
        filename: &str,
        spec: &TileSpec,
        data: &[Tile],
        colourer: Colourer,
    ) -> anyhow::Result<()>;
}

impl Renderer {
    /// Attempt to auto-match a file extension to a renderer
    pub fn try_from_file_extension(filename: &str) -> Option<Renderer> {
        use strum::EnumProperty;

        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_ascii_lowercase();

        <Renderer as strum::IntoEnumIterator>::iter().find(|ren| {
            let trial = ren.get_str("file_extension").map_or_else(
                || {
                    // No property? use the enum name
                    ren.to_string().to_ascii_lowercase()
                },
                // the property exists? convert &str to string
                std::string::ToString::to_string,
            );
            trial == extension
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::util::Listable;

    #[test]
    fn renderers_list() {
        assert_ne!(super::Renderer::elements().count(), 0);
    }
}
