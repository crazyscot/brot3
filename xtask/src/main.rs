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
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Task definition
//
// Syntax: (Command-line verb, implementing function, description for help message)

static DEBIAN_ARGS: LazyLock<DebPackageMeta> = LazyLock::new(|| {
    DebPackageMeta::new(
        "brot3",
        "brot3-ui",
        "ui",
        "Ross Younger",
        "ross@crazyscot.com",
        "generic",
    )
});

#[allow(clippy::type_complexity)]
const TASKS: util::Tasks<'_> = &[(
    "debian",
    |args| debian::debian(args, &DEBIAN_ARGS),
    "Prepare the debian files",
)];

fn main() -> anyhow::Result<()> {
    util::dispatch(TASKS)
}
