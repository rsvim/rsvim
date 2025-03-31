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

  // Get buffer arc pointer, and lock for read.
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
              Some(c) => {
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

  // Get buffer arc pointer, and lock for read.
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
                Some(c) => {
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
fn find_word_by_char<'a>(
  words: &'a Vec<&str>,
  word_accumulated_chars_index: &HashMap<usize, usize>,
  char_idx: usize,
) -> (usize, bool) {
  let mut low = 0;
  let mut high = words.len() - 1;

  while low < high {
    let mid = (low + high) / 2;

    let start_char_idx = if mid > 0 {
      *word_accumulated_chars_index.get(&(mid - 1)).unwrap()
    } else {
      0_usize
    };
    let end_char_idx = *word_accumulated_chars_index.get(&mid).unwrap();

    if start_char_idx <= char_idx && end_char_idx > char_idx {
      return (mid, char_idx + 1 == end_char_idx);
    } else if start_char_idx > char_idx {
      high = mid - 1;
    } else {
      low = mid + 1;
    }
  }

  unreachable!()
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

  // Get buffer arc pointer, and lock for read.
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
            let cloned_line = raw_buffer.as_ref().clone_line(
              l,
              0,
              height as usize * width as usize * 2 + 16 + start_dcol_on_line,
            );

            if cloned_line.is_none() {
              rows.insert(wrow, RowViewport::new(0..0));
              (rows, 0_usize, 0_usize)
            } else {
              let cloned_line = cloned_line.unwrap();
              let word_boundaries: Vec<&str> = cloned_line.split_word_bounds().collect();
              // Word index => word's chars count (only the word itself).
              let word_chars_count: HashMap<usize, usize> = word_boundaries
                .iter()
                .enumerate()
                .map(|(i, wd)| (i, wd.chars().count()))
                .collect();
              // Word index => accumulated word's chars index (from the first char until current
              // word).
              let word_accumulated_chars_index: HashMap<usize, usize> = word_boundaries
                .iter()
                .enumerate()
                .scan(0_usize, |state, (i, wd)| {
                  *state = *state + word_chars_count.get(&i).unwrap();
                  Some((i, *state))
                })
                .collect();

              let mut start_char = raw_buffer
                .as_mut()
                .char_after(l, start_dcol_on_line)
                .unwrap_or(0_usize);
              let start_fills = {
                let width_before = raw_buffer.as_mut().width_before(l, start_char);
                width_before.saturating_sub(start_dcol_on_line)
              };

              let mut end_width = start_dcol_on_line + width as usize;
              let mut end_char: Option<usize> = None;
              let mut eol = false;

              assert!(wrow < height);
              while wrow < height && !eol {
                end_char = match raw_buffer.as_mut().char_after(l, end_width) {
                  Some(c) => {
                    let (word_idx, is_last_char_in_word) =
                      find_word_by_char(&word_boundaries, &word_accumulated_chars_index, c);
                    Some(c)
                  }
                  None => {
                    eol = true;
                    Some(raw_buffer.as_mut().last_char(l).unwrap())
                  }
                };
                rows.insert(wrow, RowViewport::new(start_char..end_char.unwrap()));
                wrow += 1;
                start_char = end_char.unwrap();
                end_width = end_width + width as usize;
              }

              let end_fills = {
                let end_width_until = raw_buffer.as_mut().width_until(l, end_char.unwrap());
                if end_width_until >= end_width {
                  end_width_until - end_width
                } else {
                  0_usize
                }
              };

              (rows, start_fills, end_fills)
            }
          };

          let mut rows: BTreeMap<u16, RowViewport> = BTreeMap::new();
          let mut wcol = 0_u16;

          let mut bchars = 0_usize;
          let mut dcol = 0_usize;
          let mut start_dcol = 0_usize;
          let mut end_dcol = 0_usize;

          let mut start_c_idx = 0_usize;
          let mut end_c_idx = 0_usize;
          let mut start_c_idx_init = false;
          let mut _end_c_idx_init = false;

          let mut ch2dcols: BTreeMap<usize, (usize, usize)> = BTreeMap::new();

          let mut start_fills = 0_usize;
          let mut end_fills = 0_usize;

          // Chop the line into maximum chars can hold by current window, thus avoid those super
          // long lines for iteration performance.
          // NOTE: Use `height * width * 4` simply for a much bigger size for the total characters in
          // a viewport.
          let truncated_line = truncate_line(
            &bline,
            start_dcol_on_line,
            height as usize * width as usize * 2 + height as usize * 2 + 16,
          );
          let word_boundaries: Vec<&str> = truncated_line.split_word_bounds().collect();
          // trace!(
          //   "0-truncated_line: {:?}, word_boundaries: {:?}, wrow/wcol:{}/{}, dcol:{}/{}/{}, c_idx:{}/{}, fills:{}/{}",
          //   truncated_line, word_boundaries, wrow, wcol, dcol, start_dcol, end_dcol, start_c_idx, end_c_idx, start_fills, end_fills
          // );

          for (i, wd) in word_boundaries.iter().enumerate() {
            let (wd_chars, wd_width) = wd.chars().map(|c| (1_usize, buffer.char_width(c))).fold(
              (0_usize, 0_usize),
              |(init_chars, init_width), (count, width)| (init_chars + count, init_width + width),
            );

            // trace!(
            //   "1-l:{:?}, line:'{:?}', current_line:{:?}, i:{}, wd:{:?}",
            //   l,
            //   slice2line(&line),
            //   current_line,
            //   i,
            //   wd
            // );

            // Prefix width is still before `start_dcolumn`.
            if dcol + wd_width < start_dcol_on_line {
              dcol += wd_width;
              bchars += wd_chars;
              end_dcol = dcol;
              end_c_idx = bchars;
              // trace!(
              //   "2-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, start_dcolumn:{}",
              //   wrow,
              //   wcol,
              //   dcol,
              //   start_dcol,
              //   end_dcol,
              //   bchars,
              //   start_c_idx,
              //   end_c_idx,
              //   start_fills,
              //   end_fills,
              //   wd_chars,
              //   wd_width,
              //   start_dcolumn
              // );
              continue;
            }

            if !start_c_idx_init {
              start_c_idx_init = true;
              start_dcol = dcol;
              start_c_idx = bchars;
              start_fills = dcol - start_dcol_on_line;
              // trace!(
              //   "3-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
              //   wrow,
              //   wcol,
              //   dcol,
              //   start_dcol,
              //   end_dcol,
              //   bchars,
              //   start_c_idx,
              //   end_c_idx,
              //   start_fills,
              //   end_fills,
              //   wd_chars,
              //   wd_width
              // );
            }

            // Row column with next char will goes out of the row.
            // i.e. there's not enough space to place this word in current row.
            // There're two cases:
            // 1. The word can be placed in next empty row, i.e. the word length is less or equal to
            //    the row length of the viewport.
            // 2. The word is too long to place in an entire row, i.e. the word length is greater
            //    than the row length of the viewport.
            // Anyway, we simply go to next row and force render all of the word. If the word is too
            // long to place in an entire row, it fallbacks back to the same behavior with
            // 'line-break' option is `false`.
            if wcol as usize + wd_width > width as usize {
              // trace!(
              //   "4.1-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
              //   wrow,
              //   wcol,
              //   dcol,
              //   start_dcol,
              //   end_dcol,
              //   bchars,
              //   start_c_idx,
              //   end_c_idx,
              //   start_fills,
              //   end_fills,
              //   wd_chars,
              //   wd_width,
              //   width
              // );

              // If it happens this word starts from the beginning of the row, then we don't need to
              // start from the next row. Because this is an empty of entire row.
              // If this word starts in the middle of the row, then we will have to start a new row.
              if wcol > 0 {
                rows.insert(
                  wrow,
                  RowViewport::new(start_dcol..end_dcol, start_c_idx..end_c_idx, &ch2dcols),
                );

                // NOTE: The `end_fills` only indicates the cells at the end of the bottom row in the
                // viewport cannot show the full unicode character for those ASCII control codes or
                // other unicodes such as CJK languages.
                // But for word-wrap rendering, i.e. `line-break` option is `true`, sometimes the whole
                // word display length is out of the end of the row and it will not be displayed (and
                // in such case, we don't set `end_fills` for it).
                // So, here we need to detect the real end fills position for the word.

                let saved_end_fills = {
                  let mut tmp_wcol = wcol;
                  for c in wd.chars() {
                    let c_width = buffer.char_width(c);

                    // Column with next char will goes out of the row.
                    if tmp_wcol as usize + c_width > width as usize {
                      break;
                    }
                    tmp_wcol += c_width as u16;
                    // Column already meets the end of the row.
                    if tmp_wcol >= width {
                      break;
                    }
                  }
                  //   trace!(
                  //   "4.2-wrow/wcol/tmp_wcol:{}/{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
                  //   wrow,
                  //   wcol,
                  //   tmp_wcol,
                  //   dcol,
                  //   start_dcol,
                  //   end_dcol,
                  //   bchars,
                  //   start_c_idx,
                  //   end_c_idx,
                  //   start_fills,
                  //   end_fills,
                  //   wd_chars,
                  //   wd_width,
                  //   width
                  // );
                  width - tmp_wcol
                };

                wrow += 1;
                wcol = 0_u16;
                start_dcol = end_dcol;
                start_c_idx = bchars;
                ch2dcols.clear();

                if wrow >= height {
                  end_fills = saved_end_fills as usize;
                  //   trace!(
                  //   "5-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                  //   wrow,
                  //   wcol,
                  //   dcol,
                  //   start_dcol,
                  //   end_dcol,
                  //   bchars,
                  //   start_c_idx,
                  //   end_c_idx,
                  //   start_fills,
                  //   end_fills,
                  //   wd_chars,
                  //   wd_width,
                  //   height
                  // );
                  break;
                }
              }

              for (j, c) in wd.chars().enumerate() {
                let c_width = buffer.char_width(c);

                // Column with next char will goes out of the row.
                if wcol as usize + c_width > width as usize {
                  // trace!(
                  //   "6-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
                  //   wrow,
                  //   wcol,
                  //   dcol,
                  //   start_dcol,
                  //   end_dcol,
                  //   bchars,
                  //   j,
                  //   c,
                  //   start_c_idx,
                  //   end_c_idx,
                  //   start_fills,
                  //   end_fills,
                  //   wd_chars,
                  //   wd_width,
                  //   width
                  // );
                  rows.insert(
                    wrow,
                    RowViewport::new(start_dcol..end_dcol, start_c_idx..end_c_idx, &ch2dcols),
                  );

                  let saved_end_fills = width as usize - wcol as usize;
                  if j > 0 {
                    wrow += 1;
                  }
                  wcol = 0_u16;
                  start_dcol = end_dcol;
                  start_c_idx = bchars;
                  ch2dcols.clear();

                  if wrow >= height {
                    end_fills = saved_end_fills;
                    // trace!(
                    //   "7-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                    //   wrow,
                    //   wcol,
                    //   dcol,
                    //   start_dcol,
                    //   end_dcol,
                    //   bchars,
                    //   j,
                    //   c,
                    //   start_c_idx,
                    //   end_c_idx,
                    //   start_fills,
                    //   end_fills,
                    //   wd_chars,
                    //   wd_width,
                    //   height
                    // );
                    break;
                  }
                }

                let saved_c_idx = bchars;
                let saved_start_dcol = dcol;

                dcol += c_width;
                bchars += 1;
                end_dcol = dcol;
                end_c_idx = bchars;
                wcol += c_width as u16;

                ch2dcols.insert(saved_c_idx, (saved_start_dcol, end_dcol));

                // trace!(
                //   "8-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
                //   wrow,
                //   wcol,
                //   dcol,
                //   start_dcol,
                //   end_dcol,
                //   bchars,
                //   j,
                //   c,
                //   start_c_idx,
                //   end_c_idx,
                //   start_fills,
                //   end_fills,
                //   wd_chars,
                //   wd_width
                // );

                // Column goes out of current row.
                if wcol >= width {
                  // trace!(
                  //   "9-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
                  //   wrow,
                  //   wcol,
                  //   dcol,
                  //   start_dcol,
                  //   end_dcol,
                  //   bchars,
                  //   j,
                  //   c,
                  //   start_c_idx,
                  //   end_c_idx,
                  //   start_fills,
                  //   end_fills,
                  //   wd_chars,
                  //   wd_width,
                  //   width
                  // );
                  rows.insert(
                    wrow,
                    RowViewport::new(start_dcol..end_dcol, start_c_idx..end_c_idx, &ch2dcols),
                  );
                  assert_eq!(wcol, width);
                  wrow += 1;
                  wcol = 0_u16;
                  start_dcol = end_dcol;
                  start_c_idx = end_c_idx;
                  ch2dcols.clear();

                  if wrow >= height {
                    // trace!(
                    //   "10-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, j/c:{}/{:?}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                    //   wrow,
                    //   wcol,
                    //   dcol,
                    //   start_dcol,
                    //   end_dcol,
                    //   bchars,
                    //   j,
                    //   c,
                    //   start_c_idx,
                    //   end_c_idx,
                    //   start_fills,
                    //   end_fills,
                    //   wd_chars,
                    //   wd_width,
                    //   height
                    // );
                    break;
                  }
                }
              }
            } else {
              // Enough space to place this word in current row
              let saved_c_idx = bchars;
              let saved_start_dcol = dcol;

              dcol += wd_width;
              bchars += wd_chars;
              end_dcol = dcol;
              end_c_idx = bchars;
              wcol += wd_width as u16;

              let mut tmp_start_dcol = saved_start_dcol;
              for (k, c) in wd.chars().enumerate() {
                let c_width = buffer.char_width(c);
                let tmp_end_dcol = tmp_start_dcol + c_width;
                ch2dcols.insert(saved_c_idx + k, (tmp_start_dcol, tmp_end_dcol));
                tmp_start_dcol = tmp_end_dcol;
              }
            }

            // trace!(
            //   "9-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
            //   wrow,
            //   wcol,
            //   dcol,
            //   start_dcol,
            //   end_dcol,
            //   bchars,
            //   start_c_idx,
            //   end_c_idx,
            //   start_fills,
            //   end_fills,
            //   wd_chars,
            //   wd_width
            // );

            // End of the line.
            if i + 1 == word_boundaries.len() {
              // trace!(
              //   "10-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}",
              //   wrow,
              //   wcol,
              //   dcol,
              //   start_dcol,
              //   end_dcol,
              //   bchars,
              //   start_c_idx,
              //   end_c_idx,
              //   start_fills,
              //   end_fills,
              //   wd_chars,
              //   wd_width
              // );
              rows.insert(
                wrow,
                RowViewport::new(start_dcol..end_dcol, start_c_idx..end_c_idx, &ch2dcols),
              );
              break;
            }

            // Column goes out of current row.
            if wcol >= width {
              // trace!(
              //   "11-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, width:{}",
              //   wrow,
              //   wcol,
              //   dcol,
              //   start_dcol,
              //   end_dcol,
              //   bchars,
              //   start_c_idx,
              //   end_c_idx,
              //   start_fills,
              //   end_fills,
              //   wd_chars,
              //   wd_width,
              //   width
              // );
              rows.insert(
                wrow,
                RowViewport::new(start_dcol..end_dcol, start_c_idx..end_c_idx, &ch2dcols),
              );
              assert_eq!(wcol, width);
              wrow += 1;
              wcol = 0_u16;
              start_dcol = end_dcol;
              start_c_idx = end_c_idx;
              ch2dcols.clear();

              if wrow >= height {
                // trace!(
                //   "12-wrow/wcol:{}/{}, dcol:{}/{}/{}, bchars:{}, c_idx:{}/{}, fills:{}/{}, wd:{}/{}, height:{}",
                //   wrow,
                //   wcol,
                //   dcol,
                //   start_dcol,
                //   end_dcol,
                //   bchars,
                //   start_c_idx,
                //   end_c_idx,
                //   start_fills,
                //   end_fills,
                //   wd_chars,
                //   wd_width,
                //   height
                // );
                break;
              }
            }
          }

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
