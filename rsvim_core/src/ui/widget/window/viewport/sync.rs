//! Internal implementations for Viewport.

use crate::buf::{Buffer, BufferWk};
use crate::prelude::*;
use crate::ui::widget::window::viewport::RowViewport;
use crate::ui::widget::window::{LineViewport, ViewportOptions};
use crate::wlock;

use ropey::RopeSlice;
use std::collections::BTreeMap;
use std::ops::Range;
use std::ptr::NonNull;
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

// Given the buffer and window size, collect information from start line and column, i.e. from the
// top-left corner.
pub fn from_top_left(
  options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_dcolumn: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  // If window is zero-sized.
  let height = actual_shape.height();
  let width = actual_shape.width();
  if height == 0 || width == 0 {
    return (ViewportLineRange::default(), BTreeMap::new());
  }

  match (options.wrap, options.line_break) {
    (false, _) => _from_top_left_nowrap(options, buffer, actual_shape, start_line, start_dcolumn),
    (true, false) => {
      _from_top_left_wrap_nolinebreak(options, buffer, actual_shape, start_line, start_dcolumn)
    }
    (true, true) => {
      _from_top_left_wrap_linebreak(options, buffer, actual_shape, start_line, start_dcolumn)
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

unsafe fn end_char_and_prefills(
  mut raw_buffer: NonNull<Buffer>,
  bline: &RopeSlice,
  l: usize,
  c: usize,
  end_width: usize,
) -> (usize, usize) {
  unsafe {
    let c_width = raw_buffer.as_mut().width_at(l, c);
    if c_width > end_width {
      // If the char `c` width is greater than `end_width`, the `c` itself is the end char.
      let c_width_before = raw_buffer.as_mut().width_before(l, c);
      (c, end_width.saturating_sub(c_width_before))
    } else {
      // If the char `c` width is less than or equal to `end_width`, the char next to `c` is the end char.
      let c_next = std::cmp::min(c + 1, bline.len_chars() - 1);
      (c_next, 0_usize)
    }
  }
}

#[allow(unused_variables, clippy::explicit_counter_loop)]
/// Implements [`from_top_left`] with option `wrap=false`.
fn _from_top_left_nowrap(
  _options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_dcol_on_line: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  assert!(height > 0);
  assert!(width > 0);
  // trace!(
  //   "_collect_from_top_left_with_nowrap, actual_shape:{:?}, height/width:{:?}/{:?}",
  //   actual_shape,
  //   height,
  //   width
  // );

  // Get buffer arc pointer, and lock for write.
  let buffer = buffer.upgrade().unwrap();
  let mut buffer = wlock!(buffer);

  unsafe {
    // Fix mutable borrow on `buffer`.
    let mut raw_buffer = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

    // trace!(
    //   "buffer.get_line ({:?}):{:?}",
    //   start_line,
    //   match buffer.get_line(start_line) {
    //     Some(line) => slice2line(&line),
    //     None => "None".to_string(),
    //   }
    // );

    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    match raw_buffer.as_ref().get_rope().get_lines_at(start_line) {
      // The `start_line` is in the buffer.
      Some(buflines) => {
        // The first `wrow` in the window maps to the `start_line` in the buffer.
        let mut wrow = 0;
        let mut current_line = start_line;

        // The first `wrow` in the window maps to the `start_line` in the buffer.
        for (l, bline) in buflines.enumerate() {
          // Current row goes out of viewport.
          if wrow >= height as usize {
            break;
          }

          // trace!(
          //   "0-l:{:?}, line:'{:?}', current_line:{:?}",
          //   l,
          //   slice2line(&line),
          //   current_line
          // );

          let (start_char, start_fills, end_char, end_fills) = if bline.len_chars() == 0 {
            (0_usize, 0_usize, 0_usize, 0_usize)
          } else {
            let start_char = raw_buffer
              .as_mut()
              .char_after(l, start_dcol_on_line)
              .unwrap_or(0_usize);
            let start_fills = {
              let width_before = raw_buffer.as_mut().width_before(l, start_char);
              width_before.saturating_sub(start_dcol_on_line)
            };

            let end_width = start_dcol_on_line + width as usize;
            let (end_char, end_fills) = match raw_buffer.as_mut().char_at(l, end_width) {
              Some(c) => end_char_and_prefills(raw_buffer, &bline, l, c, end_width),
              None => {
                // If the char not found, it means the `end_width` is too long than the whole line.
                // So the char next to the line's last char is the end char.
                (bline.len_chars(), 0_usize)
              }
            };

            (start_char, start_fills, end_char, end_fills)
          };

          let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
          rows.insert(wrow as u16, RowViewport::new(start_char..end_char));
          line_viewports.insert(
            current_line,
            LineViewport::new(rows, start_fills, end_fills),
          );

          // Go to next row and line
          current_line += 1;
          wrow += 1;
        }

        // trace!("9-current_line:{}, row:{}", current_line, wrow,);
        (
          ViewportLineRange::new(start_line..current_line),
          line_viewports,
        )
      }
      None => {
        // The `start_line` is outside of the buffer.
        // trace!("10-start_line:{}", start_line);
        (ViewportLineRange::default(), BTreeMap::new())
      }
    }
  }
}

#[allow(unused_variables)]
/// Implements [`from_top_left`] with option `wrap=true` and `line-break=false`.
fn _from_top_left_wrap_nolinebreak(
  _options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_dcol_on_line: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  assert!(height > 0);
  assert!(width > 0);
  // trace!(
  //   "_collect_from_top_left_with_wrap_nolinebreak, actual_shape:{:?}, height/width:{:?}/{:?}",
  //   actual_shape,
  //   height,
  //   width
  // );

  // Get buffer arc pointer, and lock for write.
  let buffer = buffer.upgrade().unwrap();
  let mut buffer = wlock!(buffer);

  // trace!(
  //   "buffer.get_line ({:?}):'{:?}'",
  //   start_line,
  //   match buffer.get_line(start_line) {
  //     Some(line) => slice2line(&line),
  //     None => "None".to_string(),
  //   }
  // );

  unsafe {
    // Fix mutable borrow on `buffer`.
    let mut raw_buffer = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    match buffer.get_rope().get_lines_at(start_line) {
      Some(buflines) => {
        // The `start_line` is inside the buffer.

        // The first `wrow` in the window maps to the `start_line` in the buffer.
        let mut wrow = 0;
        let mut current_line = start_line;

        for (l, bline) in buflines.enumerate() {
          // Current row goes out of viewport.
          if wrow >= height {
            break;
          }

          // trace!(
          //   "0-l:{:?}, line:'{:?}', current_line:{:?}",
          //   l,
          //   slice2line(&line),
          //   current_line
          // );

          let (rows, start_fills, end_fills) = if bline.len_chars() == 0 {
            let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
            rows.insert(wrow, RowViewport::new(0..0));
            (rows, 0_usize, 0_usize)
          } else {
            let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();

            let mut start_char = raw_buffer
              .as_mut()
              .char_after(l, start_dcol_on_line)
              .unwrap_or(0_usize);
            let start_fills = {
              let width_before = raw_buffer.as_mut().width_before(l, start_char);
              width_before.saturating_sub(start_dcol_on_line)
            };

            let mut end_width = start_dcol_on_line + width as usize;
            let mut end_fills = 0_usize;

            assert!(wrow < height);
            while wrow < height {
              let (end_char, end_fills_result) = match raw_buffer.as_mut().char_at(l, end_width) {
                Some(c) => end_char_and_prefills(raw_buffer, &bline, l, c, end_width),
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
              end_width = raw_buffer.as_mut().width_before(l, end_char) + width as usize;
            }

            (rows, start_fills, end_fills)
          };

          line_viewports.insert(
            current_line,
            LineViewport::new(rows, start_fills, end_fills),
          );
          // trace!(
          //   "9-current_line:{}, wrow/wcol:{}/{}, dcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
          //   current_line,
          //   wrow,
          //   wcol,
          //   dcol,
          //   start_dcol,
          //   end_dcol,
          //   start_c_idx,
          //   end_c_idx,
          //   start_fills,
          //   end_fills
          // );

          current_line += 1;
          wrow += 1;
        }

        // trace!("10-current_line:{}, wrow:{}", current_line, wrow);
        (
          ViewportLineRange::new(start_line..current_line),
          line_viewports,
        )
      }
      None => {
        // The `start_line` is outside of the buffer.
        // trace!("11-start_line:{}", start_line);
        (ViewportLineRange::default(), BTreeMap::new())
      }
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
unsafe fn part1(
  words: &[&str],
  words_end_char_idx: &HashMap<usize, usize>,
  mut raw_buffer: NonNull<Buffer>,
  bline: &RopeSlice,
  l: usize,
  c: usize,
  end_width: usize,
  start_char: usize,
  last_word_is_too_long: &mut Option<(usize, usize, usize, usize)>,
) -> (usize, usize) {
  let (wd_idx, start_c_of_wd, end_c_of_wd) = find_word_by_char(words, words_end_char_idx, c);

  unsafe {
    let end_c_width = raw_buffer.as_mut().width_before(l, end_c_of_wd);
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

        end_char_and_prefills(raw_buffer, bline, l, start_c_of_wd - 1, end_width)
      } else {
        // Part-1.2, cut this word and force rendering it ignoring line-break behavior.
        assert_eq!(start_c_of_wd, start_char);
        // Record the position (c) where we cut the words into pieces.
        *last_word_is_too_long = Some((wd_idx, start_c_of_wd, end_c_of_wd, c));

        // If the char `c` width is greater than `end_width`, the `c` itself is the end char.
        end_char_and_prefills(raw_buffer, bline, l, c, end_width)
      }
    } else {
      assert_eq!(c + 1, end_c_of_wd);
      // The current word is not long, it can be put in current row.
      let c_next = std::cmp::min(end_c_of_wd, bline.len_chars());
      (c_next, 0_usize)
    }
  }
}

#[allow(unused_variables)]
/// Implements [`from_top_left`] with option `wrap=true` and `line-break=true`.
fn _from_top_left_wrap_linebreak(
  _options: &ViewportOptions,
  buffer: BufferWk,
  actual_shape: &U16Rect,
  start_line: usize,
  start_dcol_on_line: usize,
) -> (ViewportLineRange, BTreeMap<usize, LineViewport>) {
  let height = actual_shape.height();
  let width = actual_shape.width();

  // trace!(
  //   "_collect_from_top_left_with_wrap_linebreak, actual_shape:{:?}, height/width:{:?}/{:?}",
  //   actual_shape,
  //   height,
  //   width
  // );

  // Get buffer arc pointer, and lock for write.
  let buffer = buffer.upgrade().unwrap();
  let mut buffer = wlock!(buffer);

  trace!(
    "buffer.get_line ({:?}):'{:?}'",
    start_line,
    match buffer.get_rope().get_line(start_line) {
      Some(line) => slice2line(&line),
      None => "None".to_string(),
    }
  );

  unsafe {
    // Fix mutable borrow on `buffer`.
    let mut raw_buffer = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

    let mut line_viewports: BTreeMap<usize, LineViewport> = BTreeMap::new();

    match raw_buffer.as_ref().get_rope().get_lines_at(start_line) {
      Some(buflines) => {
        // The `start_line` is inside the buffer.

        // The first `wrow` in the window maps to the `start_line` in the buffer.
        let mut wrow = 0;
        let mut current_line = start_line;

        for (l, bline) in buflines.enumerate() {
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
                l,
                0,
                height as usize * width as usize * 2 + 16 + start_dcol_on_line,
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
              .char_after(l, start_dcol_on_line)
              .unwrap_or(0_usize);
            let start_fills = {
              let width_before = raw_buffer.as_mut().width_before(l, start_char);
              width_before.saturating_sub(start_dcol_on_line)
            };

            let mut end_width = start_dcol_on_line + width as usize;
            let mut end_fills = 0_usize;

            // Saved last word info, if it is too long to put in an entire row of window.
            // The tuple is:
            // 1. Word index.
            // 2. Start char of the word.
            // 3. End char of the word.
            // 4. Continued start char index of the word (which should be continued to rendering on
            //    current row).
            let mut last_word_is_too_long: Option<(usize, usize, usize, usize)> = None;

            assert!(wrow < height);
            while wrow < height {
              let (end_char, end_fills_result) = match raw_buffer.as_mut().char_at(l, end_width) {
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

                      match raw_buffer.as_mut().char_at(l, end_width) {
                        Some(c) => {
                          if end_c_of_last_wd > c {
                            // Part-2.1, the rest part of the word is still too long.

                            // Record the position (c) where we cut the words into pieces.
                            last_word_is_too_long =
                              Some((last_wd_idx, start_c_of_last_wd, end_c_of_last_wd, c));

                            // If the char `c` width is greater than `end_width`, the `c` itself is
                            // the end char.
                            end_char_and_prefills(raw_buffer, &bline, l, c, end_width)
                          } else {
                            // Part-2.2, the rest part of the word is not long.
                            // Thus we can go back to *normal* algorithm just like part-1.

                            part1(
                              &words,
                              &words_end_char_idx,
                              raw_buffer,
                              &bline,
                              l,
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
                        raw_buffer,
                        &bline,
                        l,
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
              end_width = raw_buffer.as_mut().width_before(l, end_char) + width as usize;
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
