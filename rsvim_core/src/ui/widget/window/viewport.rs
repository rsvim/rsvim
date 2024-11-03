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
use ropey::RopeSlice;
use std::collections::BTreeMap;
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Copy, Clone)]
/// The section information of a buffer line. One section is exactly one row in a window.
pub struct LineViewportSection {
  /// Row index in the window.
  ///
  /// NOTE: The row index is based on the window widget, i.e. the top row of the window widget is
  /// 0.
  pub row: u16,

  /// Chars length/count.
  pub chars_length: usize,

  /// Chars display length (a unicode char can occupy 1~2 terminal cells width).
  pub chars_width: u16,
}

#[derive(Debug, Clone)]
/// All the sections of a buffer line. Since one line could occupy multiple rows in a window.
pub struct LineViewport {
  pub sections: Vec<LineViewportSection>,
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
/// Rectangle of `start_line`, `end_line`, `start_column`, `end_column`.
pub struct ViewportRect {
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

#[allow(dead_code)]
fn rpslice2line(s: &RopeSlice) -> String {
  let mut builder = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
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
  //     rpslice2line(&line),
  //   );
  // } else {
  //   debug!("buffer.get_line ({:?}):None", start_line);
  // }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut max_column = start_column;

  match buffer.rope().get_lines_at(start_line) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.
      // Parse the lines from `start_line` until the end of the buffer or the window.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;
      let mut current_line = start_line;

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

        let mut sections: Vec<LineViewportSection> = vec![];
        let mut col = 0_u16;
        let mut chars_length = 0_usize;
        let mut chars_width = 0_u16;

        // Go through each char in the line.
        for (i, c) in line.chars().enumerate() {
          if i < start_column {
            continue;
          }
          if col >= width {
            break;
          }
          let char_width = strings::char_width(c, &buffer);
          if char_width == 0 && i + 1 == line.len_chars() {
            break;
          }
          if col + char_width > width {
            break;
          }
          chars_width += char_width;
          chars_length += 1;
          debug!(
            "1-row:{:?}, col:{:?}, c:{:?}, char_width:{:?}, chars_length:{:?}, chars_width:{:?}",
            row, col, c, char_width, chars_length, chars_width
          );
          col += char_width;
          max_column = std::cmp::max(i, max_column);
        }

        sections.push(LineViewportSection {
          row,
          chars_length,
          chars_width,
        });
        line_viewports.insert(current_line, LineViewport { sections });
        debug!(
          "2-current_line:{:?}, row:{:?}, chars_length:{:?}, chars_width:{:?}",
          current_line, row, chars_length, chars_width
        );
        // Go to next row and line
        current_line += 1;
        row += 1;
      }

