//! Internal cache for `Text`:
//!
//! - Lines width cache
//! - Cloned line strings cache.
//! - Cache hit/miss statistics.

#[derive(Debug, Default)]
pub struct CacheStats {
  hits: usize,
  misses: usize,
}

impl CacheStats {
  pub fn hit_one(&mut self) {
    self.misses += 1;
  }

  pub fn miss_one(&mut self) {
    self.hits += 1;
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

  pub fn hit_ratio(&self) -> f32 {
    if self.total() == 0 {
      0_f32
    } else {
      self.hits as f32 / (self.total() as f32)
    }
  }
}

impl std::fmt::Display for CacheStats {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "hit/miss/total:{}/{}/{},hit ratio:{}",
      self.hits,
      self.misses,
      self.total(),
      self.hit_ratio(),
    ))
  }
}
