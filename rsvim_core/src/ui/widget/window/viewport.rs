//! Buffer viewport on a window.

use crate::buf::BufferWk;
use crate::cart::{U16Pos, U16Rect, U16Size, URect};
use crate::defaults::grapheme::AsciiControlCode;
use crate::envar;
use crate::rlock;
use crate::ui::canvas::Cell;
use crate::ui::tree::internal::Inodeable;
use crate::ui::util::{ptr::SafeWindowRef, strings};
use crate::ui::widget::window::Window;

use geo::point;
use ropey::RopeSlice;
use std::collections::{BTreeMap, HashMap};
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Copy, Clone)]
/// The row information of a buffer line.
pub struct LineViewportRow {
  /// Start display column index for current row, starts from 0.
  ///
  /// NOTE: For the term _**display column**_, please see [`Viewport`].
  pub start_dcolumn_idx: usize,

  /// End display column index for current row.
  ///
  /// With start/end display column index, we could get several info:
  /// - The start/end char index for current row.
  /// - The total characters count.
  /// - The total character display width.
  pub end_dcolumn_idx: usize,
}

#[derive(Debug, Clone)]
/// All the displayed rows for a buffer line.
pub struct LineViewport {
  /// Maps from row index (based on the window) to a row in the buffer line, starts from 0.
  pub rows: BTreeMap<u16, LineViewportRow>,
}

#[derive(Debug, Copy, Clone)]
pub struct ViewportOptions {
  pub wrap: bool,
  pub line_break: bool,
}

