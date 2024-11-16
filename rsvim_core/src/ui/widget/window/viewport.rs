//! Buffer viewport on a window.

use crate::buf::BufferWk;
use crate::cart::{U16Pos, U16Rect, U16Size, URect};
use crate::envar;
use crate::rlock;
use crate::ui::canvas::Cell;
use crate::ui::tree::internal::Inodeable;
use crate::ui::util::ptr::SafeWindowRef;
use crate::ui::widget::window::Window;

use anyhow;
use geo::point;
use ropey::RopeSlice;
use std::collections::{BTreeMap, HashMap};
use swc_ecma_transforms_base::perf::Items;
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Copy, Clone)]
/// The row information of a buffer line.
pub struct LineViewportRow {
  /// Start display column index (in the buffer) for current row, starts from 0.
  ///
  /// NOTE: For the term _**display column**_, please see [`Viewport`].
  pub start_bcolumn: usize,

  /// First (fully displayed) char index in current row.
  /// NOTE: The char index is based on the line of the buffer, not based on the whole buffer.
  pub start_char_idx: usize,

  /// End display column index (in the buffer) for current row.
  ///
  /// NOTE: The start and end indexes are left-inclusive and right-exclusive.
  pub end_bcolumn: usize,

  /// End (next to the fully displayed) char index in current row.
  ///
  /// NOTE:
  /// The char index is based on the line of the buffer, not based on the whole buffer.
  /// The start and end indexes are left-inclusive and right-exclusive.
  pub end_char_idx: usize,
}

impl LineViewportRow {
  /// Get the chars length (count) on the row of the line.
  pub fn chars_length(&self) -> usize {
    self.end_char_idx - self.start_char_idx
  }

  /// Get the chars display width on the row of the line.
  pub fn chars_width(&self) -> usize {
    self.end_bcolumn - self.start_bcolumn
  }
}

#[derive(Debug, Clone)]
/// All the displayed rows for a buffer line.
pub struct LineViewport {
  /// Maps from row index (based on the window) to a row in the buffer line, starts from 0.
  pub rows: BTreeMap<u16, LineViewportRow>,

  /// Extra filled columns at the beginning of the line.
  ///
  /// For most cases, this value should be zero. But when the first char (indicate by
  /// `start_char_idx`) doesn't show at the first column of the row, and meanwhile the cells width
  /// is not enough for the previous character.
  ///
  /// For example:
  ///
  /// ```text
  ///              Column index in viewport -> 0   3
  ///                                          |   |
  /// 0         10        20        30    36   40  |  <- Column index in the buffer
  /// |         |         |         |     |    |   |
  /// 0         10        20        30    36   |   37  <- Char index in the buffer
  /// |         |         |         |     |    |   |
  ///                                         |---------------------|
  /// This is the beginning of the buffer.<--H|T-->But it begins to |show at here.
  /// The second line is really short!        |                     |
  /// Too short to show in viewport, luckily t|he third line is ok. |
  ///                                         |---------------------|
  /// ```
  ///
  /// The example shows the first char `B` starts at column index 3 in the viewport, and its
  /// previous char `<--HT-->` uses 8 cells width so cannot fully shows in the viewport.
  ///
  /// In this case, the variable `start_filled_columns` is 4, `start_bcolumn` is 40,
  /// `start_char_idx` is 37.
  pub start_filled_columns: usize,

