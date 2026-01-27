//! VecDeque based fixed-size ringbuf, it provides more Vec APIs than the
//! `ringbuf` crate.

use std::collections::VecDeque;

pub struct FixedDeque<T> {
  dq: VecDeque<T>,
  max_size: usize,
}

impl<T> FixedDeque<T> {
  pub fn new(max_size: usize) -> Self {
    Self {
      dq: VecDeque::with_capacity(max_size),
      max_size,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.dq.is_empty()
  }

  pub fn len(&self) -> usize {
    self.dq.len()
  }

  /// Push back
  pub fn push_back_overwrite(&mut self, value: T) {
    while self.dq.len() >= self.max_size && !self.dq.is_empty() {
      self.dq.pop_front();
    }
    self.dq.push_back(value)
  }

  pub fn try_push_back(&mut self, value: T) -> Result<(), T> {
    if self.dq.len() < self.max_size {
      self.dq.push_back(value);
      Ok(())
    } else {
      Err(value)
    }
  }

  pub fn pop_front(&mut self) -> Option<T> {
    self.dq.pop_front()
  }

  pub fn iter(&'_ self) -> std::collections::vec_deque::Iter<'_, T> {
    self.dq.iter()
  }
}
