// Direct-to-RGB colouring functions
// (c) 2024 Ross Younger

use crate::fractal::maths;

use super::{OutputsRgb8, Rgb8};

const BLACK: Rgb8 = Rgb8::new(0, 0, 0);
const WHITE: Rgb8 = Rgb8::new(255, 255, 255);

const ITERS_CLAMP_EPSILON: f64 = 0.000_01;

// /////////////////////////////////////////////////////////////
/// The colouring algorithm from rjk's ``mandy``
/// `https://github.com/ewxrjk/mandy/blob/master/lib/Color.h`
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

// /////////////////////////////////////////////////////////////
/// fanf's White Fade algorithm
/// `https://dotat.at/cgi/git/mandelbrot.git/blob/HEAD:/mandel2ppm.c`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WhiteFade {}

impl OutputsRgb8 for WhiteFade {
    fn colour_rgb8(&self, iters: f64) -> Rgb8 {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_sign_loss)]
        if iters < ITERS_CLAMP_EPSILON {
            return WHITE;
        }
        if iters.is_infinite() {
            return BLACK;
        }
        let iters = iters.ln();
        if iters < 0.0 {
            return WHITE;
        }
        Rgb8::new(
            (255.0 * 0.5 * (1.0 + (iters * 2.0).cos())) as u8,
            (255.0 * 0.5 * (1.0 + (iters * 1.5).cos())) as u8,
            (255.0 * 0.5 * (1.0 + (iters * 1.0).cos())) as u8,
        )
    }
}

/// fanf's Black Fade algorithm
/// `https://dotat.at/cgi/git/mandelbrot.git/blob/HEAD:/mandel2ppm.c`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlackFade {}

impl OutputsRgb8 for BlackFade {
    fn colour_rgb8(&self, iters: f64) -> Rgb8 {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_sign_loss)]
        if iters < ITERS_CLAMP_EPSILON {
            return BLACK;
        }
        if iters.is_infinite() {
            return BLACK;
        }
        let iters = iters.ln();
        if iters < 0.0 {
            return BLACK;
        }
        Rgb8::new(
            (255.0 * 0.5 * (1.0 - (iters * 1.0).cos())) as u8,
            (255.0 * 0.5 * (1.0 - (iters * 2.0).cos())) as u8,
            (255.0 * 0.5 * (1.0 - (iters * 3.0).cos())) as u8,
        )
    }
}

/// fanf's Monochrome Shade algorithm
/// `https://dotat.at/cgi/git/mandelbrot.git/blob/HEAD:/mandel2ppm.c`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Monochrome {}

impl OutputsRgb8 for Monochrome {
    fn colour_rgb8(&self, iters: f64) -> Rgb8 {
        #![allow(clippy::cast_possible_truncation)]
        #![allow(clippy::cast_sign_loss)]
        if iters < ITERS_CLAMP_EPSILON {
            return BLACK;
        }
        if iters < maths::E {
            return WHITE;
        }
        let shade = (255.0 / iters.ln()) as u8;
        Rgb8::new(shade, shade, shade)
    }
}

// /////////////////////////////////////////////////////////////

/// Colouring algorithm by `OneLoneCoder.com`
/// `https://github.com/OneLoneCoder/Javidx9/blob/master/PixelGameEngine/SmallerProjects/OneLoneCoder_PGE_Mandelbrot.cpp`

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OneLoneCoder {}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl OutputsRgb8 for OneLoneCoder {
    fn colour_rgb8(&self, iters: f64) -> Rgb8 {
        if iters.is_infinite() {
            //return BLACK;
        }

        Rgb8::new(
            (128.0 + 127.0 * (0.1 * iters).sin()) as u8,
            (128.0 + 127.0 * (0.1 * iters + 2.094).sin()) as u8,
            (128.0 + 127.0 * (0.1 * iters + 4.188).sin()) as u8,
        )
    }
}

// /////////////////////////////////////////////////////////////
