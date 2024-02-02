// Common support code for enums which the user can select from
// (c) 2024 Ross Younger

use std::str::FromStr;

use strum::{EnumMessage, IntoEnumIterator, VariantNames};

/// Returns a list of all available items for a given type
pub fn list_vec<T: IntoEnumIterator + std::fmt::Display>() -> impl Iterator<Item = String> {
    T::iter().map(|a| a.to_string())
}

/// Prints a list of available items for a given type
pub fn list<T: VariantNames + FromStr + std::fmt::Display + EnumMessage>(machine_parseable: bool) {
    let v = &T::VARIANTS;

    if machine_parseable {
        println!("{v:?}");
        return;
    }

    let longest = v.iter().map(|r| r.len()).max().unwrap_or(1);

    v.iter().for_each(|name| {
        // Due to an issue with EnumIter that appeared in strum 0.26.1,
        // we iterate over the names, turn them back into enum members (sigh!),
        // then query the enum member `val` for its docstring.
        let res = T::from_str(name);
        if let Ok(val) = res {
            println!(
                "  {:width$}  {}",
                name,
                val.get_documentation().unwrap_or_default(),
                width = longest
            );
        }
    });
}
