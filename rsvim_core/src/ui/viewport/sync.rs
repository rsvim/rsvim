//! Internal implementations for Viewport.

#![allow(clippy::too_many_arguments)]

use crate::buf::Buffer;
use crate::prelude::*;
use crate::ui::viewport::{LineViewport, RowViewport, ViewportOptions};

use litemap::LiteMap;
use ropey::RopeSlice;
use std::ops::Range;
#[allow(unused_imports)]
use tracing::trace;
use unicode_segmentation::UnicodeSegmentation;

use super::Viewport;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
/// Lines index inside the viewport.
pub struct ViewportLineRange {
  start_line_idx: usize,
  end_line_idx: usize,
}

impl ViewportLineRange {
  pub fn new(line_idx_range: Range<usize>) -> Self {
    Self {
      start_line_idx: line_idx_range.start,
      end_line_idx: line_idx_range.end,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.end_line_idx <= self.start_line_idx
  }

  pub fn len(&self) -> usize {
    self.end_line_idx - self.start_line_idx
  }

  // Get start line index in the buffer, starts from 0.
  pub fn start_line_idx(&self) -> usize {
    self.start_line_idx
  }

  // Get end line index in the buffer.
  pub fn end_line_idx(&self) -> usize {
    self.end_line_idx
  }
}

/// Calculate viewport from top to bottom.
pub fn sync(
  opts: &ViewportOptions,
  buffer: &Buffer,
  shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  // If window is zero-sized.
  let height = shape.height();
  let width = shape.width();
  if height == 0 || width == 0 {
    return (ViewportLineRange::default(), LiteMap::new());
  }

  match (opts.wrap(), opts.line_break()) {
    (false, _) => sync_nowrap(buffer, shape, start_line, start_column),
    (true, false) => sync_wrap_nolinebreak(buffer, shape, start_line, start_column),
    (true, true) => sync_wrap_linebreak(buffer, shape, start_line, start_column),
  }
}

fn _end_char_and_prefills(
  buffer: &Buffer,
  bline: &RopeSlice,
  l: usize,
  c: usize,
  end_width: usize,
) -> (usize, usize) {
  let c_width = buffer.width_until(l, c);
  if c_width > end_width {
    // If the char `c` width is greater than `end_width`, the `c` itself is the end char.
    let c_width_before = buffer.width_before(l, c);
    (c, end_width.saturating_sub(c_width_before))
  } else {
    // Here we use the last visible char in the line, thus avoid those invisible chars like '\n'.
    debug_assert!(bline.len_chars() > 0);
    let next_to_last_visible_char = buffer.last_char_on_line_no_empty_eol(l).unwrap() + 1;

    // If the char `c` width is less than or equal to `end_width`, the char next to `c` is the end
    // char.
    let c_next = std::cmp::min(c + 1, next_to_last_visible_char);
    (c_next, 0_usize)
  }
}

/// Returns `rows`, `start_fills`, `end_fills`, `current_row`.
fn proc_line_nowrap(
  buffer: &Buffer,
  start_column: usize,
  current_line: usize,
  current_row: u16,
  _window_height: u16,
  window_width: u16,
) -> (LiteMap<u16, RowViewport>, usize, usize, u16) {
  let bufline = buffer.get_rope().line(current_line);
  let (start_char, start_fills, end_char, end_fills) = if bufline.len_chars() == 0 {
    (0_usize, 0_usize, 0_usize, 0_usize)
  } else {
    match buffer.char_after(current_line, start_column) {
      Some(start_char) => {
        let start_fills = {
          let width_before = buffer.width_before(current_line, start_char);
          width_before.saturating_sub(start_column)
        };

        let end_width = start_column + window_width as usize;
        let (end_char, end_fills) = match buffer.char_at(current_line, end_width) {
          Some(c) => _end_char_and_prefills(buffer, &bufline, current_line, c, end_width),
          None => {
            // If the char not found, it means the `end_width` is too long than the whole line.
            // So the char next to the line's last char is the end char.
            (bufline.len_chars(), 0_usize)
          }
        };

        (start_char, start_fills, end_char, end_fills)
      }
      None => (0_usize, 0_usize, 0_usize, 0_usize),
    }
  };

  let mut rows: LiteMap<u16, RowViewport> = LiteMap::with_capacity(1);
  rows.insert(current_row, RowViewport::new(start_char..end_char));
  (rows, start_fills, end_fills, current_row)
}

/// Implements [`sync`] with option `wrap=false`.
fn sync_nowrap(
  buffer: &Buffer,
  shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  let height = shape.height();
  let width = shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut line_viewports: LiteMap<usize, LineViewport> = LiteMap::with_capacity(height as usize);

  // The first `current_row` in the window maps to the `start_line` in the buffer.
  let mut current_row = 0_u16;
  let mut current_line = start_line;

  if current_line < buffer_len_lines {
    // If `current_row` goes out of window, `current_line` goes out of buffer.
    while current_row < height && current_line < buffer_len_lines {
      let (rows, start_fills, end_fills, _) = proc_line_nowrap(
        buffer,
        start_column,
        current_line,
        current_row,
        height,
        width,
      );

      line_viewports.insert(
        current_line,
        LineViewport::new(rows, start_fills, end_fills),
      );

      // Go down to next row and line
      current_line += 1;
      current_row += 1;
    }

    (
      ViewportLineRange::new(start_line..current_line),
      line_viewports,
    )
  } else {
    (ViewportLineRange::default(), LiteMap::new())
  }
}

/// Returns `rows`, `start_fills`, `end_fills`, `current_row`.
fn proc_line_wrap_nolinebreak(
  buffer: &Buffer,
  start_column: usize,
  current_line: usize,
  mut current_row: u16,
  window_height: u16,
  window_width: u16,
) -> (LiteMap<u16, RowViewport>, usize, usize, u16) {
  let bufline = buffer.get_rope().line(current_line);
  let bufline_len_chars = bufline.len_chars();

  if bufline_len_chars == 0 {
    let mut rows: LiteMap<u16, RowViewport> = LiteMap::with_capacity(1);
    rows.insert(current_row, RowViewport::new(0..0));
    (rows, 0_usize, 0_usize, current_row)
  } else {
    let mut rows: LiteMap<u16, RowViewport> = LiteMap::with_capacity(window_height as usize);

    // let mut start_char = buffer
    match buffer.char_after(current_line, start_column) {
      Some(mut start_char) => {
        let start_fills = {
          let width_before = buffer.width_before(current_line, start_char);
          width_before.saturating_sub(start_column)
        };

        let mut end_width = start_column + window_width as usize;
        let mut end_fills = 0_usize;

        debug_assert!(current_row < window_height);
        while current_row < window_height {
          let (end_char, end_fills_result) = match buffer.char_at(current_line, end_width) {
            Some(c) => _end_char_and_prefills(buffer, &bufline, current_line, c, end_width),
            None => {
              // If the char not found, it means the `end_width` is too long than the whole line.
              // So the char next to the line's last char is the end char.
              (bufline_len_chars, 0_usize)
            }
          };
          end_fills = end_fills_result;

          rows.insert(current_row, RowViewport::new(start_char..end_char));

          // Goes out of line.
          debug_assert!(bufline.len_chars() > 0);
          if end_char > buffer.last_char_on_line_no_empty_eol(current_line).unwrap() {
            break;
          }

          // Prepare next row.
          current_row += 1;
          start_char = end_char;
          end_width = buffer.width_before(current_line, end_char) + window_width as usize;
        }

        (rows, start_fills, end_fills, current_row)
      }
      None => (rows, 0_usize, 0_usize, current_row),
    }
  }
}

/// Implements [`sync`] with option `wrap=true` and `line-break=false`.
fn sync_wrap_nolinebreak(
  buffer: &Buffer,
  shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  let height = shape.height();
  let width = shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut line_viewports: LiteMap<usize, LineViewport> = LiteMap::with_capacity(height as usize);

  // The first `current_row` in the window maps to the `start_line` in the buffer.
  let mut current_row = 0_u16;
  let mut current_line = start_line;

  if current_line < buffer_len_lines {
    // If `current_row` goes out of window, `current_line` goes out of buffer.
    while current_row < height && current_line < buffer_len_lines {
      let (rows, start_fills, end_fills, changed_current_row) = proc_line_wrap_nolinebreak(
        buffer,
        start_column,
        current_line,
        current_row,
        height,
        width,
      );
      current_row = changed_current_row;

      line_viewports.insert(
        current_line,
        LineViewport::new(rows, start_fills, end_fills),
      );

      current_line += 1;
      current_row += 1;
    }

    (
      ViewportLineRange::new(start_line..current_line),
      line_viewports,
    )
  } else {
    (ViewportLineRange::default(), LiteMap::new())
  }
}

/// Find the word index by the char index.
///
/// Returns the word index which contains this char, and whether the char is the last char in the
/// word.
fn _find_word_by_char(
  words: &[&str],
  word_end_chars_index: &LiteMap<usize, usize>,
  char_idx: usize,
) -> (usize, usize, usize) {
  // trace!("words:{words:?}, words_end_chars:{word_end_chars_index:?},char_idx:{char_idx}");
  let mut low = 0;
  let mut high = words.len() - 1;

  while low <= high {
    let mid = (low + high) / 2;

    let start_char_idx = if mid > 0 {
      *word_end_chars_index.get(&(mid - 1)).unwrap()
    } else {
      0_usize
    };
    let end_char_idx = *word_end_chars_index.get(&mid).unwrap();

    // trace!(
    //   "low:{low},high:{high},mid:{mid},start_char_idx:{start_char_idx},end_char_idx:{end_char_idx},char_idx:{char_idx}"
    // );
    if start_char_idx <= char_idx && end_char_idx > char_idx {
      // trace!(
      //   "return mid:{mid},start_char_idx:{start_char_idx},end_char_idx:{end_char_idx},char_idx:{char_idx}"
      // );
      return (mid, start_char_idx, end_char_idx);
    } else if start_char_idx > char_idx {
      high = mid - 1;
    } else {
      low = mid + 1;
    }
  }

  unreachable!()
}

/// Part-1 of the processing algorithm in [`proc_line_wrap_linebreak`].
fn _part1(
  words: &[&str],
  words_end_char_idx: &LiteMap<usize, usize>,
  buffer: &Buffer,
  bline: &RopeSlice,
  l: usize,
  c: usize,
  end_width: usize,
  start_char: usize,
  last_word_is_too_long: &mut Option<(usize, usize, usize, usize)>,
) -> (usize, usize) {
  let (wd_idx, start_c_of_wd, end_c_of_wd) = _find_word_by_char(words, words_end_char_idx, c);

  let end_c_width = buffer.width_before(l, end_c_of_wd);
  if end_c_width > end_width {
    // The current word is longer than current row, it needs to be put to next row.

    // Part-1
    // Here's the **tricky** part, there are two sub-cases in this scenario:
    // 1. For most happy cases, the word is not longer than a whole row in the
    //    window, so it can be completely put to next row.
    // 2. For very rare cases, the word is just too long to put in an entire row
    //    in the window. And in this case, we fallback to the no-line-break
    //    rendering behavior, i.e. just cut the word by chars and force rendering
    //    the word on multiple rows in the window (because otherwise there will be
    //    never enough places to put the whole word).

    if start_c_of_wd > start_char {
      // Part-1.1, simply wrapped this word to next row.
      // Here we actually use the `start_c_of_wd` as the end char for current row.

      _end_char_and_prefills(buffer, bline, l, start_c_of_wd - 1, end_width)
    } else {
      // Part-1.2, cut this word and force rendering it ignoring line-break behavior.
      debug_assert!(start_c_of_wd <= start_char);
      // Record the position (c) where we cut the words into pieces.
      *last_word_is_too_long = Some((wd_idx, start_c_of_wd, end_c_of_wd, c));

      // If the char `c` width is greater than `end_width`, the `c` itself is the end char.
      _end_char_and_prefills(buffer, bline, l, c, end_width)
    }
  } else {
    debug_assert_eq!(c + 1, end_c_of_wd);
    // The current word is not long, it can be put in current row.
    let c_next = std::cmp::min(end_c_of_wd, bline.len_chars());
    (c_next, 0_usize)
  }
}

fn _cloned_line_max_len(window_height: u16, window_width: u16) -> usize {
  window_height as usize * window_width as usize * 2 + 16
}

/// Returns `rows`, `start_fills`, `end_fills`, `current_row`.
fn proc_line_wrap_linebreak(
  buffer: &Buffer,
  start_column: usize,
  current_line: usize,
  mut current_row: u16,
  window_height: u16,
  window_width: u16,
) -> (LiteMap<u16, RowViewport>, usize, usize, u16) {
  let bufline = buffer.get_rope().line(current_line);
  if bufline.len_chars() == 0 {
    let mut rows: LiteMap<u16, RowViewport> = LiteMap::with_capacity(1);
    rows.insert(current_row, RowViewport::new(0..0));
    (rows, 0_usize, 0_usize, current_row)
  } else {
    let mut rows: LiteMap<u16, RowViewport> = LiteMap::with_capacity(window_height as usize);

    // Here clone the line with the max chars that can hold by current window/viewport,
    // i.e. the `height * width` cells count as the max chars in the line. This helps avoid
    // performance issue when iterating on super long lines.

    // Clone this line from `cloned_start_char`, thus we can limit the cloned text within the
    // window's size (i.e. height * width).
    let cloned_start_char = buffer
      .char_before(current_line, start_column)
      .unwrap_or(0_usize);

    let cloned_line = buffer
      .clone_line(
        current_line,
        cloned_start_char,
        _cloned_line_max_len(window_height, window_width),
      )
      .unwrap();

    trace!(
      "cloned_line({}):{:?}, start_column:{}",
      cloned_line.len(),
      cloned_line.as_str(),
      start_column
    );

    // Words.
    let words: Vec<&str> = cloned_line.split_word_bounds().collect();
    // Word index => its end char index (from the first char until current word).
    let words_end_char_idx = words
      .iter()
      .enumerate()
      .scan(cloned_start_char, |state, (i, wd)| {
        *state += wd.chars().count();
        Some((i, *state))
      })
      .collect::<LiteMap<usize, usize>>();

    // let mut start_char = buffer
    match buffer.char_after(current_line, start_column) {
      Some(mut start_char) => {
        let start_fills = {
          let width_before = buffer.width_before(current_line, start_char);
          width_before.saturating_sub(start_column)
        };

        let mut end_width = start_column + window_width as usize;
        let mut end_fills = 0_usize;

        // Saved last word info, if it is too long to put in an entire row of window.
        // The tuple is:
        // 1. Word index.
        // 2. Start char of the word.
        // 3. End char of the word.
        // 4. Continued start char index of the word (which should be continued to rendering on
        //    current row).
        let mut last_word_is_too_long: Option<(usize, usize, usize, usize)> = None;

        debug_assert!(current_row < window_height);
        while current_row < window_height {
          let (end_char, end_fills_result) = match buffer.char_at(current_line, end_width) {
            Some(c) => {
              match last_word_is_too_long {
                Some((
                  last_wd_idx,
                  start_c_of_last_wd,
                  end_c_of_last_wd,
                  _continued_c_of_last_wd,
                )) => {
                  // Part-2
                  // This is the following logic of part-1.2, you should see part-1 before
                  // this.
                  //
                  // If the word is too long to put in an entire row, and we cut it into
                  // pieces. In this part, we need to continue rendering the rest part of the
                  // word on current row.
                  //
                  // Here we also have two sub-cases:
                  // 1. If the rest part of the word is still too long to put in current row.
                  // 2. If the rest part of the word is not long and can be put in current row.

                  match buffer.char_at(current_line, end_width) {
                    Some(c) => {
                      if end_c_of_last_wd > c {
                        // Part-2.1, the rest part of the word is still too long.

                        // Record the position (c) where we cut the words into pieces.
                        last_word_is_too_long =
                          Some((last_wd_idx, start_c_of_last_wd, end_c_of_last_wd, c));

                        // If the char `c` width is greater than `end_width`, the `c` itself is
                        // the end char.
                        _end_char_and_prefills(buffer, &bufline, current_line, c, end_width)
                      } else {
                        // Part-2.2, the rest part of the word is not long.
                        // Thus we can go back to *normal* algorithm just like part-1.

                        _part1(
                          &words,
                          &words_end_char_idx,
                          buffer,
                          &bufline,
                          current_line,
                          c,
                          end_width,
                          start_char,
                          &mut last_word_is_too_long,
                        )
                      }
                    }
                    None => {
                      // If the char not found, it means the `end_width` is too long than the
                      // whole buffer line.
                      // So the char next to the line's last char is the end char.
                      (bufline.len_chars(), 0_usize)
                    }
                  }
                }
                None => {
                  // Part-1
                  _part1(
                    &words,
                    &words_end_char_idx,
                    buffer,
                    &bufline,
                    current_line,
                    c,
                    end_width,
                    start_char,
                    &mut last_word_is_too_long,
                  )
                }
              }
            }
            None => {
              // If the char not found, it means the `end_width` is too long than the whole line.
              // So the char next to the line's last char is the end char.
              (bufline.len_chars(), 0_usize)
            }
          };
          end_fills = end_fills_result;

          rows.insert(current_row, RowViewport::new(start_char..end_char));

          // Goes out of line.
          debug_assert!(bufline.len_chars() > 0);
          if end_char > buffer.last_char_on_line_no_empty_eol(current_line).unwrap() {
            break;
          }

          // Prepare next row.
          current_row += 1;
          start_char = end_char;
          end_width = buffer.width_before(current_line, end_char) + window_width as usize;
        }

        (rows, start_fills, end_fills, current_row)
      }
      None => (rows, 0_usize, 0_usize, current_row),
    }
  }
}

/// Implements [`sync`] with option `wrap=true` and `line-break=true`.
fn sync_wrap_linebreak(
  buffer: &Buffer,
  shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  let height = shape.height();
  let width = shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut line_viewports: LiteMap<usize, LineViewport> = LiteMap::with_capacity(height as usize);

  // The first `current_row` in the window maps to the `start_line` in the buffer.
  let mut current_row = 0_u16;
  let mut current_line = start_line;

  if current_line < buffer_len_lines {
    // If `current_row` goes out of window, `current_line` goes out of buffer.
    while current_row < height && current_line < buffer_len_lines {
      let (rows, start_fills, end_fills, changed_current_row) = proc_line_wrap_linebreak(
        buffer,
        start_column,
        current_line,
        current_row,
        height,
        width,
      );
      current_row = changed_current_row;

      line_viewports.insert(
        current_line,
        LineViewport::new(rows, start_fills, end_fills),
      );

      current_line += 1;
      current_row += 1;
    }

    (
      ViewportLineRange::new(start_line..current_line),
      line_viewports,
    )
  } else {
    (ViewportLineRange::default(), LiteMap::new())
  }
}

mod detail {
  #[derive(Debug, Copy, Clone)]
  pub struct AdjustOptions {
    pub no_leftward: bool,
    pub no_rightward: bool,
  }

