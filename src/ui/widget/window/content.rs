//! Vim window's text content widget.

use crate::buf::{Buffer, BufferWk};
use crate::cart::{IRect, U16Pos, U16Rect};
use crate::glovar;
use crate::inode_generate_impl;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::widget::window::WindowOptions;
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

  // Buffer
  buffer: BufferWk,
  // Buffer view
  view: BufferView,
  // First modified line in buffer view, index (line number) start from 0.
  // When rendering window content with line-wrap/word-wrap options, once a line is modified, it's
  // possible that the following lines are modified too.
  first_modified_line: Option<usize>,

  // Options
  options: WindowOptions,
}

impl WindowContent {
  /// Make window content from buffer. The view starts from the first line.
  pub fn new(shape: IRect, buffer: BufferWk, window_options: &WindowOptions) -> Self {
    let view = BufferView::new(Some(0), None, Some(0), Some(shape.width() as usize));
    WindowContent {
      base: InodeBase::new(shape),
      buffer,
      view,
      first_modified_line: Some(0),
      options: window_options.clone(),
    }
  }
}

// Options {
impl WindowContent {
  pub fn options(&self) -> &WindowOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: &WindowOptions) {
    self.options = options.clone();
  }

  /// Get 'wrap' option.
  pub fn wrap(&self) -> bool {
    self.options.wrap
  }

  /// Get 'line-break' option.
  pub fn link_break(&self) -> bool {
    self.options.line_break
  }

  /// Get 'break-at' option.
  pub fn break_at(&self) -> &String {
    self.options.break_at()
  }

  /// Get 'break-at' option in regex.
  pub fn break_at_regex(&self) -> &Regex {
    self.options.break_at_regex()
  }
}
// Options }

// Buffer/View {
impl WindowContent {
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
}
// Buffer/View }

#[allow(dead_code)]
fn rpslice2line(s: &RopeSlice) -> String {
  let mut builder: String = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
}

// Draw {
impl WindowContent {
  /// Get the first modified line.
  pub fn first_modified_line(&self) -> &Option<usize> {
    &self.first_modified_line
  }

  /// Set modified line. This operation should be called after editing a line on buffer.
  pub fn modify_line(&mut self, line: usize) {
    let smaller_line = match self.first_modified_line {
      Some(first_modified) => {
        if first_modified > line {
          line
        } else {
          first_modified
        }
      }
      None => line,
    };
    self.first_modified_line = Some(smaller_line);
  }

  /// Draw buffer from `start_line`
  pub fn _draw_from_top(
    &mut self,
    canvas: &mut Canvas,
    start_line: usize,
    start_column: usize,
    end_column: usize,
  ) {
    match &self.options {
      WindowOptions {
        wrap: false,
        line_break: _,
        break_at: _,
        break_at_regex: _,
      } => self._draw_from_top_for_nowrap(canvas, start_line, start_column, end_column),
      WindowOptions {
        wrap: true,
        line_break: false,
        break_at: _,
        break_at_regex: _,
      } => { /*Skip*/ }
      WindowOptions {
        wrap: true,
        line_break: true,
        break_at: _,
        break_at_regex: _,
      } => { /*Skip*/ }
    }
  }

