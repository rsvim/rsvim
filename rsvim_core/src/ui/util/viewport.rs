//! Buffer viewport on a window.

use crate::buf::BufferWk;
use crate::cart::{U16Pos, U16Size, URect};
use crate::envar;
use crate::rlock;
use crate::ui::util::ptr::SafeWindowRef;
use crate::ui::widget::window::Window;

use std::collections::BTreeMap;
use tracing::debug;

#[derive(Debug, Copy, Clone)]
/// The section information of a buffer line. One section is exactly one row in a window.
pub struct LineSection {
  // Row index in the window.
  row: u16,
  // chars length
  chars_length: usize,
  // unicode displayed length
  display_length: usize,
}

#[derive(Debug, Clone)]
/// All the sections of a buffer line. Since one line could occupy multiple rows in a window.
pub struct LineViewport {
  sections: Vec<LineSection>,
}

#[derive(Debug, Clone)]
/// The buffer viewport on a window. The range is left-inclusive right-exclusive, or top-inclusive
/// bottom-exclusive, i.e. `[start_row, end_row)` or `[start_column, end_column)`. All lines,
/// rows and columns index are start from 0.
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
  // Start row index.
  start_row: usize,
  // End row index.
  end_row: usize,
  // Start column index.
  start_col: usize,
  // End column index.
  end_col: usize,
  // Maps from buffer's each line to its displayed information in the window.
  lines: BTreeMap<usize, LineViewport>,
}

#[derive(Debug, Copy, Clone)]
// Tuple of start_row, end_row, start_column, end_column.
struct RowAndColumnResult(usize, usize, usize, usize);

// Given the buffer and window size, collect information from start line and column, i.e. from the
// top-left corner.
fn collect_from_top_left(
  window: &Window,
  start_row: usize,
  start_col: usize,
) -> (RowAndColumnResult, BTreeMap<usize, LineViewport>) {
  match (window.wrap(), window.line_break()) {
    (false, _) => _collect_from_top_left_for_nowrap(window, start_row, start_col),
    (true, false) => _collect_from_top_left_for_wrap_nolinebreak(window, start_row, start_col),
    (true, true) => _collect_from_top_left_for_wrap_linebreak(window, start_row, start_col),
  }
}

// Implement [`collect_from_top_left`] with option `wrap=false`.
fn _collect_from_top_left_for_nowrap(
  window: &Window,
  start_row: usize,
  start_col: usize,
) -> (RowAndColumnResult, BTreeMap<usize, LineViewport>) {
  let actual_shape = window.actual_shape();
  let upos: U16Pos = actual_shape.min().into();
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!("_collect_from_top_left_for_nowrap");
  // debug!(
  //   "actual shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
  //   actual_shape, upos, height, width,
  // );

  // If window is zero-sized.
  if height == 0 || width == 0 {
    return (RowAndColumnResult(0, 0, 0, 0), BTreeMap::new());
  }

  // Get buffer arc pointer
  let buffer = window.buffer().upgrade().unwrap();

  // Lock buffer for read
  let buffer = rlock!(buffer);

  // if let Some(line) = buffer.rope().get_line(start_row) {
  //   debug!(
  //     "buffer.get_line ({:?}):'{:?}'",
  //     start_row,
  //     rslice2line(&line),
  //   );
  // } else {
  //   debug!("buffer.get_line ({:?}):None", start_row);
  // }

  let mut row_and_col = RowAndColumnResult(0, 0, 0, 0);
  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

  match buffer.rope().get_lines_at(start_row) {
    Some(mut buflines) => {
      // The `start_row` is inside the buffer.
      // Parse the lines from `start_row` until the end of the buffer or the window.

      // The first `row` in the window maps to the `start_row` in the buffer.
      let mut row = 0;

      while row < height {
        match buflines.next() {
          Some(line) => {
            // If there's 1 more line in the buffer.
            let mut col = 0_u16;

            // Go through each char in the line.
            for ch in line.chars() {
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

  (RowAndColumnResult(0, 0, 0, 0), BTreeMap::new())
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=false`.
fn _collect_from_top_left_for_wrap_nolinebreak(
  window: &Window,
  start_row: usize,
  start_col: usize,
) -> (RowAndColumnResult, BTreeMap<usize, LineViewport>) {
  (RowAndColumnResult(0, 0, 0, 0), BTreeMap::new())
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=true`.
fn _collect_from_top_left_for_wrap_linebreak(
  window: &Window,
  start_row: usize,
  start_col: usize,
) -> (RowAndColumnResult, BTreeMap<usize, LineViewport>) {
  (RowAndColumnResult(0, 0, 0, 0), BTreeMap::new())
}

impl Viewport {
  pub fn new(window: &mut Window) -> Self {
    // By default the viewport start from the first line, i.e. start from 0.
    // See: <https://docs.rs/ropey/latest/ropey/struct.Rope.html#method.byte_to_line>
    let (row_and_col, lines) = collect_from_top_left(window, 0, 0);

    Viewport {
      window: SafeWindowRef::new(window),
      start_row: row_and_col.0,
      end_row: row_and_col.1,
      start_col: row_and_col.2,
      end_col: row_and_col.3,
      lines,
    }
  }
}