  impl AdjustOptions {
    pub fn no_leftward() -> Self {
      Self {
        no_leftward: true,
        no_rightward: false,
      }
    }
    pub fn no_rightward() -> Self {
      Self {
        no_leftward: false,
        no_rightward: true,
      }
    }
    pub fn all() -> Self {
      Self {
        no_leftward: false,
        no_rightward: false,
      }
    }
  }
}

mod nowrap_detail {
  use super::*;

  // spellchecker:off
  // When searching the new viewport downward, the target cursor could be not shown in it.
  //
  // For example:
  //
  // ```text
  //                                           |----------------------------------|
  // This is the beginning of the very long lin|e, which only shows the beginning |part.
  // This is the short line, it's not shown.   |                                  |
  // This is the second very long line, which s|till shows in the viewport.       |
  //                                           |----------------------------------|
  // ```
  //
  // If the target cursor is in the 2nd line, it will not be shown in the new viewport. This is
  // because old `viewport_start_column` is too big, and incase we need to place the target cursor in
  // the new viewport correctly, so we will have to move the new viewport to the left to allow cursor
  // show in it.
  //
  // ```text
  //      |----------------------------------|
  // This |is the beginning of the very long |line, which only shows the beginning part.
  // This |is the short line, it's not shown.|
  // This |is the second very long line, whic|h s|till shows in the viewport.
  //      |----------------------------------|
  // ```
  //
  // There are 2 edge cases:
  // 1. The target cursor is on the left side.
  // 1. The target cursor is on the right side.
  //
  // Returns
  // 1. If target cursor is on the left side of viewport, and we need to move the viewport more to
  //    the left side.
  // 2. If 1st is true, this is the new "start_column" after adjustments.
  // spellchecker:on
  fn to_left(
    buffer: &Buffer,
    _window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    if cfg!(debug_assertions) {
      match buffer.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }
    }

