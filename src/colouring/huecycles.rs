// Colouring algorithms that cycle around a given hue
// (c) 2024 Ross Younger

use palette::{encoding::Srgb, Hsv, RgbHue};

use super::OutputsHsvf;

/// Cycling H; Fixed S=1.0, V=1.0
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinearRainbow {}

const LINEAR_RAINBOW_WRAP: f64 = 32.0; // TODO this might become a parameter later

const BLACK: Hsv<Srgb, f32> = Hsv::new_const(RgbHue::new(0.0), 0.0, 0.0);

#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for LinearRainbow {
    fn colour_hsvf(&self, iters: f64, _: u64) -> Hsv<Srgb, f32> {
        if iters.is_infinite() {
            return BLACK;
        }
        let tau = (iters / LINEAR_RAINBOW_WRAP).fract() as f32;
        // this gives a number from 0..1, map that to the hue angle
        // TODO: offset becomes a parameter?
        let degrees = (0.5 + tau) * 360.0;
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

/// Cycling H; Fixed S=1.0, V=1.0
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogRainbow {}
#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for LogRainbow {
    fn colour_hsvf(&self, iters: f64, _: u64) -> Hsv<Srgb, f32> {
        if iters.is_infinite() {
            return BLACK;
        }
        let degrees = 60.0 * (iters.ln() as f32 + 0.5);
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

/// HSV Gradient function from <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#HSV_coloring>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HsvGradient {}
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
impl OutputsHsvf for HsvGradient {
    fn colour_hsvf(&self, iters: f64, max_iters: u64) -> Hsv<Srgb, f32> {
        if iters.is_infinite() || iters >= (max_iters as f64 - 1.0) {
            return BLACK;
        }
        let proportion = iters as f32 / max_iters as f32;
        // TODO: 0.75 becomes a parameter
        let degrees = (proportion * 360.0).powf(1.5) % 360.0;
        // TODO: value 1.0 becomes a parameter of proportion?
        Hsv::new(RgbHue::new(degrees), 1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use assert_float_eq::{afe_is_f32_near, afe_near_error_msg, assert_f32_near};
    use palette::{rgb::Srgb, IntoColor, Lch, LinSrgb, RgbHue};

    use super::{LinearRainbow, LINEAR_RAINBOW_WRAP};
    use crate::colouring::{OutputsHsvf, Rgb8, Rgbf};

    #[test]
    fn hue_cycles() {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_lossless)]
        let uut = LinearRainbow {};
        // The algorithm operates a linear cycle over the Wrap interval.
        // Therefore we expect it to average out fairly neatly.
        let mut hue_accumulator = 0.0;
        let mut previous = RgbHue::new(f32::NAN);

        for i in 0..(LINEAR_RAINBOW_WRAP as i32) {
            let res = uut.colour_hsvf(i as f64, 256);
            assert_ne!(res.hue, previous);
            previous = res.hue;
            hue_accumulator += res.hue.into_degrees();
        }
        // Figure from current implementation, not critical
        assert_f32_near!(hue_accumulator, 180.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn hsv_sanity() {
        let rgb = Rgb8::new(255, 0, 0);
        let rgb2: palette::rgb::Srgb = Rgbf::from_format(rgb);
        let hsv: palette::Hsv = rgb2.into_color();
        println!("{hsv:?}");
        assert_eq!(hsv.hue, 0.0);
        assert_eq!(hsv.value, 1.0);
        assert_eq!(hsv.saturation, 1.0);
        // This demonstrates that Value and Saturation range from 0.0 to 1.0, as we thought.
    }
}
