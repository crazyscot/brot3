// Conversion of fractal PointData into its output format
// (c) 2024 Ross Younger
mod ascii;
mod framework;
mod png;

#[allow(clippy::module_name_repetitions)]
pub use framework::{factory, RenderInstance, Renderer, Selection};
