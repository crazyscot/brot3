//! xtask to create debian package files
// (c) 2025 Ross Younger

#[allow(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
pub mod debian;

pub mod util;

use debian::DebPackageMeta;

static DEBIAN_ARGS: DebPackageMeta = DebPackageMeta {
    deb_name: "brot3",
    package_crate: "brot3-ui",
    package_dir: "ui",
    deb_fullname: "Ross Younger",
    deb_email: "ross@crazyscot.com",
    distro: "generic",
};

// ---------------------------------------------------------------------------------------------
// Task definition
//
// Syntax: (Command-line verb, implementing function, description for help message)

#[allow(clippy::type_complexity)]
const TASKS: util::Tasks<'_> = &[(
    "debian",
    |args| debian::debian(args, &DEBIAN_ARGS),
    "Prepare the debian files",
)];

fn main() -> anyhow::Result<()> {
    util::dispatch(TASKS)
}
