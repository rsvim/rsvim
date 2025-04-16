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

/// Calculate viewport with option `wrap=false` downward, from top to bottom.
pub fn downward(
  buffer: &mut Buffer,
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
    (false, _) => downward_nowrap(buffer, window_actual_shape, start_line, start_column),
    (true, false) => {
      downward_wrap_nolinebreak(buffer, window_actual_shape, start_line, start_column)
    }
    (true, true) => {
      _from_top_left_wrap_linebreak(buffer, window_actual_shape, start_line, start_column)
    }
  }
}

/// Calculate viewport with option `wrap=false` upward, from bottom to top.
pub fn upward(
  buffer: &mut Buffer,
  window_actual_shape: &U16Rect,
  window_local_options: &WindowLocalOptions,
  end_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  if height == 0 || width == 0 {
    return (ViewportLineRange::default(), BTreeMap::new());
  }

  match (
    window_local_options.wrap(),
    window_local_options.line_break(),
  ) {
    (false, _) => upward_nowrap(buffer, window_actual_shape, end_line, start_column),
    (true, false) => {
      unreachable!()
    }
    (true, true) => {
      unreachable!()
    }
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
  buffer: &mut Buffer,
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
    // If the char `c` width is less than or equal to `end_width`, the char next to `c` is the end char.
    let c_next = std::cmp::min(c + 1, bline.len_chars() - 1);
    (c_next, 0_usize)
  }
}

/// Returns `rows`, `start_fills`, `end_fills`.
fn process_line_nowrap(
  buffer: &mut Buffer,
  bufline: &RopeSlice,
  current_line: usize,
  start_column: usize,
  current_row: u16,
  _window_height: u16,
  window_width: u16,
) -> (BTreeMap<u16, RowViewport>, usize, usize) {
  let (start_char, start_fills, end_char, end_fills) = if bufline.len_chars() == 0 {
    (0_usize, 0_usize, 0_usize, 0_usize)
  } else {
    let start_char = buffer
      .char_after(current_line, start_column)
      .unwrap_or(0_usize);
    let start_fills = {
      let width_before = buffer.width_before(current_line, start_char);
      width_before.saturating_sub(start_column)
    };

    let end_width = start_column + window_width as usize;
    let (end_char, end_fills) = match buffer.char_at(current_line, end_width) {
      Some(c) => end_char_and_prefills(buffer, bufline, current_line, c, end_width),
      None => {
        // If the char not found, it means the `end_width` is too long than the whole line.
        // So the char next to the line's last char is the end char.
        (bufline.len_chars(), 0_usize)
      }
    };

    (start_char, start_fills, end_char, end_fills)
  };

  let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
  rows.insert(current_row, RowViewport::new(start_char..end_char));
  (rows, start_fills, end_fills)
}

