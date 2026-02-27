//! Save Image support
// (c) 2026 Ross Younger

use shader::push_constants::FragmentConstants;

pub(crate) fn do_save_image(path: &std::path::Path, constants: FragmentConstants) {
    eprintln!("Saving image with constants: {constants:?}");
    eprintln!("Would save image to {}", path.display());
}
