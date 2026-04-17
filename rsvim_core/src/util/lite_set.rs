//! LiteSet

use litemap::LiteMap;
use std::borrow::Borrow;
use std::cmp::Ord;

#[derive(Default, Clone)]
pub struct LiteSet<K, V> {
  data: LiteMap<K, V>,
}

impl<K, V> LiteSet<K, V> {
  pub fn new() -> Self {
    Self {
      data: LiteMap::new(),
    }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      data: LiteMap::with_capacity(capacity),
    }
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn first(&self) -> Option<(&K, &V)> {
    self.data.first()
  }

  pub fn last(&self) -> Option<(&K, &V)> {
    self.data.last()
  }

  pub fn get<Q>(&self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
  {
    self.data.get(key)
  }

  pub fn contains_key<Q>(&self, key: &Q) -> bool
  where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
  {
    self.data.contains_key(key)
  }
}
