//! Buffer viewport on a window.

use crate::arc_impl;
use crate::buf::Buffer;
use crate::prelude::*;
use crate::ui::widget::window::WindowLocalOptions;

use paste::paste;
use std::collections::BTreeMap;
use std::ops::Range;
#[allow(unused_imports)]
use tracing::trace;

pub mod sync;

#[derive(Debug, Copy, Clone)]
/// The row viewport in a buffer line.
pub struct RowViewport {
  start_char_idx: usize,
  end_char_idx: usize,
}

impl RowViewport {
  /// Make new instance.
  pub fn new(char_idx_range: Range<usize>) -> Self {
    Self {
      start_char_idx: char_idx_range.start,
      end_char_idx: char_idx_range.end,
    }
  }

  /// Get the chars length (count) on the row of the line.
  pub fn chars_length(&self) -> usize {
    self.end_char_idx - self.start_char_idx
  }

  /// First (fully displayed) char index in current row.
  ///
  /// NOTE: The start and end indexes are left-inclusive and right-exclusive.
  pub fn start_char_idx(&self) -> usize {
    self.start_char_idx
  }

  /// Get end (next to the fully displayed) char index in current row.
  ///
  /// NOTE:
  /// The char index is based on the line of the buffer, not based on the whole buffer.
  /// The start and end indexes are left-inclusive and right-exclusive.
  pub fn end_char_idx(&self) -> usize {
    self.end_char_idx
  }
}

#[derive(Debug, Clone)]
/// The buffer line viewport in a buffer.
pub struct LineViewport {
  rows: BTreeMap<u16, RowViewport>,
  start_filled_cols: usize,
  end_filled_cols: usize,
}

impl LineViewport {
  /// Make new instance.
  pub fn new(
    rows: BTreeMap<u16, RowViewport>,
    start_filled_cols: usize,
    end_filled_cols: usize,
  ) -> Self {
    Self {
      rows,
      start_filled_cols,
      end_filled_cols,
    }
  }

  /// Maps `row_idx` (in the window) => its row-wise viewport.
  /// The row index starts from 0.
  pub fn rows(&self) -> &BTreeMap<u16, RowViewport> {
    &self.rows
  }

  /// Get extra filled columns at the beginning of the line.
  ///
  /// For most cases, this value should be zero. But when the first char (indicate by
  /// `start_char_idx`) doesn't show at the first column of the row, and meanwhile the cells width
  /// is not enough for the previous character.
  ///
  /// For example:
  ///
  /// ```text
  ///              Column index in viewport -> 0   4
  ///                                          |   |
  /// 0         10        20        30    36   |   37  <- Char index in the buffer
  /// |         |         |         |     |    |   |
  ///                                         |---------------------|
  /// This is the beginning of the buffer.<--H|T-->But it begins to |show at here.
  /// The second line is really short!        |                     |
  /// Too short to show in viewport, luckily t|he third line is ok. |
  ///                                         |---------------------|
  /// ```
  ///
  /// The example shows the first char `B` starts at column index 4 in the viewport, and its
  /// previous char `<--HT-->` uses 8 cells width so cannot fully shows in the viewport.
  ///
  /// In this case, the variable `start_filled_cols` is 4, the start char index is 37.
  pub fn start_filled_cols(&self) -> usize {
    self.start_filled_cols
  }