#[allow(clippy::explicit_counter_loop)]
/// Implements [`downward`] with option `wrap=false`.
fn downward_nowrap(
  buffer: &mut Buffer,
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

  unsafe {
    // Fix multiple mutable references on `buffer`.
    let mut raw_buffer = Buffer::to_nonnull(buffer);
    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    // The first `current_row` in the window maps to the `start_line` in the buffer.
    let mut current_row = 0_u16;
    let mut current_line = start_line;

    if current_line < buffer_len_lines {
      // If `current_row` goes out of window, `current_line` goes out of buffer.
      while current_row < height && current_line < buffer_len_lines {
        let bufline = raw_buffer.as_ref().get_rope().line(current_line);

        let (rows, start_fills, end_fills) = process_line_nowrap(
          raw_buffer.as_mut(),
          &bufline,
          current_line,
          start_column,
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
}

#[allow(clippy::explicit_counter_loop)]
/// Implements [`upward`] with option `wrap=false`.
fn upward_nowrap(
  buffer: &mut Buffer,
  window_actual_shape: &U16Rect,
  end_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();
  // trace!("buffer_len_lines:{:?}", buffer_len_lines);

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  unsafe {
    // Fix multiple mutable references on `buffer`.
    let mut raw_buffer = Buffer::to_nonnull(buffer);
    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    // The first `current_row` in the window maps to the `start_line` in the buffer.
    let mut current_row: isize = height as isize - 1;
    let mut current_line: isize = end_line as isize - 1;

    if current_line >= 0 && (current_line as usize) < buffer_len_lines {
      // If `current_row` goes out of window, `current_line` goes out of buffer.
      while current_row >= 0 && current_line >= 0 && (current_line as usize) < buffer_len_lines {
        let bufline = raw_buffer.as_ref().get_rope().line(current_line as usize);

        let (rows, start_fills, end_fills) = process_line_nowrap(
          raw_buffer.as_mut(),
          &bufline,
          current_line as usize,
          start_column,
          current_row as u16,
          height,
          width,
        );

        line_viewports.insert(
          current_line as usize,
          LineViewport::new(rows, start_fills, end_fills),
        );

        // Go up to previous row and line
        current_line -= 1;
        current_row -= 1;
      }

      (
        ViewportLineRange::new((current_line + 1) as usize..end_line),
        line_viewports,
      )
    } else {
      (ViewportLineRange::default(), BTreeMap::new())
    }
  }
}

/// Returns `rows`, `start_fills`, `end_fills`, `current_row`.
fn process_line_wrap_nolinebreak(
  buffer: &mut Buffer,
  bufline: &RopeSlice,
  current_line: usize,
  start_column: usize,
  mut current_row: u16,
  window_height: u16,
  window_width: u16,
) -> (BTreeMap<u16, RowViewport>, usize, usize, u16) {
  let bufline_len_chars = bufline.len_chars();

  if bufline_len_chars == 0 {
    let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
    rows.insert(current_row, RowViewport::new(0..0));
    (rows, 0_usize, 0_usize, current_row)
  } else {
    let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();

    let mut start_char = buffer
      .char_after(current_line, start_column)
      .unwrap_or(0_usize);
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
      if end_char >= bufline_len_chars {
        break;
      }

      // Prepare next row.
      current_row += 1;
      start_char = end_char;
      end_width = buffer.width_before(current_line, end_char) + window_width as usize;
    }

    (rows, start_fills, end_fills, current_row)
  }
}

#[allow(unused_variables)]
/// Implements [`from_top_left`] with option `wrap=true` and `line-break=false`.
fn downward_wrap_nolinebreak(
  buffer: &mut Buffer,
  window_actual_shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();
  let buffer_len_lines = buffer.get_rope().len_lines();

  debug_assert!(height > 0);
  debug_assert!(width > 0);

  unsafe {
    // Fix multiple mutable references on `buffer`.
    let mut raw_buffer = Buffer::to_nonnull(buffer);
    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    // The first `wrow` in the window maps to the `start_line` in the buffer.
    let mut current_row = 0;
    let mut current_line = start_line;

    if current_line < buffer_len_lines {
      // If `current_row` goes out of window, `current_line` goes out of buffer.
      while current_row < height && current_line < buffer_len_lines {
        let bufline = raw_buffer.as_ref().get_rope().line(current_line);

        let (rows, start_fills, end_fills, changed_current_row) = process_line_wrap_nolinebreak(
          raw_buffer.as_mut(),
          &bufline,
          current_line,
          start_column,
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
  buffer: &mut Buffer,
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

#[allow(unused_variables)]
/// Implements [`from_top_left`] with option `wrap=true` and `line-break=true`.
fn _from_top_left_wrap_linebreak(
  buffer: &mut Buffer,
  window_actual_shape: &U16Rect,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = window_actual_shape.height();
  let width = window_actual_shape.width();

  // trace!(
  //   "_collect_from_top_left_with_wrap_linebreak, actual_shape:{:?}, height/width:{:?}/{:?}",
  //   actual_shape,
  //   height,
  //   width
  // );

  // trace!(
  //   "buffer.get_line ({:?}):'{:?}'",
  //   start_line,
  //   match raw_buffer.as_ref().get_rope().get_line(start_line) {
  //     Some(line) => slice2line(&line),
  //     None => "None".to_string(),
  //   }
  // );

  unsafe {
    // Fix multiple mutable references on `buffer`.
    let mut raw_buffer = Buffer::to_nonnull(buffer);
    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    match raw_buffer.as_ref().get_rope().get_lines_at(start_line) {
      Some(buflines) => {
        // The `start_line` is inside the buffer.

        // The first `wrow` in the window maps to the `start_line` in the buffer.
        let mut wrow = 0;
        let mut current_line = start_line;

        for bline in buflines {
          // Current row goes out of viewport.
          if wrow >= height {
            break;
          }

          let (rows, start_fills, end_fills) = if bline.len_chars() == 0 {
            let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
            rows.insert(wrow, RowViewport::new(0..0));
            (rows, 0_usize, 0_usize)
          } else {
            let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();

            // Here clone the line with the max chars that can hold by current window/viewport,
            // i.e. the `height * width` cells count as the max chars in the line. This helps avoid
            // performance issue when iterating on super long lines.
            let cloned_line = raw_buffer
              .as_ref()
              .clone_line(
                current_line,
                0,
                height as usize * width as usize * 2 + 16 + start_column,
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

            let mut start_char = raw_buffer
              .as_mut()
              .char_after(current_line, start_column)
              .unwrap_or(0_usize);
            let start_fills = {
              let width_before = raw_buffer.as_mut().width_before(current_line, start_char);
              width_before.saturating_sub(start_column)
            };

            let mut end_width = start_column + width as usize;
            let mut end_fills = 0_usize;

            // Saved last word info, if it is too long to put in an entire row of window.
            // The tuple is:
            // 1. Word index.
            // 2. Start char of the word.
            // 3. End char of the word.
            // 4. Continued start char index of the word (which should be continued to rendering on
            //    current row).
            let mut last_word_is_too_long: Option<(usize, usize, usize, usize)> = None;

            debug_assert!(wrow < height);
            while wrow < height {
              let (end_char, end_fills_result) =
                match raw_buffer.as_mut().char_at(current_line, end_width) {
                  Some(c) => {
                    match last_word_is_too_long {
                      Some((
                        last_wd_idx,
                        start_c_of_last_wd,
                        end_c_of_last_wd,
                        continued_c_of_last_wd,
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

                        match raw_buffer.as_mut().char_at(current_line, end_width) {
                          Some(c) => {
                            if end_c_of_last_wd > c {
                              // Part-2.1, the rest part of the word is still too long.

                              // Record the position (c) where we cut the words into pieces.
                              last_word_is_too_long =
                                Some((last_wd_idx, start_c_of_last_wd, end_c_of_last_wd, c));

                              // If the char `c` width is greater than `end_width`, the `c` itself is
                              // the end char.
                              end_char_and_prefills(
                                raw_buffer.as_mut(),
                                &bline,
                                current_line,
                                c,
                                end_width,
                              )
                            } else {
                              // Part-2.2, the rest part of the word is not long.
                              // Thus we can go back to *normal* algorithm just like part-1.

                              part1(
                                &words,
                                &words_end_char_idx,
                                raw_buffer.as_mut(),
                                &bline,
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
                            (bline.len_chars(), 0_usize)
                          }
                        }
                      }
                      None => {
                        // Part-1
                        part1(
                          &words,
                          &words_end_char_idx,
                          raw_buffer.as_mut(),
                          &bline,
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
                    (bline.len_chars(), 0_usize)
                  }
                };
              end_fills = end_fills_result;

              rows.insert(wrow, RowViewport::new(start_char..end_char));

              // Goes out of line.
              if end_char >= bline.len_chars() {
                break;
              }

              // Prepare next row.
              wrow += 1;
              start_char = end_char;
              end_width = raw_buffer.as_mut().width_before(current_line, end_char) + width as usize;
            }

            (rows, start_fills, end_fills)
          };

          line_viewports.insert(
            current_line,
            LineViewport::new(rows, start_fills, end_fills),
          );

          // trace!(
          //   "13-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}",
          //   wrow,
          //   wcol,
          //   dcol,
          //   start_dcol,
          //   end_dcol,
          //   bchars,
          //   start_c_idx,
          //   end_c_idx,
          //   start_fills,
          //   end_fills
          // );

          current_line += 1;
          wrow += 1;
        }

        // trace!("14-wrow:{}, current_line:{}", wrow, current_line);
        (
          ViewportLineRange::new(start_line..current_line),
          line_viewports,
        )
      }
      None => {
        // The `start_line` is outside of the buffer.
        // trace!("15-start_line:{}", start_line);
        (ViewportLineRange::default(), BTreeMap::new())
      }
    }
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use super::*;

  use crate::test::log::init as test_log_init;
  use std::ops::Range;
  use tracing::info;

  #[test]
  fn default_range() {
    test_log_init();

    let r1: Range<usize> = Range::default();
    info!("r1:{:?}", r1);
    info!("r1.start:{:?}, r1.end:{:?}", r1.start, r1.end);
    assert!(r1.is_empty());
    assert!(r1.start == 0);
    assert!(r1.end == 0);
  }
}
