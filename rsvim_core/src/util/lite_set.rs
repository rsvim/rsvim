//! LiteSet

use litemap::LiteMap;
use std::borrow::Borrow;
use std::cmp::Ord;

#[derive(Default, Clone)]
pub struct LiteSet<V> {
  data: LiteMap<V, bool>,
}

impl<V> LiteSet<V> {
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

  pub fn first(&self) -> Option<&V> {
    self.data.first().map(|(k, _)| k)
  }

  pub fn last(&self) -> Option<&V> {
    self.data.last().map(|(k, _)| k)
  }

  pub fn contains_key<Q>(&self, key: &Q) -> bool
  where
    V: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
  {
    self.data.contains_key(key)
  }

  pub fn clear(&mut self) {
    self.data.clear()
  }

  pub fn reserve(&mut self, additional: usize) {
    self.data.reserve(additional)
  }

  pub fn insert(&mut self, value: V) -> bool
  where
    V: Ord,
  {
    self.data.insert(value, true).is_none()
  }

  pub fn remove(&mut self, value: &V) -> bool
  where
    V: Ord,
  {
    self.data.remove(value).is_none()
  }
}