    let mut target_cursor_width = buffer.width_before(target_cursor_line, target_cursor_char);

    // For empty eol, sub extra 1 column.
    let target_is_empty_eol = buffer.is_empty_eol(target_cursor_line, target_cursor_char);
    if target_is_empty_eol {
      target_cursor_width = target_cursor_width.saturating_sub(1);
    }

    let on_left_side = target_cursor_width < target_viewport_start_column;
    if on_left_side {
      // We need to move viewport to left to show the cursor, to minimize the viewport adjustments,
      // just put the cursor at the first left char in the new viewport.
      let start_column = target_cursor_width;
      Some(start_column)
    } else {
      None
    }
  }

  // Returns
  // 1. If target cursor is on the right side of viewport, and we need to adjust/move the viewport to
  //    right.
  // 2. If 1st is true, this is the new "start_column" after adjustments.
  fn to_right(
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    let width = window_actual_shape.width();
    let viewport_end_column = target_viewport_start_column + width as usize;

    if cfg!(debug_assertions) {
      let target_viewport_start_char =
        match buffer.char_after(target_cursor_line, target_viewport_start_column) {
          Some(c) => format!(
            "{}({:?})",
            c,
            buffer.get_rope().line(target_cursor_line).char(c)
          ),
          None => "None".to_string(),
        };
      let viewport_end_char = match buffer.char_at(target_cursor_line, viewport_end_column) {
        Some(c) => format!(
          "{}({:?})",
          c,
          buffer.get_rope().line(target_cursor_line).char(c)
        ),
        None => "None".to_string(),
      };
      trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{},viewport_end_column:{},viewport_end_char:{}",
        target_cursor_line,
        target_cursor_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        target_viewport_start_column,
        target_viewport_start_char,
        viewport_end_column,
        viewport_end_char,
      );
    }