  /// Extra filled columns at the end of the row, see:
  /// [`start_filled_columns`](LineViewport::start_filled_columns).
  pub end_filled_columns: usize,
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
/// ### Case-1.1: Wrap is disabled
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
/// This is the short line, it's not shown.   |                                       |
/// This is the second very long line, which s|till shows in the viewport.            |
///                                           |---------------------------------------|
/// ```
///
/// Example-1 only shows the beginning of the line, and example-2 only shows the ending of the
/// line (with 'wrap' option is `false`, the second line is too short to show in viewport).
///
/// ### Case-1.2: Wrap is enabled
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
/// ### Case-2.1: ASCII control codes
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
/// ### Case-2.2: Unicode and CJK
///
/// ## Case-3: All together
///
/// Taking all above scenarios into consideration, when rendering a buffer in a viewport, some edge
/// cases are:
///
/// Example-7
///
/// ```text
///                                  31(LF)
/// 0  3 4                   24    30|
/// |  | |                   |      ||
///     |---------------------|
/// <--H|T-->This is the first| line.                 <- Line-1
/// This| is the second line. |                       <- Line-2
/// This| is the third line, 它 有一点点长。          <- Line-3
///     |---------------------|
/// |  |                     | |          |||
/// 0  3                     24|         36||
///                            25         37|
///                                         38(LF)
/// ```
///
/// Example-7 shows a use case with 'wrap' option is `false`, at the beginning of the first line in
/// the viewport, the horizontal tab (`<--HT-->`, use 8 cells width) cannot been fully rendered in
/// viewport. And at the end of the last line in the viewport, the Chinese character (`它`, use 2
/// cells width) also cannot been fully rendered in viewport.
///
/// For the _**display column**_ in the buffer, it's based on the display width of the unicode
/// character, not the char index in the buffer. In example-7, line-1, the start display column is
/// 4 (the `T` in the `<--HT-->`, inclusive), the end display column is 25 (the ` ` whitespace
/// after the `t`, exclusive). In line-3, the start display column is 4 (the ` ` whitespace after
/// the `s`, inclusive), the end display column is 25 (the right half part in the `它` character,
/// exclusive).
///
/// Another hidden rule is:
/// 1. When 'wrap' option is `false`, there can be multiple lines start from non-zero columns in
///    the buffer, or end at non-ending position of the line. Just like example-7, there are 3
///    lines and they all start from column 4, and for line-1 and line-3 they don't end at their
///    ending position of the line.
/// 2. When 'wrap' option is `true`, there will be at most 1 line start from non-zero columns, or
///    end at non-ending position of the line in the buffer. For example:
///
/// Example-8
///
/// ```text
/// 0  3 4  <- Column index in the line of the buffer.
/// |  | |
///     |---------------------|
/// <--H|T-->This is the first|
///     | line. It is quite lo|
///     |ng and even cannot <-|-HT--> be fully rendered in viewport.
///     |---------------------|
///      |                 ||
///      46               64|   <- Column index in the line of the buffer.
///                         65
/// ```
///
/// The example-8 shows a very long line that cannot be fully rendered in the viewport, both start
/// and ending parts are been truncated. But there will not be 2 lines that are both truncated,
/// because with 'wrap' options is `true`, if one line is too long to render, only the one line
/// will show in the viewport. If one line is not too lone and there could be another line, then at
/// least for the first line, it can be fully rendered in the viewport, and it must starts from the
/// first char of the line (as 'wrap' option requires this rendering behavior).
///
/// But what if there're characters cannot been rendered in 2nd row end in example-8? for example:
///
/// Example-9
///
/// ```text
/// 0  3 4  <- Column index in the line of the buffer.
/// |  | |
///     |---------------------|
/// <--H|T-->This is the first|
///     | line. It is quite<--|
///     |HT-->long and even ca|nnot <--HT--> be fully rendered in viewport.
///     |---------------------|
///      |    |              |
///      43   44             59   <- Column index in the line of the buffer.
/// ```
///
/// The above example is a variant from example-8, in the 2nd row end, the tab (`<--HT-->`, uses 8
/// cells width) char cannot be fully rendered. Vim doesn't allow such cases, i.e. when 'wrap'
/// option is `true`, the non-fully rendering only happens at the beginning (top left corner) of
/// viewport, and at the end (bottom right corner) of viewport. It cannot happens inside the
/// viewport. Vim forces the tab char to render in the next row. When 'wrap' option is `false`,
/// each row handles exactly one line on their own, including the non-fully rendering.
///
/// Example-10
///
/// ```text
/// 0  3 4  <- Column index in the line of the buffer.
/// |  | |
///     |---------------------|
/// <--H|T-->This is the first|
///     | line. It is quite___|
///     |<--HT-->long and even| cannot <--HT--> be fully rendered in viewport.
///     |---------------------|
///      |       |           |
///      43      44          56   <- Column index in the line of the buffer.
/// ```
///
/// The above example is a variant from example-9, when 'wrap' is `true`, in the 2nd row end, the 3
/// cells cannot place the tab char, so move it to the next row.
///
/// Finally, some most important anchors (in a viewport) are:
///
/// - `start_line`: The start line (inclusive) of the buffer, it is the first line shows at the top
///   row of the viewport.
/// - `start_bcolumn`: The start display column (inclusive) of the buffer, it is the the first cell
///   of a line displayed in the viewport.
/// - `start_filled_columns`: The filled columns at the beginning of the row in the viewport, it is
///   only useful when the first char in a line doesn't show at the first column of the top row in
///   the viewport (because the previous char cannot be fully placed within these cells).
/// - `end_line`: The end line (exclusive) of the buffer, it is next to the last line at the bottom
///   row of the viewport.
/// - `end_bcolumn`: The end display column (exclusive) of the buffer, it is next to the last cell
///   of a line displayed in the viewport.
/// - `end_filled_columns`: The filled columns at the end of the row in the viewport, it is only
///   useful when the last char in a line doesn't show at the last column at the bottom row in the
///   viewport (because the following char cannot be fully placed within these cells).
///
/// NOTE: The _**display column**_ in the buffer is the characters displayed column index, not the
/// char index of the buffer, not the cell column of the viewport/window. It's named `bcolumn`
/// (short for `buffer_column`).
///
/// When rendering a buffer, viewport will need to go through each lines and characters in the
/// buffer to ensure how it display. It can starts from 4 corners:
///
/// 1. Start from top left corner.
/// 2. Start from top right corner.
/// 3. Start from bottom left corner.
/// 4. Start from bottom right corner.
pub struct Viewport {
  // Options.
  options: ViewportOptions,

  // Buffer.
  buffer: BufferWk,

  // Actual shape.
  actual_shape: U16Rect,

  // Start line index in the buffer, starts from 0.
  start_line: usize,

  // End line index in the buffer.
  end_line: usize,

  // // Start display column index in the buffer, starts from 0.
  // start_bcolumn: usize,
  //
  // // End display column index in the buffer.
  // end_bcolumn: usize,

  // Maps from buffer line index to its displayed rows in the window.
  lines: BTreeMap<usize, LineViewport>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
/// Lines index inside [`Viewport`].
pub struct ViewportRect {
  // Start line index in the buffer, starts from 0.
  pub start_line: usize,

