//! Buffer viewport on a window.

#![allow(unused_variables)]
use crate::buf::BufferWk;
use crate::cart::{U16Pos, U16Size, URect};
use crate::defaults::grapheme::AsciiControlCode;
use crate::envar;
use crate::rlock;
use crate::ui::canvas::Cell;
use crate::ui::tree::internal::Inodeable;
use crate::ui::util::{ptr::SafeWindowRef, strings};
use crate::ui::widget::window::Window;

use geo::point;
use std::collections::BTreeMap;
use tracing::debug;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Copy, Clone)]
/// The section information of a buffer line. One section is exactly one row in a window.
pub struct LineViewportSection {
  // Row index in the window.
  pub row: u16,
  // chars length
  pub char_length: usize,
  // unicode displayed length
  pub display_length: u16,
}

#[derive(Debug, Clone)]
/// All the sections of a buffer line. Since one line could occupy multiple rows in a window.
pub struct LineViewport {
  pub sections: Vec<LineViewportSection>,
}

#[derive(Debug, Clone)]
/// The buffer viewport on a window.
///
/// When a buffer displays on a window, it starts from a specific line and column, ends at a
/// specific line and column. Here it calls `start_line`, `start_column`, `end_line`, `end_column`.
/// The range is start-inclusive end-exclusive, i.e. `[start_line, end_line)` or
/// `[start_column, end_column)`. All lines, rows and columns index are start from 0.
///
/// The viewport will handle some calculating task when rendering a buffer to terminal.
///
/// With some display options (such as ['wrap'](crate::defaults::win::WRAP) and
/// ['line-break'](crate::defaults::win::LINE_BREAK)), unicode/i18n settings or other factors, each
/// char could occupy different cell width on terminal, and each line could occupy more than 1 row
/// on a window.
///
/// To ensure these detailed positions/sizes, it will have to go through all the text contents
/// inside the window, even more. Suppose the window is a MxN (M rows, N columns) size, each go
/// through time complexity is O(MxN). We would like to keep the number of such kind of going
/// through task to about 1~2 times, or at least a constant number that doesn't increase with the
/// increase of the buffer.
pub struct Viewport {
  // Window reference.
  window: SafeWindowRef,
  // Start line number.
  start_line: usize,
  // End line number.
  end_line: usize,
  // Start column number.
  start_column: usize,
  // End column number.
  end_column: usize,
  // Maps from buffer's line number to its displayed information in the window.
  lines: BTreeMap<usize, LineViewport>,
}

#[derive(Debug, Copy, Clone, Default)]
// Tuple of start_line, end_line, start_column, end_column.
struct ViewportRect {
  pub start_line: usize,
  pub end_line: usize,
  pub start_column: usize,
  pub end_column: usize,
}

// Given the buffer and window size, collect information from start line and column, i.e. from the
// top-left corner.
fn collect_from_top_left(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  match (window.wrap(), window.line_break()) {
    (false, _) => _collect_from_top_left_for_nowrap(window, start_line, start_column),
    (true, false) => _collect_from_top_left_for_wrap_nolinebreak(window, start_line, start_column),
    (true, true) => _collect_from_top_left_for_wrap_linebreak(window, start_line, start_column),
  }
}

// Implement [`collect_from_top_left`] with option `wrap=false`.
fn _collect_from_top_left_for_nowrap(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let actual_shape = window.actual_shape();
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!("_collect_from_top_left_for_nowrap");
  // debug!(
  //   "actual shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
  //   actual_shape, upos, height, width,
  // );

  // If window is zero-sized.
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  // Get buffer arc pointer
  let buffer = window.buffer().upgrade().unwrap();

  // Lock buffer for read
  let buffer = rlock!(buffer);

  // if let Some(line) = buffer.rope().get_line(start_line) {
  //   debug!(
  //     "buffer.get_line ({:?}):'{:?}'",
  //     start_line,
  //     rslice2line(&line),
  //   );
  // } else {
  //   debug!("buffer.get_line ({:?}):None", start_line);
  // }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut current_line = start_line;

  match buffer.rope().get_lines_at(start_line) {
    Some(mut buflines) => {
      // The `start_line` is inside the buffer.
      // Parse the lines from `start_line` until the end of the buffer or the window.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;

      while row < height {
        match buflines.next() {
          Some(line) => {
            // If there's 1 more line in the buffer.
            let mut col = 0_u16;
            let mut sections: Vec<LineViewportSection> = vec![];
            let mut char_length = 0_usize;
            let mut display_length = 0_u16;

            // Go through each char in the line.
            for c in line.chars() {
              if (col as usize) < start_column {
                col += 1;
                continue;
              }
              if col as usize >= (width as usize + start_column) {
                break;
              }
              let width = strings::char_width(c, &buffer);
              display_length += width;
              char_length += 1;
              // debug!(
              //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
              //   row, col, ch, cell_upos
              // );
              col += 1;
            }

            sections.push(LineViewportSection {
              row,
              char_length,
              display_length,
            });
            line_viewports.insert(current_line, LineViewport { sections });
            current_line += 1;
          }
          None => { /* There's no more lines in the buffer. */ }
        }
        // Go to next row.
        row += 1;
      }

      (
        ViewportRect {
          start_line,
          end_line: current_line,
          start_column: 0,
          end_column: 0,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      (ViewportRect::default(), BTreeMap::new())
    }
  }
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=false`.
fn _collect_from_top_left_for_wrap_nolinebreak(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  (ViewportRect::default(), BTreeMap::new())
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=true`.
fn _collect_from_top_left_for_wrap_linebreak(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  (ViewportRect::default(), BTreeMap::new())
}

impl Viewport {
  pub fn new(window: &mut Window) -> Self {
    // By default the viewport start from the first line, i.e. start from 0.
    // See: <https://docs.rs/ropey/latest/ropey/struct.Rope.html#method.byte_to_line>
    let (row_and_col, lines) = collect_from_top_left(window, 0, 0);

    Viewport {
      window: SafeWindowRef::new(window),
      start_line: row_and_col.start_line,
      end_line: row_and_col.end_line,
      start_column: row_and_col.start_column,
      end_column: row_and_col.end_column,
      lines,
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn collect_from_top_left_for_nowrap1() {}
}
