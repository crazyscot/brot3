// (c) Ross Younger 2024

use std::fmt::{self, Display};
use std::ops::Add;
use std::str::FromStr;

/// A rectangle of some sort
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Rect<T> {
    /// Width of the thing
    pub width: T,
    /// Height of the thing
    pub height: T,
}

impl<T: Copy> Rect<T>
where
    f64: From<T>, // this means you can't have a Size<usize> on x86_64
{
    /// Computing accessor
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.width) / f64::from(self.height)
    }
}

impl<T: Copy> Rect<T> {
    /// Constructor
    pub fn new(width: T, height: T) -> Rect<T> {
        Self { width, height }
    }
}

impl<T: Display> Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "(w={} h={})", self.width, self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseSizeError;

impl<T: FromStr> FromStr for Rect<T> {
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

impl<T: std::ops::Add<Output = T>> Add for Rect<T> {
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
    use approx::assert_relative_eq;

    use super::Rect;
    use std::str::FromStr;

    #[test]
    fn aspect() {
        #![allow(clippy::float_cmp)]
        assert_eq!(Rect::new(200, 100).aspect_ratio(), 2.0);
        assert_eq!(Rect::new(100, 100).aspect_ratio(), 1.0);
        assert_eq!(Rect::new(100, 200).aspect_ratio(), 0.5);
    }

    #[test]
    fn parse_int() {
        let t = Rect::<u32>::from_str("(123,456)").unwrap();
        assert_eq!(t.width, 123);
        assert_eq!(t.height, 456);
    }
    #[test]
    fn parse_fail() {
        assert!(Rect::<u32>::from_str("(123 ,456)").is_err());
        assert!(Rect::<u32>::from_str("(123, 456)").is_err());
        assert!(Rect::<u32>::from_str("123,456)").is_err());
        assert!(Rect::<u32>::from_str("(123,456").is_err());
        assert!(Rect::<u32>::from_str("(123456)").is_err());
        assert!(Rect::<u32>::from_str("(12,34,56)").is_err());
        assert!(Rect::<u32>::from_str("(123banana,456)").is_err());
    }
    #[test]
    fn parse_float() {
        let t = Rect::<f64>::from_str("(2.0,4.0)").unwrap();
        assert_relative_eq!(t.width, 2.0);
        assert_relative_eq!(t.height, 4.0);

        let u = Rect::<f64>::from_str("(inf,nan)").unwrap();
        assert!(u.width.is_infinite());
        assert!(u.height.is_nan());
    }
    #[test]
    fn add() {
        let s = Rect::new(12, 34) + Rect::new(56, 78);
        assert_eq!(s.width, 12 + 56);
        assert_eq!(s.height, 34 + 78);
    }
}
