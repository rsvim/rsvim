//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::CompactString;
use std::convert::From;

use crate::cart::U16Rect;
use crate::ui::term::TerminalArc;
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
}
