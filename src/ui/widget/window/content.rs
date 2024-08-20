//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::CompactString;
use std::collections::VecDeque;
use std::convert::From;
use tracing::debug;

use crate::cart::{IRect, U16Rect};
use crate::ui::canvas::Canvas;
use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Debug, Clone)]
/// The VIM window content.
pub struct WindowContent {
  id: WidgetId,
  lines: VecDeque<CompactString>,
  dirty_lines: VecDeque<bool>,
  line_wrap: bool,
  word_wrap: bool,
  dirty_option: bool,
}

impl WindowContent {
  pub fn new() -> Self {
    WindowContent {
      id: uuid::next(),
      lines: VecDeque::new(),
      dirty_lines: VecDeque::new(),
      line_wrap: false,
      word_wrap: false,
      dirty_option: false,
    }
  }

  pub fn lines(&self) -> &VecDeque<CompactString> {
    &self.lines
  }

  pub fn line(&self, index: usize) -> &CompactString {
    &self.lines[index]
  }

  pub fn set_line(&mut self, index: usize, line: CompactString) {
    self.lines[index] = line;
    self.dirty_lines[index] = true;
  }

  /// Get the first line.
  pub fn front_line(&self) -> Option<&CompactString> {
    self.lines.front()
  }

  /// Get the last line.
  pub fn back_line(&self) -> Option<&CompactString> {
    self.lines.back()
  }

  pub fn front_line_mut(&mut self) -> Option<&mut CompactString> {
    self.lines.front_mut()
  }

  pub fn back_line_mut(&mut self) -> Option<&mut CompactString> {
    self.lines.back_mut()
  }

  pub fn resize(&mut self, new_len: usize) {
    self.lines.resize(new_len, CompactString::const_new(""));
    self.dirty_lines.resize(new_len, false);
  }

  pub fn push_back_line(&mut self, line: CompactString) {
    self.lines.push_back(line);
    self.dirty_lines.push_back(false);
  }

  pub fn pop_back_line(&mut self) -> Option<CompactString> {
    let result = self.lines.pop_back();
    self.dirty_lines.pop_back();
    result
  }

  pub fn push_front_line(&mut self, line: CompactString) {
    self.lines.push_front(line);
    self.dirty_lines.push_front(false);
  }

  pub fn pop_front_line(&mut self) -> Option<CompactString> {
    let result = self.lines.pop_front();
    self.dirty_lines.pop_front();
    result
  }

  pub fn line_wrap(&self) -> bool {
    self.line_wrap
  }

  pub fn set_line_wrap(&mut self, line_wrap: bool) -> bool {
    if self.line_wrap != line_wrap {
      let old_value = self.line_wrap;
      self.line_wrap = line_wrap;
      self.dirty_option = true;
      old_value
    } else {
      self.line_wrap
    }
  }

  pub fn word_wrap(&self) -> bool {
    self.word_wrap
  }

  pub fn set_word_wrap(&mut self, word_wrap: bool) -> bool {
    if self.word_wrap != word_wrap {
      let old_value = self.word_wrap;
      self.word_wrap = word_wrap;
      self.dirty_option = true;
      old_value
    } else {
      self.word_wrap
    }
  }
}

impl Default for WindowContent {
  fn default() -> Self {
    WindowContent::new()
  }
}

impl From<Vec<CompactString>> for WindowContent {
  fn from(lines: Vec<CompactString>) -> Self {
    let dirty_lines = lines.iter().map(|_| false).collect();
    WindowContent {
      id: uuid::next(),
      lines,
      dirty_lines,
      line_wrap: false,
      word_wrap: false,
      dirty_option: true,
    }
  }
}

impl Widget for WindowContent {
  fn id(&self) -> WidgetId {
    self.id
  }

  fn draw(&mut self, actual_shape: U16Rect, canvas: &mut Canvas) {
    if !self.dirty {
      return;
    }
    if self.lines.is_empty() {}

    self.dirty = false;
  }
}
