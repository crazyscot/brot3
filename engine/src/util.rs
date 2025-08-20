/// Support code for enums which the user can select from
pub mod listable;

mod rect;
pub use rect::Rect;

/// Custom formatting code for floating-point types
pub mod float_format;

/// Build-time info (from `built`)
pub mod build_info {
    #![allow(clippy::needless_raw_string_hashes, clippy::doc_markdown)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
