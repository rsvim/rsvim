//! The VIM buffer's view.

#![allow(dead_code)]

use crate::cart::URect;

#[derive(Copy, Clone, Debug)]
/// The view of the VIM buffer.
///
/// When VIM buffer shows in VIM window, it starts and ends at specific lines and columns.
pub struct BufferView {
  /// Top-left corner: `shape.min()`.
  /// Bottom-right corner: `shape.max()`.
  ///
  /// Start line number: `shape.min().y`
  /// End line number: `shape.max().y`
  /// Start column: `shape.min().x`
  /// End column: `shape.max().x`
  shape: URect,
}

impl BufferView {
  pub fn new(shape: URect) -> Self {
    BufferView { shape }
  }

  pub fn lstart(&self) -> usize {
    self.shape.min().y
  }

  pub fn set_lstart(&mut self, lstart: usize) {
    self.shape.set_min((self.shape.min().x, lstart))
  }
}

impl PartialEq for BufferView {
  fn eq(&self, other: &Self) -> bool {
    self.shape == other.shape
  }
}

impl Eq for BufferView {}

#[cfg(test)]
mod tests {
  use super::*;
}