  /// Get extra filled columns at the end of the row.
  pub fn end_filled_cols(&self) -> usize {
    self.end_filled_cols
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The cursor viewport to maintain the positions.
///
/// As explained in [`Viewport`], ASCII control codes and other unicode chars can use 0 or more
/// cells when displayed in terminal, thus when cursor moves on the window/buffer, it needs to
/// always stay on the left most cell of a unicode char. Thus this viewport maintains the cursor
/// positions by taking consideration of both terminal cell position and buffer unicode char
/// position.
///
/// NOTE: It is not a must that a window/buffer has a cursor inside it. But once it has, we will
/// always maintain this position information for it.
pub struct CursorViewport {
  // Line index.
  line_idx: usize,
  // Char index.
  char_idx: usize,
  // Row index.
  row_idx: u16,
  // Column index.
  column_idx: u16,
}

arc_impl!(CursorViewport);

impl CursorViewport {
  /// Make new instance.
  pub fn new(line_idx: usize, char_idx: usize, row_idx: u16, column_idx: u16) -> Self {
    Self {
      line_idx,
      char_idx,
      row_idx,
      column_idx,
    }
  }

  /// Get line index, starts from 0.
  pub fn line_idx(&self) -> usize {
    self.line_idx
  }

  /// Get char index, starts from 0.
  pub fn char_idx(&self) -> usize {
    self.char_idx
  }

  /// Get row index, starts from 0.
  pub fn row_idx(&self) -> u16 {
    self.row_idx
  }

  /// Get column index, starts from 0.
  pub fn column_idx(&self) -> u16 {
    self.column_idx
  }

  /// Create cursor viewport with the top-left corner position from the window viewport.
  pub fn from_top_left(viewport: &Viewport, buffer: &Buffer) -> Self {
    debug_assert!(viewport.end_line_idx() >= viewport.start_line_idx());
    if viewport.end_line_idx() == viewport.start_line_idx() {
      return Self::new(0, 0, 0, 0);
    }

    let lines = viewport.lines();
    debug_assert!(viewport.end_line_idx() > viewport.start_line_idx());
    debug_assert!(!lines.is_empty());
    debug_assert!(lines.len() == viewport.end_line_idx() - viewport.start_line_idx());
    debug_assert!(lines.first_key_value().is_some());
    debug_assert!(lines.last_key_value().is_some());
    debug_assert!(*lines.first_key_value().unwrap().0 == viewport.start_line_idx());
    debug_assert!(viewport.end_line_idx() > 0);
    debug_assert!(*lines.last_key_value().unwrap().0 == viewport.end_line_idx() - 1);
    let first_line = lines.first_key_value().unwrap();
    let line_idx = *first_line.0;
    let first_line = first_line.1;

    if first_line.rows().is_empty() {
      return Self::new(0, 0, 0, 0);
    }

    let first_row = first_line.rows().first_key_value().unwrap();
    let first_row = first_row.1;

    debug_assert!(first_row.end_char_idx() >= first_row.start_char_idx());
    if first_row.end_char_idx() == first_row.start_char_idx() {
      debug_assert_eq!(first_row.start_char_idx(), 0);
      debug_assert_eq!(first_row.end_char_idx(), 0);
      return Self::new(0, 0, 0, 0);
    }

    let char_idx = first_row.start_char_idx();
    Self::from_position(viewport, buffer, line_idx, char_idx)
  }

  /// Create cursor viewport with specified position (buffer's line/char index) from the window
  /// viewport.
  ///
  /// # Panics
  ///
  /// It panics if the line/char index are not shown in the window viewport.
  pub fn from_position(
    viewport: &Viewport,
    buffer: &Buffer,
    line_idx: usize,
    char_idx: usize,
  ) -> Self {
    debug_assert!(viewport.lines().contains_key(&line_idx));
    let line_viewport = viewport.lines().get(&line_idx).unwrap();

    let cursor_row = line_viewport
      .rows()
      .iter()
      .filter(|(_row_idx, row_viewport)| {
        // trace!(
        //   "row_viewport:{:?},start_char_idx:{},end_char_idx:{},line_idx:{},char_idx:{}",
        //   row_viewport,
        //   row_viewport.start_char_idx(),
        //   row_viewport.end_char_idx(),
        //   line_idx,
        //   char_idx
        // );
        row_viewport.start_char_idx() <= char_idx && row_viewport.end_char_idx() > char_idx
      })
      .collect::<Vec<_>>();

    if !cursor_row.is_empty() {
      debug_assert_eq!(cursor_row.len(), 1);
      let (row_idx, row_viewport) = cursor_row[0];

      let row_start_width = buffer.width_before(line_idx, row_viewport.start_char_idx());
      let char_start_width = buffer.width_before(line_idx, char_idx);
      let col_idx = (char_start_width - row_start_width) as u16;
      let row_idx = *row_idx;

      CursorViewport::new(line_idx, char_idx, row_idx, col_idx)
    } else {
      debug_assert!(line_viewport.rows().first_key_value().is_some());
      let (first_row_idx, _first_row_viewport) = line_viewport.rows().first_key_value().unwrap();
      CursorViewport::new(line_idx, char_idx, *first_row_idx, 0_u16)
    }
  }
}

#[derive(Debug, Clone)]
// spellchecker:off
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
/// - `start_dcolumn`: The start display column (inclusive) of the buffer, it is the the first cell
///   of a line displayed in the viewport.
/// - `start_filled_cols`: The filled columns at the beginning of the row in the viewport, it is
///   only useful when the first char in a line doesn't show at the first column of the top row in
///   the viewport (because the previous char cannot be fully placed within these cells).
/// - `end_line`: The end line (exclusive) of the buffer, it is next to the last line at the bottom
///   row of the viewport.
/// - `end_dcolumn`: The end display column (exclusive) of the buffer, it is next to the last cell
///   of a line displayed in the viewport.
/// - `end_filled_cols`: The filled columns at the end of the row in the viewport, it is only
///   useful when the last char in a line doesn't show at the last column at the bottom row in the
///   viewport (because the following char cannot be fully placed within these cells).
///
/// NOTE: The _**display column**_ in the buffer is the characters displayed column index, not the
/// char index of the buffer, not the cell column of the viewport/window. It's named `dcolumn`
/// (short for `displayed_column`).
///
/// When rendering a buffer, viewport will need to go through each lines and characters in the
/// buffer to ensure how it display. It can starts from 4 corners:
///
/// 1. Start from top left corner.
/// 2. Start from top right corner.
/// 3. Start from bottom left corner.
/// 4. Start from bottom right corner.
// spellchecker:on
pub struct Viewport {
  // Start line index (in the buffer), starts from 0.
  start_line_idx: usize,

  // End line index (in the buffer).
  end_line_idx: usize,

  // Start display column index (in the buffer), starts from 0.
  start_column_idx: usize,

  // Maps `line_idx` (in the buffer) => its line-wise viewports.
  lines: BTreeMap<usize, LineViewport>,
}

arc_impl!(Viewport);

#[derive(Debug, Copy, Clone)]
pub enum ViewportSearchAnchorDirection {
  Up,
  Down,
  Left,
  Right,
}

impl Viewport {
  /// Calculate viewport downward, from top to bottom.
  ///
  /// NOTE: By default the viewport should starts from (0, 0), i.e. when first open buffer in a
  /// window.
  pub fn view(
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    window_local_options: &WindowLocalOptions,
    start_line_idx: usize,
    start_column_idx: usize,
  ) -> Self {
    let (line_idx_range, lines) = sync::sync(
      buffer,
      window_actual_shape,
      window_local_options,
      start_line_idx,
      start_column_idx,
    );

    debug_assert_eq!(line_idx_range.start_line_idx(), start_line_idx);

    Viewport {
      start_line_idx: line_idx_range.start_line_idx(),
      end_line_idx: line_idx_range.end_line_idx(),
      start_column_idx,
      lines,
    }
  }

  /// Search for a new viewport anchor (i.e. `start_line`/`start_column`) with target cursor
  /// line/char position, when cursor moves downward.
  ///
  /// NOTE: If target cursor line/char position cannot be correctly shown in new viewport, the
  /// viewport will be adjusted to show target cursor correctly, with a minimal movement (for
  /// better user visuals).
  ///
  /// Returns `start_line` and `start_column` for new viewport.
  pub fn search_anchor(
    &self,
    direction: ViewportSearchAnchorDirection,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    window_local_options: &WindowLocalOptions,
    target_cursor_line_idx: usize,
    target_cursor_char_idx: usize,
  ) -> (usize, usize) {
    // If window is zero-sized.
    let height = window_actual_shape.height();
    let width = window_actual_shape.width();
    if height == 0 || width == 0 {
      return (0, 0);
    }

    match direction {
      ViewportSearchAnchorDirection::Down => sync::search_anchor_downward(
        self,
        buffer,
        window_actual_shape,
        window_local_options,
        target_cursor_line_idx,
        target_cursor_char_idx,
      ),
      ViewportSearchAnchorDirection::Up => sync::search_anchor_upward(
        self,
        buffer,
        window_actual_shape,
        window_local_options,
        target_cursor_line_idx,
        target_cursor_char_idx,
      ),
      ViewportSearchAnchorDirection::Left => sync::search_anchor_leftward(
        self,
        buffer,
        window_actual_shape,
        window_local_options,
        target_cursor_line_idx,
        target_cursor_char_idx,
      ),
      ViewportSearchAnchorDirection::Right => sync::search_anchor_rightward(
        self,
        buffer,
        window_actual_shape,
        window_local_options,
        target_cursor_line_idx,
        target_cursor_char_idx,
      ),
    }
  }

  #[cfg(not(debug_assertions))]
  fn _internal_check(&self) {}

  #[cfg(debug_assertions)]
  fn _internal_check(&self) {
    debug_assert!(self.end_line_idx >= self.start_line_idx);
    debug_assert_eq!(
      self.end_line_idx == self.start_line_idx,
      self.lines.is_empty()
    );
    debug_assert!(self.lines.first_key_value().is_some());
    debug_assert_eq!(
      *self.lines.first_key_value().unwrap().0,
      self.start_line_idx
    );
    debug_assert!(self.lines.last_key_value().is_some());
    debug_assert_eq!(
      *self.lines.last_key_value().unwrap().0,
      self.end_line_idx - 1
    );
    let mut last_line_idx: Option<usize> = None;
    let mut last_row_idx: Option<u16> = None;
    for (line_idx, line_viewport) in self.lines.iter() {
      match last_line_idx {
        Some(last_line_idx1) => debug_assert_eq!(last_line_idx1 + 1, *line_idx),
        None => { /* Skip */ }
      }
      last_line_idx = Some(*line_idx);
      let mut last_row_viewport: Option<RowViewport> = None;
      for (row_idx, row_viewport) in line_viewport.rows() {
        trace!(
          "line_idx:{:?},row_idx:{:?},last_row_idx:{:?},last_row_viewport:{:?},row_viewport:{:?}",
          line_idx, row_idx, last_row_idx, last_row_viewport, row_viewport
        );
        match last_row_idx {
          Some(last_row_idx1) => debug_assert_eq!(last_row_idx1 + 1, *row_idx),
          None => { /* Skip */ }
        }
        last_row_idx = Some(*row_idx);
        match last_row_viewport {
          Some(last_row_viewport1) => {
            //trace!(
            //  "last_row_viewport1.end_char_idx:{:?}, row_viewport.start_char_idx:{:?}",
            //  last_row_viewport1.end_char_idx(),
            //  row_viewport.start_char_idx()
            //);
            debug_assert_eq!(
              last_row_viewport1.end_char_idx(),
              row_viewport.start_char_idx()
            )
          }
          None => { /* Skip */ }
        }
        last_row_viewport = Some(*row_viewport);
      }
    }
  }

  /// Get start line index in the buffer, starts from 0.
  pub fn start_line_idx(&self) -> usize {
    self._internal_check();
    self.start_line_idx
  }

  /// Get end line index in the buffer.
  pub fn end_line_idx(&self) -> usize {
    self._internal_check();
    self.end_line_idx
  }

  /// Get start display column index in the buffer.
  pub fn start_column_idx(&self) -> usize {
    self._internal_check();
    self.start_column_idx
  }

  /// Get viewport information by lines.
  pub fn lines(&self) -> &BTreeMap<usize, LineViewport> {
    self._internal_check();
    &self.lines
  }

  /// Whether viewport is empty.
  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.lines.is_empty()
  }
}

// spellchecker:off
#[allow(unused_imports)]
#[cfg(test)]
mod tests_util {
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptions, BufferLocalOptionsBuilder};
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Tree;
  use crate::ui::tree::*;
  use crate::ui::widget::window::{Window, WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  pub fn make_nowrap() -> WindowLocalOptions {
    WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap()
  }

  pub fn make_wrap_nolinebreak() -> WindowLocalOptions {
    WindowLocalOptionsBuilder::default().build().unwrap()
  }

  pub fn make_wrap_linebreak() -> WindowLocalOptions {
    WindowLocalOptionsBuilder::default()
      .line_break(true)
      .build()
      .unwrap()
  }

  pub fn make_window(
    terminal_size: U16Size,
    buffer: BufferArc,
    window_options: &WindowLocalOptions,
  ) -> Window {
    let mut tree = Tree::new(terminal_size);
    tree.set_global_local_options(window_options);
    let window_shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    Window::new(
      window_shape,
      Arc::downgrade(&buffer),
      tree.global_local_options(),
    )
  }

  #[allow(clippy::too_many_arguments)]
  pub fn assert_viewport(
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
      actual.start_line_idx(),
      actual.end_line_idx()
    );
    for (k, v) in actual.lines().iter() {
      info!("actual-{:?}: {:?}", k, v);
    }
    info!("expect:{:?}", expect);

    assert_eq!(actual.start_line_idx(), expect_start_line);
    assert_eq!(actual.end_line_idx(), expect_end_line);
    if actual.lines().is_empty() {
      assert!(actual.end_line_idx() <= actual.start_line_idx());
    } else {
      let (first_line_idx, _first_line_viewport) = actual.lines().first_key_value().unwrap();
      let (last_line_idx, _last_line_viewport) = actual.lines().last_key_value().unwrap();
      assert_eq!(*first_line_idx, actual.start_line_idx());
      assert_eq!(*last_line_idx, actual.end_line_idx() - 1);
    }
    assert_eq!(
      actual.end_line_idx() - actual.start_line_idx(),
      actual.lines().len()
    );

    let buffer = lock!(buffer);
    let buflines = buffer
      .get_rope()
      .get_lines_at(actual.start_line_idx())
      .unwrap();
    let total_lines = expect_end_line - expect_start_line;

    for (l, line) in buflines.enumerate() {
      if l >= total_lines {
        break;
      }
      let actual_line_idx = l + expect_start_line;
      let line_viewport = actual.lines().get(&actual_line_idx).unwrap();

      info!(
        "l-{:?}, actual_line_idx:{}, line_viewport:{:?}",
        actual.start_line_idx() + l,
        actual_line_idx,
        line_viewport
      );
      info!(
        "l-{:?},start_filled_cols expect:{:?}, actual:{}, end_filled_cols expect:{:?}, actual:{}",
        actual.start_line_idx() + l,
        expect_start_fills.get(&actual_line_idx),
        line_viewport.start_filled_cols(),
        expect_end_fills.get(&actual_line_idx),
        line_viewport.end_filled_cols()
      );
      assert_eq!(
        line_viewport.start_filled_cols(),
        *expect_start_fills.get(&actual_line_idx).unwrap()
      );
      assert_eq!(
        line_viewport.end_filled_cols(),
        *expect_end_fills.get(&actual_line_idx).unwrap()
      );

      let rows = &line_viewport.rows();
      for (r, row) in rows.iter() {
        info!("row-index-{:?}, row:{:?}", r, row);

        if r > rows.first_key_value().unwrap().0 {
          let prev_r = r - 1;
          let prev_row = rows.get(&prev_r).unwrap();
          info!(
            "row-{:?}, current[{}]:{:?}, previous[{}]:{:?}",
            r, r, row, prev_r, prev_row
          );
        }
        if r < rows.last_key_value().unwrap().0 {
          let next_r = r + 1;
          let next_row = rows.get(&next_r).unwrap();
          info!(
            "row-{:?}, current[{}]:{:?}, next[{}]:{:?}",
            r, r, row, next_r, next_row
          );
        }

        let mut payload = String::new();
        for c_idx in row.start_char_idx()..row.end_char_idx() {
          let c = line.get_char(c_idx).unwrap();
          payload.push(c);
        }
        info!(
          "row-{:?}, payload actual:{:?}, expect:{:?}",
          r, payload, expect[*r as usize]
        );
        assert_eq!(payload, expect[*r as usize]);
      }
    }
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests_view_nowrap {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

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

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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
    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and smal",
      "But still it contains several t",
      "  1. When the line is small eno",
      "  2. When the line is too long ",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      5,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello,\tRSVIM!\n",
        "This\r",
        "is a quite\tsimple and small test lines.\n",
        "But still\\it\r",
        "contains\tseveral things we want to test:\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      10,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(27, 6);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "你好，\tRSVIM！\n",
        "这是\ta quite 简单而且很小的测试文字内容行。\n",
        "But still\\it\t包含了好几种我们想测试的情况：\n",
        "\t1. 当那条线\tis small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line 特别长而无法完全 to put in a row of the window content widget, there're multiple cases:\n",
        "\t* The extra\tparts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "  * The extra parts\tare split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "你好，\tRSVIM！\n",
      "这是\ta quite 简单而",  // 1 fills for '且'
      "But still\\it\t包含了", // 1 fills for '好'
      "\t1. 当那条线\t",
      "  2. When the line 特别长而",
      "\t* The extra\t",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 1), (2, 1), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      6,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new7() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new8() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![""]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new9() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new10() {
    test_log_init();

    let terminal_size = U16Size::new(13, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!",
      "This is a qui",
      "But still it ",
      "  1. When the",
      "  2. When the",
      "     * The ex",
      "     * The ex",
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests_view_nowrap_startcol {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "lo, RSVIM!",
      "s is a qui",
      " still it ",
      ". When the",
      ". When the",
      "  * The ex",
      "  * The ex",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 3);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      " RSVIM!\n",
      "s a quite ",
      "ill it con",
      "hen the li",
      "hen the li",
      " The extra",
      " The extra",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 6);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "",
      " simple an",
      "ntains sev",
      "ine is sma",
      "ine is too",
      "a parts ar",
      "a parts ar",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 15);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "",
      "",
      "",
      "a row of t",
      " of the wi",
      "and word-w",
      "r line-wra",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 60);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec!["", "", "", "", "", "", "", ""];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 500);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
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
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }
}