    let target_is_empty_eol = buffer.is_empty_eol(target_cursor_line, target_cursor_char);
    let target_cursor_width = buffer.width_until(target_cursor_line, target_cursor_char)
      + if target_is_empty_eol { 1 } else { 0 }; // For empty eol, add extra 1 column.
    let on_right_side = target_cursor_width > viewport_end_column;

    if on_right_side {
      // Move viewport to right to show the cursor, just put the cursor at the last right char in the
      // new viewport.
      let end_column = target_cursor_width;
      let start_column = end_column.saturating_sub(width as usize);
      Some(start_column)
    } else {
      None
    }
  }

  pub fn adjust_nowrap(
    opts: detail::AdjustOptions,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
    start_line: usize,
    start_column: usize,
  ) -> (usize, usize) {
    debug_assert!(!(opts.no_leftward && opts.no_rightward));

    if opts.no_leftward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_left(
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_left_side = to_left(
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_left) = start_column_on_left_side {
        return (start_line, start_column_left);
      }
    }

    if opts.no_rightward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_right(
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_right_side = to_right(
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_right) = start_column_on_right_side {
        return (start_line, start_column_right);
      }
    }

    (start_line, start_column)
  }
}

mod wrap_detail {
  use super::*;

  pub type SyncFn = fn(
    /* buffer */ &Buffer,
    /* window_actual_shape */ &U16Rect,
    /* start_line */ usize,
    /* start_column */ usize,
  ) -> (
    /* line range */ ViewportLineRange,
    /* lines_viewport */ LiteMap<usize, LineViewport>,
  );

