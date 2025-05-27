//! Internal implementations for Viewport.

#![allow(clippy::too_many_arguments)]

use crate::buf::Buffer;
use crate::prelude::*;
use crate::ui::widget::window::viewport::RowViewport;
use crate::ui::widget::window::{LineViewport, WindowLocalOptions};

use derive_builder::Builder;
use ropey::RopeSlice;
use std::collections::BTreeMap;
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
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  window_local_options: &WindowLocalOptions,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  // If window is zero-sized.
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  if height == 0 || width == 0 {
    return (ViewportLineRange::default(), BTreeMap::new());
  }

  match (
    window_local_options.wrap(),
    window_local_options.line_break(),
  ) {
    (false, _) => sync_nowrap(buffer, window_actual_shape, start_line, start_column),
    (true, false) => sync_wrap_nolinebreak(buffer, window_actual_shape, start_line, start_column),
    (true, true) => sync_wrap_linebreak(buffer, window_actual_shape, start_line, start_column),
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
    let next_to_last_visible_char = buffer.last_char_on_line_no_eol(l).unwrap() + 1;

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
) -> (BTreeMap<u16, RowViewport>, usize, usize, u16) {
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

  let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
  rows.insert(current_row, RowViewport::new(start_char..end_char));
  (rows, start_fills, end_fills, current_row)
}

/// Implements [`sync`] with option `wrap=false`.
fn sync_nowrap(
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

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
    (ViewportLineRange::default(), BTreeMap::new())
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
) -> (BTreeMap<u16, RowViewport>, usize, usize, u16) {
  let bufline = buffer.get_rope().line(current_line);
  let bufline_len_chars = bufline.len_chars();

  if bufline_len_chars == 0 {
    let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
    rows.insert(current_row, RowViewport::new(0..0));
    (rows, 0_usize, 0_usize, current_row)
  } else {
    let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();

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
          if end_char > buffer.last_char_on_line_no_eol(current_line).unwrap() {
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
  window_actual_shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

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
    (ViewportLineRange::default(), BTreeMap::new())
  }
}

/// Find the word index by the char index.
///
/// Returns the word index which contains this char, and whether the char is the last char in the
/// word.
fn _find_word_by_char(
  words: &[&str],
  word_end_chars_index: &HashMap<usize, usize>,
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

#[allow(clippy::too_many_arguments)]
/// Part-1 of the processing algorithm in `_from_top_left_wrap_linebreak`.
fn _part1(
  words: &[&str],
  words_end_char_idx: &HashMap<usize, usize>,
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

fn _cloned_line_max_len(window_height: u16, window_width: u16, start_column: usize) -> usize {
  window_height as usize * window_width as usize * 2 + 16 + start_column
}

/// Returns `rows`, `start_fills`, `end_fills`, `current_row`.
fn proc_line_wrap_linebreak(
  buffer: &Buffer,
  start_column: usize,
  current_line: usize,
  mut current_row: u16,
  window_height: u16,
  window_width: u16,
) -> (BTreeMap<u16, RowViewport>, usize, usize, u16) {
  let bufline = buffer.get_rope().line(current_line);
  if bufline.len_chars() == 0 {
    let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
    rows.insert(current_row, RowViewport::new(0..0));
    (rows, 0_usize, 0_usize, current_row)
  } else {
    let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();

    // Here clone the line with the max chars that can hold by current window/viewport,
    // i.e. the `height * width` cells count as the max chars in the line. This helps avoid
    // performance issue when iterating on super long lines.
    let cloned_line = buffer
      .clone_line(
        current_line,
        0,
        _cloned_line_max_len(window_height, window_width, start_column),
      )
      .unwrap();

    // Words.
    let words: Vec<&str> = cloned_line.split_word_bounds().collect();
    // Word index => its end char index (from the first char until current word).
    let words_end_char_idx = words
      .iter()
      .enumerate()
      .scan(0_usize, |state, (i, wd)| {
        *state += wd.chars().count();
        Some((i, *state))
      })
      .collect::<HashMap<usize, usize>>();

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
          if end_char > buffer.last_char_on_line_no_eol(current_line).unwrap() {
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
  window_actual_shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

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
    (ViewportLineRange::default(), BTreeMap::new())
  }
}

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
// spellchecker:on
//
// Returns
// 1. If target cursor is on the left side of viewport, and we need to move the viewport more to
//    the left side.
// 2. If 1st is true, this is the new "start_column" after adjustments.
fn _adjust_left_nowrap(
  buffer: &Buffer,
  _window_actual_shape: &U16Rect,
  target_viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> Option<usize> {
  let target_cursor_width = buffer.width_before(target_cursor_line, target_cursor_char);
  let on_left_side = target_cursor_width < target_viewport_start_column;

  if cfg!(debug_assertions) {
    match buffer.char_at(target_cursor_line, target_viewport_start_column) {
      Some(target_viewport_start_char) => trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),target_cursor_width:{},viewport_start_column:{},viewport_start_char:{}({:?})",
        target_cursor_line,
        target_cursor_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        target_cursor_width,
        target_viewport_start_column,
        target_viewport_start_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_viewport_start_char)
          .unwrap_or('?')
      ),
      None => trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),target_cursor_width:{},viewport_start_column:{},viewport_start_char:None",
        target_cursor_line,
        target_cursor_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        target_cursor_width,
        target_viewport_start_column,
      ),
    }
  }

  if on_left_side {
    // We need to move viewport to left to show the cursor, to minimize the viewport adjustments,
    // just put the cursor at the first left char in the new viewport.
    let start_column = buffer.width_before(target_cursor_line, target_cursor_char);
    Some(start_column)
  } else {
    None
  }
}