      debug!("3-current_line:{:?}, row:{:?}", current_line, row);
      (
        ViewportRect {
          start_line,
          end_line: current_line,
          start_column,
          end_column: max_column + 1,
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
fn _collect_from_top_left_for_wrap_nolinebreak(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let actual_shape = window.actual_shape();
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!("_collect_from_top_left_for_wrap_nolinebreak");
  debug!(
    "actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width,
  );

  // If window is zero-sized.
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  // Get buffer arc pointer
  let buffer = window.buffer.upgrade().unwrap();

  // Lock buffer for read
  let buffer = rlock!(buffer);

  if let Some(line) = buffer.rope().get_line(start_line) {
    debug!(
      "buffer.get_line ({:?}):'{:?}'",
      start_line,
      rpslice2line(&line),
    );
  } else {
    debug!("buffer.get_line ({:?}):None", start_line);
  }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut max_column = start_column;

  match buffer.rope().get_lines_at(start_line) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;
      let mut current_line = start_line;

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

        let mut sections: Vec<LineViewportSection> = vec![];
        let mut col = 0_u16;
        let mut chars_length = 0_usize;
        let mut chars_width = 0_u16;

        for (i, c) in line.chars().enumerate() {
          if i < start_column {
            continue;
          }
          if col >= width {
            debug!(
              "1-row:{:?}, col:{:?}, c:{:?}, chars_length:{:?}, chars_width:{:?}",
              row, col, c, chars_length, chars_width
            );
            max_column = std::cmp::max(i, max_column);
            sections.push(LineViewportSection {
              row,
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
            sections.push(LineViewportSection {
              row,
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
        sections.push(LineViewportSection {
          row,
          chars_length,
          chars_width,
        });
        line_viewports.insert(current_line, LineViewport { sections });
        current_line += 1;
        row += 1;
      }

      debug!("9-row:{}, current_line:{}", row, current_line);
      (
        ViewportRect {
          start_line,
          end_line: current_line,
          start_column,
          end_column: max_column + 1,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      debug!("10-no start_line:{}", start_line);
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
fn _collect_from_top_left_for_wrap_linebreak(
  window: &Window,
  start_line: usize,
  start_column: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let actual_shape = window.actual_shape();
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug!("_collect_from_top_left_for_wrap_linebreak");
  debug!(
    "actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width,
  );

  // If window is zero-sized.
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  // Get buffer arc pointer
  let buffer = window.buffer.upgrade().unwrap();

  // Lock buffer for read
  let buffer = rlock!(buffer);

  if let Some(line) = buffer.rope().get_line(start_line) {
    debug!(
      "buffer.get_line ({:?}):'{:?}'",
      start_line,
      rpslice2line(&line),
    );
  } else {
    debug!("buffer.get_line ({:?}):None", start_line);
  }

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  let mut max_column = start_column;

  match buffer.rope().get_lines_at(start_line) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.

      // The first `row` in the window maps to the `start_line` in the buffer.
      let mut row = 0;
      let mut current_line = start_line;

      for (l, line) in buflines.enumerate() {
        if row >= height {
          break;
        }
        let mut sections: Vec<LineViewportSection> = vec![];

        let mut col = 0_u16;
        let mut chars_length = 0_usize;
        let mut chars_width = 0_u16;
        let mut wd_length = 0_usize;

        // Chop the line into maximum chars can hold by current window, thus avoid those super
        // long lines for iteration performance.
        // NOTE: Use `height * width * 4`, 4 is for at most 4 bytes can hold a grapheme
        // cluster.
        let truncated_line =
          truncate_line(&line, start_column, height as usize * width as usize * 4);
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
            sections.push(LineViewportSection {
              row,
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
                sections.push(LineViewportSection {
                  row,
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

        max_column = std::cmp::max(max_column, start_column + wd_length);
        debug!(
          "8-row:{:?}, col:{:?}, chars_length:{:?}, chars_width:{:?}, max_column:{:?}",
          row, col, chars_length, chars_width, max_column
        );
        sections.push(LineViewportSection {
          row,
          chars_length,
          chars_width,
        });
        line_viewports.insert(current_line, LineViewport { sections });
        current_line += 1;
        row += 1;
      }

      debug!(
        "9-row:{}, current_line:{}, max_column:{}",
        row, current_line, max_column
      );
      (
        ViewportRect {
          start_line,
          end_line: current_line,
          start_column,
          end_column: max_column,
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
    let (rectangle, lines) = collect_from_top_left(window, 0, 0);

    Viewport {
      window: SafeWindowRef::new(window),
      start_line: rectangle.start_line,
      end_line: rectangle.end_line,
      start_column: rectangle.start_column,
      end_column: rectangle.end_column,
      lines,
    }
  }

  /// Get start line, index start from 0.
  pub fn start_line(&self) -> usize {
    self.start_line
  }

  /// Get start column, index start from 0.
  pub fn start_column(&self) -> usize {
    self.start_column
  }

  /// Get end line, index start from 0.
  pub fn end_line(&self) -> usize {
    self.end_line
  }

  /// Get end column, index start from 0.
  pub fn end_column(&self) -> usize {
    self.end_column
  }

  /// Get lines viewport
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
      actual.start_line(),
      actual.end_line(),
      actual.start_column(),
      actual.end_column()
    );
    for (k, v) in actual.lines().iter() {
      info!("actual {:?}: {:?}", k, v);
    }
    info!("expect:{:?}", expect);

    let buffer = buffer.read();
    let buflines = buffer.rope().get_lines_at(actual.start_line()).unwrap();

    let mut row = 0_usize;
    for (l, line) in buflines.enumerate() {
      if row >= size.height() as usize {
        break;
      }
      info!("l-{:?}", l);
      let line_viewport = actual.lines().get(&l).unwrap();
      let sections = line_viewport.sections.clone();
      info!("l-{:?}, line_viewport:{:?}", l, line_viewport);
      let mut line_chars = line.chars();
      for (j, sec) in sections.iter().enumerate() {
        assert_eq!(sec.row, row as u16);
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

    assert_eq!(actual.start_line(), 0);
    assert_eq!(actual.end_line(), expect_end_line);
    assert_eq!(actual.start_column(), 0);
    assert_eq!(actual.end_column(), expect_end_column);
    assert_eq!(*actual.lines().first_key_value().unwrap().0, 0);
    assert_eq!(
      *actual.lines().last_key_value().unwrap().0,
      actual.end_line() - 1
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

    assert_eq!(actual.start_line(), 0);
    assert_eq!(actual.end_line(), expect.len());
    assert_eq!(actual.start_column(), 0);
    assert_eq!(actual.end_column(), 1);
    assert_eq!(*actual.lines().first_key_value().unwrap().0, 0);
    assert_eq!(
      *actual.lines().last_key_value().unwrap().0,
      actual.end_line() - 1
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
