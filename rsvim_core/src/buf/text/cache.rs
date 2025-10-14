//! Internal cache for `Text`:
//!
//! - Lines width cache
//! - Cloned lines cache.
//! - Cache hit/miss statistics.

use crate::prelude::*;
pub use cidx::ColumnIndex;
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

// Lines width cache.
pub struct LinesWidthCache {
  cache: RefCell<CLruCache<usize, ColumnIndex, RandomState>>,
  stats: RefCell<Stats>,
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
struct ClonedLinesCacheKey {
  pub line_idx: usize,
  pub start_char_idx: usize,
  pub max_chars: usize,
}

// Cloned lines cache.
pub struct ClonedLinesCache {
  cache: RefCell<CLruCache<ClonedLinesCacheKey, Rc<String>, RandomState>>,
  stats: RefCell<Stats>,
}
