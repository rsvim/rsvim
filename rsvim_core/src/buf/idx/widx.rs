//! Display width index (line-wise) for each unicode char in vim buffer.

use crate::buf::unicode;
use crate::buf::Buffer;
//use ropey::{Rope, RopeBuilder, RopeSlice};

#[derive(Debug, Clone)]
/// Display width index (line-wise) for each unicode char in vim buffer. For each line, the
/// char/column index starts from 0.
///
/// NOTE: This structure is wrapped on a [`Vec`], all APIs follow the standard [`Vec`]'s API
/// design.
pub struct BufWindex {
  // Char index maps to its accumulate display width, i.e. from the first char/column (0) to
  // current char/column, not just the current char's display width.
  char2width: Vec<usize>,
}

impl BufWindex {
  /// Create and initialize index for the line.
  ///
  /// # Panics
  ///
  /// It panics if the line doesn't exist in the rope.
  pub fn new(buf: &Buffer, line_idx: usize) -> Self {
    let rope_slice = buf.rope.get_line(line_idx).unwrap();
    let mut char2width: Vec<usize> = Vec::with_capacity(rope_slice.len_chars());
    let mut width = 0_usize;
    for (_i, c) in rope_slice.chars().enumerate() {
      char2width.push(width);
      width += unicode::char_width(&buf.options, c);
    }
    Self { char2width }
  }

  #[cfg(not(debug_assertions))]
  pub fn _internal_check(&self) {}

  #[cfg(debug_assertions)]
  pub fn _internal_check(&self) {
    let mut last_width: Option<usize> = None;
    for (i, w) in self.char2width.iter().enumerate() {
      if i == 0 {
        assert!(self.char2width[0] == 0);
      }
      match last_width {
        Some(width_value) => {
          assert!(*w >= width_value);
        }
        None => { /* Skip */ }
      }
      last_width = Some(*w);
    }
  }

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.char2width.is_empty()
  }

  pub fn len(&self) -> usize {
    self._internal_check();
    self.char2width.len()
  }

  /// Get the display width starts from the first char 0.
  ///
  /// NOTE: This is equivalent to `get_width_between(0..=char_idx)`.
  pub fn width(&self, char_idx: usize) -> usize {
    self._internal_check();
    assert!(char_idx < self.char2width.len());
    self.char2width[char_idx]
  }

  /// Get the display width between inclusive range, i.e. [a, b].
  pub fn width_between(&self, char_idx_range: std::ops::RangeInclusive<usize>) -> usize {
    self._internal_check();
    let c_start = *char_idx_range.start();
    let c_end = *char_idx_range.end();
    assert!(c_start < self.char2width.len());
    assert!(c_end <= self.char2width.len());
    assert!(self.char2width[c_start] <= self.char2width[c_end]);
    self.char2width[c_start] - self.char2width[c_end]
  }

  pub fn update(&mut self) {}

  pub fn update_between(&mut self) {}

  pub fn splice(&mut self) {}

  pub fn truncate(&self) {}
}