  /// Implement the [`_draw_from_top`] with below window options:
  /// - [`warp`](WindowOptions::wrap) is `true`.
  /// - [`line_break`](WindowOptions::line_break) is `false`
  /// - [`break_at`](WindowOptions::break_at) will not be used since 'line-break' is `false`.
  pub fn _draw_from_top_for_wrap_nolinebreak(
    &mut self,
    canvas: &mut Canvas,
    start_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();
    debug!(
      "actual_shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
      actual_shape, upos, height, width,
    );

    // If window is zero-sized.
    if height == 0 || width == 0 {
      return;
    }

    // Get buffer arc pointer
    let buffer = self.buffer.upgrade().unwrap();

    // Lock buffer for read
    let buffer = buffer
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap();

    if let Some(line) = buffer.rope().get_line(start_line) {
      debug!(
        "buffer.get_line ({:?}):'{:?}'",
        start_line,
        rpslice2line(&line),
      );
    } else {
      debug!("buffer.get_line ({:?}):None", start_line);
    }

    match buffer.rope().get_lines_at(start_line) {
      Some(mut buflines) => {
        // The `start_line` is inside the buffer.
        // Render the lines from `start_line` till the end of the buffer or the window widget.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          match buflines.next() {
            Some(line) => {
              // For the row in current window widget, if has the line in buffer.
              let mut col = 0_u16;

              for chunk in line.chunks() {
                if col >= width {
                  row += 1;
                  col = 0_u16;
                  if row >= height {
                    break;
                  }
                }
                for ch in chunk.chars() {
                  if col >= width {
                    row += 1;
                    col = 0_u16;
                    if row >= height {
                      break;
                    }
                  }
                  if ch != '\n' {
                    let cell = Cell::from(ch);
                    let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
                    debug!(
                      "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
                      row, col, ch, cell_upos
                    );
                    canvas.frame_mut().set_cell(cell_upos, cell);
                  }
                  col += 1;
                }
              }

              // The line doesn't fill the whole row in current widget, fill left parts with empty
              // cells.
              if row < height && col < width - 1 {
                let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
                let cells_len = (width - col) as usize;
                debug!(
                  "2-row:{:?}, col:{:?}, cells upos:{:?}, cells len:{:?}",
                  row, col, cells_upos, cells_len,
                );
                canvas
                  .frame_mut()
                  .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                  .unwrap();
              }
            }
            None => {
              // If there's no more lines in the buffer, simply set the whole line to empty for
              // left parts of the window.
              let cells_upos = point!(x: upos.x(), y: row + upos.y());
              let cells_len = width as usize;
              debug!(
                "3-row:{:?}, cells upos:{:?}, cells len:{:?}",
                row, cells_upos, cells_len,
              );
              canvas
                .frame_mut()
                .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                .unwrap();
            }
          }
          // Iterate to next row.
          row += 1;
        }
      }
      None => {
        // The `start_line` is outside of the buffer.
        // Render the whole window contents as empty cells.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          // There's no lines in the buffer, simply set the whole line to empty.
          let cells_upos = point!(x: upos.x(), y: row + upos.y());
          let cells_len = width as usize;
          debug!(
            "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
            row, cells_upos, cells_len,
          );
          canvas
            .frame_mut()
            .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
            .unwrap();
          row += 1;
        }
      }
    }
  }

  /// Implement the [`_draw_from_top`] with below options:
  /// - [`warp`](WindowOptions::wrap) is `false`.
  /// - [`line_break`](WindowOptions::line_break) and [`break_at`](WindowOptions::break_at) will
  ///   not be used since 'wrap' is `false`.
  pub fn _draw_from_top_for_nowrap(
    &mut self,
    canvas: &mut Canvas,
    start_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();
    // debug!(
    //   "actual shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
    //   actual_shape, upos, height, width,
    // );

    // If window is zero-sized.
    if height == 0 || width == 0 {
      return;
    }

    // Get buffer arc pointer
    let buffer = self.buffer.upgrade().unwrap();

    // Lock buffer for read
    let buffer = buffer
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap();

    // if let Some(line) = buffer.rope().get_line(start_line) {
    //   debug!(
    //     "buffer.get_line ({:?}):'{:?}'",
    //     start_line,
    //     rslice2line(&line),
    //   );
    // } else {
    //   debug!("buffer.get_line ({:?}):None", start_line);
    // }

    match buffer.rope().get_lines_at(start_line) {
      Some(mut buflines) => {
        // The `start_line` is inside the buffer.
        // Render the lines from `start_line` till the end of the buffer or the window widget.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          match buflines.next() {
            Some(line) => {
              // For the row in current window widget, if has the line in buffer.
              let mut col = 0_u16;

              for chunk in line.chunks() {
                if col >= width {
                  break;
                }
                for ch in chunk.chars() {
                  if col >= width {
                    break;
                  }
                  if ch != '\n' {
                    let cell = Cell::from(ch);
                    let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
                    // debug!(
                    //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
                    //   row, col, ch, cell_upos
                    // );
                    canvas.frame_mut().set_cell(cell_upos, cell);
                  }
                  col += 1;
                }
              }

              // The line doesn't fill the whole row in current widget, fill left parts with empty
              // cells.
              if row < height && col < width - 1 {
                let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
                let cells_len = (width - col) as usize;
                // debug!(
                //   "2-row:{:?}, col:{:?}, cells upos:{:?}, cells len:{:?}",
                //   row, col, cells_upos, cells_len,
                // );
                canvas
                  .frame_mut()
                  .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                  .unwrap();
              }
            }
            None => {
              // If there's no more lines in the buffer, simply set the whole line to empty for
              // left parts of the window.
              let cells_upos = point!(x: upos.x(), y: row + upos.y());
              let cells_len = width as usize;
              // debug!(
              //   "3-row:{:?}, cells upos:{:?}, cells len:{:?}",
              //   row, cells_upos, cells_len,
              // );
              canvas
                .frame_mut()
                .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                .unwrap();
            }
          }
          // Iterate to next row.
          row += 1;
        }
      }
      None => {
        // The `start_line` is outside of the buffer.
        // Render the whole window contents as empty cells.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          // There's no lines in the buffer, simply set the whole line to empty.
          let cells_upos = point!(x: upos.x(), y: row + upos.y());
          let cells_len = width as usize;
          // debug!(
          //   "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
          //   row, cells_upos, cells_len,
          // );
          canvas
            .frame_mut()
            .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
            .unwrap();
          row += 1;
        }
      }
    }
  }

  /// Draw buffer from `end_line` in reverse order.
  pub fn _draw_from_bottom(
    &mut self,
    _canvas: &mut Canvas,
    _end_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    unimplemented!()
  }
}
// Draw }

