//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::{CompactString, ToCompactString};
use crossterm::style::{Attributes, Color};
use geo::point;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::time::Duration;
use tracing::debug;

use crate::buf::{Buffer, BufferWk};
use crate::cart::{IRect, U16Pos, U16Rect};
use crate::glovar;
use crate::inode_generate_impl;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::widget::Widgetable;
use crate::uuid;

#[derive(Debug, Copy, Clone, Default)]
/// The view of a buffer. The range is left-open right-closed, or top-open bottom-closed, i.e.
/// `[start_line, end_line)` or `[start_column, end_column)`.
pub struct BufferView {
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

  // Buffer
  buffer: BufferWk,
  // Buffer view
  view: BufferView,
  // Modified lines in buffer view, index start from 0. This dataset dedups lines iterating on
  // buffer view in each drawing.
  modified_lines: BTreeSet<usize>,

  // Options
  line_wrap: bool,
  word_wrap: bool,
}

impl WindowContent {
  /// Make window content from buffer. The view starts from the first line.
  pub fn new(shape: IRect, buffer: BufferWk) -> Self {
    let view = BufferView::new(Some(0), None, Some(0), None);
    WindowContent {
      base: InodeBase::new(shape),
      buffer,
      view,
      modified_lines: (0..shape.height()).map(|l| l as usize).collect(),
      line_wrap: false,
      word_wrap: false,
    }
  }

  // Options {

  /// Get line-wrap option.
  pub fn line_wrap(&self) -> bool {
    self.line_wrap
  }

  /// Set line-wrap option.
  pub fn set_line_wrap(&mut self, line_wrap: bool) {
    self.line_wrap = line_wrap;
  }

  /// Get word-wrap option.
  pub fn word_wrap(&self) -> bool {
    self.word_wrap
  }

  /// Set word-wrap option.
  pub fn set_word_wrap(&mut self, word_wrap: bool) {
    self.word_wrap = word_wrap;
  }

  // Options }

  // Buffer/View {

  /// Get buffer reference.
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }

  /// Set buffer reference.
  pub fn set_buffer(&mut self, buffer: BufferWk) {
    self.buffer = buffer;
  }

  /// Get start line, index start from 0.
  pub fn start_line(&self) -> Option<usize> {
    self.view.start_line
  }

  /// Set start line.
  ///
  /// This operation will unset the end line. Because with different line-wrap/word-wrap options,
  /// the window may contains less lines than its height. We cannot know the end line unless
  /// iterating over the buffer from start line.
  pub fn set_start_line(&mut self, line: usize) {
    self.view.start_line = Some(line);
    self.view.end_line = None;
  }

  /// Get end line, index start from 0.
  pub fn end_line(&self) -> Option<usize> {
    self.view.end_line
  }

  /// Set end line.
  ///
  /// This operation will unset the start line. Because with different line-wrap/word-wrap options,
  /// the window may contains less lines than the height. We cannot know the start line unless
  /// reversely iterating over the buffer from end line.
  pub fn set_end_line(&mut self, lend: usize) {
    self.view.end_line = Some(lend);
    self.view.start_line = None;
  }

  /// Get start column, index start from 0.
  pub fn start_column(&self) -> Option<usize> {
    self.view.start_column
  }

  /// Set start column.
  ///
  /// This operation also calculates the end column based on widget's width, and set it as well.
  pub fn set_start_column(&mut self, cstart: usize) {
    self.view.start_column = Some(cstart);
    self.view.end_column = Some(cstart + self.base.actual_shape().width() as usize);
  }

  /// Get end column, index start from 0.
  pub fn end_column(&self) -> Option<usize> {
    self.view.end_column
  }

  /// Set end column.
  ///
  /// This operation also calculates the start column based on widget's width, and set it as well.
  pub fn set_end_column(&mut self, cend: usize) {
    self.view.end_column = Some(cend);
    self.view.start_column = Some(cend - self.base.actual_shape().width() as usize);
  }

  // Buffer/View }

  // Modified {

  /// Get modified lines (in the view).
  pub fn modified_lines(&self) -> &BTreeSet<usize> {
    &self.modified_lines
  }

  /// Set all lines (in the view) to modified.
  pub fn modify_all_lines(&mut self) {
    self.modified_lines = (0..self.shape().height()).map(|l| l as usize).collect();
  }

  /// Clear all modified lines. This operation should be called after drawing to canvas.
  pub fn clear_modified_lines(&mut self) {
    self.modified_lines = BTreeSet::new();
  }

  /// Set modified line. This operation should be called after editing a line on buffer.
  pub fn modify_line(&mut self, line: usize) -> bool {
    self.modified_lines.insert(line)
  }

  /// Reset modified line to unmodified.
  pub fn unmodify_line(&mut self, line_no: &usize) -> bool {
    self.modified_lines.remove(line_no)
  }

  // Modified }

  fn draw_from_start_line(
    &mut self,
    canvas: &mut Canvas,
    start_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    let actual_shape = self.actual_shape();
    let actual_pos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();

    // Get buffer arc pointer
    let buffer = self.buffer.upgrade().unwrap();

    // Lock buffer for read
    let buffer = buffer
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap();

    let total_lines = buffer.rope().len_lines();
    if start_line <= total_lines {
      let mut buffer_lines = buffer.rope().lines_at(start_line);
      for row in 0..height {
        match buffer_lines.next() {
          Some(one_line) => {
            // Write the line.
            let mut col = 0_u16;
            for chunk in one_line.chunks() {
              let cells: Vec<Cell> = chunk.chars().map(Cell::from).collect();
              let cells_len = cells.len();
              canvas
                .frame_mut()
                .set_cells_at(point!(x: col, y: row + actual_pos.y()), cells);
              col += cells_len as u16;
            }

            // Clear the left parts (at the end) of the line.
            canvas.frame_mut().set_cells_at(
              point!(x: col, y: row  + actual_pos.y()),
              vec![Cell::empty(); (width - col) as usize],
            );
          }
          None => {
            // Set empty line
            canvas.frame_mut().set_cells_at(
              point!(x: actual_pos.x(), y: row + actual_pos.y()),
              vec![Cell::empty(); width as usize],
            );
          }
        }
      }
    }
  }

  fn draw_from_end_line(
    &mut self,
    canvas: &mut Canvas,
    end_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    unimplemented!()
  }
}

inode_generate_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {
    match self.view {
      BufferView {
        start_line: Some(start_line),
        end_line: _,
        start_column: Some(start_column),
        end_column: Some(end_column),
      } => self.draw_from_start_line(canvas, start_line, start_column, end_column),
      BufferView {
        start_line: _,
        end_line: Some(end_line),
        start_column: Some(start_column),
        end_column: Some(end_column),
      } => self.draw_from_end_line(canvas, end_line, start_column, end_column),
      _ => {
        unreachable!("Invalid buffer view")
      }
    }
  }
}
