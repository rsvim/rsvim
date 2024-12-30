//! Chars and display width index (line-wise) for vim buffer.

use crate::buf::Buffer;
//use ropey::{Rope, RopeBuilder, RopeSlice};

#[derive(Debug, Clone)]
/// Chars and display width index (line-wise) for vim buffer.
///
/// NOTE: For each line, the char/column index starts from 0.
pub struct BufCindex {
  // Char index maps to its accumulate display width, i.e. from the first char/column (0) to
  // current char/column, not just the current char's display width.
  char2width: Vec<usize>,
}

impl BufCindex {
  /// Create and initialize index for the line.
  ///
  /// # Panics
  ///
  /// It panics if the line doesn't exist in the rope.
  pub fn new(buf: &Buffer, line_idx: usize) -> Self {
    let rope_slice = buf.rope.get_line(line_idx).unwrap();
    let mut char2width: Vec<usize> = Vec::new();
    for (i, c) in rope_slice.chars().enumerate() {}
    Self {
      char2width: Vec::new(),
    }
  }

  #[cfg(not(debug_assertions))]
  pub fn _internal_check(&self) {}

  #[cfg(debug_assertions)]
  pub fn _internal_check(&self) {}

  pub fn is_empty(&self) -> bool {
    self.char2width.is_empty()
  }

  pub fn len(&self) -> usize {
    self.char2width.len()
  }
}
