// Direct-to-RGB colouring functions
// (c) 2024 Ross Younger

use super::{OutputsRgb8, Rgb8};

/// The colouring algorithm from rjk's ``mandy``
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mandy {}

impl OutputsRgb8 for Mandy {
    fn colour_rgb8(&self, iters: f64) -> Rgb8 {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_sign_loss)]
        // inf -> black, that's all good with us.
        let c = 2.0 * std::f64::consts::PI * iters.sqrt();
        Rgb8::new(
            (((c / 5.0).cos() + 1.0) * 127.0) as u8,
            (((c / 7.0).cos() + 1.0) * 127.0) as u8,
            (((c / 11.0).cos() + 1.0) * 127.0) as u8,
        )
    }
}
