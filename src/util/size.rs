// (c) Ross Younger 2024

use std::ops::Add;
use std::str::FromStr;

/// Co-ordinate pair describing the size of something
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size<T> {
    /// Width of the thing
    pub width: T,
    /// Height of the thing
    pub height: T,
}

impl<T: Copy> Size<T>
where
    f64: From<T>,
{
    /// Computing accessor
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.width) / f64::from(self.height)
    }

    /// Constructor
    pub fn new(width: T, height: T) -> Size<T> {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseSizeError;

impl<T: FromStr> FromStr for Size<T> {
    type Err = ParseSizeError;

    fn from_str(item: &str) -> Result<Self, Self::Err> {
        let (w, h) = item
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(ParseSizeError)?;
        let width = w.parse::<T>().map_err(|_| ParseSizeError)?;
        let height = h.parse::<T>().map_err(|_| ParseSizeError)?;
        Ok(Self { width, height })
    }
}

impl<T: std::ops::Add<Output = T>> Add for Size<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};

    use super::Size;
    use std::str::FromStr;

    #[test]
    fn aspect() {
        #![allow(clippy::float_cmp)]
        assert_eq!(Size::new(200, 100).aspect_ratio(), 2.0);
        assert_eq!(Size::new(100, 100).aspect_ratio(), 1.0);
        assert_eq!(Size::new(100, 200).aspect_ratio(), 0.5);
    }

    #[test]
    fn parse_int() {
        let t = Size::<u32>::from_str("(123,456)").unwrap();
        assert_eq!(t.width, 123);
        assert_eq!(t.height, 456);
    }
    #[test]
    fn parse_fail() {
        assert!(Size::<u32>::from_str("(123 ,456)").is_err());
        assert!(Size::<u32>::from_str("(123, 456)").is_err());
        assert!(Size::<u32>::from_str("123,456)").is_err());
        assert!(Size::<u32>::from_str("(123,456").is_err());
        assert!(Size::<u32>::from_str("(123456)").is_err());
        assert!(Size::<u32>::from_str("(12,34,56)").is_err());
        assert!(Size::<u32>::from_str("(123banana,456)").is_err());
    }
    #[test]
    fn parse_float() {
        let t = Size::<f64>::from_str("(2.0,4.0)").unwrap();
        assert_f64_near!(t.width, 2.0);
        assert_f64_near!(t.height, 4.0);

        let u = Size::<f64>::from_str("(inf,nan)").unwrap();
        assert!(u.width.is_infinite());
        assert!(u.height.is_nan());
    }
    #[test]
    fn add() {
        let s = Size::new(12, 34) + Size::new(56, 78);
        assert_eq!(s.width, 12 + 56);
        assert_eq!(s.height, 34 + 78);
    }
}