  // Type alias for `proc_line_*` functions.
  pub type ProcessLineFn = fn(
    /* buffer */ &Buffer,
    /* start_column */ usize,
    /* current_line */ usize,
    /* mut current_row */ u16,
    /* window_height */ u16,
    /* window_width */ u16,
  ) -> (
    /* rows */ LiteMap<u16, RowViewport>,
    /* start_fills */ usize,
    /* end_fills */ usize,
    /* next_current_row */ u16,
  );

  pub fn maximized_viewport_height(height: u16) -> u16 {
    height.saturating_add(3)
  }

  fn find_start_char(
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
    mut start_column: usize,
  ) -> usize {
    let bufline = buffer.get_rope().line(target_cursor_line);
    let bufline_len_char = bufline.len_chars();
    let bufline_chars_width = buffer.width_until(target_cursor_line, bufline_len_char);

    while start_column < bufline_chars_width {
      let (rows, _start_fills, _end_fills, _) = proc_fn(
        buffer,
        start_column,
        target_cursor_line,
        0_u16,
        window_actual_shape.height(),
        window_actual_shape.width(),
      );
      let (_last_row_idx, last_row_viewport) = rows.last().unwrap();
      if last_row_viewport.end_char_idx() > target_cursor_char {
        return start_column;
      }
      start_column += 1;
    }

    unreachable!()
  }

  fn reverse_search_start_column(
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> usize {
    let target_is_empty_eol = buffer.is_empty_eol(target_cursor_line, target_cursor_char);
    let target_cursor_width = buffer.width_until(target_cursor_line, target_cursor_char)
      + if target_is_empty_eol { 1 } else { 0 }; // For empty eol, add extra 1 column.

    let approximate_start_column = target_cursor_width.saturating_sub(
      (window_actual_shape.height() as usize) * (window_actual_shape.width() as usize),
    );

    find_start_char(
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      approximate_start_column,
    )
  }

  // For case-1
  fn to_left_1(
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    let mut start_column = target_viewport_start_column;

    if cfg!(debug_assertions) {
      match buffer.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }
    }

    // spellchecker:off
    // NOTE: The `cannot_fully_contains_target_cursor_line` in caller functions is calculated with
    // `start_column=0`, but here the `start_column` actually belongs to the old viewport, which
    // can be greater than 0. If the tail of target cursor line doesn't fully use the spaces of the
    // viewport, this is not good because we didn't make full use of the viewport. Thus we should
    // try to mov viewport to left side to use all of the spaces in the viewport.
    //
    // For example:
    //
    // ```text
    //                                           |----------------------------------|
    // This is the beginning of the very long lin|e, which only shows the beginning |
    //                                           |part.                             |
    //                                           |----------------------------------|
    // ```
    //
    // Apparently we can move viewport to left to use more spaces, like this:
    //
    // ```text
    //              |----------------------------------|
    // This is the b|eginning of the very long line, wh|
    //              |ich only shows the beginning part.|
    //              |----------------------------------|
    // ```
    //
    // Which is a much better algorithm.
    // spellchecker:on

    let mut on_left_side = false;

    debug_assert!(buffer.get_rope().get_line(target_cursor_line).is_some());
    let last_char = buffer
      .last_char_on_line(target_cursor_line) // Also consider empty eol char.
      .unwrap_or(0_usize);

    let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
      buffer,
      start_column,
      target_cursor_line,
      0_u16,
      maximized_viewport_height(window_actual_shape.height()),
      window_actual_shape.width(),
    );

    let extra_space_left = match preview_target_rows.last() {
      Some((_last_row_idx, last_row_viewport)) => last_row_viewport.end_char_idx() > last_char,
      None => true,
    };

    // If there is extra space left in viewport, i.e. viewport is not fully used, we need to do a
    // reverse search to try to locate the better `start_column`.
    if extra_space_left {
      let start_column_include_last_visible_char = reverse_search_start_column(
        proc_fn,
        buffer,
        window_actual_shape,
        target_cursor_line,
        last_char,
      );
      if start_column > start_column_include_last_visible_char {
        start_column = start_column_include_last_visible_char;
        on_left_side = true;
      }
    }

    // If `target_cursor_char` is still out of viewport, then we still need to move viewport to
    // left.
    let mut target_cursor_width = buffer.width_before(target_cursor_line, target_cursor_char);

    // For empty eol, sub extra 1 column.
    let target_is_empty_eol = buffer.is_empty_eol(target_cursor_line, target_cursor_char);
    if target_is_empty_eol {
      target_cursor_width = target_cursor_width.saturating_sub(1);
    }

    if target_cursor_width < start_column {
      on_left_side = true;
      start_column = target_cursor_width;
    }

    if on_left_side {
      return Some(start_column);
    }

