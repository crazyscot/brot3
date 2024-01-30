// Colour space typing helpers
// (c) 2024 Ross Younger

use palette::FromColor;
use palette::{Hsv, Srgb};

use super::framework::OutputsRgb8;

/// RGB type, f32 storage
pub type Rgbf = palette::Srgb<f32>;
/// RGB type, u8 storage
pub type Rgb8 = palette::Srgb<u8>;

/// A colouring algorithm that outputs HSV colours
pub trait OutputsHsvf {
    /// Colouring function
    fn colour_hsvf(&self, iters: f64, max_iters: u64) -> Hsv<palette::encoding::Srgb, f32>;
}

/// Auto conversion helper
impl<T: OutputsHsvf> OutputsRgb8 for T {
    #[inline]
    fn colour_rgb8(&self, iters: f64, max_iters: u64) -> Rgb8 {
        let hsv = self.colour_hsvf(iters, max_iters);
        Srgb::<f32>::from_color(hsv).into_format::<u8>()
    }
}

/// Test algorithm, doesn't do anything useful
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct White {}
impl OutputsRgb8 for White {
    fn colour_rgb8(&self, _: f64, _: u64) -> Rgb8 {
        Rgb8::new(255, 255, 255)
    }
}

// Test algorithm
struct WhiteHSV {}
impl OutputsHsvf for WhiteHSV {
    fn colour_hsvf(&self, _: f64, _: u64) -> Hsv<palette::encoding::Srgb, f32> {
        Hsv::new(0.0, 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputsRgb8, WhiteHSV};
    use palette::{rgb, FromColor, Hsv, IntoColor, RgbHue, Srgb};

    use crate::colouring::{OutputsHsvf, Rgb8, Rgbf};

    #[test]
    fn red_conversion() {
        let hsv = Hsv::new(RgbHue::from_degrees(0.0), 1.0, 1.0);
        let rgb: Rgbf = hsv.into_color();
        let expected = Rgbf::new(1.0, 0.0, 0.0);
        assert_eq!(rgb, expected);

        let rgb8: Rgb8 = Rgb8::from_format(rgb);
        println!("{rgb:?} -> {rgb8:?}");
        let packed = rgb8.into_u32::<rgb::channels::Rgba>();
        assert_eq!(0xff00_00ff, packed);
    }

    #[test]
    fn hsv_conversions() {
        {
            let rgb2 = Srgb::new(255u8, 255u8, 255u8);
            let rgb3 = Srgb::<f32>::from_format(rgb2);
            let hsv2: Hsv = Hsv::from_color(rgb3);
            println!("A {rgb2:?} -> {rgb3:?} -> {hsv2:?}");
            let yyy: Srgb<f32> = hsv2.into_color();
            let zzz: Srgb<u8> = yyy.into_format();
            println!("B {hsv2:?} -> {yyy:?} -> {zzz:?}");
            assert_eq!(rgb2, zzz);
        }
        {
            let alg = WhiteHSV {};
            let raw = alg.colour_hsvf(42.0, 256);
            let result = alg.colour_rgb8(42.0, 256);
            println!("C {raw:?} -> {result:?}");

            let res2 = Srgb::<f32>::from_color(raw);
            println!("D {raw:?} -> {res2:?}");
            assert_eq!(result, Srgb::new(255, 255, 255));
            assert_eq!(res2.into_format::<u8>(), Srgb::new(255, 255, 255));
        }
    }
}
