// Common support code for enums which the user can select from
// (c) 2024 Ross Younger

use std::fmt::Display;

use heck::ToKebabCase;
use serde::Serialize;
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

#[derive(Serialize, Clone, Debug)]
/// A representation of a listable item
pub struct ListItem {
    /// Item name (in kebab case)
    pub name: String,
    /// Item description
    pub description: String,
}

/// Returns an iterator of available items for a given type and their descriptions.
/// Item names are returned in kebab case.
/// This call respects the ``hide_from_list`` flag.
pub fn list_kebab_case<T: Listable>() -> impl Iterator<Item = ListItem> {
    elements::<T>(false).map(|item| ListItem {
        name: item.to_string().to_kebab_case(),
        description: item.get_documentation().unwrap_or_default().to_string(),
    })
}

/// Returns an iterator of available items for a given type and their descriptions.
/// Item names are returned in original case.
/// This call respects the ``hide_from_list`` flag.
pub fn list_original_case<T: Listable>() -> impl Iterator<Item = ListItem> {
    elements::<T>(false).map(|item| ListItem {
        name: item.to_string(),
        description: item.get_documentation().unwrap_or_default().to_string(),
    })
}

/// Prints a list of available items for a given type, respecting the ``hide_from_list`` flag
pub fn print_list<T: Listable>() {
    let v: Vec<ListItem> = list_kebab_case::<T>().collect();

    let longest = v.iter().map(|item| item.name.len()).max().unwrap_or(1);

    for item in v {
        println!(
            "  {:width$}  {}",
            item.name,
            item.description,
            width = longest
        );
    }
}