  // End line index in the buffer.
  pub end_line: usize,
  // // Start display column index in the buffer, starts from 0.
  // pub start_bcolumn: usize,
  //
  // // End display column index in the buffer.
  // pub end_bcolumn: usize,
}

// Given the buffer and window size, collect information from start line and column, i.e. from the
// top-left corner.
fn _sync_from_top_left(
  options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_bcolumn: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  // If window is zero-sized.
  let height = actual_shape.height();
  let width = actual_shape.width();
  if height == 0 || width == 0 {
    return (ViewportRect::default(), BTreeMap::new());
  }

  match (options.wrap, options.line_break) {
    (false, _) => {
      _sync_from_top_left_nowrap(options, buffer, actual_shape, start_line, start_bcolumn)
    }
    (true, false) => {
      _sync_from_top_left_wrap_nolinebreak(options, buffer, actual_shape, start_line, start_bcolumn)
    }
    (true, true) => {
      _sync_from_top_left_wrap_linebreak(options, buffer, actual_shape, start_line, start_bcolumn)
    }
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

// Implement [`_sync_from_top_left`] with option `wrap=false`.
fn _sync_from_top_left_nowrap(
  _options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_bcolumn: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug_assert!(height > 0);
  debug_assert!(width > 0);
  debug!(
    "_collect_from_top_left_with_nowrap, actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width
  );

  // Get buffer arc pointer, and lock for read.
  let buffer = buffer.upgrade().unwrap();
  let buffer = rlock!(buffer);

  debug!(
    "buffer.get_line ({:?}):{:?}",
    start_line,
    match buffer.get_line(start_line) {
      Some(line) => rpslice2line(&line),
      None => "None".to_string(),
    }
  );

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  // let mut max_bcolumn = start_bcolumn;

  match buffer.get_lines_at(start_line) {
    // The `start_line` is in the buffer.
    Some(buflines) => {
      // The first `wrow` in the window maps to the `start_line` in the buffer.
      let mut wrow = 0;
      let mut current_line = start_line;

      for (l, line) in buflines.enumerate() {
        // Current row goes out of viewport.
        if wrow >= height {
          break;
        }

        debug!(
          "0-l:{:?}, line:'{:?}', current_line:{:?}",
          l,
          rpslice2line(&line),
          current_line
        );

        let mut rows: BTreeMap<u16, LineViewportRow> = BTreeMap::new();
        let mut wcol = 0_u16;

        let mut bcol = 0_usize;
        let mut start_bcol = 0_usize;
        let mut end_bcol = 0_usize;

        let mut start_c_idx = 0_usize;
        let mut end_c_idx = 0_usize;
        let mut start_c_idx_init = false;
        let mut _end_c_idx_init = false;

        let mut start_fills = 0_usize;
        let mut end_fills = 0_usize;

        // Go through each char in the line.
        for (i, c) in line.chars().enumerate() {
          let c_width = buffer.char_width(c);

          // Prefix width is still before `start_bcolumn`.
          if bcol + c_width < start_bcolumn {
            bcol += c_width;
            end_bcol = bcol;
            end_c_idx = i;
            debug!(
              "1-wrow/wcol:{}/{}, c:{:?}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, start_bcolumn:{}",
              wrow, wcol, c, c_width, bcol, start_bcol, end_bcol, start_c_idx, end_c_idx, start_fills, end_fills, start_bcolumn
            );
            continue;
          }

          if !start_c_idx_init {
            start_c_idx_init = true;
            start_bcol = bcol;
            start_c_idx = i;
            start_fills = bcol - start_bcolumn;
            debug!(
              "2-wrow/wcol:{}/{}, c:{:?}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, start_bcolumn:{}",
              wrow, wcol, c, c_width, bcol, start_bcol, end_bcol, start_c_idx, end_c_idx, start_fills, end_fills, start_bcolumn
            );
          }

          // Row column with next char will go out of the row.
          if wcol as usize + c_width > width as usize {
            end_fills = width as usize - wcol as usize;
            debug!(
              "4-wrow/wcol:{}/{}, c:{:?}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            break;
          }

          bcol += c_width;
          end_bcol = bcol;
          end_c_idx = i + 1;
          wcol += c_width as u16;
          debug!(
            "5-wrow/wcol:{}/{}, c:{:?}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
            wrow,
            wcol,
            c,
            c_width,
            bcol,
            start_bcol,
            end_bcol,
            start_c_idx,
            end_c_idx,
            start_fills,
            end_fills
          );

          // End of the line.
          if i + 1 == line.len_chars() {
            debug!(
              "6-wrow/wcol:{}/{}, c:{:?}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            break;
          }

          // Row column goes out of the row.
          if wcol >= width {
            debug!(
              "7-wrow/wcol:{}/{}, c:{:?}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            break;
          }
        }

        line_viewports.insert(
          current_line,
          LineViewport {
            rows,
            start_filled_columns: start_fills,
            end_filled_columns: end_fills,
          },
        );
        debug!(
          "8-current_line:{}, wrow/wcol:{}/{}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
          current_line,
          wrow,
          wcol,
          bcol,
          start_bcol,
          end_bcol,
          start_c_idx,
          end_c_idx,
          start_fills,
          end_fills
        );
        // Go to next row and line
        current_line += 1;
        wrow += 1;
      }

      debug!("9-current_line:{}, row:{}", current_line, wrow,);
      (
        ViewportRect {
          start_line,
          end_line: current_line,
          // start_bcolumn,
          // end_bcolumn: max_bcolumn + 1,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      debug!("10-start_line:{}", start_line);
      (ViewportRect::default(), BTreeMap::new())
    }
  }
}

// Implement [`_sync_from_top_left`] with option `wrap=true` and `line-break=false`.
fn _sync_from_top_left_wrap_nolinebreak(
  _options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_bcolumn: usize,
) -> (ViewportRect, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  debug_assert!(height > 0);
  debug_assert!(width > 0);
  debug!(
    "_collect_from_top_left_with_wrap_nolinebreak, actual_shape:{:?}, height/width:{:?}/{:?}",
    actual_shape, height, width
  );

  // Get buffer arc pointer, and lock for read.
  let buffer = buffer.upgrade().unwrap();
  let buffer = rlock!(buffer);

  debug!(
    "buffer.get_line ({:?}):'{:?}'",
    start_line,
    match buffer.get_line(start_line) {
      Some(line) => rpslice2line(&line),
      None => "None".to_string(),
    }
  );

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

  match buffer.get_lines_at(start_line) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.

      // The first `wrow` in the window maps to the `start_line` in the buffer.
      let mut wrow = 0;
      let mut current_line = start_line;

      for (l, line) in buflines.enumerate() {
        // Current row goes out of viewport.
        if wrow >= height {
          break;
        }

        debug!(
          "0-l:{:?}, line:'{:?}', current_line:{:?}",
          l,
          rpslice2line(&line),
          current_line
        );

        let mut rows: BTreeMap<u16, LineViewportRow> = BTreeMap::new();
        let mut wcol = 0_u16;

        let mut bcol = 0_usize;
        let mut start_bcol = 0_usize;
        let mut end_bcol = 0_usize;

        let mut start_c_idx = 0_usize;
        let mut end_c_idx = 0_usize;
        let mut start_c_idx_init = false;
        let mut _end_c_idx_init = false;

        let mut start_fills = 0_usize;
        let mut end_fills = 0_usize;

        for (i, c) in line.chars().enumerate() {
          let c_width = buffer.char_width(c);

          // Prefix width is still before `start_bcolumn`.
          if bcol + c_width < start_bcolumn {
            bcol += c_width;
            end_bcol = bcol;
            end_c_idx = i;
            debug!(
              "1-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, start_bcolumn:{}",
              wrow, wcol, c, c_width, bcol, start_bcol, end_bcol, start_c_idx, end_c_idx, start_fills, end_fills, start_bcolumn
            );
            continue;
          }

          if !start_c_idx_init {
            start_c_idx_init = true;
            start_bcol = bcol;
            start_c_idx = i;
            start_fills = bcol - start_bcolumn;
            debug!(
              "2-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
            );
          }

          // Column with next char will goes out of the row.
          if wcol as usize + c_width > width as usize {
            debug!(
              "3-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, width:{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              width
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            let saved_end_fills = width as usize - wcol as usize;
            wrow += 1;
            wcol = 0_u16;
            start_bcol = end_bcol;
            start_c_idx = end_c_idx;
            if wrow >= height {
              end_fills = saved_end_fills;
              debug!(
                "4-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, height:{}",
                wrow,
                wcol,
                c,
                c_width,
                bcol,
                start_bcol,
                end_bcol,
                start_c_idx,
                end_c_idx,
                start_fills,
                end_fills,
                height
              );
              break;
            }
          }

          bcol += c_width;
          end_bcol = bcol;
          end_c_idx = i + 1;
          wcol += c_width as u16;

          debug!(
            "5-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
            wrow,
            wcol,
            c,
            c_width,
            bcol,
            start_bcol,
            end_bcol,
            start_c_idx,
            end_c_idx,
            start_fills,
            end_fills
          );

          // End of the line.
          if i + 1 == line.len_chars() {
            debug!(
              "6-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            break;
          }

          // Column goes out of current row.
          if wcol >= width {
            debug!(
              "7-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, width:{}",
              wrow,
              wcol,
              c,
              c_width,
              bcol,
              start_bcol,
              end_bcol,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              width
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            debug_assert_eq!(wcol, width);
            wrow += 1;
            wcol = 0_u16;
            start_bcol = end_bcol;
            start_c_idx = end_c_idx;
            if wrow >= height {
              debug!(
                "8-wrow/wcol:{}/{}, c:{}/{:?}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}, height:{}",
                wrow,
                wcol,
                c,
                c_width,
                bcol,
                start_bcol,
                end_bcol,
                start_c_idx,
                end_c_idx,
                start_fills,
                end_fills,
                height
              );
              break;
            }
          }
        }

        line_viewports.insert(
          current_line,
          LineViewport {
            rows,
            start_filled_columns: start_fills,
            end_filled_columns: end_fills,
          },
        );
        debug!(
          "9-current_line:{}, wrow/wcol:{}/{}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
          current_line,
          wrow,
          wcol,
          bcol,
          start_bcol,
          end_bcol,
          start_c_idx,
          end_c_idx,
          start_fills,
          end_fills
        );
        current_line += 1;
        wrow += 1;
      }

      debug!("10-current_line:{}, wrow:{}", current_line, wrow);
      (
        ViewportRect {
          start_line,
          end_line: current_line,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      debug!("11-start_line:{}", start_line);
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

// Implement [`_sync_from_top_left`] with option `wrap=true` and `line-break=true`.
fn _sync_from_top_left_wrap_linebreak(
  _options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_bcolumn: usize,
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

  debug!(
    "buffer.get_line ({:?}):'{:?}'",
    start_line,
    match buffer.get_line(start_line) {
      Some(line) => rpslice2line(&line),
      None => "None".to_string(),
    }
  );

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();
  // let mut max_column = start_bcolumn;

  match buffer.get_lines_at(start_line) {
    Some(buflines) => {
      // The `start_line` is inside the buffer.

      // The first `wrow` in the window maps to the `start_line` in the buffer.
      let mut wrow = 0;
      let mut current_line = start_line;

      for (l, line) in buflines.enumerate() {
        // Current row goes out of viewport.
        if wrow >= height {
          break;
        }

        let mut rows: BTreeMap<u16, LineViewportRow> = BTreeMap::new();
        let mut wcol = 0_u16;

        let mut bchars = 0_usize;
        let mut bcol = 0_usize;
        let mut start_bcol = 0_usize;
        let mut end_bcol = 0_usize;

        let mut start_c_idx = 0_usize;
        let mut end_c_idx = 0_usize;
        let mut start_c_idx_init = false;
        let mut _end_c_idx_init = false;

        let mut start_fills = 0_usize;
        let mut end_fills = 0_usize;

        // Chop the line into maximum chars can hold by current window, thus avoid those super
        // long lines for iteration performance.
        // NOTE: Use `height * width * 4` simply for a much bigger size for the total characters in
        // a viewport.
        let truncated_line = truncate_line(
          &line,
          start_bcolumn,
          height as usize * width as usize * 2 + height as usize * 2 + 16,
        );
        let word_boundaries: Vec<&str> = truncated_line.split_word_bounds().collect();
        debug!(
          "0-truncated_line: {:?}, word_boundaries: {:?}, wrow/wcol:{}/{}, bcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
          truncated_line, word_boundaries, wrow, wcol, bcol, start_bcol, end_bcol, start_c_idx, end_c_idx, start_fills, end_fills
        );

        for (i, wd) in word_boundaries.iter().enumerate() {
          let (wd_chars, wd_width) = wd.chars().map(|c| (1_usize, buffer.char_width(c))).fold(
            (0_usize, 0_usize),
            |(init_chars, init_width), (c_count, c_width)| {
              (init_chars + c_count, init_width + c_width)
            },
          );

          debug!(
            "1-l:{:?}, line:'{:?}', current_line:{:?}, i:{}, wd:{:?}",
            l,
            rpslice2line(&line),
            current_line,
            i,
            wd
          );

          // Prefix width is still before `start_bcolumn`.
          if bcol + wd_width < start_bcolumn {
            bcol += wd_width;
            bchars += wd_chars;
            end_bcol = bcol;
            end_c_idx = bchars;
            debug!(
              "2-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, start_bcolumn:{}",
              wrow,
              wcol,
              bcol,
              start_bcol,
              end_bcol,
              bchars,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              wd_chars,
              wd_width,
              start_bcolumn
            );
            continue;
          }

          if !start_c_idx_init {
            start_c_idx_init = true;
            start_bcol = bcol;
            start_c_idx = bchars;
            start_fills = bcol - start_bcolumn;
            debug!(
              "3-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
              wrow,
              wcol,
              bcol,
              start_bcol,
              end_bcol,
              bchars,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              wd_chars,
              wd_width
            );
          }

          // Row column with next char will goes out of the row, i.e. there's not enough space to
          // place this word in current row.
          // There're two cases:
          // 1. The word can be placed in next empty row, i.e. the word length is less or equal to
          //    the row length of the viewport.
          // 2. The word is too long to place in an entire row, i.e. the word length is greater
          //    than the row length of the viewport.
          // Anyway, we simply go to next row and force render all of the word. If the word is too
          // long to place in an entire row, it fallbacks back to the same behavior with
          // 'line-break' option is `false`.
          if wcol as usize + wd_width > width as usize {
            debug!(
              "4-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
              wrow,
              wcol,
              bcol,
              start_bcol,
              end_bcol,
              bchars,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              wd_chars,
              wd_width,
              width
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            let saved_end_fills = width as usize - wcol as usize;
            wrow += 1;
            wcol = 0_u16;
            start_bcol = end_bcol + 1;
            start_c_idx = bchars;

            if wrow >= height {
              end_fills = saved_end_fills;
              debug!(
                "5-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                wrow,
                wcol,
                bcol,
                start_bcol,
                end_bcol,
                bchars,
                start_c_idx,
                end_c_idx,
                start_fills,
                end_fills,
                wd_chars,
                wd_width,
                height
              );
              break;
            }

            for (j, c) in wd.chars().enumerate() {
              let c_width = buffer.char_width(c);

              // Column with next char will goes out of the row.
              if wcol as usize + c_width > width as usize {
                debug!(
                  "6-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
                  wrow,
                  wcol,
                  bcol,
                  start_bcol,
                  end_bcol,
                  bchars,
                  j,
                  c,
                  start_c_idx,
                  end_c_idx,
                  start_fills,
                  end_fills,
                  wd_chars,
                  wd_width,
                  width
                );
                rows.insert(
                  wrow,
                  LineViewportRow {
                    start_bcolumn: start_bcol,
                    start_char_idx: start_c_idx,
                    end_bcolumn: end_bcol,
                    end_char_idx: end_c_idx,
                  },
                );

                let saved_end_fills = width as usize - wcol as usize;
                wrow += 1;
                wcol = 0_u16;
                start_bcol = end_bcol;
                start_c_idx = bchars;

                if wrow >= height {
                  end_fills = saved_end_fills;
                  debug!(
                    "7-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                    wrow,
                    wcol,
                    bcol,
                    start_bcol,
                    end_bcol,
                    bchars,
                    j,
                    c,
                    start_c_idx,
                    end_c_idx,
                    start_fills,
                    end_fills,
                    wd_chars,
                    wd_width,
                    height
                  );
                  break;
                }
              }

              bcol += c_width;
              bchars += 1;
              end_bcol = bcol;
              end_c_idx = bchars;
              wcol += c_width as u16;

              debug!(
                "8-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
                wrow,
                wcol,
                bcol,
                start_bcol,
                end_bcol,
                bchars,
                j,
                c,
                start_c_idx,
                end_c_idx,
                start_fills,
                end_fills,
                wd_chars,
                wd_width
              );

              // Column goes out of current row.
              if wcol >= width {
                debug!(
                  "9-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
                  wrow,
                  wcol,
                  bcol,
                  start_bcol,
                  end_bcol,
                  bchars,
                  j,
                  c,
                  start_c_idx,
                  end_c_idx,
                  start_fills,
                  end_fills,
                  wd_chars,
                  wd_width,
                  width
                );
                rows.insert(
                  wrow,
                  LineViewportRow {
                    start_bcolumn: start_bcol,
                    start_char_idx: start_c_idx,
                    end_bcolumn: end_bcol,
                    end_char_idx: end_c_idx,
                  },
                );
                debug_assert_eq!(wcol, width);
                wrow += 1;
                wcol = 0_u16;
                start_bcol = end_bcol;
                start_c_idx = end_c_idx;

                if wrow >= height {
                  debug!(
                    "10-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                    wrow,
                    wcol,
                    bcol,
                    start_bcol,
                    end_bcol,
                    bchars,
                    j,
                    c,
                    start_c_idx,
                    end_c_idx,
                    start_fills,
                    end_fills,
                    wd_chars,
                    wd_width,
                    height
                  );
                  break;
                }
              }
            }
          }
          // Row column with next char will goes out of the row.
          else {
            // Enough space to place this word in current row
            bcol += wd_width;
            bchars += wd_chars;
            end_bcol = bcol;
            end_c_idx = bchars;
            wcol += wd_width as u16;
          }

          debug!(
            "9-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
            wrow,
            wcol,
            bcol,
            start_bcol,
            end_bcol,
            bchars,
            start_c_idx,
            end_c_idx,
            start_fills,
            end_fills,
            wd_chars,
            wd_width
          );

          // End of the line.
          if i + 1 == word_boundaries.len() {
            debug!(
              "10-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
              wrow,
              wcol,
              bcol,
              start_bcol,
              end_bcol,
              bchars,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              wd_chars,
              wd_width
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            break;
          }

          // Column goes out of current row.
          if wcol >= width {
            debug!(
              "11-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
              wrow,
              wcol,
              bcol,
              start_bcol,
              end_bcol,
              bchars,
              start_c_idx,
              end_c_idx,
              start_fills,
              end_fills,
              wd_chars,
              wd_width,
              width
            );
            rows.insert(
              wrow,
              LineViewportRow {
                start_bcolumn: start_bcol,
                start_char_idx: start_c_idx,
                end_bcolumn: end_bcol,
                end_char_idx: end_c_idx,
              },
            );
            debug_assert_eq!(wcol, width);
            wrow += 1;
            wcol = 0_u16;
            start_bcol = end_bcol;
            start_c_idx = end_c_idx;

            if wrow >= height {
              debug!(
                "12-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                wrow,
                wcol,
                bcol,
                start_bcol,
                end_bcol,
                bchars,
                start_c_idx,
                end_c_idx,
                start_fills,
                end_fills,
                wd_chars,
                wd_width,
                height
              );
              break;
            }
          }
        }

        line_viewports.insert(
          current_line,
          LineViewport {
            rows,
            start_filled_columns: start_fills,
            end_filled_columns: end_fills,
          },
        );
        debug!(
          "13-wrow/wcol:{}/{}, bcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}",
          wrow,
          wcol,
          bcol,
          start_bcol,
          end_bcol,
          bchars,
          start_c_idx,
          end_c_idx,
          start_fills,
          end_fills
        );
        current_line += 1;
        wrow += 1;
      }

      debug!("14-wrow:{}, current_line:{}", wrow, current_line);
      (
        ViewportRect {
          start_line,
          end_line: current_line,
          // start_bcolumn,
          // end_bcolumn: max_column,
        },
        line_viewports,
      )
    }
    None => {
      // The `start_line` is outside of the buffer.
      debug!("15-start_line:{}", start_line);
      (ViewportRect::default(), BTreeMap::new())
    }
  }
}

impl Viewport {
  pub fn new(options: &ViewportOptions, buffer: BufferWk, actual_shape: &U16Rect) -> Self {
    // By default the viewport start from the first line, i.e. starts from 0.
    let (rectangle, lines) = _sync_from_top_left(options, buffer.clone(), actual_shape, 0, 0);

    Viewport {
      options: *options,
      buffer,
      actual_shape: *actual_shape,
      start_line: rectangle.start_line,
      end_line: rectangle.end_line,
      // start_bcolumn: rectangle.start_bcolumn,
      // end_bcolumn: rectangle.end_bcolumn,
      lines,
    }
  }

  /// Get start line index in the buffer, starts from 0.
  pub fn start_line(&self) -> usize {
    self.start_line
  }

  /// Get end line index in the buffer.
  pub fn end_line(&self) -> usize {
    self.end_line
  }

  /// Get viewport information by lines.
  pub fn lines(&self) -> &BTreeMap<usize, LineViewport> {
    &self.lines
  }

  /// Sync from top-left corner, i.e. `start_line` and `start_bcolumn`.
  pub fn sync_from_top_left(&mut self, start_line: usize, start_bcolumn: usize) {
    let (rectangle, lines) = _sync_from_top_left(
      &self.options,
      self.buffer.clone(),
      &self.actual_shape,
      start_line,
      start_bcolumn,
    );
  }
}

impl Viewport {
  /// Get options.
  pub fn options(&self) -> &ViewportOptions {
    &self.options
  }

  /// Set options.
  pub fn set_options(&mut self, options: &ViewportOptions) {
    self.options = *options;
  }

  /// Get buffer.
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }

  /// Set buffer.
  pub fn set_buffer(&mut self, buffer: BufferWk) {
    self.buffer = buffer;
  }

  /// Get actual shape.
  pub fn actual_shape(&self) -> &U16Rect {
    &self.actual_shape
  }

  /// Set actual shape.
  pub fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
    self.actual_shape = *actual_shape;
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

  fn make_viewport_from_size(
    size: U16Size,
    buffer: BufferArc,
    window_options: &WindowLocalOptions,
  ) -> Viewport {
    let mut tree = Tree::new(size);
    tree.set_local_options(window_options);
    let window_shape = IRect::new((0, 0), (size.width() as isize, size.height() as isize));
    let window = Window::new(window_shape, Arc::downgrade(&buffer), &mut tree);
    let options = ViewportOptions {
      wrap: window.options().wrap(),
      line_break: window.options().line_break(),
    };
    Viewport::new(&options, window.buffer(), window.actual_shape())
  }

  #[allow(clippy::too_many_arguments)]
  fn do_test_sync_from_top_left(
    buffer: BufferArc,
    actual: &Viewport,
    expect: &Vec<&str>,
    expect_start_line: usize,
    expect_end_line: usize,
    expect_start_fills: &BTreeMap<usize, usize>,
    expect_end_fills: &BTreeMap<usize, usize>,
  ) {
    info!(
      "actual start_line/end_line:{:?}/{:?}",
      actual.start_line(),
      actual.end_line()
    );
    for (k, v) in actual.lines().iter() {
      info!("actual-{:?}: {:?}", k, v);
    }
    info!("expect:{:?}", expect);

    assert_eq!(actual.start_line(), expect_start_line);
    assert_eq!(actual.end_line(), expect_end_line);
    if !actual.lines().is_empty() {
      let (first_line_idx, _first_line_viewport) = actual.lines().first_key_value().unwrap();
      let (last_line_idx, _last_line_viewport) = actual.lines().last_key_value().unwrap();
      assert_eq!(*first_line_idx, actual.start_line());
      assert_eq!(*last_line_idx, actual.end_line() - 1);
    }
    assert_eq!(
      actual.end_line() - actual.start_line(),
      actual.lines().len()
    );
    assert_eq!(
      actual.end_line() - actual.start_line(),
      expect_start_fills.len()
    );
    assert_eq!(
      actual.end_line() - actual.start_line(),
      expect_end_fills.len()
    );

    let buffer = buffer.read();
    let buflines = buffer.get_lines_at(actual.start_line()).unwrap();
    let total_lines = expect_end_line - expect_start_line;

    for (l, line) in buflines.enumerate() {
      if l >= total_lines {
        break;
      }
      let actual_line_idx = l + expect_start_line;
      let line_viewport = actual.lines().get(&actual_line_idx).unwrap();

      info!(
        "l-{:?}, line_viewport:{:?}, actual_line_idx:{}, expect_start_fills:{:?}, expect_end_fills:{:?}",
        l, line_viewport, actual_line_idx, expect_start_fills, expect_end_fills
      );
      assert_eq!(
        line_viewport.start_filled_columns,
        *expect_start_fills.get(&actual_line_idx).unwrap()
      );
      assert_eq!(
        line_viewport.end_filled_columns,
        *expect_end_fills.get(&actual_line_idx).unwrap()
      );

      let rows = &line_viewport.rows;
      for (r, row) in rows.iter() {
        let mut payload = String::new();
        for c_idx in row.start_char_idx..row.end_char_idx {
          payload.push(line.get_char(c_idx).unwrap());
        }
        info!(
          "row-{:?}, actual:{:?}, expect:{:?}",
          r, payload, expect[*r as usize]
        );
        assert_eq!(payload, expect[*r as usize]);
      }
    }
  }

  #[test]
  fn sync_from_top_left_nowrap1() {
    test_log_init();

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
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    do_test_sync_from_top_left(
      buffer.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn sync_from_top_left_nowrap2() {
    test_log_init();

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
      "Hello, RSVIM!\n",
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
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    do_test_sync_from_top_left(
      buffer.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn sync_from_top_left_nowrap3() {
    test_log_init();

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
      "Hello, RSVIM!\n",
      "This is a quite simple and smal",
      "But still it contains several t",
      "  1. When the line is small eno",
      "  2. When the line is too long ",
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    do_test_sync_from_top_left(
      buffer.clone(),
      &actual,
      &expect,
      0,
      5,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn sync_from_top_left_nowrap4() {
    test_log_init();

    let buffer = make_empty_buffer();
    let expect = vec![""];

    let size = U16Size::new(20, 20);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn sync_from_top_left_nowrap5() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "Hello,\tRSVIM!\n",
      "This\r",
      "is a quite\tsimple and small test lines.\n",
      "But still\\it\r",
      "contains\tseveral things we want to test:\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello,", // 4 fills for '\t'
      "This\r",
      "is a quite",
      "But still\\",
      "contains", // 2 fills for '\t'
      "\t1.",
      "\t2.",
      "\t", // 2 fills for '\t'
      "\t", // 2 fills for '\t'
      "",
    ];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
      (8, 0),
      (9, 0),
    ]
    .into_iter()
    .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![
      (0, 4),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 2),
      (5, 0),
      (6, 0),
      (7, 2),
      (8, 2),
      (9, 0),
    ]
    .into_iter()
    .collect();
    do_test_sync_from_top_left(
      buffer.clone(),
      &actual,
      &expect,
      0,
      10,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_nowrap6() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "你好，\tRSVIM！\n",
      "这是\ta quite 简单而且很小的测试文字内容行。\n",
      "But still\\it\t包含了好几种我们想测试的情况：\n",
      "\t1. 当那条线\tis small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line 特别长而无法完全 to put in a row of the window content widget, there're multiple cases:\n",
      "\t* The extra\tparts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "  * The extra parts\tare split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "你好，\tRSVIM！\n",
      "这是\ta quite 简单而",  // 1 fills for '且'
      "But still\\it\t包含了", // 1 fills for '好'
      "\t1. 当那条线\t",
      "  2. When the line 特别长而",
      "\t* The extra\t",
    ];

    let size = U16Size::new(27, 6);
    let options = WindowLocalOptions::builder().wrap(false).build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 1), (2, 1), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
    do_test_sync_from_top_left(
      buffer.clone(),
      &actual,
      &expect,
      0,
      6,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak1() {
    test_log_init();

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
      "IM!\n",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.\n",
      "But still ",
      "it contain",
      "s several ",
    ];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    do_test_sync_from_top_left(buffer, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak2() {
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains sever",
      "al things we want to test:\n",
      "  1. When the line is small",
      " enough to completely put i",
      "nside a row of the window c",
      "ontent widget, then the lin",
      "e-wrap and word-wrap doesn'",
      "t affect the rendering.\n",
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
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      5,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak3() {
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
      "Hello, RSVIM!\n",
      "This is a quite simple and smal",
      "l test lines.\n",
      "But still it contains several t",
      "hings we want to test:\n",
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    do_test_sync_from_top_left(buffer, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak4() {
    let buffer = make_empty_buffer();
    let expect = vec![""];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    do_test_sync_from_top_left(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak5() {
    let buffer = make_buffer_from_lines(vec![
      "\t\t* The extra parts are\tsplit into the next\trow,\tif either line-wrap or word-wrap options are been set. If the extra\tparts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "\t\t* The extra par",
      "ts are\tsplit into the ne",
      "xt\trow,\tif either",
      " line-wrap or word-wrap options",
      " are been set. If the extra",
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 4)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak6() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "But still it contains several things we want to test:\n",
      "\t\t1. When\tthe line\tis small\tenough to\tcompletely put\tinside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
    ]);
    let expect = vec![
      "But still it contains several t",
      "hings we want to test:\n",
      "\t\t1. When\t",
      "the line\tis small",
      "\tenough to\tcomple",
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak7() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "But still it contains several things we want to test:\n",
      "\t\t1. When\tthe line\tis small\tenough\tto\tcompletely put\tinside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
    ]);
    let expect = vec![
      "But still it contains several t",
      "hings we want to test:\n",
      "\t\t1. When\t",
      "the line\tis small",
      "\tenough\tto", // 7 fills
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 7)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak8() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "但它仍然contains several things 我们想要测试的文字内容：\n",
      "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
    ]);
    let expect = vec![
      "但它仍然contains several things",
      " 我们想要测试的文字内容：\n",
      "\t第一，当一行文字内容太",
      "小了，然后可以完全的放进窗口的",
      "一行之中，那么行wrap和词wrap两", // 1 fills
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_nolinebreak9() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "但它仍然contains several th\tings 我们想要测试的文字内容：\n",
      "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
    ]);
    let expect = vec![
      "但它仍然contains several th",
      "\tings 我们想要测试的文字",
      "内容：\n",
      "\t第一，当一行文字内容太",
      "小了，然后可以完全的放进窗口的",
      "一行之中，那么行wrap和词wrap两", // 1 fills
    ];

    let size = U16Size::new(31, 5);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(false)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak1() {
    test_log_init();

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
      "RSVIM!\n",
      "This is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      "But still ",
      "it ",
    ];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 7)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak2() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is to\to long to be completely p\tut in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains ",
      "several things we want to ",
      "test:\n",
      "  1. When the line is small",
      " enough to completely put ",
      "inside a row of the window ",
      "content widget, then the ",
      "line-wrap and word-wrap ",
      "doesn't affect the ",
      "rendering.\n",
      "  2. When the line is to",
      "\to long to be ", // 6 fills
    ];

    let size = U16Size::new(27, 15);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 6)]
      .into_iter()
      .collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      5,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak3() {
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "  1. When the line is small ",
      "enough to completely put inside",
      " a row of the window content ",
      "widget, then the line-wrap and ",
      "word-wrap doesn't affect the ",
      "rendering.\n",
      "",
    ];

    let size = U16Size::new(31, 11);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak4() {
    test_log_init();

    let buffer = make_empty_buffer();
    let expect = vec![""];

    let size = U16Size::new(10, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    do_test_sync_from_top_left(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak5() {
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "  1. When the line is small ",
      "enough to completely put inside",
      " a row of the window content ",
      "widget, then the line-wrap and ",
      "word-wrap doesn't affect the ",
    ];

    let size = U16Size::new(31, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 2)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak6() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
      "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "\t\t第一，当一行文",
      "本内容的长度足够短，短到可以完",
      "整的放入一个窗口（的一行）之中",
      "，那么基于行的换行和基于单词的",
      "换行两个选项都不会影响渲染的最",
    ];

    let size = U16Size::new(31, 10);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 1)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn sync_from_top_left_wrap_linebreak7() {
    test_log_init();

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
      "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "\t\t第一，当一行文",
      "本内容的长度足够短，短到可以完",
      "整的放入一个窗口（的一行）之中",
      "，那么基于行的换行和基于单词的",
      "换行两个选项都不会影响渲染的最",
      "终效果。\n",
    ];

    let size = U16Size::new(31, 11);
    let options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    let actual = make_viewport_from_size(size, buffer.clone(), &options);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    do_test_sync_from_top_left(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }
}
