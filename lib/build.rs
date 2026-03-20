//! Build script for the library crate

use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        spirv: { target_arch = "spirv" },
    }
}
