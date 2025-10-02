//! Two dimensional array types

#![allow(missing_debug_implementations)]

use glam::UVec2;

#[derive(Clone, Copy)]
/// A read-only view of a two-dimensional array, that uses a borrowed slice (one-dimensional) as storage.
///
/// Note that coordinates are mapped to the array in (y,x) order.
pub struct GridRef<'a, T> {
    /// Dimensions of the underlying buffer
    size: UVec2,
    buffer: &'a [T],
}

impl<'a, T: Copy> GridRef<'a, T> {
    /// Constructs a new `GridRef` of a given size and type.
    ///
    /// The length of storage `buffer` must be at least `size.x * size.y`.
    /// ```
    /// # use shader_util::GridRef;
    /// use glam::uvec2;
    /// let buf = vec![0,0,0,42];
    /// let gr = GridRef::new(uvec2(2,2), &buf);
    /// assert_eq!(gr.get(uvec2(1,1)), 42);
    /// ```
    ///
    /// # Panics
    ///
    /// In a debug build, if the underlying slice is not large enough to hold `size.x * size.y` items.
    /// ```should_panic
    /// # use shader_util::GridRef;
    /// use glam::uvec2;
    /// let buf = vec![0];
    /// let _gr = GridRef::new(uvec2(2,2), &buf);
    /// ```
    pub fn new(size: UVec2, buffer: &'a [T]) -> Self {
        debug_assert!(
            buffer.len() >= (size.x * size.y).try_into().unwrap(),
            "storage not large enough" // rust-analyzer false positive with the rust-gpu toolchain; https://github.com/Rust-GPU/rust-cuda/issues/261 refers
        );
        Self { size, buffer }
    }

    #[must_use]
    /// Accesses a given grid co-ordinate
    ///
    /// # Panics
    ///
    /// If the requested co-ordinates are outside of the underlying storage
    ///
    /// ```should_panic
    /// # use shader_util::GridRef;
    /// use glam::uvec2;
    /// let buf = vec![0,0,0,42];
    /// let gr = GridRef::new(uvec2(2, 2), &buf);
    /// assert_eq!(gr.get(uvec2(3, 3)), 42);
    /// ```
    pub fn get(&self, p: UVec2) -> T {
        self.buffer[(p.y * self.size.x + p.x) as usize]
    }
}

/// A mutable view of a two-dimensional array, that uses a borrowed slice (one-dimensional) as storage.
/// This is the mutable version of [`GridRef`].
pub struct GridRefMut<'a, T> {
    size: UVec2,
    buffer: &'a mut [T],
}

impl<'a, T: Copy> GridRefMut<'a, T> {
    /// Constructs a new `GridRefMut` of a given size and type.
    ///
    /// The length of storage `buffer` must be at least `size.x * size.y`.
    ///
    /// Note that coordinates are mapped to the array in (y,x) order.
    ///
    /// ```
    /// # use shader_util::GridRefMut;
    /// use glam::uvec2;
    /// let mut buf = [42, 43, 44, 45];
    /// let gr = GridRefMut::new(uvec2(2,2), &mut buf);
    /// assert_eq!(gr.get(uvec2(0,1)), 44);
    /// ```
    ///
    /// # Panics
    ///
    /// In a debug build, if the underlying slice is not large enough to hold `size.x * size.y` items.
    ///
    /// ```should_panic
    /// # use shader_util::GridRefMut;
    /// use glam::uvec2;
    /// let mut buf = [0;1];
    /// let _gr = GridRefMut::new(uvec2(2,2), &mut buf);
    /// ```
    pub fn new(size: UVec2, buffer: &'a mut [T]) -> Self {
        debug_assert!(
            buffer.len() >= (size.x * size.y).try_into().unwrap(),
            "storage not large enough" // rust-analyzer false positive with the rust-gpu toolchain; https://github.com/Rust-GPU/rust-cuda/issues/261 refers
        );
        Self { size, buffer }
    }

