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
