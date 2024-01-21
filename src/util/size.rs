// (c) Ross Younger 2024

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

#[cfg(test)]
mod tests {
    use super::Size;

    #[test]
    fn aspect() {
        #![allow(clippy::float_cmp)]
        assert_eq!(Size::new(200, 100).aspect_ratio(), 2.0);
        assert_eq!(Size::new(100, 100).aspect_ratio(), 1.0);
        assert_eq!(Size::new(100, 200).aspect_ratio(), 0.5);
    }
}
