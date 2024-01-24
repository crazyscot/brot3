// Colour space typing - tests only for now
// (c) 2024 Ross Younger

#[cfg(test)]
mod tests {
    use palette::{rgb, IntoColor};

    use crate::colouring::{Hsvf, Rgb8, Rgbf};

    #[test]
    fn red() {
        let hsv = Hsvf::new(0.0, 1.0, 1.0);
        let rgb: Rgbf = hsv.into_color();
        let expected = Rgbf::new(1.0, 0.0, 0.0);
        assert_eq!(rgb, expected);

        let rgb8: Rgb8 = Rgb8::from_format(rgb);
        let packed = rgb8.into_u32::<rgb::channels::Rgba>();
        assert_eq!(0xff00_00ff, packed);
    }
}
