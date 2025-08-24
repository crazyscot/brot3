// Common support code for enums which the user can select from
// (c) 2024 Ross Younger

use std::{fmt::Display, str::FromStr};

use heck::ToKebabCase;
use serde::Serialize;
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

/// A compound trait for Listable operations
/// see e.g. ``fractal::framework::Selection``
pub trait Listable
: Display + EnumMessage + IntoEnumIterator/* derive strum::EnumIter */ + EnumProperty + FromStr /* derive strum_macros::EnumString */
{
    /// Returns an iterator of elements of a listable type, filtering out any marked as hidden
    #[must_use]
    fn elements() -> impl Iterator<Item = Self> {
        Self::iter().filter(move |x| x.get_str("hide_from_list").is_none())
    }

    /// Returns an iterator of available items for a given type and their descriptions.
    /// Item names are returned in kebab case.
    /// This call respects the ``hide_from_list`` flag.
    #[must_use]
    fn list_kebab_case() -> impl Iterator<Item = ListItem> {
        Self::elements().map(|item| ListItem {
            name: item.to_string().to_kebab_case(),
            description: item.get_documentation().unwrap_or_default().to_string(),
        })
    }

    /// Returns an iterator of available items for a given type and their descriptions.
    /// Item names are returned in original case.
    /// This call respects the ``hide_from_list`` flag.
    #[must_use]
    fn list_original_case() -> impl Iterator<Item = ListItem> {
        Self::elements().map(|item| ListItem {
            name: item.to_string(),
            description: item.get_documentation().unwrap_or_default().to_string(),
        })
    }
    /// Prints a list of available items for a given type, respecting the ``hide_from_list`` flag
    fn print_list() {
        let v: Vec<ListItem> = Self::list_kebab_case().collect();

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
}

#[derive(Serialize, Clone, Debug)]
/// A representation of a listable item
pub struct ListItem {
    /// Item name (in kebab case)
    pub name: String,
    /// Item description
    pub description: String,
}
