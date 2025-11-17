//! Internal cache for `Text`:
//!
//! - Lines width cache
//! - Cloned lines cache.
//! - Cache hit/miss statistics.

use crate::buf::text::cidx::ColumnIndex;
use crate::prelude::*;
use lru::LruCache;
use std::hash::Hash;
use std::rc::Rc;

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
pub struct Stats {
  hits: usize,
  misses: usize,
}

#[cfg(debug_assertions)]
impl Stats {
  pub fn hit(&mut self) {
    self.hits += 1;
  }

  pub fn miss(&mut self) {
    self.misses += 1;
  }

  pub fn hits(&self) -> usize {
    self.hits
  }

  pub fn misses(&self) -> usize {
    self.misses
  }

  pub fn total(&self) -> usize {
    self.hits + self.misses
  }

  pub fn ratio(&self) -> f32 {
    if self.total() == 0 {
      0_f32
    } else {
      self.hits as f32 / (self.total() as f32)
    }
  }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for Stats {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.total() == 0 {
      Ok(())
    } else {
      f.write_fmt(format_args!(
        "hit/miss/total:{}/{}/{},hit ratio:{}",
        self.hits,
        self.misses,
        self.total(),
        self.ratio(),
      ))
    }
  }
}

fn _cached_size(canvas_size: U16Size) -> std::num::NonZeroUsize {
  std::num::NonZeroUsize::new(canvas_size.height as usize * 3 + 3).unwrap()
}

#[derive(Debug)]
// Internal cache implementation.
pub struct GenericCache<K: Copy + Eq + Hash, V> {
  cache: LruCache<K, V, RandomState>,

  #[cfg(debug_assertions)]
  stats: Stats,
}

impl<K: Copy + Eq + Hash, V> GenericCache<K, V> {
  pub fn new(canvas_size: U16Size) -> Self {
    let cache_size = _cached_size(canvas_size);
    Self {
      cache: LruCache::with_hasher(cache_size, RandomState::default()),

      #[cfg(debug_assertions)]
      stats: Stats::default(),
    }
  }

  #[cfg(debug_assertions)]
  pub fn stats(&self) -> &Stats {
    &self.stats
  }

  fn _get_or_insert_impl<F>(&mut self, k: &K, f: F)
  where
    F: FnOnce() -> Option<V>,
  {
    #[cfg(debug_assertions)]
    {
      if self.cache.contains(k) {
        self.stats.hit();
      } else {
        self.stats.miss();
      }
    }

    if !self.cache.contains(k)
      && let Some(v) = f()
    {
      self.cache.put(*k, v);
    }
  }

  pub fn get_or_insert<F>(&mut self, k: &K, f: F) -> Option<&V>
  where
    F: FnOnce() -> Option<V>,
  {
    self._get_or_insert_impl(k, f);
    self.cache.get(k)
  }

  pub fn get_or_insert_mut<F>(&mut self, k: &K, f: F) -> Option<&mut V>
  where
    F: FnOnce() -> Option<V>,
  {
    self._get_or_insert_impl(k, f);
    self.cache.get_mut(k)
  }

  pub fn resize(&mut self, canvas_size: U16Size) {
    let new_cache_size = _cached_size(canvas_size);
    if new_cache_size > self.cache.cap() {
      self.cache.resize(new_cache_size);
    }
  }

  pub fn clear(&mut self) {
    self.cache.clear();
  }

  pub fn retain<F>(&mut self, f: F)
  where
    F: Fn(/* key */ &K) -> bool,
  {
    let to_be_removed: Vec<K> = self
      .cache
      .iter()
      .filter(|(key, _)| !f(key))
      .map(|(key, _)| *key)
      .collect();
    for key in to_be_removed.iter() {
      self.cache.pop(key);
    }
  }
}

/// Cached lines width.
///
/// Key `line_idx` => Value `ColumnIndex`.
pub type CachedWidth = GenericCache<usize, ColumnIndex>;

/// Cache key for `CachedLines`.
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct CachedLinesKey {
  pub line_idx: usize,
  pub start_char_idx: usize,
  pub max_chars: usize,
}

/// Cached cloned lines.
///
/// Key `CachedLinesKey` (line_idx, start_char_idx, max_chars) => Value `Rc<String>`.
pub type CachedLines = GenericCache<CachedLinesKey, Rc<String>>;
