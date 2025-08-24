// Conversion of fractal PointData into its output format
// (c) 2024 Ross Younger
mod ascii;
mod framework;
mod png;

pub use framework::{IRenderer, Renderer};
pub use png::Png;
