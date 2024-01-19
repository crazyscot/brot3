//! Yet Another Fractal Plotter

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![deny(elided_lifetimes_in_paths)]
#![deny(explicit_outlives_requirements)]
#![deny(meta_variable_misuse)]
#![deny(missing_abi)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![warn(missing_docs)]
#![deny(non_ascii_idents)]
#![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unreachable_pub)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(unused_lifetimes)]
#![deny(unused_results)]
#![warn(unused_tuple_struct_fields)]
#![deny(variant_size_differences)]

/// Command Line Interface helpers
pub mod cli;
/// Fractal interface
pub mod fractal;
/// Fractal renderers
pub mod render;
/// General utility code
pub mod util;
