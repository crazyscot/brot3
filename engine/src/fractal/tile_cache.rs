// Tile data cache
// (c) 2024 Ross Younger

use std::sync::Arc;

use quick_cache::sync::Cache;

use super::{Tile, TileSpec};

type KeyT = TileSpec;
type ValueT = Arc<Tile>;

/// A cache for tiles, keyed by their specs.
/// This is a thin wrapper to `quick_cache`.
/// Keys are `TileSpec`. Values are `Tile` (wrapped in `Arc`).
#[derive(Debug)]
pub struct TileCache {
    cache: Cache<KeyT, ValueT>,
}

impl TileCache {
    /// Creates a new cache
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Cache::new(capacity),
        }
    }

    /// Empties out the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Insert a new value. Note that the key need not be specified, as `Tile` already contains a copy of its `TileSpec`.
    pub fn insert(&self, value: Tile) {
        if value.max_iter_plotted != value.spec.max_iter_requested() {
            return; // Not valid to cache
        }
        self.cache.insert(value.spec.clone(), value.into());
    }

    /// Retrieves an item from the cache
    #[must_use]
    pub fn get(&self, key: &KeyT) -> Option<ValueT> {
        self.cache.get(key)
    }
    /// Retrieves an item from the cache without updating the LRU records
    #[must_use]
    pub fn peek(&self, key: &KeyT) -> Option<ValueT> {
        self.cache.peek(key)
    }

    /// Removes an item from the cache
    #[must_use]
    pub fn remove(&self, key: &KeyT) -> Option<(KeyT, ValueT)> {
        self.cache.remove(key)
    }

    /// Replaces an item with another
    pub fn replace(&self, key: KeyT, value: ValueT, soft: bool) -> Result<(), (KeyT, ValueT)> {
        self.cache.replace(key, value, soft)
    }

    /// Tests whether the cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Queries the number of items currently in cache
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        colouring,
        fractal::{self, Location, Point, Size, Tile, TileSpec},
        util::Rect,
    };

    use super::TileCache;

    fn test_tilespec(frac: fractal::Selection, col: colouring::Selection) -> TileSpec {
        TileSpec::new(
            Location::Origin(Point { re: 0.0, im: -0.5 }),
            Size::AxesLength(Point { re: -1.0, im: 2.0 }),
            Rect::new(200, 400),
            fractal::factory(frac),
            32,
            colouring::factory(col),
        )
    }

    #[test]
    fn cache_works() {
        let uut = TileCache::new(10);
        let spec1 = test_tilespec(
            fractal::Selection::Mandel3,
            colouring::Selection::LinearRainbow,
        );
        let mut tile1 = Tile::new(&spec1, 0);
        tile1.plot();

        let spec2 = test_tilespec(
            fractal::Selection::Original,
            colouring::Selection::LinearRainbow,
        );
        let mut tile2 = Tile::new(&spec2, 0);
        tile2.plot();

        // Cache starts empty
        assert_eq!(uut.len(), 0);
        assert!(uut.get(&spec1).is_none());

        // Multiple inserts with the same key only insert once
        let tile1a = tile1.clone();
        uut.insert(tile1);
        assert_eq!(uut.len(), 1);
        uut.insert(tile1a);
        assert_eq!(uut.len(), 1);

        // A different key does insert
        uut.insert(tile2);
        assert_eq!(uut.len(), 2);
    }

    #[test]
    fn no_cache_unplotted() {
        let uut = TileCache::new(10);
        let spec1 = test_tilespec(
            fractal::Selection::Original,
            colouring::Selection::LinearRainbow,
        );
        let tile1 = Tile::new(&spec1, 0);
        assert_eq!(uut.len(), 0);
        uut.insert(tile1);
        assert_eq!(uut.len(), 0);
    }
}
