/// Filename wrangling sugar
pub mod filename;

/// Support code for enums which the user can select from
pub mod listable;

mod rect;
pub use rect::Rect;

/// Custom formatting code for floating-point types
pub mod float_format;

/// Build-time info (from `built`)
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
