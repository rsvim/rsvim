//! The VIM Window.

use compact_str::CompactString;

use crate::cart::IRect;
use crate::define_widget_base_helpers;
use crate::ui::tree::NodeId;
use crate::ui::widget::{Widget, WidgetBase};

/// The VIM window.
pub struct Window {
  base: WidgetBase,
  lines: Vec<CompactString>,
}

impl Window {
  pub fn new(rect: IRect, zindex: usize) -> Self {
    let base = WidgetBase::new(rect, zindex);
    Window {
      base,
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

  pub fn push_line(&mut self, line: CompactString) {
    self.lines.push(line);
  }

  pub fn insert_line(&mut self, index: usize, line: CompactString) {
    self.lines.insert(index, line);
  }

  pub fn remove_line(&mut self, index: usize) -> CompactString {
    self.lines.remove(index)
  }
}

impl Widget for Window {
  define_widget_base_helpers!();

  fn draw(&mut self) {
    todo!();
  }
}
