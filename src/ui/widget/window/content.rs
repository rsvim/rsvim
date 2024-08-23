//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::{CompactString, ToCompactString};
use crossterm::style::{Attributes, Color};
use geo::point;
use std::collections::VecDeque;
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
struct BufferView {
  /// Start line number
  pub lstart: Option<usize>,
  /// End line number.
  pub lend: Option<usize>,
  /// Start column.
  pub cstart: Option<usize>,
  /// End column.
  pub cend: Option<usize>,
}

impl BufferView {
  pub fn new(
    lstart: Option<usize>,
    lend: Option<usize>,
    cstart: Option<usize>,
    cend: Option<usize>,
  ) -> Self {
    BufferView {
      lstart,
      lend,
      cstart,
      cend,
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
/// * Start line (`lstart`).
/// * End line (`lend`).
/// * Start column (`cstart`).
/// * End column (`cend`).
///
/// We can always calculates the two fields based on the other two fields on the diagonal corner,
/// with window size, buffer's text contents, and the line-wrap/word-wrap options.
pub struct WindowContent {
  base: InodeBase,

  // Buffer
  buffer: BufferWk,
  // Buffer view
  view: BufferView,

  // Options
  line_wrap: bool,
  word_wrap: bool,
}

impl WindowContent {
  pub fn new(shape: IRect, buffer: BufferWk) -> Self {
    let view = BufferView::new(Some(0), None, Some(0), None);
    WindowContent {
      base: InodeBase::new(shape),
      buffer,
      view,
      line_wrap: false,
      word_wrap: false,
    }
  }

  pub fn line_wrap(&self) -> bool {
    self.line_wrap
  }

  pub fn set_line_wrap(&mut self, line_wrap: bool) {
    self.line_wrap = line_wrap;
  }

  pub fn word_wrap(&self) -> bool {
    self.word_wrap
  }

  pub fn set_word_wrap(&mut self, word_wrap: bool) {
    self.word_wrap = word_wrap;
  }

  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }

  pub fn set_buffer(&mut self, buffer: BufferWk) {
    self.buffer = buffer;
  }

  pub fn view_lstart(&self) -> Option<usize> {
    self.view.lstart
  }

  pub fn set_view_lstart(&mut self, lstart: usize) {
    self.view.lstart = Some(lstart);
    self.view.lend = None;
  }

  pub fn view_lend(&self) -> Option<usize> {
    self.view.lend
  }

  pub fn set_view_lend(&mut self, lend: usize) {
    self.view.lend = Some(lend);
    self.view.lstart = None;
  }

  pub fn view_cstart(&self) -> Option<usize> {
    self.view.cstart
  }

  pub fn set_view_cstart(&mut self, cstart: usize) {
    self.view.cstart = Some(cstart);
    self.view.cend = Some(cstart + self.base.actual_shape().width() as usize);
  }

  pub fn view_cend(&self) -> Option<usize> {
    self.view.cend
  }

  pub fn set_view_cend(&mut self, cend: usize) {
    self.view.cend = Some(cend);
    self.view.cstart = Some(cend - self.base.actual_shape().width() as usize);
  }
}

inode_generate_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {
    match self.view {
      BufferView {
        lstart: Some(lstart),
        lend: _,
        cstart: Some(_cstart),
        cend: Some(_cend),
      } => {
        let actual_shape = self.actual_shape();
        let actual_pos: U16Pos = actual_shape.min().into();
        let height = actual_shape.height();
        let width = actual_shape.width();

        // Get buffer Arc pointer
        if let Some(buffer) = self.buffer.upgrade() {
          // Lock buffer for read
          if let Some(buffer) = buffer.try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT())) {
            let total_lines = buffer.rope().len_lines();
            if lstart <= total_lines {
              let mut buffer_lines = buffer.rope().lines_at(lstart);
              for row in 0..height {
                match buffer_lines.next() {
                  Some(one_line) => {
                    // Write the line.
                    let mut col = 0_u16;
                    for chunk in one_line.chunks() {
                      let cells: Vec<Cell> = chunk.chars().map(Cell::from).collect();
                      let cells_len = cells.len();
                      canvas.frame_mut().splice_cells_at(
                        point!(x: col, y: row + actual_pos.y()),
                        (width - col) as usize,
                        cells,
                      );
                      col += cells_len as u16;
                    }

                    // Clear the left parts (at the end) of the line.
                    canvas.frame_mut().splice_cells_repeatedly_at(
                      point!(x: col, y: row  + actual_pos.y()),
                      (width - col) as usize,
                      Cell::none(),
                    );
                  }
                  None => {
                    // Set empty line
                    canvas.frame_mut().splice_cells_repeatedly_at(
                      point!(x: actual_pos.x(), y: row + actual_pos.y()),
                      width as usize,
                      Cell::none(),
                    );
                  }
                }
              }
            }
          }
        }

        // Failed to upgrade to Arc pointer or lock, don't do anything and keep the current contents.
      }
      BufferView {
        lstart: _,
        lend: Some(_lend),
        cstart: Some(_cstart),
        cend: Some(_cend),
      } => {
        unreachable!("Not implement")
      }
      _ => {
        unreachable!("Missing buffer view")
      }
    }
  }
}
