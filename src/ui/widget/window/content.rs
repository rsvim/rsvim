//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::CompactString;
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
  lines: Vec<CompactString>,
  line_wrap: bool,
  word_wrap: bool,
}

impl WindowContent {
  pub fn new() -> Self {
    WindowContent {
      id: uuid::next(),
      lines: vec![],
      line_wrap: false,
      word_wrap: false,
    }
  }

  pub fn lines(&self) -> &Vec<CompactString> {
    &self.lines
  }

  pub fn lines_mut(&mut self) -> &mut Vec<CompactString> {
    &mut self.lines
  }

  pub fn line(&self, index: usize) -> &CompactString {
    &self.lines[index]
  }

  pub fn line_mut(&mut self, index: usize) -> &mut CompactString {
    &mut self.lines[index]
  }

  pub fn line_wrap(&self) -> bool {
    self.line_wrap
  }

  pub fn set_line_wrap(&mut self, line_wrap: bool) -> bool {
    let old_value = self.line_wrap;
    self.line_wrap = line_wrap;
    old_value
  }

  pub fn word_wrap(&self) -> bool {
    self.word_wrap
  }

  pub fn set_word_wrap(&mut self, word_wrap: bool) -> bool {
    let old_value = self.word_wrap;
    self.word_wrap = word_wrap;
    old_value
  }
}

impl Default for WindowContent {
  fn default() -> Self {
    WindowContent::new()
  }
}

impl From<Vec<CompactString>> for WindowContent {
  fn from(lines: Vec<CompactString>) -> Self {
    WindowContent {
      id: uuid::next(),
      lines,
      line_wrap: false,
      word_wrap: false,
    }
  }
}

impl Widget for WindowContent {
  fn id(&self) -> WidgetId {
    self.id
  }

  fn draw(&mut self, actual_shape: U16Rect, _canvas: &mut Canvas) {
    // Do nothing.
    debug!("draw, actual shape:{:?}", actual_shape);
  }
}