inode_generate_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {
    match self.view {
      BufferView {
        start_line: Some(start_line),
        end_line: _,
        start_column: Some(start_column),
        end_column: Some(end_column),
      } => self._draw_from_top(canvas, start_line, start_column, end_column),
      BufferView {
        start_line: _,
        end_line: Some(end_line),
        start_column: Some(start_column),
        end_column: Some(end_column),
      } => self._draw_from_bottom(canvas, end_line, start_column, end_column),
      _ => {
        error!("Invalid view: {:?}", self.view);
        unreachable!("Invalid view")
      }
    }

    // Reset first modified line after drawing.
    self.first_modified_line = None;
  }
}

#[cfg(test)]
mod tests {
  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  use super::*;
  use crate::buf::BufferArc;
  use crate::cart::U16Size;
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  #[allow(dead_code)]
  static INIT: Once = Once::new();

  fn make_buffer_from_file(filename: String) -> BufferArc {
    let rop: Rope = Rope::from_reader(BufReader::new(File::open(filename).unwrap())).unwrap();
    let buf: Buffer = Buffer::from(rop);
    Buffer::to_arc(buf)
  }

  fn make_buffer_from_lines(lines: Vec<&str>) -> BufferArc {
    let mut rop: RopeBuilder = RopeBuilder::new();
    for line in lines.iter() {
      rop.append(line);
    }
    let buf: Buffer = Buffer::from(rop);
    Buffer::to_arc(buf)
  }

  fn make_empty_buffer() -> BufferArc {
    let buf: Buffer = RopeBuilder::new().into();
    Buffer::to_arc(buf)
  }

  #[test]
  fn _draw_from_top_for_nowrap1() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let window_options = WindowOptions::builder().wrap(false).build();
    let window_content_shape = IRect::new((0, 0), (10, 10));
    let mut window_content = WindowContent::new(
      window_content_shape,
      Arc::downgrade(&buffer),
      &window_options,
    );
    let canvas_size = U16Size::new(10, 10);
    let mut canvas = Canvas::new(canvas_size);

