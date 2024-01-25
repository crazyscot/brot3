// Fractal colouring
// Note, this module is named 'colouring' to avoid name confusion with the 'palette' crate !
// (c) 2024 Ross Younger

use palette::Srgb;

/// Selection & dispatch framework
pub mod framework;
pub use framework::{factory, ColourerInstance, OutputsRgb8, Selection};

/// Hue cycling algorithms
pub mod huecycles;

/// Direct-to-RGB algorithms
pub mod direct_rgb;

// Colour space and conversion helpers
mod types;
pub use types::OutputsHsvf;

/// RGB type, f32 storage
pub type Rgbf = palette::rgb::Rgb<Srgb, f32>;
/// RGB type, u8 storage
pub type Rgb8 = palette::rgb::Rgb<Srgb, u8>;
/// HSV type, f32 storage
pub type Hsvf = palette::hsv::Hsv<Srgb, f32>;