#[derive(Debug, Clone)]
/// The viewport for a buffer.
///
/// There are several factors affecting the final display effects when a window showing a buffer:
///
/// 1. Window (display) options (such as ['wrap'](crate::defaults::win),
///    ['line-break'](crate::defaults::win)), they decide how does the line in the buffer renders
///    in the window.
/// 2. ASCII control codes and unicode, they decide how does a character renders in the window.
///
/// ## Case-1: Line-wrap and word-wrap
///
/// When 'wrap' option is `false`, and there's one very long line which is longer than the width
/// of the window. The viewport has to truncate the line, and only shows part of it. For example:
///
/// Example-1
///
/// ```text
/// |-------------------------------------|
/// |This is the beginning of the very lon|g line, which only shows the beginning part.
/// |-------------------------------------|
/// ```
///
/// Example-2
///
/// ```text
///                                           |---------------------------------------|
/// This is the beginning of the very long lin|e, which only shows the beginning part.|
///                                           |---------------------------------------|
/// ```
///
/// Example-1 only shows the beginning of the line, and example-2 only shows the ending of the
/// line.
///
/// When 'wrap' option is `true`, the long line will take multiple rows and try to use the whole
/// window to render all of it, while still been truncated if it's just too long to show within the
/// whole window. For example:
///
/// Example-3
///
/// ```text
/// |-------------------------------------|
/// |This is the beginning of the very lon|
/// |g line, which only shows the beginnin|
/// |g part.                              |
/// |-------------------------------------|
/// ```
///
/// Example-4
///
/// ```text
///  This is the beginning of the very lon
/// |-------------------------------------|
/// |g line, which only shows the beginnin|
/// |g part? No, even it sets `wrap=true`,|
/// | it is still been truncated because t|
/// |-------------------------------------|
///  he whole window cannot render it.
/// ```
///
/// Example-3 shows `wrap=true` can help a window renders the very long line if the window itself
/// is big enough. And example-4 shows the very long line will still be truncated if it's too long.
///
/// ## Case-2: ASCII control codes and unicodes
///
/// Most characters use 1 cell width in the terminal, such as alphabets (A-Z), numbers (0-9) and
/// punctuations. But ASCII control codes can use 2 or more cells width. For example:
///
/// Example-5
///
/// ```text
/// ^@^A^B^C^D^E^F^G^H<--HT-->
/// ^K^L^M^N^O^P^Q^R^S^T^U^V^W^X^Y^Z^[^\^]^^^_
/// ```
///
/// Example-5 shows how ASCII control codes render in Vim. Most of them use 2 cells width, except
/// horizontal tab (HT, 9) renders as 8 spaces (shows as `<--HT-->` in the example), and line feed
/// (LF, 10) renders as empty, i.e. 0 cell width, it just starts a new line.
///
/// Some unicode, especially Chinese/Japanese/Korean, can use 2 cells width as well. For example:
///
/// Example-6
///
/// ```text
/// 你好，Vim！
/// こんにちは、Vim！
/// 안녕 Vim!
/// ```
///
/// Taking all above scenarios into consideration, when rendering a buffer in a viewport, the most
/// important several anchors are:
///
/// - The start line of the buffer, which shows at the top row of the viewport (inclusive).
/// - The start display column of the buffer, which shows at the beginning cell of the viewport
///   (inclusive).
/// - The end line of the buffer, which is next to the bottom row of the viewport (exclusive).
/// - The end display column of the buffer, which is next to the last cell of the viewport
///   (exclusive).
///
/// NOTE: The _**display column**_ is neither the character index of the buffer, nor the column of
/// the window. When character using multiple cells width meet the wrap options and truncated line,
/// this becomes a little bit complicated. For example:
///
/// Example-7
///
/// ```text
///                                  31(LF)
/// 0  3 4                   24    30|
/// |  | |                   |      ||
///     |---------------------|
/// <--H|T-->This is the first| line.
/// This| is the second line. |
/// This| is the third line, 它 有一点点长。
///     |---------------------|
/// |  |                     | |          |||
/// 0  3                     24|         36||
///                            25         37|
///                                         38(LF)
/// ```
///
/// Example-7 shows a use case, at the beginning of the viewport, the horizontal tab
/// (`<--HT-->`, use 8 cells width) is been truncated. And at the end of the viewport, the Chinese
/// character (`它`, use 2 cells width) is also been truncated.
///
/// For the X-axis in the viewport, here use the term _**display column**_, it's based on the
/// display width of the character, not the index in the buffer. In example-7, the start display
/// column is 4 (the `T` in the `<--HT-->`, inclusive), the end display column is 26 (the right half
/// part in the `它` character, exclusive).
///
/// To distinguish the display column (in the buffer) from the cell column (in the
/// window/terminal) in the source code, it's named `dcolumn` (short for `display_column`).
///
/// When rendering a buffer, viewport will need to go through each lines and characters in the
/// buffer to ensure how it display. The going through can start from 4 anchors:
///
/// 1. Start from top left corner, i.e. the start line (`start_line_idx`) and start display column
///    (`start_dcolumn_idx`).
/// 2. Start from top right corner, i.e. the start line (`start_line_idx`) and end display column
///    (`end_dcolumn_idx`).
/// 3. Start from bottom left corner, i.e. the end line (`end_line_idx`) and start display column
///    (`start_dcolumn_idx`).
/// 4. Start from bottom right corner, i.e. the end line (`end_line_idx`) and end display column
///    (`end_dcolumn_idx`).
pub struct Viewport {
  // Options.
  options: ViewportOptions,

  // Buffer.
  buffer: BufferWk,

  // Actual shape.
  actual_shape: U16Rect,

  // Start line index in the buffer, starts from 0.
  start_line_idx: usize,

  // End line index in the buffer.
  end_line_idx: usize,

  // Start display column index in the buffer, starts from 0.
  start_dcolumn_idx: usize,

  // End display column index in the buffer.
  end_dcolumn_idx: usize,

  // Maps from buffer line index to its displayed rows in the window.
  lines: BTreeMap<usize, LineViewport>,