    /// Creates a read-only copy borrowing this struct's buffer
    ///
    /// ```
    /// # use shader_util::{GridRef, GridRefMut};
    /// use glam::uvec2;
    /// let mut buf = vec![42; 4];
    /// let mut grm = GridRefMut::new(uvec2(2, 2), &mut buf);
    /// grm.set(uvec2(0, 0), 0);
    ///
    /// let gr = grm.as_ref();
    /// assert_eq!(gr.get(uvec2(0, 0)), 0);
    /// // The normal borrowing rules apply:
    /// assert_eq!(grm.get(uvec2(0, 0)), 0);
    /// ```
    #[must_use]
    pub fn as_ref(&self) -> GridRef<'_, T> {
        GridRef::new(self.size, self.buffer)
    }

    #[must_use]
    /// Accesses a given grid co-ordinate
    ///
    /// # Panics
    ///
    /// If the requested co-ordinates are outside of the underlying storage
    pub fn get(&self, p: UVec2) -> T {
        self.buffer[(p.y * self.size.x + p.x) as usize]
    }

    /// Writes an item to a given grid co-ordinate
    ///
    /// # Panics
    ///
    /// If the requested co-ordinates are outside of the underlying storage
    pub fn set(&mut self, p: UVec2, value: T) {
        self.buffer[(p.y * self.size.x + p.x) as usize] = value;
    }

    /// Swaps the values at two locations in the grid
    ///
    /// # Panics
    ///
    /// If either of the requested co-ordinates are outside of the underlying storage
    /// ```
    /// # use shader_util::GridRefMut;
    /// use glam::uvec2;
    /// let mut buf = [0, 1, 2, 3];
    /// let c00 = uvec2(0, 0);
    /// let c01 = uvec2(0, 1);
    /// let mut grid = GridRefMut::new(uvec2(2,2), &mut buf);
    /// assert_eq!(grid.get(c00), 0);
    /// grid.swap(c00, c01);
    /// assert_eq!(grid.get(c00), 2);
    /// ```
    pub fn swap(&mut self, a: UVec2, b: UVec2) {
        // we can't use std::mem::swap here as spirv is `no_std`
        let tmp = self.get(a);
        self.set(a, self.get(b));
        self.set(b, tmp);
    }
}

#[cfg(not(target_arch = "spirv"))]
/// A two-dimensional array that uses an owned [`Vec`] as storage.
/// **Not available in `no_std` (GPU) environments.**
///
/// See also [`GridRef`] and [`GridRefMut`].
pub struct Grid<T> {
    /// Dimensions of the grid
    pub size: UVec2,
    /// Underlying storage
    pub buffer: Vec<T>,
}

#[cfg(not(target_arch = "spirv"))]
impl<T> Grid<T>
where
    T: Default + Clone + Copy,
{
    /// Constructs a new `Grid` of a given size and type.
    ///
    /// The underlying storage `buffer` will be created of the correct size,
    /// with all elements default-initialised.
    #[must_use]
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            buffer: vec![Default::default(); (size.x * size.y) as usize],
        }
    }

    /// Creates a new read-only [`GridRef`] borrowing this struct's storage
    #[must_use]
    pub fn as_ref(&self) -> GridRef<'_, T> {
        GridRef::new(self.size, &self.buffer)
    }

    /// Creates a new mutable [`GridRefMut`] borrowing this struct's storage
    pub fn as_ref_mut(&mut self) -> GridRefMut<'_, T> {
        GridRefMut::new(self.size, &mut self.buffer)
    }

    /// Resizes the grid storage
    ///
    /// Any new elements created are default-initialised.
    ///
    /// The elements are renumbered linearly for the changed dimensions, but bear in mind
    /// that they are mapped in (y,x) coordinate order.
    /// ```
    /// # use shader_util::Grid;
    /// use glam::uvec2;
    /// let c00 = uvec2(0, 0);
    /// let c11 = uvec2(1, 1);
    /// let c22 = uvec2(2, 2);
    /// let mut grid = Grid::new(c11);
    /// grid.set(c00, 42);
    /// grid.resize(c22);
    /// assert_eq!(grid.get(c00), 42);
    /// assert_eq!(grid.get(c11), 0);
    /// ```
    pub fn resize(&mut self, size: UVec2) {
        self.size = size;
        let length = (size.x * size.y) as usize;
        if length > self.buffer.len() {
            self.buffer.resize(length, Default::default());
        }
    }

    /// Accesses a given grid co-ordinate
    ///
    /// # Panics
    ///
    /// If the requested co-ordinates are outside of the underlying storage
    #[must_use]
    pub fn get(&self, p: UVec2) -> T {
        self.as_ref().get(p)
    }

    /// Accesses a given grid co-ordinate
    ///
    /// # Panics
    ///
    /// If the requested co-ordinates are outside of the underlying storage
    pub fn set(&mut self, p: UVec2, value: T) {
        self.as_ref_mut().set(p, value);
    }

    /// Swaps two grid elements
    ///
    /// # Panics
    ///
    /// If either of the requested co-ordinates are outside of the underlying storage
    ///
    /// ```
    /// # use shader_util::Grid;
    /// use glam::uvec2;
    /// let c00 = uvec2(0, 0);
    /// let c01 = uvec2(0, 1);
    /// let mut grid = Grid::new(uvec2(2,2));
    /// grid.set(c01, 2);
    /// assert_eq!(grid.get(c00), 0);
    /// grid.swap(c00, c01);
    /// assert_eq!(grid.get(c00), 2);
    /// ```
    pub fn swap(&mut self, a: UVec2, b: UVec2) {
        self.as_ref_mut().swap(a, b);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::GridRef;

    #[test]
    fn larger_buf_ok() {
        let buf = vec![1, 2, 3, 4, 5];
        let _gr = GridRef::new(glam::uvec2(2, 1), &buf);
    }
}
