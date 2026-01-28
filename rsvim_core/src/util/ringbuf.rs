//! VecDeque based fixed-size ringbuf.

use std::collections::VecDeque;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Debug, Clone)]
/// Single-ended Ring buffer.
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

  pub fn occupied_len(&self) -> usize {
    self.data.len()
  }

  pub fn vacant_len(&self) -> usize {
    self.max_size
  }

  /// Force push, remove front items if ring buffer is full.
  pub fn push_overwrite(&mut self, value: T) {
    while self.data.len() >= self.max_size && !self.data.is_empty() {
      self.data.pop_front();
    }
    self.data.push_back(value)
  }

  /// Try push, fail and don't remove front items if ring buffer is full.
  pub fn try_push(&mut self, value: T) -> Result<(), T> {
    if self.data.len() < self.max_size {
      self.data.push_back(value);
      Ok(())
    } else {
      Err(value)
    }
  }

  pub fn iter(&'_ self) -> std::collections::vec_deque::Iter<'_, T> {
    self.data.iter()
  }

  pub fn first(&mut self) -> Option<T> {
    self.data.pop_front()
  }

  pub fn first_mut(&mut self) -> Option<T> {
    self.data.pop_front()
  }

  pub fn last(&self) -> Option<&T> {
    self.data.back()
  }

  pub fn last_mut(&mut self) -> Option<&mut T> {
    self.data.back_mut()
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
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    self.data.index(index)
  }
}

impl<T> IndexMut<usize> for RingBuffer<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    self.data.index_mut(index)
  }
}

#[derive(Debug, Clone)]
/// Double-ended ring buffer.
pub struct DeRingBuffer<T> {
  data: VecDeque<T>,
  max_size: usize,
}

impl<T> DeRingBuffer<T> {
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

  pub fn max_size(&self) -> usize {
    self.max_size
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

  pub fn back(&self) -> Option<&T> {
    self.data.back()
  }

  pub fn back_mut(&mut self) -> Option<&mut T> {
    self.data.back_mut()
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

impl<T> Index<usize> for DeRingBuffer<T> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    self.data.index(index)
  }
}

impl<T> IndexMut<usize> for DeRingBuffer<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    self.data.index_mut(index)
  }
}
