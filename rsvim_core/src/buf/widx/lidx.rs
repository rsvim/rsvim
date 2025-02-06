//! Display width index (line-wise) for vim buffer.

use crate::buf::opt::BufferLocalOptions;
use crate::buf::widx::cidx::ColIndex;

use ropey::Rope;
use std::collections::BTreeMap;
use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
/// Display width index (line-wise) for vim buffer. It manages all the
/// [`ColIndex`](crate::buf::ColIndex) and handles the details.
pub struct LineIndex {
  line2cidx: BTreeMap<usize, ColIndex>,
}

impl LineIndex {
  /// Create new index.
  pub fn new() -> Self {
    Self {
      line2cidx: BTreeMap::new(),
    }
  }

  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_before(
    &mut self,
    options: &BufferLocalOptions,
    rope: &Rope,
    line_idx: usize,
    char_idx: usize,
  ) -> usize {
    self.line2cidx.entry(line_idx).or_default();
    let rope_line = rope.line(line_idx);
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .width_before(options, &rope_line, char_idx)
  }

  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_until(
    &mut self,
    options: &BufferLocalOptions,
    rope: &Rope,
    line_idx: usize,
    char_idx: usize,
  ) -> usize {
    self.line2cidx.entry(line_idx).or_default();
    let rope_line = rope.line(line_idx);
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .width_until(options, &rope_line, char_idx)
  }

  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_before(
    &mut self,
    options: &BufferLocalOptions,
    rope: &Rope,
    line_idx: usize,
    width: usize,
  ) -> Option<usize> {
    self.line2cidx.entry(line_idx).or_default();
    let rope_line = rope.line(line_idx);
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .char_before(options, &rope_line, width)
  }

  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_until(
    &mut self,
    options: &BufferLocalOptions,
    rope: &Rope,
    line_idx: usize,
    width: usize,
  ) -> Option<usize> {
    self.line2cidx.entry(line_idx).or_default();
    let rope_line = rope.line(line_idx);
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .char_until(options, &rope_line, width)
  }

  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_after(
    &mut self,
    options: &BufferLocalOptions,
    rope: &Rope,
    line_idx: usize,
    width: usize,
  ) -> Option<usize> {
    self.line2cidx.entry(line_idx).or_default();
    let rope_line = rope.line(line_idx);
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .char_after(options, &rope_line, width)
  }

  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn last_char(
    &mut self,
    options: &BufferLocalOptions,
    rope: &Rope,
    line_idx: usize,
  ) -> Option<usize> {
    self.line2cidx.entry(line_idx).or_default();
    let rope_line = rope.line(line_idx);
    self
      .line2cidx
      .get_mut(&line_idx)
      .unwrap()
      .last_char(options, &rope_line)
  }

  /// Reset tail of cache on one line, start from specified char index.
  pub fn reset_line_since_char(&mut self, line_idx: usize, char_idx: usize) {
    match self.line2cidx.get_mut(&line_idx) {
      Some(cidx) => cidx.truncate_by_char(char_idx),
      None => { /* Do Nothing. */ }
    }
  }

  /// Reset tail of cache on one line, start from specified width.
  pub fn reset_line_since_width(&mut self, line_idx: usize, width: usize) {
    match self.line2cidx.get_mut(&line_idx) {
      Some(cidx) => cidx.truncate_by_width(width),
      None => { /* Do Nothing. */ }
    }
  }

  /// Truncate lines at the tail, start from specified line index.
  pub fn truncate(&mut self, start_line_idx: usize) {
    self.line2cidx.retain(|&l, _| l < start_line_idx);
  }

  /// Remove one specified line.
  pub fn remove(&mut self, line_idx: usize) {
    self.line2cidx.remove(&line_idx);
  }

  /// Retain multiple specified lines.
  pub fn retain(&mut self, lines_idx: HashSet<usize>) {
    self.line2cidx.retain(|l, _| lines_idx.contains(l));
  }

  /// Clear.
  pub fn clear(&mut self) {
    self.line2cidx.clear()
  }
}
