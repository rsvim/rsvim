//! Internal cache for `Text`:
//!
//! - Lines width cache
//! - Cloned lines cache.
//! - Cache hit/miss statistics.

#![allow(dead_code)]

use crate::buf::text::cidx::ColumnIndex;
use crate::prelude::*;
use clru::CLruCache;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Stats {
  hits: usize,
  misses: usize,
}

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

impl std::fmt::Display for Stats {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "hit/miss/total:{}/{}/{},hit ratio:{}",
      self.hits,
      self.misses,
      self.total(),
      self.ratio(),
    ))
  }
}

fn _cached_size(canvas_size: U16Size) -> std::num::NonZeroUsize {
  std::num::NonZeroUsize::new(canvas_size.height() as usize * 3 + 3).unwrap()
}

type LinesWidthCache = CLruCache<usize, ColumnIndex, RandomState>;

// Cached lines width.
pub struct CachedLinesWidth {
  cache: LinesWidthCache,
  stats: Stats,
}

impl CachedLinesWidth {
  pub fn new(canvas_size: U16Size) -> Self {
    let cache_size = _cached_size(canvas_size);
    Self {
      cache: CLruCache::with_hasher(cache_size, RandomState::default()),
      stats: Stats::default(),
    }
  }

  pub fn get_or_insert<F>(&mut self, k: &usize, f: F) -> &mut ColumnIndex
  where
    F: FnOnce() -> ColumnIndex,
  {
    if !self.cache.contains(k) {
      let v = f();
      self.cache.put(*k, v);
      self.stats.miss();
    } else {
      self.stats.hit();
    }

    self.cache.get_mut(k).unwrap()
  }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
struct CachedClonedLinesKey {
  pub line_idx: usize,
  pub start_char_idx: usize,
  pub max_chars: usize,
}

// Cached cloned lines.
pub struct CachedClonedLines {
  cache: RefCell<CLruCache<CachedClonedLinesKey, Rc<String>, RandomState>>,
  stats: RefCell<Stats>,
}