    window_content._draw_from_top_for_nowrap(&mut canvas, 0, 0, 10);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    let expect = buffer
      .read()
      .rope()
      .lines()
      .take(10)
      .map(|l| l.as_str().unwrap().chars().take(10).collect::<String>())
      .collect::<Vec<_>>();
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 10);
    assert!(expect.len() <= 10);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 10);
      if i < expect.len() {
        let e = expect[i].clone();
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 10].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap2() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let window_options = WindowOptions::builder().wrap(false).build();
    let window_content_shape = IRect::new((0, 0), (27, 15));
    let mut window_content = WindowContent::new(
      window_content_shape,
      Arc::downgrade(&buffer),
      &window_options,
    );
    let canvas_size = U16Size::new(27, 15);
    let mut canvas = Canvas::new(canvas_size);

    window_content._draw_from_top_for_nowrap(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    let expect = buffer
      .read()
      .rope()
      .lines()
      .skip(1)
      .take(15)
      .map(|l| l.as_str().unwrap().chars().take(27).collect::<String>())
      .collect::<Vec<_>>();
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 15);
    assert!(expect.len() <= 15);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 27);
      if i < expect.len() {
        let e = expect[i].clone();
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap3() {
    // INIT.call_once(test_log_init);

    let buffer = make_empty_buffer();
    let window_options = WindowOptions::builder().wrap(false).build();
    let window_content_shape = IRect::new((0, 0), (20, 18));
    let mut window_content = WindowContent::new(
      window_content_shape,
      Arc::downgrade(&buffer),
      &window_options,
    );
    let canvas_size = U16Size::new(20, 18);
    let mut canvas = Canvas::new(canvas_size);

    window_content._draw_from_top_for_nowrap(&mut canvas, 0, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    assert_eq!(actual.len(), 18);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 20);
      info!("{:?} a:{:?}", i, a);
      assert!(a
        .chars()
        .filter(|c| *c != ' ')
        .collect::<Vec<_>>()
        .is_empty());
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak1() {
    INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello, RSV",
      "IM!       ",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.      ",
      "But still ",
      "it contain",
      "s several ",
    ];

    let window_options = WindowOptions::builder().wrap(false).build();
    let window_content_shape = IRect::new((0, 0), (10, 10));
    let mut window_content = WindowContent::new(
      window_content_shape,
      Arc::downgrade(&buffer),
      &window_options,
    );
    let canvas_size = U16Size::new(10, 10);
    let mut canvas = Canvas::new(canvas_size);

    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 0, 0, 10);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 10);
    assert!(expect.len() <= 10);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 10);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 10].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak2() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let window_options = WindowOptions::builder().wrap(false).build();
    let window_content_shape = IRect::new((0, 0), (27, 15));
    let mut window_content = WindowContent::new(
      window_content_shape,
      Arc::downgrade(&buffer),
      &window_options,
    );
    let canvas_size = U16Size::new(27, 15);
    let mut canvas = Canvas::new(canvas_size);

    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    let expect = buffer
      .read()
      .rope()
      .lines()
      .skip(1)
      .take(15)
      .map(|l| l.as_str().unwrap().chars().take(27).collect::<String>())
      .collect::<Vec<_>>();
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 15);
    assert!(expect.len() <= 15);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 27);
      if i < expect.len() {
        let e = expect[i].clone();
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak3() {
    // INIT.call_once(test_log_init);

    let buffer = make_empty_buffer();
    let window_options = WindowOptions::builder().wrap(false).build();
    let window_content_shape = IRect::new((0, 0), (20, 18));
    let mut window_content = WindowContent::new(
      window_content_shape,
      Arc::downgrade(&buffer),
      &window_options,
    );
    let canvas_size = U16Size::new(20, 18);
    let mut canvas = Canvas::new(canvas_size);

    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 0, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    assert_eq!(actual.len(), 18);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 20);
      info!("{:?} a:{:?}", i, a);
      assert!(a
        .chars()
        .filter(|c| *c != ' ')
        .collect::<Vec<_>>()
        .is_empty());
    }
  }
}