// #[allow(unused_imports)]
// #[cfg(test)]
// mod tests_upward_nowrap {
//   use super::tests_util::*;
//   use super::*;
//
//   use crate::buf::BufferLocalOptionsBuilder;
//   use crate::prelude::*;
//   use crate::test::buf::make_buffer_from_lines;
//   use crate::test::log::init as test_log_init;
//   use crate::ui::tree::*;
//   use crate::wlock;
//
//   #[test]
//   fn update1() {
//     test_log_init();
//
//     let terminal_size = U16Size::new(10, 10);
//     let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
//     let win_opts = make_nowrap();
//
//     let buf = make_buffer_from_lines(
//       terminal_size.height(),
//       buf_opts,
//       vec![
//         "Hello, RSVIM!\n",
//         "This is a quite simple and small test lines.\n",
//         "But still it contains several things we want to test:\n",
//         "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
//         "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
//         "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
//         "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
//       ],
//     );
//
//     let expect = vec![
//       "",
//       "",
//       "",
//       "",
//       "",
//       "",
//       "Hello, RSV",
//       "This is a ",
//       "But still ",
//       "  1. When ",
//     ];
//
//     let window = make_window(terminal_size, buf.clone(), &win_opts);
//     let actual = {
//       let mut buf = wlock!(buf);
//       Viewport::_upward(&mut buf, window.actual_shape(), &win_opts, 4, 0)
//     };
//     let expect_fills: BTreeMap<usize, usize> = vec![
//       (0, 0),
//       (1, 0),
//       (2, 0),
//       (3, 0),
//       (4, 0),
//       (5, 0),
//       (6, 0),
//       (7, 0),
//       (8, 0),
//       (9, 0),
//     ]
//     .into_iter()
//     .collect();
//     assert_viewport(
//       buf.clone(),
//       &actual,
//       &expect,
//       0,
//       4,
//       &expect_fills,
//       &expect_fills,
//     );
//
//     let expect = vec![
//       "",
//       "",
//       "",
//       "",
//       "Hello, RSV",
//       "This is a ",
//       "But still ",
//       "  1. When ",
//       "  2. When ",
//       "     * The",
//     ];
//
//     let window = make_window(terminal_size, buf.clone(), &win_opts);
//     let actual = {
//       let mut buf = wlock!(buf);
//       Viewport::_upward(&mut buf, window.actual_shape(), &win_opts, 6, 0)
//     };
//     let expect_fills: BTreeMap<usize, usize> = vec![
//       (0, 0),
//       (1, 0),
//       (2, 0),
//       (3, 0),
//       (4, 0),
//       (5, 0),
//       (6, 0),
//       (7, 0),
//       (8, 0),
//       (9, 0),
//     ]
//     .into_iter()
//     .collect();
//     assert_viewport(
//       buf.clone(),
//       &actual,
//       &expect,
//       0,
//       6,
//       &expect_fills,
//       &expect_fills,
//     );
//   }
//
//   #[test]
//   fn update2() {
//     test_log_init();
//
//     let terminal_size = U16Size::new(31, 5);
//     let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
//     let win_opts = make_nowrap();
//
//     let buf = make_buffer_from_lines(
//       terminal_size.height(),
//       buf_opts,
//       vec![
//         "Hello, RSVIM!\n",
//         "This is a quite simple and small test lines.\n",
//         "But still it contains several things we want to test:\n",
//         "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
//         "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
//         "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
//         "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
//       ],
//     );
//     let expect = vec![
//       "Hello, RSVIM!\n",
//       "This is a quite simple and smal",
//       "But still it contains several t",
//       "  1. When the line is small eno",
//       "  2. When the line is too long ",
//     ];
//
//     let window = make_window(terminal_size, buf.clone(), &win_opts);
//     let actual = {
//       let mut buf = wlock!(buf);
//       Viewport::_upward(&mut buf, window.actual_shape(), &win_opts, 5, 0)
//     };
//     let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
//       .into_iter()
//       .collect();
//     assert_viewport(
//       buf.clone(),
//       &actual,
//       &expect,
//       0,
//       5,
//       &expect_fills,
//       &expect_fills,
//     );
//
//     let expect = vec![
//       "But still it contains several t",
//       "  1. When the line is small eno",
//       "  2. When the line is too long ",
//       "     * The extra parts are been",
//       "     * The extra parts are spli",
//     ];
//
//     let window = make_window(terminal_size, buf.clone(), &win_opts);
//     let actual = {
//       let mut buf = wlock!(buf);
//       Viewport::_upward(&mut buf, window.actual_shape(), &win_opts, 7, 0)
//     };
//     let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
//       .into_iter()
//       .collect();
//     assert_viewport(
//       buf.clone(),
//       &actual,
//       &expect,
//       2,
//       7,
//       &expect_fills,
//       &expect_fills,
//     );
//   }
//
//   #[test]
//   fn update3() {
//     test_log_init();
//
//     let terminal_size = U16Size::new(20, 20);
//     let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
//     let win_opts = make_nowrap();
//
//     let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![""]);
//     let expect = vec![
//       "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
//     ];
//
//     let window = make_window(terminal_size, buf.clone(), &win_opts);
//     let actual = {
//       let mut buf = wlock!(buf);
//       Viewport::_upward(&mut buf, window.actual_shape(), &win_opts, 1, 0)
//     };
//     let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
//     assert_viewport(
//       buf.clone(),
//       &actual,
//       &expect,
//       0,
//       1,
//       &expect_fills,
//       &expect_fills,
//     );
//   }
//
//   #[test]
//   fn update4() {
//     test_log_init();
//
//     let terminal_size = U16Size::new(20, 20);
//     let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
//     let win_opts = make_nowrap();
//
//     let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![""]);
//     let expect = vec![
//       "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
//     ];
//
//     let window = make_window(terminal_size, buf.clone(), &win_opts);
//     let actual = {
//       let mut buf = wlock!(buf);
//       Viewport::_upward(&mut buf, window.actual_shape(), &win_opts, 1, 0)
//     };
//     let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
//     assert_viewport(
//       buf.clone(),
//       &actual,
//       &expect,
//       0,
//       1,
//       &expect_fills,
//       &expect_fills,
//     );
//   }
// }