// Returns
// 1. If target cursor is on the right side of viewport, and we need to adjust/move the viewport to
//    right.
// 2. If 1st is true, this is the new "start_column" after adjustments.
fn _adjust_right_nowrap(
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> Option<usize> {
  let width = window_actual_shape.width();
  let viewport_end_column = target_viewport_start_column + width as usize;
  let target_cursor_width = buffer.width_until(target_cursor_line, target_cursor_char);
  let on_right_side = target_cursor_width > viewport_end_column;

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
      "target_cursor_line:{},target_cursor_char:{}({:?}),target_cursor_width:{},viewport_start_column:{},viewport_start_char:{},viewport_end_column:{},viewport_end_char:{}",
      target_cursor_line,
      target_cursor_char,
      buffer
        .get_rope()
        .line(target_cursor_line)
        .get_char(target_cursor_char)
        .unwrap_or('?'),
      target_cursor_width,
      target_viewport_start_column,
      target_viewport_start_char,
      viewport_end_column,
      viewport_end_char,
    );
  }

  if on_right_side {
    // Move viewport to right to show the cursor, just put the cursor at the last right char in the
    // new viewport.
    let end_column = buffer.width_until(target_cursor_line, target_cursor_char);
    let start_column = end_column.saturating_sub(width as usize);
    Some(start_column)
  } else {
    None
  }
}

#[derive(Debug, Copy, Clone, Builder)]
struct AdjustOptions {
  #[builder(default = false)]
  pub disable_detect_leftward: bool,

  #[builder(default = false)]
  pub disable_detect_rightward: bool,
}

