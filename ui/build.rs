//! Build-time crate:
//! Before compiling our rust we must let slint do its stuff.

fn main() {
    // Caution: You can only usefully compile one slint file at the present time. https://github.com/slint-ui/slint/issues/3217
    slint_build::compile("ui/mainui.slint").unwrap();
}
