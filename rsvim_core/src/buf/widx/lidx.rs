//! Display width index (line-wise) for vim buffer.

use crate::buf::opt::BufferLocalOptions;
use crate::buf::widx::cidx::ColIndex;

use ropey::RopeSlice;
use std::collections::BTreeMap;
use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
/// Display width index (line-wise) for vim buffer. It manages all the
/// [`ColIndex`](crate::buf::ColIndex) and handles the details.
pub struct LineLindex {
  line2cidx: BTreeMap<usize, ColIndex>,
}

impl LineLindex {
  /// Create new index.
  pub fn new() -> Self {
    Self {
      line2cidx: BTreeMap::new(),
    }
  }

  /// Get the prefix display width on line `line_idx`, in char index range `[0,char_idx)`,
  /// left-inclusive and right-exclusive.
  ///
  /// NOTE: This is equivalent to `width_until(line_idx, char_idx-1)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if `char_idx <= 0`.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than the line
  ///    length.
  pub fn width_before(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    line_idx: usize,
    char_idx: usize,
  ) -> usize {
    if !self.line2cidx.contains_key(&line_idx) {
      self.line2cidx.insert(line_idx, ColIndex::new());
    }
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .width_before(options, rope_line, char_idx)
  }

  /// Get the prefix display width on line `line_idx`, char index range `[0,char_idx]`, both sides
  /// are inclusive.
  ///
  /// NOTE: This is equivalent to `width_before(line_idx, char_idx+1)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if the line length is 0, i.e. the line itself is empty.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than or equal to
  ///    the line length.
  pub fn width_until(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    line_idx: usize,
    char_idx: usize,
  ) -> usize {
    if !self.line2cidx.contains_key(&line_idx) {
      self.line2cidx.insert(line_idx, ColIndex::new());
    }
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .width_until(options, rope_line, char_idx)
  }

  /// Get the right-most char index which the width is less than the specified width, on line
  /// `line_idx`.
  ///
  /// Note:
  /// 1. The specified width is exclusive, i.e. the returned char index's width is always less than
  ///    the specified width, but cannot be greater than or equal to it.
  /// 2. For all the char indexes which the width is less, it returns the right-most char index.
  ///
  /// # Return
  ///
  /// 1. It returns None if the line length is 0, i.e. the line itself is empty, or there's no such
  ///    char.
  /// 2. It returns the right-most char index if `width` is inside the line.
  /// 3. It returns the last char index of the line if `width` is greater than or equal to
  ///    the line's whole display width.
  pub fn char_before(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    line_idx: usize,
    width: usize,
  ) -> Option<usize> {
    if !self.line2cidx.contains_key(&line_idx) {
      self.line2cidx.insert(line_idx, ColIndex::new());
    }
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .char_before(options, rope_line, width)
  }

  /// Get the right-most char index which the width is greater than or equal to the specified
  /// width, on line `line_idx`.
  ///
  /// Note:
  /// 1. The specified width is inclusive, i.e. the returned char index's width is greater than or
  ///    equal to the specified width, but cannot be less than it.
  /// 2. For all the char indexes which the width is greater or equal, it returns the right-most
  ///    char index.
  ///
  /// # Return
  ///
  /// 1. It returns None if the line length is 0, i.e. the line itself is empty, or there's no such
  ///    char.
  /// 2. It returns the right-most char index if `width` is inside the line.
  /// 3. It returns the last char index of the line if `width` is greater than or equal to
  ///    the line's whole display width.
  pub fn char_until(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    line_idx: usize,
    width: usize,
  ) -> Option<usize> {
    if !self.line2cidx.contains_key(&line_idx) {
      self.line2cidx.insert(line_idx, ColIndex::new());
    }
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .char_until(options, rope_line, width)
  }

  /// Truncate cache on one line, start from specified char index.
  pub fn truncate_line_since_char(&mut self, line_idx: usize, char_idx: usize) {
    match self.line2cidx.get_mut(&line_idx) {
      Some(cidx) => cidx.truncate_by_char(char_idx),
      None => { /* Do Nothing. */ }
    }
  }

  /// Truncate cache on one line, start from specified width.
  pub fn truncate_line_since_width(&mut self, line_idx: usize, width: usize) {
    match self.line2cidx.get_mut(&line_idx) {
      Some(cidx) => cidx.truncate_by_width(width),
      None => { /* Do Nothing. */ }
    }
  }

  /// Truncate multiple lines, start from specified line index.
  pub fn truncate(&mut self, start_line_idx: usize) {
    self.line2cidx.retain(|&l, _| l < start_line_idx);
  }

  /// Truncate multiple lines, start from specified line index.
  pub fn remove(&mut self, line_idx: usize) {
    self.line2cidx.remove(&line_idx);
  }

  /// Retain multiple lines, start from specified line index.
  pub fn retain(&mut self, lines_idx: HashSet<usize>) {
    self.line2cidx.retain(|l, _| lines_idx.contains(l));
  }
}
