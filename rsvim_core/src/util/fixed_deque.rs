//! VecDeque based fixed-size ringbuf, it provides more Vec APIs than the
//! `ringbuf` crate.

use crate::prelude::*;
use std::collections::VecDeque;

pub struct FixedDeque<T> {
  dq: VecDeque<T>,
  max_size: usize,
}

impl<T> FixedDeque<T> {
  pub fn new(size: usize) -> Self {
    Self {
      dq: VecDeque::with_capacity(size),
      max_size: size,
    }
  }

  /// Push back
  pub fn push_back_overwrite(&mut self, value: T) {
    while self.dq.len() >= self.max_size && !self.dq.is_empty() {
      self.dq.pop_front();
    }
    self.dq.push_back(value)
  }

  pub fn try_push_back(&mut self, value: T) -> TheResult<()> {
    if self.dq.len() < self.max_size {
      self.dq.push_back(value);
      Ok(())
    } else {
      Err(TheErr::FixedDequeTryPushFailed)
    }
  }

  pub fn pop_front(&mut self) -> Option<T> {
    self.dq.pop_front()
  }
}
