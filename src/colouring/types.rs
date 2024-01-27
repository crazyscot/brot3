// Colour space typing helpers
// (c) 2024 Ross Younger

use palette::IntoColor;
use palette::Srgb;

use super::framework::OutputsRgb8;

/// RGB type, f32 storage
pub type Rgbf = palette::rgb::Rgb<Srgb, f32>;
/// RGB type, u8 storage
pub type Rgb8 = palette::rgb::Rgb<Srgb, u8>;
/// HSV type, f32 storage
pub type Hsvf = palette::hsv::Hsv<Srgb, f32>;

/// A colouring algorithm that outputs HSV colours
pub trait OutputsHsvf {
    /// Colouring function
    fn colour_hsvf(&self, iters: f64) -> Hsvf;
}

/// Auto conversion helper
impl<T: OutputsHsvf> OutputsRgb8 for T {
    #[inline]
    fn colour_rgb8(&self, iters: f64) -> Rgb8 {
        let hsv = self.colour_hsvf(iters);
        let rgb: Rgbf = hsv.into_color();
        Rgb8::from_format(rgb)
    }
}

/// Test algorithm, doesn't do anything useful
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct White {}
impl OutputsRgb8 for White {
    fn colour_rgb8(&self, _: f64) -> Rgb8 {
        Rgb8::new(255, 255, 255)
    }
}

// Test algorithm
struct WhiteHSV {}
impl OutputsHsvf for WhiteHSV {
    fn colour_hsvf(&self, _: f64) -> Hsvf {
        Hsvf::new(0.0, 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputsRgb8, WhiteHSV};
    use palette::{rgb, IntoColor};

    use crate::colouring::{Hsvf, Rgb8, Rgbf};

    #[test]
    fn red_conversion() {
        let hsv = Hsvf::new(0.0, 1.0, 1.0);
        let rgb: Rgbf = hsv.into_color();
        let expected = Rgbf::new(1.0, 0.0, 0.0);
        assert_eq!(rgb, expected);

        let rgb8: Rgb8 = Rgb8::from_format(rgb);
        let packed = rgb8.into_u32::<rgb::channels::Rgba>();
        assert_eq!(0xff00_00ff, packed);
    }

    #[test]
    fn hsv_autoconvert() {
        let alg = WhiteHSV {};
        let result = alg.colour_rgb8(42.0);
        assert_eq!(result.red, 255);
        assert_eq!(result.green, 255);
        assert_eq!(result.blue, 255);
    }
}
