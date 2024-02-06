// Common support code for enums which the user can select from
// (c) 2024 Ross Younger

use std::fmt::Display;

use strum::{EnumMessage, EnumProperty, VariantArray};

/// A compound trait for Listable operations
/// see e.g. ``fractal::framework::Selection``
pub trait Listable: Display + EnumMessage + EnumProperty + VariantArray {}

/// Returns an iterator of elements of a listable type
pub fn elements<T: Listable>(include_hidden: bool) -> impl Iterator<Item = &'static T> {
    T::VARIANTS
        .iter()
        .filter(move |x| include_hidden || x.get_str("hide_from_list").is_none())
}

/// Prints a list of available items for a given type, respecting the ``hide_from_list`` flag
pub fn list<T: Listable>() {
    let v: Vec<_> = elements::<T>(false).collect();

    let longest = v
        .iter()
        .map(|item| item.to_string().len())
        .max()
        .unwrap_or(1);

    for item in v {
        println!(
            "  {:width$}  {}",
            item,
            item.get_documentation().unwrap_or_default(),
            width = longest
        );
    }
}
