//! VecDeque based fixed-size ringbuf, it provides more Vec APIs than the
//! `ringbuf` crate.

use std::collections::VecDeque;

pub struct FixedDeque<T> {
  dq: VecDeque<T>,
  size: usize,
}

impl<T> FixedDeque<T> {
  pub fn new(size: usize) -> Self {
    Self {
      dq: VecDeque::with_capacity(size),
      size,
    }
  }
}