    None
  }

  fn to_right_1(
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    let height = window_actual_shape.height();
    let width = window_actual_shape.width();

    let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
      buffer,
      target_viewport_start_column,
      target_cursor_line,
      0_u16,
      height,
      width,
    );

    debug_assert!(preview_target_rows.last().is_some());
    let (_last_row_idx, last_row_viewport) = preview_target_rows.last().unwrap();

    let on_right_side = last_row_viewport.end_char_idx() > last_row_viewport.start_char_idx()
      && target_cursor_char >= last_row_viewport.end_char_idx();

    if on_right_side {
      let start_column = reverse_search_start_column(
        proc_fn,
        buffer,
        window_actual_shape,
        target_cursor_line,
        target_cursor_char,
      );
      Some(start_column)
    } else {
      None
    }
  }

  // For case-1: cannot fully contains target cursor line.
  pub fn adjust_wrap_1(
    opts: detail::AdjustOptions,
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
    start_line: usize,
    start_column: usize,
  ) -> (usize, usize) {
    debug_assert!(!(opts.no_leftward && opts.no_rightward));

    if opts.no_leftward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_left_1(
            proc_fn,
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_left_side = to_left_1(
        proc_fn,
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_left) = start_column_on_left_side {
        return (start_line, start_column_left);
      }
    }

    if opts.no_rightward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_right_1(
            proc_fn,
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_right_side = to_right_1(
        proc_fn,
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_right) = start_column_on_right_side {
        return (start_line, start_column_right);
      }
    }

    (start_line, start_column)
  }

  fn to_left_2_1(
    _proc_fn: ProcessLineFn,
    buffer: &Buffer,
    _window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    if cfg!(debug_assertions) {
      match buffer.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }

      let target_cursor_width = buffer.width_before(target_cursor_line, target_cursor_char);
      debug_assert_eq!(target_viewport_start_column, 0_usize);
      let on_left_side = target_cursor_width < target_viewport_start_column;
      debug_assert!(!on_left_side);
    }

    None
  }

  fn to_right_2_1(
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    debug_assert_eq!(target_viewport_start_column, 0_usize);
    let height = window_actual_shape.height();
    let width = window_actual_shape.width();

    let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
      buffer,
      target_viewport_start_column,
      target_cursor_line,
      0_u16,
      height,
      width,
    );

    debug_assert!(preview_target_rows.last().is_some());
    let (_last_row_idx, last_row_viewport) = preview_target_rows.last().unwrap();

    let on_right_side = last_row_viewport.end_char_idx() > last_row_viewport.start_char_idx()
      && target_cursor_char >= last_row_viewport.end_char_idx();

    if on_right_side {
      // The `on_right_side=true` happens only when `target_cursor_char` is the empty eol, and the
      // `target_cursor_char` is out of viewport.
      debug_assert!(buffer.is_empty_eol(target_cursor_line, target_cursor_char));
      let start_column = reverse_search_start_column(
        proc_fn,
        buffer,
        window_actual_shape,
        target_cursor_line,
        target_cursor_char,
      );
      Some(start_column)
    } else {
      None
    }
  }

  // For case-2.1: only contains target cursor line.
  pub fn adjust_wrap_2_1(
    opts: detail::AdjustOptions,
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
    start_line: usize,
    start_column: usize,
  ) -> (usize, usize) {
    debug_assert!(!(opts.no_leftward && opts.no_rightward));

    if opts.no_leftward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_left_2_1(
            proc_fn,
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_left_side = to_left_2_1(
        proc_fn,
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_left) = start_column_on_left_side {
        return (start_line, start_column_left);
      }
    }

    if opts.no_rightward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_right_2_1(
            proc_fn,
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_right_side = to_right_2_1(
        proc_fn,
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_right) = start_column_on_right_side {
        return (start_line, start_column_right);
      }
    }

    (start_line, start_column)
  }

  fn to_left_2_2(
    _proc_fn: ProcessLineFn,
    buffer: &Buffer,
    _window_actual_shape: &U16Rect,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    if cfg!(debug_assertions) {
      match buffer.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          buffer
            .get_rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }

      let target_cursor_width = buffer.width_before(target_cursor_line, target_cursor_char);
      debug_assert_eq!(target_viewport_start_column, 0_usize);
      let on_left_side = target_cursor_width < target_viewport_start_column;
      debug_assert!(!on_left_side);
    }

    None
  }

  fn to_right_2_2(
    proc_fn: ProcessLineFn,
    lines_viewport: &LiteMap<usize, LineViewport>,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_viewport_start_line: usize,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<(usize, usize)> {
    debug_assert_eq!(target_viewport_start_column, 0_usize);

    let height = window_actual_shape.height();
    let width = window_actual_shape.width();

    debug_assert!(lines_viewport.contains_key(&target_cursor_line));
    let current_target_rows = lines_viewport.get(&target_cursor_line).unwrap().rows();
    debug_assert!(current_target_rows.last().is_some());
    let (current_last_row_idx, current_last_row_viewport) = current_target_rows.last().unwrap();

    let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
      buffer,
      target_viewport_start_column,
      target_cursor_line,
      0_u16,
      height,
      width,
    );

    let fully_show = preview_target_rows.len() == current_target_rows.len();
    let is_empty_eol = buffer.is_empty_eol(target_cursor_line, target_cursor_char);
    let is_last_row = *current_last_row_idx == height.saturating_sub(1);
    let out_of_view = current_last_row_viewport.end_char_idx()
      > current_last_row_viewport.start_char_idx()
      && target_cursor_char >= current_last_row_viewport.end_char_idx();
    let on_right_side = fully_show && is_empty_eol && is_last_row && out_of_view;

    if on_right_side {
      // The `target_cursor_line` must not to be the 1st line in the viewport (because in
      // case-2.1, the viewport contains multiple lines and the empty eol of target cursor line is
      // out of viewport, it has to be at the bottom-right corner).
      debug_assert!(target_cursor_line > target_viewport_start_line);
      // Then we simply add 1 extra line to `start_line`, instead of gives 1 extra column to
      // `start_column` (compared other cases).
      Some((target_viewport_start_line + 1, target_viewport_start_column))
    } else {
      None
    }
  }

  // For case-2.2: contains multiple lines, include target cursor line.
  pub fn adjust_wrap_2_2(
    opts: detail::AdjustOptions,
    proc_fn: ProcessLineFn,
    lines_viewport: &LiteMap<usize, LineViewport>,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
    target_cursor_char: usize,
    start_line: usize,
    start_column: usize,
  ) -> (usize, usize) {
    debug_assert!(!(opts.no_leftward && opts.no_rightward));

    if opts.no_leftward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_left_2_2(
            proc_fn,
            buffer,
            window_actual_shape,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_left_side = to_left_2_2(
        proc_fn,
        buffer,
        window_actual_shape,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some(start_column_left) = start_column_on_left_side {
        return (start_line, start_column_left);
      }
    }

    if opts.no_rightward {
      if cfg!(debug_assertions) {
        debug_assert!(
          to_right_2_2(
            proc_fn,
            lines_viewport,
            buffer,
            window_actual_shape,
            start_line,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_line_column_on_right_side = to_right_2_2(
        proc_fn,
        lines_viewport,
        buffer,
        window_actual_shape,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some((start_line_right, start_column_right)) = start_line_column_on_right_side {
        return (start_line_right, start_column_right);
      }
    }

    (start_line, start_column)
  }

  pub fn reverse_search_start_line(
    proc_fn: ProcessLineFn,
    buffer: &Buffer,
    window_actual_shape: &U16Rect,
    target_cursor_line: usize,
  ) -> usize {
    let height = window_actual_shape.height();
    let width = window_actual_shape.width();

    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n < height as usize) && (current_line >= 0) {
      let (rows, _start_fills, _end_fills, _) =
        proc_fn(buffer, 0_usize, current_line as usize, 0_u16, height, width);
      n += rows.len();

      if current_line == 0 || n >= height as usize {
        break;
      }

      current_line -= 1;
    }

    if (current_line as usize) < target_cursor_line && n > (height as usize) {
      current_line as usize + 1
    } else {
      current_line as usize
    }
  }
}

// Search a new viewport anchor (`start_line`, `start_column`) downward, i.e. when cursor moves
// down, and possibly scrolling buffer if cursor reaches the window bottom.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_downward(
  viewport: &Viewport,
  opts: &ViewportOptions,
  buffer: &Buffer,
  shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must move downward.
  debug_assert!(target_cursor_line >= viewport.start_line_idx());

  let buffer_len_lines = buffer.get_rope().len_lines();
  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_downward_nowrap(
      viewport,
      buffer,
      shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_downward_wrap(
      sync_wrap_nolinebreak,
      proc_line_wrap_nolinebreak,
      viewport,
      buffer,
      shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_downward_wrap(
      sync_wrap_linebreak,
      proc_line_wrap_linebreak,
      viewport,
      buffer,
      shape,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_downward_nowrap(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  debug_assert!(viewport.lines().last().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last().unwrap();

  let start_line = if target_cursor_line <= last_line {
    // Target cursor line is still inside current viewport.
    // Still use the old viewport start line.
    viewport_start_line
  } else {
    // Target cursor line goes out of current viewport, i.e. we will have to scroll viewport down
    // to show the target cursor.

    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n + 1 < height as usize) && (current_line >= 0) {
      let current_row = 0_u16;
      let (rows, _start_fills, _end_fills, _) = proc_line_nowrap(
        buffer,
        viewport_start_column,
        current_line as usize,
        current_row,
        height,
        width,
      );
      n += rows.len();

      if current_line == 0 {
        break;
      }

      current_line -= 1;
    }

    current_line as usize
  };

  nowrap_detail::adjust_nowrap(
    detail::AdjustOptions::all(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    viewport.start_column_idx(),
  )
}

// NOTE: For `wrap=true` algorithm, we split it into several cases:
// 1. The viewport cannot fully contain the target cursor line, i.e. the line is too long and
//    have to be truncated to place in the viewport.
// 2. The viewport can contain the target cursor line, i.e. the line is not too long. And further
//    we can split this into more sub cases:
//    2.1 The viewport only contains the target cursor line. And we have a very specific edge
//      case when considering the empty eol:
//        a) The last visible char of target cursor line is at the bottom-right corner of the
//        viewport, and thus the empty eol is actually out of viewport.
//        b) Otherwise the empty eol of target cursor line is not out of viewport.
//    2.2 The viewport not only contains the target cursor line, i.e. it contains at least 2
//      lines. And we have a very specific edge case for empty eol:
//        a) The target cursor line is the last line in viewport, and its last visible char is at
//        the bottom-right corner, and thus the empty eol is out of viewport.
//        b) Otherwise the empty eol of target cursor line is not out of viewport.
fn search_anchor_downward_wrap(
  sync_fn: wrap_detail::SyncFn,
  proc_fn: wrap_detail::ProcessLineFn,
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
    buffer,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line = preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line = preview_target_rows.len() == height as usize;

  if cannot_fully_contains_target_cursor_line {
    // Case-1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // For `start_column`, still use old `viewport_start_column` and wait to be adjusted.
    let start_line = target_cursor_line;
    let start_column = viewport_start_column;
    wrap_detail::adjust_wrap_1(
      detail::AdjustOptions::all(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else if only_contains_target_cursor_line {
    // Case-2.1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // Force `start_column` to be 0, because viewport can contains this line.
    let start_line = target_cursor_line;
    let start_column = 0_usize;
    wrap_detail::adjust_wrap_2_1(
      detail::AdjustOptions::all(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else {
    // Case-2.2
    // For `start_line`, first try to put `target_cursor_line` as the last line in the viewport and
    // locate the 1st line (by reversely searching each line from bottom to top), then compare the
    // 1st line with current `start_line`, choose the bigger one.
    // Force `start_column` to be 0, because viewport can contains the line.
    let start_line = wrap_detail::reverse_search_start_line(
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
    );
    let start_line = std::cmp::max(start_line, viewport_start_line);
    let start_column = 0_usize;
    let (_new_line_range, new_lines_viewport) =
      sync_fn(buffer, window_actual_shape, start_line, start_column);
    wrap_detail::adjust_wrap_2_2(
      detail::AdjustOptions::all(),
      proc_fn,
      &new_lines_viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  }
}

// Search a new viewport anchor (`start_line`, `start_column`) upward, i.e. when cursor moves up,
// and possibly scrolling buffer if cursor reaches the window top.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_upward(
  viewport: &Viewport,
  opts: &ViewportOptions,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must move upward.
  debug_assert!(target_cursor_line < viewport.end_line_idx());

  let buffer_len_lines = buffer.get_rope().len_lines();
  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_upward_nowrap(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_upward_wrap(
      sync_wrap_nolinebreak,
      proc_line_wrap_nolinebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_upward_wrap(
      sync_wrap_linebreak,
      proc_line_wrap_linebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_upward_nowrap(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let _viewport_start_column = viewport.start_column_idx();

  debug_assert!(viewport.lines().first().is_some());
  let (&first_line, _first_line_viewport) = viewport.lines().first().unwrap();

  let start_line = if target_cursor_line >= first_line {
    // Target cursor line is still inside current viewport.
    // Still use the old viewport start line.
    viewport_start_line
  } else {
    // Target cursor line goes out of current viewport, i.e. we will have to scroll viewport up
    // to show the target cursor.

    target_cursor_line
  };

  nowrap_detail::adjust_nowrap(
    detail::AdjustOptions::all(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    viewport.start_column_idx(),
  )
}

fn search_anchor_upward_wrap(
  sync_fn: wrap_detail::SyncFn,
  proc_fn: wrap_detail::ProcessLineFn,
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
    buffer,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line = preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line = preview_target_rows.len() == height as usize;

  if cannot_fully_contains_target_cursor_line {
    // Case-1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // For `start_column`, still use old `viewport_start_column` and wait to be adjusted.
    let start_line = target_cursor_line;
    let start_column = viewport_start_column;
    wrap_detail::adjust_wrap_1(
      detail::AdjustOptions::all(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else if only_contains_target_cursor_line {
    // Case-2.1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // Force `start_column` to be 0, because viewport can contains this line.
    let start_line = target_cursor_line;
    let start_column = 0_usize;
    wrap_detail::adjust_wrap_2_1(
      detail::AdjustOptions::all(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else {
    // Case-2.2
    // For `start_line`, simply force it to be `target_cursor_line` because we are moving viewport
    // to upper, thus the `target_cursor_line` must be the 1st line in viewport.
    // Force `start_column` to be 0, because viewport can contains the line.
    let start_line = std::cmp::min(target_cursor_line, viewport_start_line);
    let start_column = 0_usize;
    let (_new_line_range, new_lines_viewport) =
      sync_fn(buffer, window_actual_shape, start_line, start_column);
    wrap_detail::adjust_wrap_2_2(
      detail::AdjustOptions::all(),
      proc_fn,
      &new_lines_viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  }
}

// Search a new viewport anchor (`start_line`, `start_column`) leftward, i.e. when cursor moves
// left, and possibly scrolling buffer if cursor reaches the window left border.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_leftward(
  viewport: &Viewport,
  opts: &ViewportOptions,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must stay in viewport.
  debug_assert!(
    target_cursor_line >= viewport.start_line_idx() && target_cursor_line < viewport.end_line_idx()
  );

  let buffer_len_lines = buffer.get_rope().len_lines();
  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_leftward_nowrap(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_leftward_wrap(
      proc_line_wrap_nolinebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_leftward_wrap(
      proc_line_wrap_linebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_leftward_nowrap(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // adjust horizontally
  let start_line = viewport.start_line_idx();
  let start_column = viewport.start_column_idx();

  nowrap_detail::adjust_nowrap(
    detail::AdjustOptions::no_rightward(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

fn search_anchor_leftward_wrap(
  proc_fn: wrap_detail::ProcessLineFn,
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
    buffer,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line = preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line = preview_target_rows.len() == height as usize;

  if cannot_fully_contains_target_cursor_line {
    // Case-1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // For `start_column`, still use old `viewport_start_column` and wait to be adjusted.
    let start_line = target_cursor_line;
    let start_column = viewport_start_column;
    wrap_detail::adjust_wrap_1(
      detail::AdjustOptions::no_rightward(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else if only_contains_target_cursor_line {
    // Case-2.1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // Force `start_column` to be 0, because viewport can contains this line.
    let start_line = target_cursor_line;
    let start_column = 0_usize;
    wrap_detail::adjust_wrap_2_1(
      detail::AdjustOptions::no_rightward(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else {
    // Case-2.2
    // For `start_line`, simply force it to be the old `viewport_start_line` because we are not
    // going to move viewport upward/downward (only leftward/rightward). Thus the value won't
    // change.
    // Force `start_column` to be 0, because viewport can contains the line.
    let start_line = viewport_start_line;
    let start_column = 0_usize;
    wrap_detail::adjust_wrap_2_2(
      detail::AdjustOptions::no_rightward(),
      proc_fn,
      viewport.lines(),
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  }
}

// Search a new viewport anchor (`start_line`, `start_column`) rightward, i.e. when cursor moves
// left, and possibly scrolling buffer if cursor reaches the window left border.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_rightward(
  viewport: &Viewport,
  opts: &ViewportOptions,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must stay in viewport.
  debug_assert!(
    target_cursor_line >= viewport.start_line_idx() && target_cursor_line < viewport.end_line_idx()
  );

  let buffer_len_lines = buffer.get_rope().len_lines();
  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_rightward_nowrap(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_rightward_wrap(
      proc_line_wrap_nolinebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_rightward_wrap(
      proc_line_wrap_linebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_rightward_nowrap(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // adjust horizontally
  let start_line = viewport.start_line_idx();
  let start_column = viewport.start_column_idx();

  nowrap_detail::adjust_nowrap(
    detail::AdjustOptions::no_leftward(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

fn search_anchor_rightward_wrap(
  proc_fn: wrap_detail::ProcessLineFn,
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (preview_target_rows, _preview_target_start_fills, _preview_target_end_fills, _) = proc_fn(
    buffer,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line = preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line = preview_target_rows.len() == height as usize;

  if cannot_fully_contains_target_cursor_line {
    // Case-1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // For `start_column`, still use old `viewport_start_column` and wait to be adjusted.
    let start_line = target_cursor_line;
    let start_column = viewport_start_column;
    wrap_detail::adjust_wrap_1(
      detail::AdjustOptions::no_leftward(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else if only_contains_target_cursor_line {
    // Case-2.1
    // For `start_line`, force it to be `target_cursor_line`, because viewport only contains this
    // line.
    // Force `start_column` to be 0, because viewport can contains this line.
    let start_line = target_cursor_line;
    let start_column = 0_usize;
    wrap_detail::adjust_wrap_2_1(
      detail::AdjustOptions::no_leftward(),
      proc_fn,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  } else {
    // Case-2.2
    // For `start_line`, simply force it to be the old `viewport_start_line` because we are not
    // going to move viewport upward/downward (only leftward/rightward). Thus the value won't
    // change.
    // Force `start_column` to be 0, because viewport can contains the line.
    let start_line = viewport_start_line;
    let start_column = 0_usize;
    wrap_detail::adjust_wrap_2_2(
      detail::AdjustOptions::no_leftward(),
      proc_fn,
      viewport.lines(),
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  }
}
