//! VecDeque based fixed-size ringbuf.

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
  dq: VecDeque<T>,
  max_size: usize,
}

impl<T> RingBuffer<T> {
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

  /// Force push back, remove front items if deque is full.
  pub fn push_back_overwrite(&mut self, value: T) {
    while self.dq.len() >= self.max_size && !self.dq.is_empty() {
      self.dq.pop_front();
    }
    self.dq.push_back(value)
  }

  /// Try push back, fail and don't remove front items if deque is full.
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

  pub fn pop_back(&mut self) -> Option<T> {
    self.dq.pop_back()
  }

  pub fn iter(&'_ self) -> std::collections::vec_deque::Iter<'_, T> {
    self.dq.iter()
  }

  pub fn drain<R>(
    &mut self,
    range: R,
  ) -> std::collections::vec_deque::Drain<'_, T>
  where
    R: std::ops::RangeBounds<usize>,
  {
    self.dq.drain(range)
  }
}
