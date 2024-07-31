//! VIM window's text content widget.

use compact_str::CompactString;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::widget::Widget;

/// The VIM window content.
#[derive(Debug, Clone, Default)]
pub struct WindowContent {
  lines: Vec<CompactString>,
}

impl WindowContent {
  pub fn new(lines: Vec<CompactString>) -> Self {
    WindowContent { lines }
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

impl Widget for WindowContent {
  fn draw(&mut self, _actual_shape: U16Rect, _terminal: TerminalWk) {}
}
