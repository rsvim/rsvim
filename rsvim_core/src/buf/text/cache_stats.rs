//! Cache hit/miss status.

#[derive(Debug, Default)]
struct CacheStats {
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
}

impl std::fmt::Display for CacheStats {
  fn to_string(&self) -> String {
    format!(
      "hit/miss/total:{}/{}/{},hit ratio:{}",
      self.hits,
      self.misses,
      self.total(),
      if self.total() == 0 {
        0.0_f32
      } else {
        self.hits as f32 / (self.total() as f32)
      }
    )
  }
}
