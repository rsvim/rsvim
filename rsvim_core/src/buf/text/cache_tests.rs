use super::cache::*;

#[cfg(test)]
mod stats {

  #[test]
  fn test1() {
    let mut stats = CacheStats::default();
    assert_eq!(stats.hits(), 0);
    assert_eq!(stats.misses(), 0);
    assert_eq!(stats.total(), 0);
    assert_eq!(stats.ratio(), 0_f32);
    stats.hit();
    stats.miss();
    assert_eq!(stats.hits(), 1);
    assert_eq!(stats.misses(), 1);
    assert_eq!(stats.total(), 2);
    assert_eq!(stats.ratio(), 0.5_f32);
  }
}
