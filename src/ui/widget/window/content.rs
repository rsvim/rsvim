//! Vim window's text content widget.

use crate::buf::{Buffer, BufferWk};
use crate::cart::{IRect, U16Pos, U16Rect, U16Size};
use crate::glovar;
use crate::inode_generate_impl;
use crate::ui::canvas::internal::iframe::Iframe;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::widget::window::WindowLocalOptions;
use crate::ui::widget::Widgetable;

use crossterm::style::{Attributes, Color};
use geo::point;
use regex::Regex;
use ropey::RopeSlice;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::time::Duration;
use tracing::{debug, error};

#[derive(Debug, Copy, Clone, Default)]
/// The view of a buffer. The range is left-inclusive right-exclusive, or top-inclusive
/// bottom-exclusive, i.e. `[start_line, end_line)` or `[start_column, end_column)`.
struct BufferView {
  /// Start line number
  pub start_line: Option<usize>,
  /// End line number.
  pub end_line: Option<usize>,
  /// Start column.
  pub start_column: Option<usize>,
  /// End column.
  pub end_column: Option<usize>,
}

impl BufferView {
  pub fn new(
    start_line: Option<usize>,
    end_line: Option<usize>,
    start_column: Option<usize>,
    end_column: Option<usize>,
  ) -> Self {
    BufferView {
      start_line,
      end_line,
      start_column,
      end_column,
    }
  }
}

#[derive(Debug, Clone)]
/// The content of VIM window.
///
/// Besides buffer and window, here introduce several terms and concepts:
///
/// * Line: One line of text content in a buffer.
/// * Row/column: The width/height of a window.
/// * View: A window only shows part of a buffer when the buffer is too big to put all the text
///   contents in the window. When a buffer shows in a window, thus the window starts and ends at
///   specific lines and columns of the buffer.
///
/// There are two options related to the view:
/// [line-wrap and word-wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap), so we have 4
/// kinds of views.
///
/// * Both line-wrap and word-wrap enabled.
/// * Line-wrap enabled and word-wrap disabled.
/// * Line-wrap disabled and word-wrap enabled.
/// * Both Line-wrap and word-wrap disabled.
///
/// For the first 3 kinds of view, when a window that has `X` rows height, it may contains less
/// than `X` lines of a buffer. Because very long lines or words can take extra spaces and trigger
/// line breaks. The real lines the window can contain needs a specific algorithm to calculate.
///
/// For the last kind of view, it contains exactly `X` lines of a buffer at most, but the lines
/// longer than the window's width are truncated by the window's boundary.
///
/// A view contains 4 fields:
///
/// * Start line.
/// * End line.
/// * Start column.
/// * End column.
///
/// We can always calculates the two fields based on the other two fields on the diagonal corner,
/// with window size, buffer's text contents, and the line-wrap/word-wrap options.
pub struct WindowContent {
  base: InodeBase,
  frame: Iframe,
}

impl WindowContent {
  /// Make window content.
  pub fn new(shape: IRect) -> Self {
    WindowContent {
      base: InodeBase::new(shape),
      // NOTE: When create window content, it doesn't know itself actual shape. The actual shape
      // will be update when inserted into its parent node.
      frame: Iframe::new(U16Size::new(0, 0)),
    }
  }

  pub fn frame(&self) -> &Iframe {
    &self.frame
  }

  /// Call this method only after it's been inserted to parent node, or shape been changed.
  pub fn sync_cells_size(&mut self) {
    self
      .frame
      .set_size(U16Size::from(*self.base.actual_shape()));
  }

  pub fn set_cell(&mut self, pos: U16Pos, cell: Cell) -> Cell {
    self.frame.set_cell(pos, cell)
  }

  pub fn try_set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Option<Vec<Cell>> {
    self.frame.try_set_cells_at(pos, cells)
  }
}

#[allow(dead_code)]
fn rpslice2line(s: &RopeSlice) -> String {
  let mut builder: String = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
}

inode_generate_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {
    for row in 0..self.actual_shape().height() {
      for col in 0..self.actual_shape().width() {
        let pos = U16Pos::new(col, row);
        canvas
          .frame_mut()
          .set_cell(pos, self.frame.get_cell(pos).clone());
      }
    }
  }
}

#[cfg(test)]
mod tests {}
