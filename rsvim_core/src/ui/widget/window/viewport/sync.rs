//! Internal implementations for Viewport.

use crate::buf::Buffer;
use crate::prelude::*;
use crate::ui::widget::window::viewport::RowViewport;
use crate::ui::widget::window::{LineViewport, WindowLocalOptions};

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

#[allow(dead_code)]
fn slice2line(s: &RopeSlice) -> String {
  let mut builder = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
}

fn end_char_and_prefills(
  buffer: &Buffer,
  bline: &RopeSlice,
  l: usize,
  c: usize,
  end_width: usize,
) -> (usize, usize) {
  let c_width = buffer.width_at(l, c);
  if c_width > end_width {
    // If the char `c` width is greater than `end_width`, the `c` itself is the end char.
    let c_width_before = buffer.width_before(l, c);
    (c, end_width.saturating_sub(c_width_before))
  } else {
    // Here we use the last visible char in the line, thus avoid those invisible chars like '\n'.
    debug_assert!(bline.len_chars() > 0);
    let next_to_last_visible_char = buffer.last_visible_char_on_line(l).unwrap() + 1;

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
          Some(c) => end_char_and_prefills(buffer, &bufline, current_line, c, end_width),
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
  // trace!("buffer.len_lines:{:?}", buffer_len_lines);

  debug_assert!(height > 0);
  debug_assert!(width > 0);

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
            Some(c) => end_char_and_prefills(buffer, &bufline, current_line, c, end_width),
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
          if end_char > buffer.last_visible_char_on_line(current_line).unwrap() {
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

#[allow(unused_variables)]
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

  debug_assert!(height > 0);
  debug_assert!(width > 0);

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
fn find_word_by_char(
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
fn part1(
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
  let (wd_idx, start_c_of_wd, end_c_of_wd) = find_word_by_char(words, words_end_char_idx, c);

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

      end_char_and_prefills(buffer, bline, l, start_c_of_wd - 1, end_width)
    } else {
      // Part-1.2, cut this word and force rendering it ignoring line-break behavior.
      debug_assert_eq!(start_c_of_wd, start_char);
      // Record the position (c) where we cut the words into pieces.
      *last_word_is_too_long = Some((wd_idx, start_c_of_wd, end_c_of_wd, c));

      // If the char `c` width is greater than `end_width`, the `c` itself is the end char.
      end_char_and_prefills(buffer, bline, l, c, end_width)
    }
  } else {
    debug_assert_eq!(c + 1, end_c_of_wd);
    // The current word is not long, it can be put in current row.
    let c_next = std::cmp::min(end_c_of_wd, bline.len_chars());
    (c_next, 0_usize)
  }
}

fn cloned_line_max_len(window_height: u16, window_width: u16, start_column: usize) -> usize {
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
        cloned_line_max_len(window_height, window_width, start_column),
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
                        end_char_and_prefills(buffer, &bufline, current_line, c, end_width)
                      } else {
                        // Part-2.2, the rest part of the word is not long.
                        // Thus we can go back to *normal* algorithm just like part-1.

                        part1(
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
                  part1(
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
          if end_char > buffer.last_visible_char_on_line(current_line).unwrap() {
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

#[allow(unused_variables)]
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
  // If window is zero-sized.
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  if height == 0 || width == 0 {
    return (0, 0);
  }

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
    (true, false) => search_anchor_downward_wrap_nolinebreak(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_downward_wrap_linebreak(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

// spellchecker:off
// When searching the new viewport downward, the target cursor could be not shown in it.
//
// For example:
//
// ```
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
// ```
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
// 1. If target cursor is on the left side of viewport, and we need to adjust/move the viewport to
//    left.
// 2. If 1st is true, this is the new "start_column" after adjustments.
fn left_downward_nowrap(
  buffer: &Buffer,
  _window_actual_shape: &U16Rect,
  _viewport_start_line: usize,
  viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (bool, usize) {
  // If target cursor char is on the left of the old target viewport.
  let on_left_side = match buffer.char_after(target_cursor_line, viewport_start_column) {
    Some(c) => {
      trace!(
        "target_cursor_line:{},target_cursor_char:{},viewport_start_line:{},viewport_start_column:{},c:{}",
        target_cursor_line, target_cursor_char, _viewport_start_line, viewport_start_column, c
      );
      c > target_cursor_char
    }
    None => true,
  };

  if on_left_side {
    // We need to move viewport to left to show the cursor, to minimize the viewport adjustments,
    // just put the cursor at the first left char in the new viewport.
    let start_column = buffer.width_before(target_cursor_line, target_cursor_char);
    (true, start_column)
  } else {
    (false, 0_usize)
  }
}

// Returns
// 1. If target cursor is on the right side of viewport, and we need to adjust/move the viewport to
//    right.
// 2. If 1st is true, this is the new "start_column" after adjustments.
fn right_downward_nowrap(
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  _viewport_start_line: usize,
  viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (bool, usize) {
  let width = window_actual_shape.width();
  let viewport_end_column = viewport_start_column + width as usize;

  // Target cursor line end.
  let on_right_side = match buffer.char_at(target_cursor_line, viewport_end_column) {
    Some(c) => {
      trace!(
        "target_cursor_line:{},target_cursor_char:{},viewport_start_line:{},viewport_start_column:{},c:{}",
        target_cursor_line, target_cursor_char, _viewport_start_line, viewport_start_column, c
      );
      c < target_cursor_char
    }
    None => false,
  };

  if on_right_side {
    // Move viewport to right to show the cursor, just put the cursor at the last right char in the
    // new viewport.
    let end_column = buffer.width_at(target_cursor_line, target_cursor_char);
    let start_column = end_column.saturating_sub(width as usize);
    (true, start_column)
  } else {
    (false, 0_usize)
  }
}

fn adjust_downward_horizontally_nowrap(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
  start_line: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();

  let (on_left_side, start_column_on_left_side) = left_downward_nowrap(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_left_side {
    return (start_line, start_column_on_left_side);
  }

  let (on_right_side, start_column_on_right_side) = right_downward_nowrap(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_right_side {
    return (start_line, start_column_on_right_side);
  }

  (start_line, viewport_start_column)
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

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_visible_char_on_line(target_cursor_line)
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

  adjust_downward_horizontally_nowrap(
    viewport,
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
  )
}

fn line_head_not_show(viewport: &Viewport, line_idx: usize) -> bool {
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

fn line_tail_not_show(viewport: &Viewport, buffer: &Buffer, line_idx: usize) -> bool {
  if viewport.start_line_idx() > line_idx || viewport.end_line_idx() <= line_idx {
    return false;
  }

  debug_assert!(viewport.lines().contains_key(&line_idx));
  debug_assert!(buffer.get_rope().get_line(line_idx).is_some());
  let bufline_last_visible_char = buffer
    .last_visible_char_on_line(line_idx)
    .unwrap_or(0_usize);

  let line_viewport = viewport.lines().get(&line_idx).unwrap();
  let rows = line_viewport.rows();
  debug_assert!(rows.last_key_value().is_some());
  let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();
  last_row_viewport.end_char_idx().saturating_sub(1) < bufline_last_visible_char
}

/// Returns `start_column`
fn revert_search_line_start_wrap_nolinebreak(
  buffer: &Buffer,
  line_idx: usize,
  last_char: usize,
  window_height: u16,
  window_width: u16,
) -> usize {
  let bufline = buffer.get_rope().line(line_idx);
  let bufline_len_chars = bufline.len_chars();
  debug_assert!(bufline_len_chars > 0);

  // Approximately calculate the beginning char of the line in window viewport, by directly
  // subtract `window_width * window_height`.
  let last_char_width = buffer.width_at(line_idx, last_char);
  let approximate_start_width =
    last_char_width.saturating_sub(window_width as usize * window_height as usize);
  let mut start_char = buffer
    .char_at(line_idx, approximate_start_width)
    .unwrap_or(0_usize);
  trace!(
    "line_idx:{},last_char:{}({:?}),last_char_width:{},approximate_start_width:{},start_char:{}({:?})",
    line_idx,
    last_char,
    bufline.char(last_char),
    last_char_width,
    approximate_start_width,
    start_char,
    bufline.char(start_char),
  );

  while start_char < bufline_len_chars {
    let start_column = buffer.width_before(line_idx, start_char);
    let (rows, _start_fills, _end_fills, _) = proc_line_wrap_nolinebreak(
      buffer,
      start_column,
      line_idx,
      0_u16,
      window_height,
      window_width,
    );
    let (last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();
    trace!(
      "start_column:{},last_row_viewport.end_char:{}({:?}),last_row_idx:{}",
      start_column,
      last_row_viewport.end_char_idx(),
      bufline.get_char(last_row_viewport.end_char_idx()),
      last_row_idx
    );
    if last_char < last_row_viewport.end_char_idx() {
      return start_column;
    }
    start_char += 1;
  }

  unreachable!()
}

// NOTE: For `wrap=true, linebreak=false`, if there's any head/tail not fully rendered, it means
// there will be only 1 line shows in current window viewport. Because the `wrap` will force the
// 2nd line wait to show until the **current** line get fully rendered.
fn right_downward_wrap_nolinebreak(
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  _viewport_start_line: usize,
  _viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (bool, usize) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (rows, _start_fills, _end_fills, _) =
    proc_line_wrap_nolinebreak(buffer, 0, target_cursor_line, 0_u16, height, width);

  debug_assert!(rows.last_key_value().is_some());
  let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();

  let on_right_side = target_cursor_char >= last_row_viewport.end_char_idx();

  if on_right_side {
    let start_column = revert_search_line_start_wrap_nolinebreak(
      buffer,
      target_cursor_line,
      target_cursor_char,
      height,
      width,
    );
    (true, start_column)
  } else {
    (false, 0_usize)
  }
}

fn adjust_downward_horizontally_wrap_nolinebreak(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
  start_line: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();

  let (on_left_side, start_column_on_left_side) = left_downward_nowrap(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_left_side {
    return (start_line, start_column_on_left_side);
  }

  let (on_right_side, start_column_on_right_side) = right_downward_wrap_nolinebreak(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_right_side {
    return (start_line, start_column_on_right_side);
  }

  (start_line, viewport_start_column)
}

fn adjust_current_line(
  current_line: isize,
  target_cursor_line: usize,
  window_height: u16,
  n: usize,
) -> usize {
  if (current_line as usize) < target_cursor_line {
    if n > (window_height as usize) {
      current_line as usize + 1
    } else {
      current_line as usize
    }
  } else {
    current_line as usize
  }
}

fn search_anchor_downward_wrap_nolinebreak(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let _viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_visible_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().last_key_value().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last_key_value().unwrap();

  let target_cursor_line_not_fully_show = line_head_not_show(viewport, target_cursor_line)
    || line_tail_not_show(viewport, buffer, target_cursor_line);

  let start_line = if target_cursor_line <= last_line && !target_cursor_line_not_fully_show {
    viewport_start_line
  } else {
    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n < height as usize) && (current_line >= 0) {
      let current_row = 0_u16;
      let (rows, _start_fills, _end_fills, _) =
        proc_line_wrap_nolinebreak(buffer, 0, current_line as usize, current_row, height, width);
      n += rows.len();

      if current_line == 0 || n >= height as usize {
        break;
      }

      current_line -= 1;
    }

    adjust_current_line(current_line, target_cursor_line, height, n)
  };

  adjust_downward_horizontally_wrap_nolinebreak(
    viewport,
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
  )
}

// For `wrap=true,linebreak=true`, the `start_char` have to start from a valid word
// beginning, i.e. a unicode segment, not a arbitrary char index.
fn find_start_char_by_word(
  buffer: &Buffer,
  bufline: &RopeSlice,
  line_idx: usize,
  start_char: usize,
) -> usize {
  if start_char > 0 {
    let last_segment_char = start_char;
    let mut start_segment_char = start_char;
    loop {
      let c_value = bufline.char(start_segment_char);
      if c_value.is_whitespace() {
        break;
      }
      if start_segment_char == 0 {
        break;
      }
      start_segment_char = start_segment_char.saturating_sub(1);
    }
    let cloned_segment = buffer
      .clone_line(
        line_idx,
        start_segment_char,
        last_segment_char.saturating_sub(start_segment_char) + 1,
      )
      .unwrap();
    debug_assert!(!cloned_segment.is_empty());
    let segment_words: Vec<&str> = cloned_segment.split_word_bounds().collect();
    debug_assert!(!segment_words.is_empty());
    // Word index => its (start char index, end char index)
    let segment_words_char_idx = segment_words
      .iter()
      .enumerate()
      .scan(start_segment_char, |state, (i, wd)| {
        let old_state = *state;
        *state += wd.chars().count();
        Some((i, (old_state, *state)))
      })
      .collect::<HashMap<usize, (usize, usize)>>();
    debug_assert!(!segment_words_char_idx.is_empty());
    let mut result = last_segment_char;

    for (w, _word) in segment_words.iter().rev().enumerate() {
      let (word_start_char, _word_end_char) = segment_words_char_idx.get(&w).unwrap();
      if *word_start_char <= last_segment_char {
        result = *word_start_char;
        break;
      }
    }

    result
  } else {
    0_usize
  }
}

fn left_downward_wrap_linebreak(
  buffer: &Buffer,
  _window_actual_shape: &U16Rect,
  _viewport_start_line: usize,
  viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (bool, usize) {
  let on_left_side = match buffer.char_after(target_cursor_line, viewport_start_column) {
    Some(c) => {
      trace!(
        "target_cursor_line:{},target_cursor_char:{},viewport_start_line:{},viewport_start_column:{},c:{}",
        target_cursor_line, target_cursor_char, _viewport_start_line, viewport_start_column, c
      );
      c > target_cursor_char
    }
    None => true,
  };

  if on_left_side {
    debug_assert!(buffer.get_rope().get_line(target_cursor_line).is_some());
    let bufline = buffer.get_rope().line(target_cursor_line);
    let start_char =
      find_start_char_by_word(buffer, &bufline, target_cursor_line, target_cursor_char);
    let start_column = buffer.width_before(target_cursor_line, start_char);
    (true, start_column)
  } else {
    (false, 0_usize)
  }
}

/// Returns `start_column`
fn revert_search_line_start_wrap_linebreak(
  buffer: &Buffer,
  line_idx: usize,
  last_char: usize,
  window_height: u16,
  window_width: u16,
) -> usize {
  let bufline = buffer.get_rope().line(line_idx);
  let bufline_len_chars = bufline.len_chars();
  debug_assert!(bufline_len_chars > 0);

  // Approximately calculate the beginning char of the line in window viewport, by directly
  // subtract `window_width * window_height`.
  let last_char_width = buffer.width_at(line_idx, last_char);
  let approximate_start_width =
    last_char_width.saturating_sub(window_width as usize * window_height as usize);
  let start_char = buffer
    .char_at(line_idx, approximate_start_width)
    .unwrap_or(0_usize);

  // For `wrap=true,linebreak=true`, the approximate `start_char` have to start from a valid word
  // beginning, i.e. a unicode segment, not a arbitrary char index.
  let mut start_char = find_start_char_by_word(buffer, &bufline, line_idx, start_char);

  trace!(
    "line_idx:{},last_char:{}({:?}),last_char_width:{},approximate_start_width:{},start_char:{}({:?})",
    line_idx,
    last_char,
    bufline.char(last_char),
    last_char_width,
    approximate_start_width,
    start_char,
    bufline.char(start_char),
  );

  let cloned_line = buffer
    .clone_line(
      line_idx,
      start_char,
      cloned_line_max_len(
        window_height,
        window_width,
        buffer.width_before(line_idx, start_char),
      ),
    )
    .unwrap();
  let words: Vec<&str> = cloned_line.split_word_bounds().collect();
  // Word index => its (start char index, end char index)
  let words_char_idx = words
    .iter()
    .enumerate()
    .scan(start_char, |state, (i, wd)| {
      let old_state = *state;
      *state += wd.chars().count();
      Some((i, (old_state, *state)))
    })
    .collect::<HashMap<usize, (usize, usize)>>();
  let mut word_idx = 0_usize;

  while start_char < bufline_len_chars {
    let start_column = buffer.width_before(line_idx, start_char);
    let (rows, _start_fills, _end_fills, _) = proc_line_wrap_linebreak(
      buffer,
      start_column,
      line_idx,
      0_u16,
      window_height,
      window_width,
    );
    let (last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();
    trace!(
      "start_column:{},last_row_viewport.end_char:{}({:?}),last_row_idx:{}",
      start_column,
      last_row_viewport.end_char_idx(),
      bufline.get_char(last_row_viewport.end_char_idx()),
      last_row_idx
    );
    if last_char < last_row_viewport.end_char_idx() {
      return start_column;
    }

    // Set `start_char` to next word beginning char index.
    word_idx += 1;
    let (next_word_start_char, _next_word_end_char) = words_char_idx.get(&word_idx).unwrap();
    start_char = *next_word_start_char;
  }

  unreachable!()
}

fn right_downward_wrap_linebreak(
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  _viewport_start_line: usize,
  _viewport_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (bool, usize) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  let (rows, _start_fills, _end_fills, _) =
    proc_line_wrap_linebreak(buffer, 0, target_cursor_line, 0_u16, height, width);

  debug_assert!(rows.last_key_value().is_some());
  let (_last_row_idx, last_row_viewport) = rows.last_key_value().unwrap();

  let on_right_side = target_cursor_char >= last_row_viewport.end_char_idx();

  if on_right_side {
    let start_column = revert_search_line_start_wrap_linebreak(
      buffer,
      target_cursor_line,
      target_cursor_char,
      height,
      width,
    );
    (true, start_column)
  } else {
    (false, 0_usize)
  }
}

fn search_anchor_downward_wrap_linebreak(
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

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_visible_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().last_key_value().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last_key_value().unwrap();

  let target_cursor_line_not_fully_show = line_head_not_show(viewport, target_cursor_line)
    || line_tail_not_show(viewport, buffer, target_cursor_line);

  let start_line = if target_cursor_line <= last_line && !target_cursor_line_not_fully_show {
    viewport_start_line
  } else {
    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n < height as usize) && (current_line >= 0) {
      let (rows, _start_fills, _end_fills, _) =
        proc_line_wrap_linebreak(buffer, 0, current_line as usize, 0_u16, height, width);
      n += rows.len();

      if current_line == 0 || n >= height as usize {
        break;
      }

      current_line -= 1;
    }

    adjust_current_line(current_line, target_cursor_line, height, n)
  };

  let (on_left_side, start_column_on_left_side) = left_downward_wrap_linebreak(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_left_side {
    return (start_line, start_column_on_left_side);
  }

  let (on_right_side, start_column_on_right_side) = right_downward_wrap_linebreak(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_right_side {
    return (start_line, start_column_on_right_side);
  }

  (start_line, viewport_start_column)
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
  // If window is zero-sized.
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  if height == 0 || width == 0 {
    return (0, 0);
  }

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
    (true, false) => search_anchor_upward_wrap_nolinebreak(
      viewport,
      buffer,
      window_actual_shape,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_upward_wrap_linebreak(
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
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_visible_char_on_line(target_cursor_line)
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

  adjust_downward_horizontally_nowrap(
    viewport,
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
  )
}

fn search_anchor_upward_wrap_nolinebreak(
  viewport: &Viewport,
  buffer: &Buffer,
  window_actual_shape: &U16Rect,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let _viewport_start_column = viewport.start_column_idx();
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_visible_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().last_key_value().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last_key_value().unwrap();

  let target_cursor_line_not_fully_show = line_head_not_show(viewport, target_cursor_line)
    || line_tail_not_show(viewport, buffer, target_cursor_line);

  let start_line = if target_cursor_line <= last_line && !target_cursor_line_not_fully_show {
    viewport_start_line
  } else {
    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n < height as usize) && (current_line >= 0) {
      let current_row = 0_u16;
      let (rows, _start_fills, _end_fills, _) =
        proc_line_wrap_nolinebreak(buffer, 0, current_line as usize, current_row, height, width);
      n += rows.len();

      if current_line == 0 || n >= height as usize {
        break;
      }

      current_line -= 1;
    }

    adjust_current_line(current_line, target_cursor_line, height, n)
  };

  adjust_downward_horizontally_wrap_nolinebreak(
    viewport,
    buffer,
    window_actual_shape,
    target_cursor_line,
    target_cursor_char,
    start_line,
  )
}

fn search_anchor_upward_wrap_linebreak(
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

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  let target_cursor_line = std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    buffer
      .last_visible_char_on_line(target_cursor_line)
      .unwrap_or(0_usize),
  );

  debug_assert!(viewport.lines().last_key_value().is_some());
  let (&last_line, _last_line_viewport) = viewport.lines().last_key_value().unwrap();

  let target_cursor_line_not_fully_show = line_head_not_show(viewport, target_cursor_line)
    || line_tail_not_show(viewport, buffer, target_cursor_line);

  let start_line = if target_cursor_line <= last_line && !target_cursor_line_not_fully_show {
    viewport_start_line
  } else {
    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n < height as usize) && (current_line >= 0) {
      let (rows, _start_fills, _end_fills, _) =
        proc_line_wrap_linebreak(buffer, 0, current_line as usize, 0_u16, height, width);
      n += rows.len();

      if current_line == 0 || n >= height as usize {
        break;
      }

      current_line -= 1;
    }

    adjust_current_line(current_line, target_cursor_line, height, n)
  };

  let (on_left_side, start_column_on_left_side) = left_downward_wrap_linebreak(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_left_side {
    return (start_line, start_column_on_left_side);
  }

  let (on_right_side, start_column_on_right_side) = right_downward_wrap_linebreak(
    buffer,
    window_actual_shape,
    viewport_start_line,
    viewport_start_column,
    target_cursor_line,
    target_cursor_char,
  );

  if on_right_side {
    return (start_line, start_column_on_right_side);
  }

  (start_line, viewport_start_column)
}