#[allow(unused_imports)]
#[cfg(test)]
mod tests_view_wrap_nolinebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn new2() {
    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      5,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new3() {
    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and smal",
      "l test lines.\n",
      "But still it contains several t",
      "hings we want to test:\n",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn new4() {
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new5() {
    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "\t\t* The extra parts are\tsplit into the next\trow,\tif either line-wrap or word-wrap options are been set. If the extra\tparts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "\t\t* The extra par",
      "ts are\tsplit into the ne",
      "xt\trow,\tif either",
      " line-wrap or word-wrap options",
      " are been set. If the extra",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 4)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t\t1. When\tthe line\tis small\tenough to\tcompletely put\tinside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      ],
    );
    let expect = vec![
      "But still it contains several t",
      "hings we want to test:\n",
      "\t\t1. When\t",
      "the line\tis small",
      "\tenough to\tcomple",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new7() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t\t1. When\tthe line\tis small\tenough\tto\tcompletely put\tinside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      ],
    );
    let expect = vec![
      "But still it contains several t",
      "hings we want to test:\n",
      "\t\t1. When\t",
      "the line\tis small",
      "\tenough\tto", // 7 fills
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 7)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new8() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "但它仍然contains several things 我们想要测试的文字内容：\n",
        "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
      ],
    );
    let expect = vec![
      "但它仍然contains several things",
      " 我们想要测试的文字内容：\n",
      "\t第一，当一行文字内容太",
      "小了，然后可以完全的放进窗口的",
      "一行之中，那么行wrap和词wrap两", // 1 fills
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new9() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "但它仍然contains several th\tings 我们想要测试的文字内容：\n",
        "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
      ],
    );
    let expect = vec![
      "但它仍然contains several th",
      "\tings 我们想要测试的文字",
      "内容：\n",
      "\t第一，当一行文字内容太",
      "小了，然后可以完全的放进窗口的",
      "一行之中，那么行wrap和词wrap两", // 1 fills
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new10() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new11() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new12() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![""]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new13() {
    test_log_init();

    let terminal_size = U16Size::new(13, 8);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!",
      "This is a qui",
      "te simple and",
      " small test l",
      "ines.\n",
      "But still it ",
      "contains seve",
      "ral things we",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn update1() {
    let terminal_size = U16Size::new(15, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite",
      " simple and sma",
      "ll test lines.\n",
      "But still it co",
      "ntains several ",
      "things we want ",
      "to test:\n",
      "  1. When the l",
      "ine is small en",
      "ough to complet",
      "ely put inside ",
      "a row of the wi",
      "ndow content wi",
      "dget, then the ",
    ];
    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      4,
      &expect_fills,
      &expect_fills,
    );

    let expect = vec![
      "But still it co",
      "ntains several ",
      "things we want ",
      "to test:\n",
      "  1. When the l",
      "ine is small en",
      "ough to complet",
      "ely put inside ",
      "a row of the wi",
      "ndow content wi",
      "dget, then the ",
      "line-wrap and w",
      "ord-wrap doesn'",
      "t affect the re",
      "ndering.\n",
    ];
    let actual = {
      let buf = lock!(buf);
      Viewport::view(&buf, window.actual_shape(), &win_opts, 2, 0)
    };
    let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      2,
      4,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update2() {
    let terminal_size = U16Size::new(15, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite",
      " simple and sma",
      "ll test lines.\n",
      "But still it co",
      "ntains several ",
      "things we want ",
      "to test:\n",
      "  1. When the l",
      "ine is small en",
      "ough to complet",
      "ely put inside ",
      "a row of the wi",
      "ndow content wi",
      "dget, then the ",
    ];
    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      4,
      &expect_fills,
      &expect_fills,
    );

    let expect = vec![
      "     * The extr",
      "a parts are spl",
      "it into the nex",
      "t row, if eithe",
      "r line-wrap or ",
      "word-wrap optio",
      "ns are been set",
      ". If the extra ",
      "parts are still",
      " too long to pu",
      "t in the next r",
      "ow, repeat this",
      " operation agai",
      "n and again. Th",
      "is operation al",
    ];
    let actual = {
      let buf = lock!(buf);
      Viewport::view(&buf, window.actual_shape(), &win_opts, 6, 0)
    };
    let expect_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      6,
      7,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update3() {
    let terminal_size = U16Size::new(15, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite",
      " simple and sma",
      "ll test lines.\n",
      "",
    ];
    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      3,
      &expect_fills,
      &expect_fills,
    );

    let expect = vec!["This is a quite", " simple and sma", "ll test lines.\n", ""];
    let actual = {
      let buf = lock!(buf);
      Viewport::view(&buf, window.actual_shape(), &win_opts, 1, 0)
    };
    let expect_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      1,
      3,
      &expect_fills,
      &expect_fills,
    );
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests_view_wrap_nolinebreak_startcol {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::*;

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "lo, RSVIM!",
      "s is a qui",
      "te simple ",
      "and small ",
      "test lines",
      ".\n",
      " still it ",
      "contains s",
      "everal thi",
      "ngs we wan",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 3);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn update2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "lo, RSVIM!",
      "s is a qui",
      "te simple ",
      "and small ",
      "test lines",
      ".\n",
      " still it ",
      "contains s",
      "everal thi",
      "ngs we wan",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 3);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn update3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "",
      " simple an",
      "d small te",
      "st lines.\n",
      "ntains sev",
      "eral thing",
      "s we want ",
      "to test:\n",
      "ine is sma",
      "ll enough ",
      "to complet",
      "ely put in",
      "side a row",
      " of the wi",
      "ndow conte",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 15);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 4, &expect_fills, &expect_fills);
  }

  #[test]
  fn update4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "",
      "",
      "a row of t",
      "he window ",
      "content wi",
      "dget, then",
      " the line-",
      "wrap and w",
      "ord-wrap d",
      "oesn't aff",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 1, 60);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 1, 4, &expect_fills, &expect_fills);
  }

  #[test]
  fn update5() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "但它仍然contains several th\tings 我们想要测试的文字内容：\n",
        "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
      ],
    );
    let expect = vec![
      "ins several th\tings 我们",
      "想要测试的文字内容：\n",
      "当一行文字内容太小了，然后可以",
      "完全的放进窗口的一行之中，那么",
      "行wrap和词wrap两个选项并不会影",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = {
      let buf = lock!(buf);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 13);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests_view_wrap_linebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
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
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is to\to long to be completely p\tut in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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
      "\to long to be ",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(
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
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new5() {
    let terminal_size = U16Size::new(31, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(31, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new7() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new8() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple ",
      "andsmalltestlineswithoutevenane",
      "wlinebreakbecausewewanttotestho",
      "witwillhappensifthereisaverylon",
      "gwordthatcannotbeenpplaceinside",
      "arowofthewindowcontent.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "\t\t第一，当一行文",
      "本内容的长度足够短，短到可以完",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new9() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
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
  fn new10() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contai\tseveral things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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
      "it contai",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
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
  fn new11() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, 那么行换行和单词换行选项都不会影响最终的渲染效果。\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!",
      "This is a ",
      "quite simple ",
      "andsmalltestl",
      "ineswithoutev",
      "enanewlinebre",
      "akbecausewewa",
      "nttotesthowit",
      "willhappensif",
      "thereisaveryl",
      "ongwordthatca",
      "nnotbeenpplac",
      "einsidearowof",
      "thewindowcont",
      "ent.\n",
      "But still it ",
      "contains ",
      "several ",
      "things we ",
      "want to test:",
      "  1. When the",
      " line is ",
      "small enough ",
      "to completely",
      " put inside a",
      " row of the ",
      "window ",
      "content ",
      "widget, 那么",
      "行换行和单词",
      "换行选项都不",
      "会影响最终的",
      "渲染效果。\n",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new12() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, 那么行换行和单词换行选项都不会影响最终的渲染效果。\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!",
      "This is a ",
      "quite simple ",
      "andsmalltestl",
      "ineswithoutev",
      "enanewlinebre",
      "akbecausewewa",
      "nttotesthowit",
      "willhappensif",
      "thereisaveryl",
      "ongwordthatca",
      "nnotbeenpplac",
      "einsidearowof",
      "thewindowcont",
      "ent.\n",
      "But still it ",
      "contains ",
      "several ",
      "things we ",
      "want to test:",
      "  1. When the",
      " line is ",
      "small enough ",
      "to completely",
      " put inside a",
      " row of the ",
      "window ",
      "content ",
      "widget, 那么",
      "行换行和单词",
      "换行选项都不",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn new13() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new14() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(terminal_size.height(), buf_opts, vec![""]);
    let expect = vec![""];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new15() {
    test_log_init();

    let terminal_size = U16Size::new(13, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!",
      "This is a ",
      "quite simple ",
      "and small ",
      "test lines.\n",
      "But still it ",
      "contains ",
      "several ",
      "things we ",
      "want to test:",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = lock!(window.viewport()).clone();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests_view_wrap_linebreak_startcol {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "lo, RSVIM!",
      "s is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      " still it ",
      "contains ",
      "several ",
    ];

    let mut window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = {
      let buf = lock!(buffer);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 3);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
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
  fn update2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      " RSVIM!\n",
      "s a quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      "ill it ",
      "contains ",
      "several ",
      "things we ",
    ];

    let mut window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = {
      let buf = lock!(buffer);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 6);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
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
  fn update3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "",
      "le and ",
      "small test",
      " lines.\n",
      "s several ",
      "things we ",
      "want to ",
      "test:\n",
      "s small ",
      "enough to ",
    ];

    let mut window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = {
      let buf = lock!(buffer);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 20);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn update4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "",
      "",
      "",
      "a row of ",
      "the window",
      " content ",
      "widget, ",
      "then the ",
      "line-wrap ",
      "and word-",
    ];

    let mut window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = {
      let buf = lock!(buffer);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 60);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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
  fn update5() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "",
      " simple ",
      "andsmalltestlineswithoutevenane",
      "wlinebreakbecausewewanttotestho",
      "witwillhappensifthereisaverylon",
      "gwordthatcannotbeenpplaceinside",
      "arowofthewindowcontent.\n",
      "ntains several things we want ",
      "to test:\n",
      "第一，当一行文本内容的长度足够",
      "短，短到可以完整的放入一个窗口",
    ];

    let mut window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = {
      let buf = lock!(buffer);
      let window_actual_shape = window.actual_shape();
      let window_local_options = window.options();
      let viewport = Viewport::view(&buf, window_actual_shape, window_local_options, 0, 15);
      window.set_viewport(Viewport::to_arc(viewport));
      lock!(window.viewport()).clone()
    };
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 1)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
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

#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_downward_nowrap {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 15;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 1;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 2), (4, 2), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["ut still it conta", "1. When", "2. When", "\t3.", "\t4."];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 3;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 1);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 7), (4, 7), (5, 7), (6, 7)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 3), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 3;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "",
        "nd small test lin",
        "veral things we w",
        "he\tline",
        "t\t\t",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 40;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "ut\tinside.", ""];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 130;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 112);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "mpletely\tp", ":\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 100;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 95);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "", "", "not\tset."];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 100;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 145);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 2)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["", "\tcompletel", "put:\n", "\tand", "if\teither"];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 50;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 7), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 6), (6, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "",
        "nd small test lin",
        "veral things we w",
        "he\tline",
        "t\t\t",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 40;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "ut\tinside.", ""];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 130;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 112);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "to\tcom", "etely\tput:"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 79);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 4), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "inside.\n", "", "options\ta"];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 80;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 120);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 2), (4, 0), (5, 1)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
        "\t3. The ex",
        "\t4. The ex",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 1;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 8);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_downward_wrap_nolinebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 15;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "small\tenou",
        "gh\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 56);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "if\teither",
        "\tline-wrap",
        "\tor",
        "\tword-wrap",
        "\toptions",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 43;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "small\tenou",
        "gh\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 58;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 56);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "xtra parts are sp",
        "lit into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 10;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["line\tis", "\tsmall", "\tenough", "\tto", "\tcompletel"];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 34);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_downward_wrap_linebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 15;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\tto", "\t", "completely", "\tput", "\tinside.\n"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 69);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 41);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "if\teither",
        "\tline-wrap",
        "\tor",
        "\tword-wrap",
        "\toptions",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 43;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\tto", "\t", "completely", "\tput", "\tinside.\n"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 58;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 69);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 41);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        " extra parts are ",
        "split into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 10;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 22);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["is\tsmall", "\tenough", "\tto", "\t", "completely"];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 46);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 44);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        " truncated if",
        "\tboth",
        "\tline-wrap",
        "\tand",
        "\tword-wrap",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 43);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![" split into the", "\tnext", "\trow,", "\tif", "\teither"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 38);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_upward_nowrap {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["is\tsmall", "long", "runcated if", "into the\tn", ""];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 40;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 45);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 1), (4, 7), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 1), (4, 6), (5, 6), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["to\tcom", "etely\tput:", "e-wrap\tand", "if\te", ""];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 79);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4), (4, 0), (5, 0), (6, 6), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["to\tcom", "etely\tput:", "e-wrap\tand", "if\te", ""];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 38;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 79);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4), (4, 0), (5, 0), (6, 6), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["put\tinsi", "", "wrap\toptio", "line-wrap\t", ""];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 55;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 109);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
        "arts are split in",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 30);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 4), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "ll test lines.\n",
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 32;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 30);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 4), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "SVIM!\n",
        "a quite simple an",
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 8;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 8);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["put\tinsid", "", "rap\toption", "ine-wrap\to", ""];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 70;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 110);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 1), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["inside.\n", "", "options\ta", "or\tw", ""];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 80;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 120);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 0), (5, 1), (6, 6), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["o\tcomplete", "\tput:\n", "p\tand", "if\teither", ""];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 84);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 1), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 5), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["o\tcomplete", "\tput:\n", "p\tand", "if\teither", ""];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 36;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 84);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 1), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 5), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
        "arts are split in",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 30);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 4), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "ll test lines.\n",
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 32;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 30);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 4), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "SVIM!\n",
        "a quite simple an",
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 8;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 8);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_upward_wrap_nolinebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "nd again. This op",
        "eration also eats",
        " more rows in the",
        " window, thus it ",
        "may contains less",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 280;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 287);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["d\tword-wra", "p\toptions", "\tare", "\tnot", "\tset.\n"];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 95);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "small\tenou",
        "gh\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 56);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
        "ains several thin",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 8;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["next\trow,", "\tif", "\teither", "\tline-wrap", "\tor"];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 70;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 61);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 80;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["line\tis", "\tsmall", "\tenough", "\tto", "\tcompletel"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 36;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 34);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
        "ains several thin",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 32;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 8;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_upward_wrap_linebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "eats more rows in",
        " the window, thus",
        " it may contains ",
        "less lines in the",
        " buffer.\n",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 295;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 317);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["\tand", "\tword-wrap", "\toptions", "\tare", "\tnot"];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 41);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["enough\tto", "\t", "completely", "\tput", "\tinside.\n"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 69);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
        "contains several ",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 8;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["next\trow,", "\tif", "\teither", "\tline-wrap", "\tor"];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 70;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 61);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 80;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 41);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["\tis", "\tsmall", "\tenough", "\tto", "\t"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 36;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 38);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
        "contains several ",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 32;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 8;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Up,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_horizontally_nowrap {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 5;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Right,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 12;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 13;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Right,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 10;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Left,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
      ];

      let actual = {
        let target_cursor_line = 0;
        let target_cursor_char = 2;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Left,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "",
        "nd small test lin",
        "veral things we w",
        "he\tline",
        "t\t\t",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 40;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "ut\tinside.", ""];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 130;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 112);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "mpletely\tp", ":\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 100;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 95);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "", "", "not\tset."];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 100;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 145);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 2)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["", "\tcompletel", "put:\n", "\tand", "if\teither"];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 50;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 7), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 6), (6, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "",
        "nd small test lin",
        "veral things we w",
        "he\tline",
        "t\t\t",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 40;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "ut\tinside.", ""];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 130;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 112);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "to\tcom", "etely\tput:"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 79);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 4), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "inside.\n", "", "options\ta"];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 80;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 120);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 2), (4, 0), (5, 1)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
        "\t3. The ex",
        "\t4. The ex",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 1;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 8);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_horizontally_wrap_nolinebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 15;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "small\tenou",
        "gh\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 56);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "if\teither",
        "\tline-wrap",
        "\tor",
        "\tword-wrap",
        "\toptions",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 43;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "small\tenou",
        "gh\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 58;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 56);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "xtra parts are sp",
        "lit into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 10;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["line\tis", "\tsmall", "\tenough", "\tto", "\tcompletel"];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 34);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 24);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
