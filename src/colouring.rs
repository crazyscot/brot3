// Fractal colouring
// Note, this module is named 'colouring' to avoid name confusion with the 'palette' crate !
// (c) 2024 Ross Younger

/// Selection & dispatch framework
mod framework;
pub use framework::{factory, ColourerInstance, OutputsRgb8, Selection};

/// Hue cycling algorithms
pub mod huecycles;

/// Direct-to-RGB algorithms
pub mod direct_rgb;

// Colour space and conversion helpers
mod types;
pub use types::{Hsvf, OutputsHsvf, Rgb8, Rgbf};