  // Cached char idx and its display width on the line, to improve performance when the viewport
  // moves and the `start_dcolumn_idx` is not 0.
  //
  // For example when the line is very long, and cursor keeps moving to the right side on the line,
  // viewport will have to always sum up the display width of all the chars before the
  // `start_dcolumn_idx`, which is `O(N)` (`N` is the chars count before the `start_dcolumn_idx`,
  // or more generally speaking, `N` is the length of the line).
  //
  // The structure maps from the line index to the (char_idx, display width) pair. Once a line is
  // been modified, the related cache for the line should be reset.
  prev_display_width: HashMap<usize, (usize, usize)>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
/// Rectangle of line/dcolumn index inside [`Viewport`].
pub struct ViewportRect {
  // Start line index in the buffer, starts from 0.
  pub start_line_idx: usize,

  // End line index in the buffer.
  pub end_line_idx: usize,

  // Start display column index in the buffer, starts from 0.
  pub start_dcolumn_idx: usize,

  // End display column index in the buffer.
  pub end_dcolumn_idx: usize,
}

// Given the buffer and window size, collect information from start line and column, i.e. from the
// top-left corner.
fn collect_from_top_left(
  options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line_idx: usize,
  start_dcolumn_idx: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  // If window is zero-sized.
  let height = actual_shape.height();
  let width = actual_shape.width();
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  match (options.wrap, options.line_break) {
    (false, _) => _collect_from_top_left_with_nowrap(
      options,
      buffer,
      actual_shape,
      start_line_idx,
      start_dcolumn_idx,
    ),
    (true, false) => _collect_from_top_left_with_wrap_nolinebreak(
      options,
      buffer,
      actual_shape,
      start_line_idx,
      start_dcolumn_idx,
    ),
    (true, true) => _collect_from_top_left_with_wrap_linebreak(
      options,
      buffer,
      actual_shape,
      start_line_idx,
      start_dcolumn_idx,
    ),
  }
}

#[allow(dead_code)]
fn rpslice2line(s: &RopeSlice) -> String {
  let mut builder = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
}

// Implement [`collect_from_top_left`] with option `wrap=false`.
fn _collect_from_top_left_with_nowrap(
  options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line_idx: usize,
  start_dcolumn_idx: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!(
    "_collect_from_top_left_with_nowrap, actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width
  );

  // Get buffer arc pointer, and lock for read.
  let buffer = buffer.upgrade().unwrap();
  let buffer = rlock!(buffer);

  debug!(
    "buffer.get_line ({:?}):{:?}",
    start_line_idx,
    match buffer.get_line(start_line_idx) {
      Some(line) => rpslice2line(&line),
      None => "None".to_string(),
    }
  );

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut max_dcolumn_idx = start_dcolumn_idx;

  match buffer.get_lines_at(start_line_idx) {
    // The `start_line_idx` is in the buffer.
    Some(buflines) => {
      // The first `row` in the window maps to the `start_line_idx` in the buffer.
      let mut row = 0;
      let mut current_line_idx = start_line_idx;

      for (l, line) in buflines.enumerate() {
        if row >= height {
          break;
        }
        debug!(
          "0-l:{:?}, line:'{:?}', current_line:{:?}",
          l,
          rpslice2line(&line),
          current_line_idx
        );

        let mut rows: BTreeMap<u16, LineViewportRow> = BTreeMap::new();
        let mut col = 0_u16;
        let mut start_dcol = 0_usize;
        let mut end_dcolumn_idx = 0_u16;

        let mut buffer_column_index = buffer.get_column_index(current_line_idx).unwrap();

        // Go through each char in the line.
        for (i, c) in line.chars().enumerate() {
          let display_width = buffer_column_index.get_width_until_char(i).unwrap();
          if display_width < start_dcolumn_idx {
            continue;
          }
          if col >= width {
            break;
          }
          if display_width == 0 && i + 1 == line.len_chars() {
            break;
          }
          if col + display_width > width {
            break;
          }
          end_dcolumn_idx += display_width;
          start_dcol += 1;
          debug!(
            "1-row:{:?}, col:{:?}, c:{:?}, char_width:{:?}, chars_length:{:?}, chars_width:{:?}",
            row, col, c, display_width, start_dcol, end_dcolumn_idx
          );
          col += display_width;
          max_dcolumn_idx = std::cmp::max(i, max_dcolumn_idx);
        }

        rows.push(LineViewportRow {
          row_idx: row,
          start_dcolumn_idx: start_dcol,
          chars_width: end_dcolumn_idx,
        });
        line_viewports.insert(current_line_idx, LineViewport { rows });
        debug!(
          "2-current_line:{:?}, row:{:?}, chars_length:{:?}, chars_width:{:?}",
          current_line_idx, row, start_dcol, end_dcolumn_idx
        );
        // Go to next row and line
        current_line_idx += 1;
        row += 1;
      }

      debug!("3-current_line:{:?}, row:{:?}", current_line_idx, row);
      (
        ViewportRect {
          start_line_idx,
          end_line_idx: current_line_idx,
          start_dcolumn_idx: start_line_idx,
          end_dcolumn_idx: max_dcolumn_idx + 1,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      debug!("4-no start_line");
      (ViewportRect::default(), BTreeMap::new())
    }
  }
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=false`.
fn _collect_from_top_left_with_wrap_nolinebreak(
  options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line_idx: usize,
  start_dcolumn_idx: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!(
    "_collect_from_top_left_with_wrap_nolinebreak, actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width
  );

  // Get buffer arc pointer, and lock for read.
  let buffer = buffer.upgrade().unwrap();
  let buffer = rlock!(buffer);

  if let Some(line) = buffer.get_line(start_line_idx) {
    debug!(
      "buffer.get_line ({:?}):'{:?}'",
      start_line_idx,
      rpslice2line(&line),
    );
  } else {
    debug!("buffer.get_line ({:?}):None", start_line_idx);
  }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut max_column = start_dcolumn_idx;

  match buffer.get_lines_at(start_line_idx) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;
      let mut current_line = start_line_idx;

      for (l, line) in buflines.enumerate() {
        if row >= height {
          break;
        }
        debug!(
          "0-l:{:?}, line:'{:?}', current_line:{:?}",
          l,
          rpslice2line(&line),
          current_line
        );

        let mut sections: Vec<LineViewportRow> = vec![];
        let mut col = 0_u16;
        let mut chars_length = 0_usize;
        let mut chars_width = 0_u16;

        for (i, c) in line.chars().enumerate() {
          if i < start_dcolumn_idx {
            continue;
          }
          if col >= width {
            debug!(
              "1-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?}",
              row, col, c, chars_length, chars_width
            );
            max_column = std::cmp::max(i, max_column);
            sections.push(LineViewportRow {
              row_idx: row,
              chars_length,
              chars_width,
            });
            row += 1;
            col = 0_u16;
            chars_length = 0_usize;
            chars_width = 0_u16;
            if row >= height {
              debug!(
                    "2-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?} height/width:{:?}/{:?}",
                    row, col, c, chars_length, chars_width, height, width
                  );
              break;
            }
          }

          let char_width = strings::char_width(c, &buffer);
          if char_width == 0 && i + 1 == line.len_chars() {
            debug!(
                    "3-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?} i:{}, line.len_chars:{}",
                    row, col, c, chars_length, chars_width, i, line.len_chars()
                  );
            break;
          }
          if col + char_width > width {
            debug!(
                    "4-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?} col({})+char_width({}) > width({})",
                    row, col, c, chars_length, chars_width, col, char_width, width
                  );
            max_column = std::cmp::max(i, max_column);
            sections.push(LineViewportRow {
              row_idx: row,
              chars_length,
              chars_width,
            });
            row += 1;
            col = 0_u16;
            chars_length = 0_usize;
            chars_width = 0_u16;
            if row >= height {
              debug!(
                    "5-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?} height/width:{}/{}",
                    row, col, c, chars_length, chars_width, height, width
                  );
              break;
            }
          }
          chars_width += char_width;
          chars_length += 1;
          col += char_width;
          max_column = std::cmp::max(i, max_column);
          debug!(
            "6-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?}",
            row, col, c, chars_length, chars_width
          );
        }

        debug!(
          "7-row:{:?}, col:{:?}, chars_length:{:?}, chars_width:{:?}, current_line:{}",
          row, col, chars_length, chars_width, current_line
        );
        sections.push(LineViewportRow {
          row_idx: row,
          chars_length,
          chars_width,
        });
        line_viewports.insert(current_line, LineViewport { rows: sections });
        current_line += 1;
        row += 1;
      }

      debug!("9-row:{}, current_line:{}", row, current_line);
      (
        ViewportRect {
          start_line_idx,
          end_line_idx: current_line,
          start_dcolumn_idx,
          end_dcolumn_idx: max_column + 1,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      debug!("10-no start_line:{}", start_line_idx);
      (ViewportRect::default(), BTreeMap::new())
    }
  }
}

fn truncate_line(line: &RopeSlice, start_column: usize, max_bytes: usize) -> String {
  let mut builder = String::new();
  builder.reserve(max_bytes);
  for (i, c) in line.chars().enumerate() {
    if i < start_column {
      continue;
    }
    if builder.len() > max_bytes {
      return builder;
    }
    builder.push(c);
  }
  builder
}

// Implement [`collect_from_top_left`] with option `wrap=true` and `line-break=true`.
fn _collect_from_top_left_with_wrap_linebreak(
  options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line_idx: usize,
  start_dcolumn_idx: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!(
    "_collect_from_top_left_with_wrap_linebreak, actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width
  );

  // Get buffer arc pointer, and lock for read.
  let buffer = buffer.upgrade().unwrap();
  let buffer = rlock!(buffer);

  if let Some(line) = buffer.get_line(start_line_idx) {
    debug!(
      "buffer.get_line ({:?}):'{:?}'",
      start_line_idx,
      rpslice2line(&line),
    );
  } else {
    debug!("buffer.get_line ({:?}):None", start_line_idx);
  }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut max_column = start_dcolumn_idx;

  match buffer.get_lines_at(start_line_idx) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;
      let mut current_line = start_line_idx;

      for (l, line) in buflines.enumerate() {
        if row >= height {
          break;
        }
        let mut sections: Vec<LineViewportRow> = vec![];

        let mut col = 0_u16;
        let mut chars_length = 0_usize;
        let mut chars_width = 0_u16;
        let mut wd_length = 0_usize;

        // Chop the line into maximum chars can hold by current window, thus avoid those super
        // long lines for iteration performance.
        // NOTE: Use `height * width * 4`, 4 is for at most 4 bytes can hold a grapheme
        // cluster.
        let truncated_line = truncate_line(
          &line,
          start_dcolumn_idx,
          height as usize * width as usize * 4,
        );
        let word_boundaries: Vec<&str> = truncated_line.split_word_bounds().collect();
        debug!(
          "0-truncated_line: {:?}, word_boundaries: {:?}",
          truncated_line, word_boundaries
        );

        for (i, wd) in word_boundaries.iter().enumerate() {
          if row >= height {
            break;
          }
          debug!(
            "1-l:{:?}, line:'{:?}', current_line:{:?}, max_column:{:?}",
            l,
            rpslice2line(&line),
            current_line,
            max_column
          );

          let (wd_chars, wd_width) = wd
            .chars()
            .map(|c| (1_usize, strings::char_width(c, &buffer) as usize))
            .fold(
              (0_usize, 0_usize),
              |(acc_chars, acc_width), (c_count, c_width)| {
                (acc_chars + c_count, acc_width + c_width)
              },
            );

          if wd_width == 0 && i + 1 == word_boundaries.len() {
            debug!(
              "2-row:{:?}, col:{:?}, wd_chars:{:?}, wd_width:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
              row, col, wd_chars, wd_width, chars_length,  chars_width, max_column
            );
            break;
          }

          if wd_width + col as usize <= width as usize {
            // Enough space to place this word in current row
            chars_length += wd_chars;
            chars_width += wd_width as u16;
            col += wd_width as u16;
            wd_length += wd_width;
            debug!(
              "3-row:{:?}, col:{:?}, wd_chars:{:?}, wd_width:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
              row, col, wd_chars, wd_width, chars_length, chars_width, max_column
            );
          } else {
            // Not enough space to place this word in current row.
            // There're two cases:
            // 1. The word can be placed in next empty row (since the column idx `col` will
            //    start from 0 in next row).
            // 2. The word is still too long to place in an entire row, so next row still
            //    cannot place it.
            // Anyway, we simply go to next row, and force render all of the word.
            sections.push(LineViewportRow {
              row_idx: row,
              chars_length,
              chars_width,
            });
            row += 1;
            col = 0_u16;
            chars_length = 0_usize;
            chars_width = 0_u16;

            if row >= height {
              debug!(
                  "4-row:{:?}, col:{:?}, wd_chars:{:?}, wd_width:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
                  row, col, wd_chars, wd_width, chars_length, chars_width, max_column
                );
              break;
            }

            for c in wd.chars() {
              if col >= width {
                sections.push(LineViewportRow {
                  row_idx: row,
                  chars_length,
                  chars_width,
                });
                row += 1;
                col = 0_u16;
                chars_length = 0_usize;
                chars_width = 0_u16;
                if row >= height {
                  debug!(
                      "5-row:{:?}, col:{:?}, wd_chars:{:?}, wd_width:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
                        row, col, wd_chars, wd_width, chars_length, chars_width, max_column
                    );
                  break;
                }
              }
              let char_width = strings::char_width(c, &buffer);
              if col + char_width > width {
                debug!( "6-row:{:?}, col:{:?}, wd_chars:{:?}, wd_width:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
                    row, col, wd_chars, wd_width, chars_length, chars_width, max_column
                  );
                break;
              }
              chars_width += char_width;
              chars_length += 1;
              col += char_width;
              wd_length += char_width as usize;
              debug!(
              "7-row:{:?}, col:{:?}, wd_chars:{:?}, wd_width:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
              row, col, wd_chars, wd_width, chars_length, chars_width, max_column
            );
            }
          }
        }

        max_column = std::cmp::max(max_column, start_dcolumn_idx + wd_length);
        debug!(
          "8-row:{:?}, col:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
          row, col, chars_length, chars_width, max_column
        );
        sections.push(LineViewportRow {
          row_idx: row,
          chars_length,
          chars_width,
        });
        line_viewports.insert(current_line, LineViewport { rows: sections });
        current_line += 1;
        row += 1;
      }

      debug!(
        "9-row:{}, current_line:{}, max_column:{}",
        row, current_line, max_column
      );
      (
        ViewportRect {
          start_line_idx,
          end_line_idx: current_line,
          start_dcolumn_idx,
          end_dcolumn_idx: max_column,
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
  pub fn new(options: &ViewportOptions, buffer: BufferWk, actual_shape: &U16Rect) -> Self {
    // By default the viewport start from the first line, i.e. starts from 0.
    let (rectangle, lines) = collect_from_top_left(options, buffer.clone(), actual_shape, 0, 0);

    Viewport {
      options: *options,
      buffer,
      actual_shape: *actual_shape,
      start_line_idx: rectangle.start_line_idx,
      end_line_idx: rectangle.end_line_idx,
      start_dcolumn_idx: rectangle.start_dcolumn_idx,
      end_dcolumn_idx: rectangle.end_dcolumn_idx,
      lines,
    }
  }

  /// Get start line index in the buffer, starts from 0.
  pub fn start_line_idx(&self) -> usize {
    self.start_line_idx
  }

  /// Get end line index in the buffer.
  pub fn end_line_idx(&self) -> usize {
    self.end_line_idx
  }

  /// Get start display column index in the buffer, starts from 0.
  pub fn start_dcolumn_idx(&self) -> usize {
    self.start_dcolumn_idx
  }

  /// Get end display column index in the buffer.
  ///
  /// NOTE: The `end_dcolumn_idx` indicates the maximum display column index of all the buffer
  /// lines displayed in the window. This is useful when the lines are been truncated and cannot
  /// display all of them in the window.
  pub fn end_dcolumn_idx(&self) -> usize {
    self.end_dcolumn_idx
  }

  /// Get viewport information by lines.
  pub fn lines(&self) -> &BTreeMap<usize, LineViewport> {
    &self.lines
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::BufferArc;
  use crate::cart::{IRect, U16Size};
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Tree;
  use crate::ui::widget::window::WindowLocalOptions;

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  #[allow(dead_code)]
  static INIT: Once = Once::new();

  fn make_viewport_from_size(
    size: U16Size,
    buffer: BufferArc,
    window_options: &WindowLocalOptions,
  ) -> Viewport {
    let mut tree = Tree::new(size);
    tree.set_local_options(window_options);
    let window_shape = IRect::new((0, 0), (size.width() as isize, size.height() as isize));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &mut tree);
    Viewport::new(&mut window)
  }

  fn _test_collect_from_top_left(
    size: U16Size,
    buffer: BufferArc,
    actual: &Viewport,
    expect: &Vec<&str>,
    expect_end_line: usize,
    expect_end_column: usize,
  ) {
    info!(
      "actual start_line/end_line:{:?}/{:?}, start_column/end_column:{:?}/{:?}",
      actual.start_line_idx(),
      actual.end_line_idx(),
      actual.start_dcolumn_idx(),
      actual.end_dcolumn_idx()
    );
    for (k, v) in actual.lines().iter() {
      info!("actual {:?}: {:?}", k, v);
    }
    info!("expect:{:?}", expect);

    let buffer = buffer.read();
    let buflines = buffer.get_lines_at(actual.start_line_idx()).unwrap();

    let mut row = 0_usize;
    for (l, line) in buflines.enumerate() {
      if row >= size.height() as usize {
        break;
      }
      info!("l-{:?}", l);
      let line_viewport = actual.lines().get(&l).unwrap();
      let sections = line_viewport.rows.clone();
      info!("l-{:?}, line_viewport:{:?}", l, line_viewport);
      let mut line_chars = line.chars();
      for (j, sec) in sections.iter().enumerate() {
        assert_eq!(sec.row_idx, row as u16);
        let mut payload = String::new();
        for _k in 0..sec.chars_length {
          payload.push(line_chars.next().unwrap());
        }
        info!(
          "j-{:?}, payload:{:?}, expect[row-{:?}]:{:?}",
          j, payload, row, expect[row]
        );
        assert_eq!(payload, expect[row]);
        row += 1;
      }
    }

    assert_eq!(actual.start_line_idx(), 0);
    assert_eq!(actual.end_line_idx(), expect_end_line);
    assert_eq!(actual.start_dcolumn_idx(), 0);
    assert_eq!(actual.end_dcolumn_idx(), expect_end_column);
    assert_eq!(*actual.lines().first_key_value().unwrap().0, 0);
    assert_eq!(
      *actual.lines().last_key_value().unwrap().0,
      actual.end_line_idx() - 1
    );
  }

  #[test]
  fn collect_from_top_left_for_nowrap1() {
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
      "This is a ",
      "But still ",
      "  1. When ",
      "  2. When ",
      "     * The",
      "     * The",
      "",
    ];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 8, 10);
  }

  #[test]
  fn collect_from_top_left_for_nowrap2() {
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
      "Hello, RSVIM!",
      "This is a quite simple and ",
      "But still it contains sever",
      "  1. When the line is small",
      "  2. When the line is too l",
      "     * The extra parts are ",
      "     * The extra parts are ",
      "",
    ];

    let size = U16Size::new(27, 15);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 8, 27);
  }

  #[test]
  fn collect_from_top_left_for_nowrap3() {
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
      "Hello, RSVIM!",
      "This is a quite simple and smal",
      "But still it contains several t",
      "  1. When the line is small eno",
      "  2. When the line is too long ",
      "     * The extra parts are been",
      "     * The extra parts are spli",
      "",
    ];

    let size = U16Size::new(31, 11);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 8, 31);
  }

  #[test]
  fn collect_from_top_left_for_nowrap4() {
    // INIT.call_once(test_log_init);

    let buffer = make_empty_buffer();
    let expect = vec![""];

    let size = U16Size::new(20, 20);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer, &options);
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);

    assert_eq!(actual.start_line_idx(), 0);
    assert_eq!(actual.end_line_idx(), expect.len());
    assert_eq!(actual.start_dcolumn_idx(), 0);
    assert_eq!(actual.end_dcolumn_idx(), 1);
    assert_eq!(*actual.lines().first_key_value().unwrap().0, 0);
    assert_eq!(
      *actual.lines().last_key_value().unwrap().0,
      actual.end_line_idx() - 1
    );
  }

  #[test]
  fn collect_from_top_left_for_wrap_nolinebreak1() {
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
    let expect = vec![
      "Hello, RSV",
      "IM!",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.",
      "But still ",
      "it contain",
      "s several ",
      "",
    ];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 3, 44);
  }

  #[test]
  fn collect_from_top_left_for_wrap_nolinebreak2() {
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
      "Hello, RSVIM!",
      "This is a quite simple and ",
      "small test lines.",
      "But still it contains sever",
      "al things we want to test:",
      "  1. When the line is small",
      " enough to completely put i",
      "nside a row of the window c",
      "ontent widget, then the lin",
      "e-wrap and word-wrap doesn'",
      "t affect the rendering.",
      "  2. When the line is too l",
      "ong to be completely put in",
      " a row of the window conten",
      "t widget, there're multiple",
      "",
    ];

    let size = U16Size::new(27, 15);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 5, 158);
  }

  #[test]
  fn collect_from_top_left_for_wrap_nolinebreak3() {
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
      "Hello, RSVIM!",
      "This is a quite simple and smal",
      "l test lines.",
      "But still it contains several t",
      "hings we want to test:",
      "  1. When the line is small eno",
      "ugh to completely put inside a ",
      "row of the window content widge",
      "t, then the line-wrap and word-",
      "wrap doesn't affect the renderi",
      "ng.",
      "",
    ];

    let size = U16Size::new(31, 11);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 4, 158);
  }

  #[test]
  fn collect_from_top_left_for_wrap_nolinebreak4() {
    let buffer = make_empty_buffer();
    let expect = vec![""];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 1, 1);
  }

  #[test]
  fn collect_from_top_left_for_wrap_linebreak1() {
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
    let expect = vec![
      "Hello, ",
      "RSVIM!",
      "This is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".",
      "But still ",
      "it ",
      "",
    ];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 3, 44);
  }

  #[test]
  fn collect_from_top_left_for_wrap_linebreak2() {
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
    let expect = vec![
      "Hello, RSVIM!",
      "This is a quite simple and ",
      "small test lines.",
      "But still it contains ",
      "several things we want to ",
      "test:",
      "  1. When the line is small",
      " enough to completely put ",
      "inside a row of the window ",
      "content widget, then the ",
      "line-wrap and word-wrap ",
      "doesn't affect the ",
      "rendering.",
      "  2. When the line is too ",
      "long to be completely put ",
      "",
    ];

    let size = U16Size::new(27, 15);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 5, 158);
  }

  #[test]
  fn collect_from_top_left_for_wrap_linebreak3() {
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
      "Hello, RSVIM!",
      "This is a quite simple and ",
      "small test lines.",
      "But still it contains several ",
      "things we want to test:",
      "  1. When the line is small ",
      "enough to completely put inside",
      " a row of the window content ",
      "widget, then the line-wrap and ",
      "word-wrap doesn't affect the ",
      "rendering.",
      "",
    ];

    let size = U16Size::new(31, 11);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 4, 158);
  }

  #[test]
  fn collect_from_top_left_for_wrap_linebreak4() {
    // INIT.call_once(test_log_init);

    let buffer = make_empty_buffer();
    let expect = vec![""];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    _test_collect_from_top_left(size, buffer, &actual, &expect, 1, 0);
  }
}
