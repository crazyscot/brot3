// Common support code for enums which the user can select from
// (c) 2024 Ross Younger

use strum::{EnumMessage, IntoEnumIterator};

/// Returns a list of all available items for a given type
#[must_use]
pub fn list_vec<T: IntoEnumIterator + std::fmt::Display>() -> Vec<String> {
    T::iter().map(|a| a.to_string()).collect()
}

/// Implementation of 'list'
pub fn list<T: IntoEnumIterator + std::fmt::Display + EnumMessage>(machine_parseable: bool) {
    if machine_parseable {
        println!("{:?}", list_vec::<T>());
        return;
    }

    println!("Available fractals:");
    let longest = T::iter().map(|r| r.to_string().len()).max().unwrap_or(1);

    let _ = T::iter()
        .map(|r| {
            println!(
                "  {:width$}  {}",
                r.to_string(),
                r.get_documentation().unwrap_or_default(),
                width = longest
            );
        })
        .collect::<Vec<_>>();
}