#[allow(unused_imports)]
#[cfg(test)]
mod tests_search_anchor_horizontally_wrap_linebreak {
  use super::tests_util::*;
  use super::*;

  use crate::buf::BufferLocalOptionsBuilder;
  use crate::lock;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Inodeable;

  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 15;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\tto", "\t", "completely", "\tput", "\tinside.\n"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 60;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 69);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 35;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 41);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "if\teither",
        "\tline-wrap",
        "\tor",
        "\tword-wrap",
        "\toptions",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 85);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 43;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 0);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\tto", "\t", "completely", "\tput", "\tinside.\n"];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 58;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 69);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 41);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline-",
        "wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 64);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        " extra parts are ",
        "split into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = {
        let target_cursor_line = 6;
        let target_cursor_char = 10;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 6);
        assert_eq!(start_column, 22);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 7;
        let target_cursor_char = 0;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 7);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = lock!(window.borrow().viewport()).clone();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["is\tsmall", "\tenough", "\tto", "\t", "completely"];

      let actual = {
        let target_cursor_line = 1;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 1);
        assert_eq!(start_column, 46);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
          &viewport,
          &buf,
          target_cursor_line,
          target_cursor_char,
        )));
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["\tlong", "\tto", "\t", "completely", "\tput:\n"];

      let actual = {
        let target_cursor_line = 2;
        let target_cursor_char = 37;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 2);
        assert_eq!(start_column, 44);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        " truncated if",
        "\tboth",
        "\tline-wrap",
        "\tand",
        "\tword-wrap",
      ];

      let actual = {
        let target_cursor_line = 3;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 3);
        assert_eq!(start_column, 43);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![" split into the", "\tnext", "\trow,", "\tif", "\teither"];

      let actual = {
        let target_cursor_line = 4;
        let target_cursor_char = 30;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 4);
        assert_eq!(start_column, 38);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![""];

      let actual = {
        let target_cursor_line = 5;
        let target_cursor_char = 82;

        let mut window = window.borrow_mut();
        let old = lock!(window.viewport()).clone();
        let buf = lock!(buf);
        let (start_line, start_column) = old.search_anchor(
          ViewportSearchAnchorDirection::Down,
          &buf,
          window.actual_shape(),
          window.options(),
          target_cursor_line,
          target_cursor_char,
        );
        assert_eq!(start_line, 5);
        assert_eq!(start_column, 0);

        let viewport = Viewport::view(
          &buf,
          window.actual_shape(),
          window.options(),
          start_line,
          start_column,
        );
        window.set_viewport(Viewport::to_arc(viewport));
        lock!(window.viewport()).clone()
      };

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
// spellchecker:on