fn _adjust_horizontally_nowrap(
  opts: AdjustOptions,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
  start_line: usize,
  start_column: usize,
) -> (usize, usize) {
  debug_assert!(!(opts.disable_detect_leftward && opts.disable_detect_rightward));

  if opts.disable_detect_leftward {
    if cfg!(debug_assertions) {
      debug_assert!(
        _adjust_left_nowrap(
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
    let start_column_on_left_side = _adjust_left_nowrap(
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

  if opts.disable_detect_rightward {
    if cfg!(debug_assertions) {
      debug_assert!(
        _adjust_right_nowrap(
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
    let start_column_on_right_side = _adjust_right_nowrap(
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

fn _line_head_not_show(viewport: &Viewport, line_idx: usize) -> bool {
  if viewport.start_line_idx() > line_idx || viewport.end_line_idx() <= line_idx {
    return false;
  }
  debug_assert!(viewport.lines().contains_key(&line_idx));
  let line_viewport = viewport.lines().get(&line_idx).unwrap();
  let rows = line_viewport.rows();
  debug_assert!(rows.first_key_value().is_some());
  let (_first_row_idx, first_row_viewport) = rows.first_key_value().unwrap();
  first_row_viewport.start_char_idx() > 0
}

fn _line_tail_not_show(viewport: &Viewport, buffer: &Buffer, line_idx: usize) -> bool {
  if viewport.start_line_idx() > line_idx || viewport.end_line_idx() <= line_idx {
    return false;
  }

  debug_assert!(viewport.lines().contains_key(&line_idx));
  debug_assert!(buffer.get_rope().get_line(line_idx).is_some());
  let bufline_last_visible_char = buffer.last_char_on_line_no_eol(line_idx).unwrap_or(0_usize);

  let line_viewport = viewport.lines().get(&line_idx).unwrap();
  let rows = line_viewport.rows();
  debug_assert!(rows.last_key_value().is_some());
  let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();
  last_row_viewport.end_char_idx().saturating_sub(1) < bufline_last_visible_char
}

// Type alias for `proc_line_*` functions.
type ProcessLineFn = fn(
  /* buffer */ &Buffer,
  /* start_column */ usize,
  /* current_line */ usize,
  /* mut current_row */ u16,
  /* window_height */ u16,
  /* window_width */ u16,
) -> (
  /* rows */ BTreeMap<u16, RowViewport>,
  /* start_fills */ usize,
  /* end_fills */ usize,
  /* next_current_row */ u16,
);

// For `wrap=true`.
fn _find_start_char_wrap(
  proc: ProcessLineFn,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  line_idx: usize,
  last_char: usize,
  mut start_column: usize,
) -> usize {
  let bufline = buffer.get_rope().line(line_idx);
  let bufline_len_char = bufline.len_chars();
  let bufline_chars_width = buffer.width_until(line_idx, bufline_len_char);

  while start_column < bufline_chars_width {
    let (rows, _start_fills, _end_fills, _) = proc(
      buffer,
      start_column,
      line_idx,
      0_u16,
      window_actual_shape.height(),
      window_actual_shape.width(),
    );
    let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();
    if last_row_viewport.end_char_idx() > last_char {
      return start_column;
    }
    start_column += 1;
  }

  unreachable!()
}

// For `wrap=true,linebreak=false`, when the whole viewport only contains 1 line and the line
// cannot fully show (i.e. the line head/tail are been truncated), and also we have confirmed the
// last char index.
// In such case, we needs to calculate the `start_column`.
fn _revert_search_start_column_wrap(
  proc: ProcessLineFn,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  line_idx: usize,
  last_char: usize,
) -> usize {
  let last_char_width = buffer.width_until(line_idx, last_char);
  let approximate_start_column = last_char_width.saturating_sub(
    (window_actual_shape.height() as usize) * (window_actual_shape.width() as usize),
  );
  _find_start_char_wrap(
    proc,
    buffer,
    window_actual_shape,
    line_idx,
    last_char,
    approximate_start_column,
  )
}

fn _adjust_left_wrap(
  proc: ProcessLineFn,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  cannot_fully_contains_target_cursor_line: bool,
  target_viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> Option<usize> {
  let mut start_column = target_viewport_start_column;
  let target_cursor_width = buffer.width_before(target_cursor_line, target_cursor_char);

  if cfg!(debug_assertions) {
    match buffer.char_at(target_cursor_line, target_viewport_start_column) {
      Some(target_viewport_start_char) => trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),target_cursor_width:{},viewport_start_column:{},viewport_start_char:{}({:?})",
        target_cursor_line,
        target_cursor_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        target_cursor_width,
        target_viewport_start_column,
        target_viewport_start_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_viewport_start_char)
          .unwrap_or('?')
      ),
      None => trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),target_cursor_width:{},viewport_start_column:{},viewport_start_char:None",
        target_cursor_line,
        target_cursor_char,
        buffer
          .get_rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        target_cursor_width,
        target_viewport_start_column,
      ),
    }
  }

  let mut on_left_side = target_cursor_width < start_column;

  if on_left_side {
    // We need to move viewport to left to show the cursor, to minimize the viewport adjustments,
    // just put the cursor at the first left char in the new viewport.
    start_column = buffer.width_before(target_cursor_line, target_cursor_char);
  }

  // spellchecker:off
  // If `target_cursor_line` doesn't show its head (i.e. the `target_viewport_start_column` > 0,
  // and the viewport only contains 1 line, and the line is just too lone to fully show), and the
  // `target_cursor_line`'s end char is not at the bottom-right corner of the viewport. For
  // example:
  //
  // ```text
  //                                           |----------------------------------|
  // This is the beginning of the very long lin|e, which only shows the beginning |
  //                                           |part.                             |
  //                                           |                                  |
  //                                           |----------------------------------|
  // ```
  //
  // Apparently we can move the `target_viewport_start_column` more to left, thus the
  // `target_cursor_line` can be put in this way:
  //
  // ```text
  // |----------------------------------|
  // |This is the beginning of the very |
  // |long lin|e, which only shows the b|
  // |eginning part.                    |
  // |----------------------------------|
  // ```
  //
  // Which is much better for `wrap=true`.
  // spellchecker:on

  let (target_cursor_rows, _target_cursor_start_fills, _target_cursor_end_fills, _) = proc(
    buffer,
    start_column,
    target_cursor_line,
    0_u16,
    window_actual_shape.height(),
    window_actual_shape.width(),
  );
  if cannot_fully_contains_target_cursor_line
    && target_cursor_rows.len() < window_actual_shape.height() as usize
  {
    let last_visible_char = buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize);
    let start_column_included_last_visible_char = _revert_search_start_column_wrap(
      proc,
      buffer,
      window_actual_shape,
      target_cursor_line,
      last_visible_char,
    );
    if start_column > start_column_included_last_visible_char {
      start_column = start_column_included_last_visible_char;
      on_left_side = true;
    }
  }

  if on_left_side {
    Some(start_column)
  } else {
    None
  }
}

fn _adjust_right_wrap(
  proc: ProcessLineFn,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  _cannot_fully_contains_target_cursor_line: bool,
  target_viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> Option<usize> {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (rows, _start_fills, _end_fills, _) = proc(
    buffer,
    target_viewport_start_column,
    target_cursor_line,
    0_u16,
    height,
    width,
  );

  debug_assert!(rows.last_key_value().is_some());
  let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();

  // NOTE: If out of viewport, the viewport must only contains 1 line.
  let out_of_viewport = last_row_viewport.end_char_idx() > last_row_viewport.start_char_idx()
    && target_cursor_char >= last_row_viewport.end_char_idx();

  if out_of_viewport {
    let start_column = _revert_search_start_column_wrap(
      proc,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    );
    return Some(start_column);
  }

  let last_row_width = if last_row_viewport.end_char_idx() > last_row_viewport.start_char_idx() {
    0_usize
  } else {
    let last_start_column =
      buffer.width_before(target_cursor_line, last_row_viewport.start_char_idx());
    buffer
      .width_until(target_cursor_line, target_cursor_char)
      .saturating_sub(last_start_column)
  };
  let eol_at_viewport_end =
    _target_cursor_is_at_eol(buffer, target_cursor_line, target_cursor_char)
      && last_row_width == width as usize;

  if eol_at_viewport_end {
    let start_column = _revert_search_start_column_wrap(
      proc,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    );
    return Some(start_column);
  }

  None
}

fn _adjust_horizontally_wrap(
  opts: AdjustOptions,
  proc: ProcessLineFn,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  cannot_fully_contains_target_cursor_line: bool,
  target_cursor_line: usize,
  target_cursor_char: usize,
  start_line: usize,
  start_column: usize,
) -> (usize, usize) {
  debug_assert!(!(opts.disable_detect_leftward && opts.disable_detect_rightward));

  if opts.disable_detect_leftward {
    if cfg!(debug_assertions) {
      debug_assert!(
        _adjust_left_wrap(
          proc,
          buffer,
          window_actual_shape,
          cannot_fully_contains_target_cursor_line,
          start_column,
          target_cursor_line,
          target_cursor_char,
        )
        .is_none()
      );
    }
  } else {
    let start_column_on_left_side = _adjust_left_wrap(
      proc,
      buffer,
      window_actual_shape,
      cannot_fully_contains_target_cursor_line,
      start_column,
      target_cursor_line,
      target_cursor_char,
    );

    if let Some(start_column_left) = start_column_on_left_side {
      return (start_line, start_column_left);
    }
  }

  if opts.disable_detect_rightward {
    if cfg!(debug_assertions) {
      debug_assert!(
        _adjust_right_wrap(
          proc,
          buffer,
          window_actual_shape,
          cannot_fully_contains_target_cursor_line,
          start_column,
          target_cursor_line,
          target_cursor_char,
        )
        .is_none()
      );
    }
  } else {
    let start_column_on_right_side = _adjust_right_wrap(
      proc,
      buffer,
      window_actual_shape,
      cannot_fully_contains_target_cursor_line,
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

fn _adjust_current_line(
  current_line: isize,
  target_cursor_line: usize,
  window_height: u16,
  n: usize,
) -> usize {
  if (current_line as usize) < target_cursor_line && n > (window_height as usize) {
    current_line as usize + 1
  } else {
    current_line as usize
  }
}

// Search a new viewport anchor (`start_line`, `start_column`) downward, i.e. when cursor moves
// down, and possibly scrolling buffer if cursor reaches the window bottom.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_downward(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  window_local_options: &WindowLocalOptions,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  match (
    window_local_options.wrap(),
    window_local_options.line_break(),
  ) {
    (false, _) => search_anchor_downward_nowrap(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_downward_wrap(
      proc_line_wrap_nolinebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_downward_wrap(
      proc_line_wrap_linebreak,
      viewport,
      buffer,
      window_actual_shape,
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().last_key_value().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last_key_value().unwrap();

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

  _adjust_horizontally_nowrap(
    AdjustOptionsBuilder::default().build().unwrap(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    viewport.start_column_idx(),
  )
}

fn search_anchor_downward_wrap(
  proc: ProcessLineFn,
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().last_key_value().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last_key_value().unwrap();

  // NOTE: For `wrap=true`, if a line's head/tail not fully rendered, it means there will be only 1
  // line shows in current window viewport. Because the `wrap` will force the 2nd line wait to show
  // until the **current** line get fully rendered.

  let target_cursor_line_not_fully_show = _line_head_not_show(viewport, target_cursor_line)
    || _line_tail_not_show(viewport, buffer, target_cursor_line);

  let (start_line, start_column, cannot_fully_contains_target_cursor_line) =
    if target_cursor_line <= last_line && !target_cursor_line_not_fully_show {
      (viewport_start_line, viewport_start_column, false)
    } else {
      // Try to fill the viewport with `start_column=0`, and we can know how many rows the
      // `target_cursor_line` needs to fill into current viewport.
      let (target_cursor_rows, _target_cursor_start_fills, _target_cursor_end_fills, _) = proc(
        buffer,
        0,
        target_cursor_line,
        0_u16,
        height.saturating_add(10),
        width,
      );

      // 1. If the `target_cursor_line` can fully show in current viewport, then we force the
      // `start_column` to 0.
      //
      // 2. Otherwise it means the current viewport can only contains 1 line, i.e. the
      // `target_cursor_line`, and it is still possible to add some `start_column` if the line is too
      // long. And in such case, the `start_line` will always be the `target_cursor_line` because
      // the line is too big to show in current viewport, and we don't need other lines.
      let cannot_fully_contains_target_cursor_line = target_cursor_rows.len() > height as usize;
      let (start_line, start_column) = if !cannot_fully_contains_target_cursor_line {
        let mut n = 0_usize;
        let mut current_line = target_cursor_line as isize;

        while (n < height as usize) && (current_line >= 0) {
          let (rows, _start_fills, _end_fills, _) =
            proc(buffer, 0_usize, current_line as usize, 0_u16, height, width);
          n += rows.len();

          if current_line == 0 || n >= height as usize {
            break;
          }

          current_line -= 1;
        }

        (
          _adjust_current_line(current_line, target_cursor_line, height, n),
          0_usize,
        )
      } else {
        (target_cursor_line, viewport_start_column)
      };

      (
        start_line,
        start_column,
        cannot_fully_contains_target_cursor_line,
      )
    };

  _adjust_horizontally_wrap(
    AdjustOptionsBuilder::default().build().unwrap(),
    proc,
    buffer,
    window_actual_shape,
    cannot_fully_contains_target_cursor_line,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

// Search a new viewport anchor (`start_line`, `start_column`) upward, i.e. when cursor moves up,
// and possibly scrolling buffer if cursor reaches the window top.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_upward(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  window_local_options: &WindowLocalOptions,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  match (
    window_local_options.wrap(),
    window_local_options.line_break(),
  ) {
    (false, _) => search_anchor_upward_nowrap(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_upward_wrap(
      proc_line_wrap_nolinebreak,
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_upward_wrap(
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().first_key_value().is_some());
  let (&first_line, _first_line_viewport) = viewport.lines().first_key_value().unwrap();

  let start_line = if target_cursor_line >= first_line {
    // Target cursor line is still inside current viewport.
    // Still use the old viewport start line.
    viewport_start_line
  } else {
    // Target cursor line goes out of current viewport, i.e. we will have to scroll viewport up
    // to show the target cursor.

    target_cursor_line
  };

  _adjust_horizontally_nowrap(
    AdjustOptionsBuilder::default().build().unwrap(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    viewport.start_column_idx(),
  )
}

fn search_anchor_upward_wrap(
  proc: ProcessLineFn,
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().first_key_value().is_some());
  let (&first_line, _first_line_viewport) = viewport.lines().first_key_value().unwrap();

  let target_cursor_line_not_fully_show = _line_head_not_show(viewport, target_cursor_line)
    || _line_tail_not_show(viewport, buffer, target_cursor_line);

  let (start_line, start_column, cannot_fully_contains_target_cursor_line) =
    if target_cursor_line >= first_line && !target_cursor_line_not_fully_show {
      (viewport_start_line, viewport_start_column, false)
    } else {
      let (target_cursor_rows, _target_cursor_start_fills, _target_cursor_end_fills, _) = proc(
        buffer,
        0,
        target_cursor_line,
        0_u16,
        height.saturating_add(10),
        width,
      );

      let cannot_fully_contains_target_cursor_line = target_cursor_rows.len() > height as usize;
      let (start_line, start_column) = if !cannot_fully_contains_target_cursor_line {
        (target_cursor_line, 0_usize)
      } else {
        (target_cursor_line, viewport_start_column)
      };

      (
        start_line,
        start_column,
        cannot_fully_contains_target_cursor_line,
      )
    };

  _adjust_horizontally_wrap(
    AdjustOptionsBuilder::default().build().unwrap(),
    proc,
    buffer,
    window_actual_shape,
    cannot_fully_contains_target_cursor_line,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

// Search a new viewport anchor (`start_line`, `start_column`) leftward, i.e. when cursor moves
// left, and possibly scrolling buffer if cursor reaches the window left border.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_leftward(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  window_local_options: &WindowLocalOptions,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  match (
    window_local_options.wrap(),
    window_local_options.line_break(),
  ) {
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  // adjust horizontally
  let start_line = viewport.start_line_idx();
  let start_column = viewport.start_column_idx();

  _adjust_horizontally_nowrap(
    AdjustOptionsBuilder::default()
      .disable_detect_rightward(true)
      .build()
      .unwrap(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

fn search_anchor_leftward_wrap(
  proc: ProcessLineFn,
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().first_key_value().is_some());
  let (&first_line, _first_line_viewport) = viewport.lines().first_key_value().unwrap();

  let target_cursor_line_not_fully_show = _line_head_not_show(viewport, target_cursor_line)
    || _line_tail_not_show(viewport, buffer, target_cursor_line);

  let (start_column, cannot_fully_contains_target_cursor_line) =
    if target_cursor_line >= first_line && !target_cursor_line_not_fully_show {
      (viewport_start_column, false)
    } else {
      let (target_cursor_rows, _target_cursor_start_fills, _target_cursor_end_fills, _) = proc(
        buffer,
        0,
        target_cursor_line,
        0_u16,
        height.saturating_add(10),
        width,
      );
      let cannot_fully_contains_target_cursor_line = target_cursor_rows.len() > height as usize;
      let start_column = if !cannot_fully_contains_target_cursor_line {
        0_usize
      } else {
        viewport_start_column
      };

      (start_column, cannot_fully_contains_target_cursor_line)
    };

  _adjust_horizontally_wrap(
    AdjustOptionsBuilder::default()
      .disable_detect_rightward(true)
      .build()
      .unwrap(),
    proc,
    buffer,
    window_actual_shape,
    cannot_fully_contains_target_cursor_line,
    target_cursor_line,
    target_cursor_char,
    viewport_start_line,
    start_column,
  )
}

// Search a new viewport anchor (`start_line`, `start_column`) rightward, i.e. when cursor moves
// left, and possibly scrolling buffer if cursor reaches the window left border.
//
// Returns `start_line`, `start_column` for the new viewport.
pub fn search_anchor_rightward(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  window_local_options: &WindowLocalOptions,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  match (
    window_local_options.wrap(),
    window_local_options.line_break(),
  ) {
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  // adjust horizontally
  let start_line = viewport.start_line_idx();
  let start_column = viewport.start_column_idx();

  _adjust_horizontally_nowrap(
    AdjustOptionsBuilder::default()
      .disable_detect_leftward(true)
      .build()
      .unwrap(),
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

fn search_anchor_rightward_wrap(
  proc: ProcessLineFn,
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
  let buffer_len_lines = buffer.get_rope().len_lines();

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_char_on_line_no_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().first_key_value().is_some());
  let (&first_line, _first_line_viewport) = viewport.lines().first_key_value().unwrap();

  let target_cursor_line_not_fully_show = _line_head_not_show(viewport, target_cursor_line)
    || _line_tail_not_show(viewport, buffer, target_cursor_line);

  let (start_column, cannot_fully_contains_target_cursor_line) =
    if target_cursor_line >= first_line && !target_cursor_line_not_fully_show {
      (viewport_start_column, false)
    } else {
      let (target_cursor_rows, _target_cursor_start_fills, _target_cursor_end_fills, _) = proc(
        buffer,
        0,
        target_cursor_line,
        0_u16,
        height.saturating_add(10),
        width,
      );
      let cannot_fully_contains_target_cursor_line = target_cursor_rows.len() > height as usize;
      let start_column = if !cannot_fully_contains_target_cursor_line {
        0_usize
      } else {
        viewport_start_column
      };

      (start_column, cannot_fully_contains_target_cursor_line)
    };

  // adjust horizontally
  _adjust_horizontally_wrap(
    AdjustOptionsBuilder::default()
      .disable_detect_leftward(true)
      .build()
      .unwrap(),
    proc,
    buffer,
    window_actual_shape,
    cannot_fully_contains_target_cursor_line,
    target_cursor_line,
    target_cursor_char,
    viewport_start_line,
    start_column,
  )
}
