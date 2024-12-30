//! Display width index (line-wise) for each unicode char in vim buffer.

use crate::buf::unicode;
use crate::buf::Buffer;
//use ropey::{Rope, RopeBuilder, RopeSlice};

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
/// Display width index (line-wise) for each unicode char in vim buffer. For each line, the
/// char/column index starts from 0.
///
/// This structure is mostly like a prefix-sum tree structure.
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
    let char2width = buf
      .rope
      .get_line(line_idx)
      .unwrap()
      .chars()
      .scan(0_usize, |acc, c| {
        let width = *acc + unicode::char_width(&buf.options, c);
        *acc = width;
        Some(width)
      })
      .collect::<Vec<usize>>();
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

  /// Update a specific char's width, and re-calculate all display width since this char.
  ///
  /// NOTE: This operation is `O(N)`, where `N` is the chars count of current line.
  pub fn update(&mut self, char_idx: usize, width: usize) {
    self._internal_check();
    assert!(char_idx < self.char2width.len());
    if width > self.char2width[char_idx] {
      let diff = width - self.char2width[char_idx];
      for (_i, w) in self.char2width.iter_mut().skip(char_idx).enumerate() {
        *w += diff;
      }
    } else if width < self.char2width[char_idx] {
      let diff = self.char2width[char_idx] - width;
      for (_i, w) in self.char2width.iter_mut().skip(char_idx).enumerate() {
        *w -= diff;
      }
    }
  }

  /// Update a range of chars and their width, and re-calculate all display width since the first
  /// char in the range.
  ///
  /// NOTE: This operation is `O(N)`, where `N` is the chars count of current line.
  pub fn update_between(&mut self, char2width: &BTreeMap<usize, usize>) {}

  /// Push (append) a specific char's width.
  ///
  /// NOTE: This operation is `O(1)`.
  pub fn push(&mut self, width: usize) {}

  /// Extend (append) multiple chars and their display width, and re-calculate all display width
  /// for the extended chars.
  ///
  /// NOTE: This operation is `O(M)`, where `M` is the chars count of the extended chars.
  pub fn extend(&mut self, char2width: &Vec<usize>) {}

  /// Replace a range of chars and their display width, with a new range, and re-calculate all
  /// display width since the first char in the newly added range of chars.
  ///
  /// NOTE: This operation is `O(N+M)`, where `N` is the chars count of current line, `M` is the
  /// chars count of the new range.
  pub fn splice(&mut self) {}

  /// Shorten (remove/truncate) the chars since a specific char index. This operation doesn't need
  /// to trigger re-calculation.
  ///
  /// NOTE: This operation is `O(1)`.
  pub fn truncate(&mut self) {}

  /// Remove a specific range of chars, and re-calculate all display width since the start index in
  /// the removed range.
  ///
  /// NOTE: This operation is `O(N)`, where `N` is the chars count of current line.
  pub fn drain(&mut self) {}
}
