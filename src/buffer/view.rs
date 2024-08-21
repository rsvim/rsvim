//! The VIM buffer's view.

#![allow(dead_code)]

use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::buffer::{Buffer, BufferId};

/// The view of the VIM buffer.
///
/// When VIM buffer shows in VIM window, it starts and ends at specific lines and columns.
///
/// There are two options related to the view:
/// [line-wrap and word-wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap), so we have 4
/// kinds of views.
///
/// * [`BothWrappedBufferView`]: Both line-wrap and word-wrap enabled.
/// * [`LineWrappedBufferView`]: Line-wrap enabled and word-wrap disabled.
/// * [`WordWrappedBufferView`]: Line-wrap disabled and word-wrap enabled.
/// * [`NoneWrappedBufferView`]: Both Line-wrap and word-wrap disabled.
///
/// For the first 3 kinds of view, when a window that has `X` rows height, it may contains less
/// than `X` lines for a buffer. Because very long lines or words can take extra spaces and
/// trigger line breaks. The real lines the window can contain needs a specific algorithm to
/// calculate.
pub trait BufferView: Sized + std::fmt::Debug + Copy + Clone + PartialEq + Eq {
  /// Get the buffer reference.
  fn reference(&self) -> &Buffer;

  /// Get the mutable buffer reference.
  fn reference_mut(&mut self) -> &mut Buffer;

  /// Get start line.
  fn lstart(&self) -> usize;
  /// Set start line.
  fn set_lstart(&mut self, lstart: usize);

  /// Get end line.
  fn lend(&self) -> usize;
  /// Set end line.
  fn set_lend(&mut self, lend: usize);

  /// Get start column.
  fn cstart(&self) -> usize;
  /// Set start column.
  fn set_cstart(&mut self, cstart: usize);

  /// Get end column.
  fn cend(&self) -> usize;
  /// Set end column.
  fn set_cend(&mut self, cend: usize);
}

#[derive(Copy, Clone, Debug)]
/// Base buffer view.
struct BufferViewBase<'a> {
  // Buffer reference
  pub buffer: NonNull<Buffer>,
  pub phantom: PhantomData<&'a mut Buffer>,
  // Start line
  pub lstart: usize,
  // End line
  pub lend: usize,
  // Start column
  pub cstart: usize,
  // End column
  pub cend: usize,
}

impl<'a> BufferViewBase<'a> {
  pub fn new(
    buffer: &'a mut Buffer,
    lstart: usize,
    lend: usize,
    cstart: usize,
    cend: usize,
  ) -> Self {
    BufferViewBase {
      buffer: NonNull::new(buffer as *mut Buffer).unwrap(),
      phantom: PhantomData,
      lstart,
      lend,
      cstart,
      cend,
    }
  }
}

impl<'a> PartialEq for BufferViewBase<'a> {
  fn eq(&self, other: &Self) -> bool {
    unsafe {
      self.buffer.as_ref().id() == other.buffer.as_ref().id()
        && self.lstart == other.lstart
        && self.lend == other.lend
        && self.cstart == other.cstart
        && self.cend == other.cend
    }
  }
}

impl<'a> Eq for BufferViewBase<'a> {}

#[derive(Copy, Clone, Debug)]
pub struct BothWrappedBufferView<'a> {
  base: BufferViewBase<'a>,
}

impl<'a> BothWrappedBufferView<'a> {
  pub fn new(
    buffer: &'a mut Buffer,
    lstart: usize,
    lend: usize,
    cstart: usize,
    cend: usize,
  ) -> Self {
    BothWrappedBufferView {
      base: BufferViewBase::new(buffer, lstart, lend, cstart, cend),
    }
  }
}

impl<'a> BufferView for BothWrappedBufferView<'a> {
  /// Get buffer reference.
  fn reference(&self) -> &Buffer {
    unsafe { self.base.buffer.as_ref() }
  }

  /// Get mutable buffer reference.
  fn reference_mut(&mut self) -> &mut Buffer {
    unsafe { self.base.buffer.as_mut() }
  }

  /// Get start line.
  fn lstart(&self) -> usize {
    self.base.lstart
  }

  fn set_lstart(&mut self, lstart: usize) {}

  /// Get end line.
  fn lend(&self) -> usize {
    self.base.lend
  }

  fn set_lend(&mut self, lend: usize) {}

  /// Get start column.
  fn cstart(&self) -> usize {
    self.base.cstart
  }

  fn set_cstart(&mut self, cstart: usize) {}

  /// Get end column.
  fn cend(&self) -> usize {
    self.base.cend
  }

  fn set_cend(&mut self, cend: usize) {}
}

impl<'a> PartialEq for BothWrappedBufferView<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.base == other.base
  }
}

impl<'a> Eq for BothWrappedBufferView<'a> {}

#[derive(Copy, Clone, Debug)]
pub struct LineWrappedBufferView<'a> {
  base: BufferViewBase<'a>,
}

#[derive(Copy, Clone, Debug)]
pub struct WordWrappedBufferView {
  base: BufferViewBase,
}

#[derive(Copy, Clone, Debug)]
pub struct NoneWrappedBufferView {
  base: BufferViewBase,
}

#[cfg(test)]
mod tests {
  use super::*;
}
