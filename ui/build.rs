//! Build-time crate:
//! Before compiling our rust we must let slint do its stuff.

fn main() {
    // Caution: You can only usefully compile one slint file at the present time. https://github.com/slint-ui/slint/issues/3217
    slint_build::compile_with_config(
        "ui/mainui.slint",
        slint_build::CompilerConfiguration::new().with_library_paths(vivi_ui::import_paths()),
    )
    .unwrap();
}
