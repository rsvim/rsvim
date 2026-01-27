//! VecDeque based fixed-size ringbuf.

use std::collections::VecDeque;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
  data: VecDeque<T>,
  max_size: usize,
}

impl<T> RingBuffer<T> {
  pub fn new(max_size: usize) -> Self {
    Self {
      data: VecDeque::with_capacity(max_size),
      max_size,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  /// Force push back, remove front items if deque is full.
  pub fn push_back_overwrite(&mut self, value: T) {
    while self.data.len() >= self.max_size && !self.data.is_empty() {
      self.data.pop_front();
    }
    self.data.push_back(value)
  }

  /// Try push back, fail and don't remove front items if deque is full.
  pub fn try_push_back(&mut self, value: T) -> Result<(), T> {
    if self.data.len() < self.max_size {
      self.data.push_back(value);
      Ok(())
    } else {
      Err(value)
    }
  }

  pub fn pop_front(&mut self) -> Option<T> {
    self.data.pop_front()
  }

  pub fn pop_back(&mut self) -> Option<T> {
    self.data.pop_back()
  }

  pub fn iter(&'_ self) -> std::collections::vec_deque::Iter<'_, T> {
    self.data.iter()
  }

  pub fn drain<R>(
    &mut self,
    range: R,
  ) -> std::collections::vec_deque::Drain<'_, T>
  where
    R: std::ops::RangeBounds<usize>,
  {
    self.data.drain(range)
  }
}

impl<T> Index<usize> for RingBuffer<T> {
  type Output = usize;

  fn index(&self, index: usize) -> &Self::Output {
    self.data.index(index)
  }
}
