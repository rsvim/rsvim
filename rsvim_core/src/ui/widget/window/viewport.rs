//! Buffer viewport on a window.

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
use icu::segmenter::WordSegmenter;
use ropey::RopeSlice;
use std::collections::BTreeMap;
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

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
/// specific line and column. Here it calls `start_line`, `start_column`, `end_line`, and there's
/// no `end_column`.
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
  // Maps from buffer's line number to its displayed information in the window.
  lines: BTreeMap<usize, LineViewport>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
// Tuple of start_line, end_line, start_column.
pub struct ViewportRect {
  pub start_line: usize,
  pub end_line: usize,
  pub start_column: usize,
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
            let mut sections: Vec<LineViewportSection> = vec![];

            let mut col = 0_u16;
            let mut char_length = 0_usize;
            let mut display_length = 0_u16;

            // Go through each char in the line.
            for (i, c) in line.chars().enumerate() {
              if i < start_column {
                continue;
              }
              if col >= width {
                break;
              }
              let char_width = strings::char_width(c, &buffer);
              if col + char_width >= width {
                break;
              }
              display_length += char_width;
              char_length += 1;
              // debug!(
              //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
              //   row, col, ch, cell_upos
              // );
              col += char_width;
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
          start_column,
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
  let actual_shape = window.actual_shape();
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!("_collect_from_top_left_for_wrap_nolinebreak");
  // debug!(
  //   "actual_shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
  //   actual_shape, upos, height, width,
  // );

  // If window is zero-sized.
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  // Get buffer arc pointer
  let buffer = window.buffer.upgrade().unwrap();

  // Lock buffer for read
  let buffer = rlock!(buffer);

  // if let Some(line) = buffer.rope().get_line(start_line) {
  //   debug!(
  //     "buffer.get_line ({:?}):'{:?}'",
  //     start_line,
  //     rpslice2line(&line),
  //   );
  // } else {
  //   debug!("buffer.get_line ({:?}):None", start_line);
  // }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut current_line = start_line;

  match buffer.rope().get_lines_at(start_line) {
    Some(mut buflines) => {
      // The `start_line` is inside the buffer.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;

      while row < height {
        match buflines.next() {
          Some(line) => {
            // If there's 1 more line in the buffer.
            let mut sections: Vec<LineViewportSection> = vec![];

            let mut col = 0_u16;
            let mut char_length = 0_usize;
            let mut display_length = 0_u16;

            for (i, c) in line.chars().enumerate() {
              if i < start_column {
                continue;
              }
              if col >= width {
                sections.push(LineViewportSection {
                  row,
                  char_length,
                  display_length,
                });
                row += 1;
                col = 0_u16;
                char_length = 0_usize;
                display_length = 0_u16;
                if row >= height {
                  break;
                }
              }

              let char_width = strings::char_width(c, &buffer);
              if col + char_width >= width {
                row += 1;
                sections.push(LineViewportSection {
                  row,
                  char_length,
                  display_length,
                });
                col = 0_u16;
                char_length = 0_usize;
                display_length = 0_u16;
                if row >= height {
                  break;
                }
              }
              display_length += char_width;
              char_length += 1;

              // debug!(
              //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
              //   row, col, ch, cell_upos
              // );
              col += char_width;
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
        // Iterate to next row.
        row += 1;
      }

      (
        ViewportRect {
          start_line,
          end_line: current_line,
          start_column,
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

fn truncate_line(line: &RopeSlice, max_bytes: usize) -> String {
  let mut builder = String::new();
  builder.reserve(max_bytes);
  for chunk in line.chunks() {
    if builder.len() > max_bytes {
      return builder;
    }
    builder.push_str(chunk);
  }
  builder
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=true`.
fn _collect_from_top_left_for_wrap_linebreak(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let actual_shape = window.actual_shape();
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!("_collect_from_top_left_for_wrap_linebreak");
  // debug!(
  //   "actual_shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
  //   actual_shape, upos, height, width,
  // );

  // If window is zero-sized.
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  // Get buffer arc pointer
  let buffer = window.buffer.upgrade().unwrap();

  // Lock buffer for read
  let buffer = rlock!(buffer);

  // if let Some(line) = buffer.rope().get_line(start_line) {
  //   debug!(
  //     "buffer.get_line ({:?}):'{:?}'",
  //     start_line,
  //     rpslice2line(&line),
  //   );
  // } else {
  //   debug!("buffer.get_line ({:?}):None", start_line);
  // }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut current_line = start_line;

  match buffer.rope().get_lines_at(start_line) {
    Some(mut buflines) => {
      // The `start_line` is inside the buffer.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;

      while row < height {
        match buflines.next() {
          Some(line) => {
            // If there's 1 more line in the buffer.
            let mut sections: Vec<LineViewportSection> = vec![];

            let mut col = 0_u16;
            let mut char_length = 0_usize;
            let mut display_length = 0_u16;

            // Chop the line into maximum chars can hold by current window, thus avoid those super
            // long lines for iteration performance.
            // NOTE: Use `height * width * 4`, 4 is for at most 4 bytes can hold a grapheme
            // cluster.
            let truncated_line = truncate_line(&line, height as usize * width as usize * 4);
            let word_boundaries: Vec<&str> = truncated_line.split_word_bounds().collect();
            debug!(
              "1-truncated_line: {:?}, word_boundaries: {:?}",
              truncated_line, word_boundaries
            );

            #[allow(unused_variables)]
            for (i, wd) in word_boundaries.iter().enumerate() {
              if row >= height {
                break;
              }
              let wd_width = wd
                .chars()
                .map(|c| strings::char_width(c, &buffer) as usize)
                .sum::<usize>();

              if wd_width + col as usize <= width as usize {
                // Enough space to place this word in current row
                char_length += wd.chars().count();
                display_length += wd_width as u16;
                col += wd_width as u16;
              } else {
                // Not enough space to place this word in current row.
                // There're two cases:
                // 1. The word can be placed in next empty row (since the column idx `col` will
                //    start from 0 in next row).
                // 2. The word is still too long to place in an entire row, so next row still
                //    cannot place it.
                // Anyway, we simply go to next row, and force render all of the word.
                sections.push(LineViewportSection {
                  row,
                  char_length,
                  display_length,
                });
                row += 1;
                col = 0_u16;
                char_length = 0_usize;
                display_length = 0_u16;
                if row >= height {
                  break;
                }

                for (j, c) in wd.chars().enumerate() {
                  if j < start_column {
                    continue;
                  }
                  if col >= width {
                    sections.push(LineViewportSection {
                      row,
                      char_length,
                      display_length,
                    });
                    row += 1;
                    col = 0_u16;
                    char_length = 0_usize;
                    display_length = 0_u16;
                    if row >= height {
                      break;
                    }
                  }
                  let char_width = strings::char_width(c, &buffer);
                  if col + char_width >= width {
                    break;
                  }
                  display_length += char_width;
                  char_length += 1;
                  // debug!(
                  //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
                  //   row, col, ch, cell_upos
                  // );
                  col += char_width;
                }
              }
            }

            line_viewports.insert(current_line, LineViewport { sections });
            current_line += 1;
          }
          None => { /* There's no more lines in the buffer. */ }
        }
        row += 1;
      }

      (
        ViewportRect {
          start_line,
          end_line: current_line,
          start_column,
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
      lines,
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn collect_from_top_left_for_nowrap1() {}
}
