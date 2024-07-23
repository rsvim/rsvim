//! The VIM Window.

use compact_str::CompactString;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;
use crate::ui::widget::Widget;
use crate::uuid;

/// The VIM window.
#[derive(Clone)]
pub struct Window {
  id: NodeId,
  lines: Vec<CompactString>,
}

impl Window {
  pub fn new(lines: Vec<CompactString>) -> Self {
    Window {
      id: uuid::next(),
      lines,
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

impl Default for Window {
  fn default() -> Self {
    Window {
      id: uuid::next(),
      lines: vec![],
    }
  }
}

impl Widget for Window {
  fn id(&self) -> NodeId {
    self.id
  }

  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {}
}
