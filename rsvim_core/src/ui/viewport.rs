//! Fundamental viewport for all kinds of buffer typeset/rendering in UI widgets.

use crate::buf::text::Text;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::TreeNodeId;
use crate::ui::widget::window::opt::WindowOptions;

use litemap::LiteMap;
use std::ops::Range;

pub mod draw;
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
  rows: LiteMap<u16, RowViewport>,
  start_filled_cols: usize,
  end_filled_cols: usize,
}

impl LineViewport {
  /// Make new instance.
  pub fn new(
    rows: LiteMap<u16, RowViewport>,
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
  pub fn rows(&self) -> &LiteMap<u16, RowViewport> {
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

arc_ptr!(CursorViewport);

impl CursorViewport {
  /// Make new instance.
  pub fn new(
    line_idx: usize,
    char_idx: usize,
    row_idx: u16,
    column_idx: u16,
  ) -> Self {
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
  pub fn from_top_left(viewport: &Viewport, text: &Text) -> Self {
    debug_assert!(viewport.end_line_idx() >= viewport.start_line_idx());
    if viewport.end_line_idx() == viewport.start_line_idx() {
      return Self::new(0, 0, 0, 0);
    }

    let lines = viewport.lines();
    debug_assert!(viewport.end_line_idx() > viewport.start_line_idx());
    debug_assert!(!lines.is_empty());
    debug_assert!(
      lines.len() == viewport.end_line_idx() - viewport.start_line_idx()
    );
    debug_assert!(lines.first().is_some());
    debug_assert!(lines.last().is_some());
    debug_assert!(*lines.first().unwrap().0 == viewport.start_line_idx());
    debug_assert!(viewport.end_line_idx() > 0);
    debug_assert!(*lines.last().unwrap().0 == viewport.end_line_idx() - 1);
    let first_line = lines.first().unwrap();
    let line_idx = *first_line.0;
    let first_line = first_line.1;

    if first_line.rows().is_empty() {
      return Self::new(0, 0, 0, 0);
    }

    let first_row = first_line.rows().first().unwrap();
    let first_row = first_row.1;

    debug_assert!(first_row.end_char_idx() >= first_row.start_char_idx());
    if first_row.end_char_idx() == first_row.start_char_idx() {
      debug_assert_eq!(first_row.start_char_idx(), 0);
      debug_assert_eq!(first_row.end_char_idx(), 0);
      return Self::new(0, 0, 0, 0);
    }

    let char_idx = first_row.start_char_idx();
    Self::from_position(viewport, text, line_idx, char_idx)
  }

  /// Create cursor viewport with specified position (buffer's line/char index) from the window
  /// viewport.
  ///
  /// # Panics
  ///
  /// It panics if the line/char index are not shown in the window viewport.
  pub fn from_position(
    viewport: &Viewport,
    text: &Text,
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
        row_viewport.start_char_idx() <= char_idx
          && row_viewport.end_char_idx() > char_idx
      })
      .collect::<Vec<_>>();

    if !cursor_row.is_empty() {
      debug_assert_eq!(cursor_row.len(), 1);
      let (row_idx, row_viewport) = cursor_row[0];

      let mut row_start_width =
        text.width_before(line_idx, row_viewport.start_char_idx());

      // Subtract `start_filled_cols` if the row is the first row in the line.
      let (first_row_idx, _first_row_viewport) =
        line_viewport.rows.first().unwrap();
      if first_row_idx == row_idx {
        debug_assert!(row_start_width >= line_viewport.start_filled_cols());
        row_start_width -= line_viewport.start_filled_cols();
      };

      let char_start_width = text.width_before(line_idx, char_idx);
      let col_idx = (char_start_width - row_start_width) as u16;
      let row_idx = *row_idx;

      CursorViewport::new(line_idx, char_idx, row_idx, col_idx)
    } else {
      let target_is_eol = text.is_eol(line_idx, char_idx);
      if target_is_eol {
        // The target cursor is eol, and it doesn't have a space to put in the viewport, it
        // indicates:
        //
        // 1. The window must be `wrap=true`
        // 2. The viewport must contains `line_idx+1`.
        // 3. The target cursor position is out of viewport.
        //
        // The cursor will be put in the position `(next line, 0-column)`.

        let next_line_idx = line_idx + 1;
        debug_assert!(viewport.lines().contains_key(&next_line_idx));
        let next_line_viewport = viewport.lines().get(&next_line_idx).unwrap();
        debug_assert!(next_line_viewport.rows().first().is_some());
        let (first_row_idx, _first_row_viewport) =
          next_line_viewport.rows().first().unwrap();
        CursorViewport::new(line_idx, char_idx, *first_row_idx, 0_u16)
      } else {
        debug_assert!(line_viewport.rows().first().is_some());
        let (first_row_idx, _first_row_viewport) =
          line_viewport.rows().first().unwrap();
        CursorViewport::new(line_idx, char_idx, *first_row_idx, 0_u16)
      }
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
  lines: LiteMap<usize, LineViewport>,
}

arc_ptr!(Viewport);

#[derive(Debug, Copy, Clone)]
pub enum ViewportSearchDirection {
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
    opts: &WindowOptions,
    text: &Text,
    shape: &U16Rect,
    start_line: usize,
    start_column: usize,
  ) -> Self {
    let (line_idx_range, lines) =
      sync::sync(opts, text, shape, start_line, start_column);

    debug_assert_eq!(line_idx_range.start_line_idx(), start_line);

    Viewport {
      start_line_idx: line_idx_range.start_line_idx(),
      end_line_idx: line_idx_range.end_line_idx(),
      start_column_idx: start_column,
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
    direction: ViewportSearchDirection,
    opts: &WindowOptions,
    text: &Text,
    shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> (usize, usize) {
    // If window is zero-sized.
    let height = shape.height();
    let width = shape.width();
    if height == 0 || width == 0 {
      return (0, 0);
    }

    match direction {
      ViewportSearchDirection::Down => sync::search_anchor_downward(
        self,
        opts,
        text,
        shape,
        target_cursor_line,
        target_cursor_char,
      ),
      ViewportSearchDirection::Up => sync::search_anchor_upward(
        self,
        opts,
        text,
        shape,
        target_cursor_line,
        target_cursor_char,
      ),
      ViewportSearchDirection::Left => sync::search_anchor_leftward(
        self,
        opts,
        text,
        shape,
        target_cursor_line,
        target_cursor_char,
      ),
      ViewportSearchDirection::Right => sync::search_anchor_rightward(
        self,
        opts,
        text,
        shape,
        target_cursor_line,
        target_cursor_char,
      ),
    }
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    debug_assert!(self.end_line_idx >= self.start_line_idx);
    debug_assert_eq!(
      self.end_line_idx == self.start_line_idx,
      self.lines.is_empty()
    );
    debug_assert!(self.lines.first().is_some());
    debug_assert_eq!(*self.lines.first().unwrap().0, self.start_line_idx);
    debug_assert!(self.lines.last().is_some());
    debug_assert_eq!(*self.lines.last().unwrap().0, self.end_line_idx - 1);
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
        // trace!(
        //   "line_idx:{:?},row_idx:{:?},last_row_idx:{:?},last_row_viewport:{:?},row_viewport:{:?}",
        //   line_idx, row_idx, last_row_idx, last_row_viewport, row_viewport
        // );
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
  pub fn lines(&self) -> &LiteMap<usize, LineViewport> {
    self._internal_check();
    &self.lines
  }

  /// Whether viewport is empty.
  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.lines.is_empty()
  }

  pub fn draw(&self, text: &Text, actual_shape: &U16Rect, canvas: &mut Canvas) {
    draw::draw(self, text, actual_shape, canvas);
  }
}

pub trait ViewportEditable {
  fn editable_viewport(&self) -> ViewportArc;

  fn set_editable_viewport(&mut self, viewport: ViewportArc);

  fn editable_cursor_viewport(&self) -> CursorViewportArc;

  fn set_editable_cursor_viewport(
    &mut self,
    cursor_viewport: CursorViewportArc,
  );

  fn editable_options(&self) -> &WindowOptions;

  fn editable_actual_shape(&self) -> &U16Rect;

  fn move_editable_cursor_to(&mut self, x: isize, y: isize) -> Option<IRect>;

  fn editable_cursor_id(&self) -> Option<TreeNodeId>;
}
