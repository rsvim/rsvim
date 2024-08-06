//! VIM window's text content widget.

use compact_str::CompactString;

use crate::cart::U16Rect;
use crate::ui::term::TerminalArc;
use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

/// The VIM window content.
#[derive(Debug, Clone)]
pub struct WindowContent {
  id: WidgetId,
  lines: Vec<CompactString>,
}

impl WindowContent {
  pub fn new() -> Self {
    WindowContent {
      id: uuid::next(),
      lines: vec![],
    }
  }

  pub fn lines(&self) -> &Vec<CompactString> {
    &self.lines
  }

  pub fn lines_mut(&mut self) -> &mut Vec<CompactString> {
    &mut self.lines
  }

  pub fn set_lines(&mut self, lines: Vec<CompactString>) {
    self.lines = lines;
  }

  pub fn line(&self, index: usize) -> &CompactString {
    &self.lines[index]
  }

  pub fn line_mut(&mut self, index: usize) -> &mut CompactString {
    &mut self.lines[index]
  }

  pub fn set_line(&mut self, index: usize, line: CompactString) {
    self.lines[index] = line;
  }
}

impl Default for WindowContent {
  fn default() -> Self {
    WindowContent::new()
  }
}

impl Widget for WindowContent {
  fn id(&self) -> WidgetId {
    self.id
  }
}
