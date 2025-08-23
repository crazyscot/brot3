// Colouring algorithms that cycle around a given hue
// (c) 2024 Ross Younger

use palette::{Hsv, LabHue, Lch, RgbHue, convert::FromColorUnclamped, encoding::Srgb};

use super::{OutputsHsvf, OutputsRgb8, Rgb8};

/// Cycling H; Fixed S=1.0, V=1.0
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LinearRainbow {}

const LINEAR_RAINBOW_WRAP: f32 = 32.0; // TODO this might become a parameter later

const BLACK_HSV: Hsv<Srgb, f32> = Hsv::new_const(RgbHue::new(0.0), 0.0, 0.0);
const BLACK_RGB: Rgb8 = Rgb8::new(0, 0, 0);

#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for LinearRainbow {
    fn colour_hsvf(&self, iters: f32, _: u32) -> Hsv<Srgb, f32> {
        if iters.is_infinite() {
            return BLACK_HSV;
        }
        let tau = (iters / LINEAR_RAINBOW_WRAP).fract();
        // this gives a number from 0..1, map that to the hue angle
        // TODO: offset becomes a parameter?
        // TODO: Wrap becomes a function of max_iter? with a parameter?
        let degrees = (0.5 + tau) * 360.0;
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

/// Cycling H; Fixed S=1.0, V=1.0
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LogRainbow {}
#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for LogRainbow {
    fn colour_hsvf(&self, iters: f32, _: u32) -> Hsv<Srgb, f32> {
        if iters.is_infinite() {
            return BLACK_HSV;
        }
        let degrees = 60.0 * iters.ln();
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

/// Cycling H; Fixed S=1.0, V=1.0
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SqrtRainbow {}
#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for SqrtRainbow {
    fn colour_hsvf(&self, iters: f32, _: u32) -> Hsv<Srgb, f32> {
        if iters.is_infinite() {
            return BLACK_HSV;
        }
        let degrees = 20.0 * iters.sqrt();
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

/// HSV Gradient function from <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#HSV_coloring>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HsvGradient {}
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for HsvGradient {
    fn colour_hsvf(&self, iters: f32, max_iters: u32) -> Hsv<Srgb, f32> {
        if iters.is_infinite() || iters >= (max_iters as f32 - 1.0) {
            return BLACK_HSV;
        }
        let proportion = iters / max_iters as f32;
        // TODO: 0.75 becomes a parameter
        let degrees = (proportion * 360.0).powf(1.5) % 360.0;
        // TODO: value 1.0 becomes a parameter of proportion?
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

/// LCH Gradient function from <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#LCH_coloring>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LchGradient {}
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
impl OutputsRgb8 for LchGradient {
    fn colour_rgb8(&self, iters: f32, max_iters: u32) -> Rgb8 {
        if iters.is_infinite() {
            return BLACK_RGB;
        }

        let s = iters / max_iters as f32;
        let v = 1.0 - (std::f32::consts::PI * s).cos().powi(2);
        let lightness = 75.0 - (75.0 * v);
        let hue = LabHue::new((s * 360.0).powf(1.5) % 360.0);
        let lch = Lch::new(lightness, 28.0 + lightness, hue);
        palette::Srgb::<f32>::from_color_unclamped(lch).into_format::<u8>()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use palette::{IntoColor, Lch, RgbHue, convert::FromColorUnclamped, rgb::Srgb};

    use super::{LINEAR_RAINBOW_WRAP, LinearRainbow};
    use crate::colouring::{OutputsHsvf, Rgb8};

    #[test]
    fn hue_cycles() {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_precision_loss)]
        let uut = LinearRainbow {};
        // The algorithm operates a linear cycle over the Wrap interval.
        // Therefore we expect it to average out fairly neatly.
        let mut hue_accumulator = 0.0;
        let mut previous = RgbHue::new(f32::NAN);

        for i in 0..(LINEAR_RAINBOW_WRAP as i32) {
            let res = uut.colour_hsvf(i as f32, 256);
            assert_ne!(res.hue, previous);
            previous = res.hue;
            hue_accumulator += res.hue.into_degrees();
        }
        // Figure from current implementation, not critical
        assert_relative_eq!(hue_accumulator, 180.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn hsv_sanity() {
        let rgb = Rgb8::new(255, 0, 0);
        let rgb2: palette::rgb::Srgb = Srgb::<f32>::from_format(rgb);
        let hsv: palette::Hsv = rgb2.into_color();
        println!("{hsv:?}");
        assert_eq!(hsv.hue, 0.0);
        assert_eq!(hsv.value, 1.0);
        assert_eq!(hsv.saturation, 1.0);
        // This demonstrates that Value and Saturation range from 0.0 to 1.0, as we thought.
    }
    #[test]
    #[allow(clippy::float_cmp)]
    fn lch_conversion() {
        // this wasn't so much a unit test of my types as it was figuring out how to use palette properly
        let rgb = Rgb8::new(255, 0, 255);
        let rgb2 = Srgb::<f32>::from_format(rgb);
        let lch: Lch = rgb2.into_color();
        println!("A {rgb:?} -> {lch:?}");

        let rgb3 = Srgb::<f32>::from_color_unclamped(lch);
        let rgb4 = Rgb8::from_format(rgb3);
        println!("B {lch:?} -> {rgb4:?}");
        assert_eq!(rgb, rgb4);
    }
}
