//! Viewport internal algorithms.

use super::CursorViewport;
use super::Viewport;
use crate::buf::text::Text;
use crate::prelude::*;
use crate::ui::viewport::LineViewport;
use crate::ui::viewport::RowViewport;
use crate::ui::widget::window::opt::WindowOptions;
use icu::segmenter::WordSegmenter;
use icu::segmenter::options::WordBreakInvariantOptions;
use itertools::Itertools;
use litemap::LiteMap;
use ropey::RopeSlice;
use std::ops::Range;

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

  pub fn start_line_idx(&self) -> usize {
    self.start_line_idx
  }

  pub fn end_line_idx(&self) -> usize {
    self.end_line_idx
  }
}

/// Calculate viewport from top to bottom.
pub fn sync(
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  // If window is zero-sized.
  if size.is_zero() {
    return (ViewportLineRange::default(), LiteMap::new());
  }

  match (opts.wrap(), opts.line_break()) {
    (false, _) => nowrap_sync(text, size, start_line, start_column),
    (true, false) => {
      wrap_nolinebreak_sync(text, size, start_line, start_column)
    }
    (true, true) => wrap_linebreak_sync(text, size, start_line, start_column),
  }
}

// Returns (end_char, end_filled_cols)
fn _end_char_and_filled_cols(
  text: &Text,
  buffer_line: &RopeSlice,
  current_line: usize,
  end_width_char: usize,
  end_width: usize,
) -> (usize, usize) {
  let c_width = text.width_until(current_line, end_width_char);
  if c_width > end_width {
    // If `the width of end_width_char > end_width`, then `end_width_char`
    // itself is the end char. And there are possibly `end_filled_cols`.
    let c_width_before = text.width_before(current_line, end_width_char);
    (end_width_char, end_width.saturating_sub(c_width_before))
  } else {
    // Here we use the last visible char in the line, thus avoid invisible
    // chars like line-break ('\n').
    debug_assert!(buffer_line.len_chars() > 0);
    let next_to_last_visible_char = text
      .last_char_idx_on_line_exclude_eol(current_line)
      .unwrap_or(0_usize)
      + 1;

    // If `the width of end_width_char <= end_width`, the char next to
    // `end_width_char` is the end char.
    let c_next = std::cmp::min(end_width_char + 1, next_to_last_visible_char);
    (c_next, 0_usize)
  }
}

#[allow(unused_assignments)]
/// Returns `rows`, `start_fills`, `end_fills`, `last_row` (in `rows`).
fn nowrap_line_process(
  text: &Text,
  start_column: usize,
  current_line: usize,
  current_row: u16,
  _window_height: u16,
  window_width: u16,
) -> (LiteMap<u16, RowViewport>, usize, usize, u16) {
  let buffer_line = text.rope().line(current_line);

  let mut rows: LiteMap<u16, RowViewport> = LiteMap::with_capacity(1);

  let mut start_fills: usize = 0;
  let mut end_fills: usize = 0;

  if buffer_line.len_chars() == 0 {
    // If current line is empty
    rows.insert(current_row, RowViewport::new(0..0));
    start_fills = 0;
    end_fills = 0;
  } else {
    if let Some(start_char) = text.char_after(current_line, start_column) {
      start_fills = {
        let width_before = text.width_before(current_line, start_char);
        width_before.saturating_sub(start_column)
      };

      let end_width = start_column + window_width as usize;
      let (end_char, end_f) = match text.char_at(current_line, end_width) {
        Some(end_width_char) => _end_char_and_filled_cols(
          text,
          &buffer_line,
          current_line,
          end_width_char,
          end_width,
        ),
        None => {
          // If the char not found, it means the `end_width` is too long
          // than the whole line. So the char next to the line's last
          // char is the end char.
          (buffer_line.len_chars(), 0)
        }
      };
      end_fills = end_f;
      rows.insert(current_row, RowViewport::new(start_char..end_char));
    } else {
      // If current line is empty
      start_fills = 0;
      end_fills = 0;
      rows.insert(current_row, RowViewport::new(0..0));
    }
  }

  (rows, start_fills, end_fills, current_row)
}

/// Implements [`sync`] with option `wrap=false`.
fn nowrap_sync(
  text: &Text,
  size: &U16Size,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  let window_height = size.height();
  let window_width = size.width();
  let buffer_len_lines = text.rope().len_lines();

  let mut line_viewports: LiteMap<usize, LineViewport> =
    LiteMap::with_capacity(window_height as usize);

  // Current row in the window maps to the `start_line` in the buffer.
  let mut current_row = 0_u16;
  let mut current_line = start_line;

  if current_line < buffer_len_lines {
    // If `current_row` goes out of window, `current_line` goes out of buffer.
    while current_row < window_height && current_line < buffer_len_lines {
      let (rows, start_fills, end_fills, _last_row) = nowrap_line_process(
        text,
        start_column,
        current_line,
        current_row,
        window_height,
        window_width,
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

#[allow(unused_assignments)]
/// Returns `rows`, `start_fills`, `end_fills`, `last_row` (in `rows`).
fn wrap_nolinebreak_line_process(
  text: &Text,
  start_column: usize,
  current_line: usize,
  current_row: u16,
  window_height: u16,
  window_width: u16,
) -> (LiteMap<u16, RowViewport>, usize, usize, u16) {
  let buffer_line = text.rope().line(current_line);
  let mut rows: LiteMap<u16, RowViewport> =
    LiteMap::with_capacity(std::cmp::min(
      buffer_line.len_chars() / (window_width as usize),
      window_height as usize,
    ));

  let mut current_row = current_row;
  let mut start_fills: usize = 0;
  let mut end_fills: usize = 0;

  if buffer_line.len_chars() == 0 {
    // If current line is empty.
    rows.insert(current_row, RowViewport::new(0..0));
    start_fills = 0;
    end_fills = 0;
  } else {
    if let Some(mut start_char) = text.char_after(current_line, start_column) {
      start_fills = {
        let width_before = text.width_before(current_line, start_char);
        width_before.saturating_sub(start_column)
      };

      let mut end_width = start_column + window_width as usize;

      debug_assert!(current_row < window_height);
      while current_row < window_height {
        let (end_char, end_f) = match text.char_at(current_line, end_width) {
          Some(end_width_char) => _end_char_and_filled_cols(
            text,
            &buffer_line,
            current_line,
            end_width_char,
            end_width,
          ),
          None => {
            // If the char not found, it means the `end_width` is too long than the whole line.
            // So the char next to the line's last char is the end char.
            (buffer_line.len_chars(), 0_usize)
          }
        };
        end_fills = end_f;

        rows.insert(current_row, RowViewport::new(start_char..end_char));

        // Goes out of line.
        debug_assert!(buffer_line.len_chars() > 0);
        if end_char
          > text
            .last_char_idx_on_line_exclude_eol(current_line)
            .unwrap_or(0_usize)
        {
          break;
        }

        // Prepare next row.
        current_row += 1;
        start_char = end_char;
        end_width =
          text.width_before(current_line, end_char) + window_width as usize;
      }
    } else {
      start_fills = 0;
      end_fills = 0;
    }
  }

  (rows, start_fills, end_fills, current_row)
}

/// Implements [`sync`] with option `wrap=true` and `line-break=false`.
fn wrap_nolinebreak_sync(
  text: &Text,
  size: &U16Size,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  let window_height = size.height();
  let window_width = size.width();
  let buffer_len_lines = text.rope().len_lines();

  let mut line_viewports: LiteMap<usize, LineViewport> =
    LiteMap::with_capacity(window_height as usize);

  // The first `current_row` in the window maps to the `start_line` in the buffer.
  let mut current_row = 0_u16;
  let mut current_line = start_line;

  if current_line < buffer_len_lines {
    // If `current_row` goes out of window, `current_line` goes out of buffer.
    while current_row < window_height && current_line < buffer_len_lines {
      let (rows, start_fills, end_fills, last_row) =
        wrap_nolinebreak_line_process(
          text,
          start_column,
          current_line,
          current_row,
          window_height,
          window_width,
        );
      current_row = last_row;

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

// Part-1 of the processing algorithm in [`wrap_linebreak_line_process`].
// Returns `(end_char, end_filled_cols)`.
fn _part1(
  words_boundary_char: &FoldMap<usize, (usize, usize)>,
  words_char_to_index: &FoldMap<usize, usize>,
  text: &Text,
  buffer_line: &RopeSlice,
  current_line: usize,
  end_width_char: usize,
  end_width: usize,
  start_char: usize,
  last_word_is_too_long: &mut Option<(usize, usize, usize, usize)>,
) -> (usize, usize) {
  let end_wd_idx = words_char_to_index.get(&end_width_char).unwrap();
  let end_wd_bound = words_boundary_char.get(end_wd_idx).unwrap();
  let start_c_of_end_wd = end_wd_bound.0;
  let end_c_of_end_wd = end_wd_bound.1;

  let end_c_width = text.width_before(current_line, end_c_of_end_wd);
  if end_c_width > end_width {
    // The current word is longer than current row, it needs to be put to next
    // row.

    // Part-1
    // Here's the **tricky** part, there are two sub-cases in this scenario:
    // 1. For most happy cases, the word is not longer than a whole row in the
    //    window, so it can be completely put to next row.
    // 2. For very rare cases, the word is just too long to put in an entire
    //    row in the window. We have to fallback to the no-line-break rendering
    //    behavior, i.e. just cut the word by chars and force rendering the
    //    word on multiple rows in the window (because otherwise there will
    //    never be enough places to put this word).

    if start_c_of_end_wd > start_char {
      // Part-1.1, simply wrapped this word to next row.
      // Here we use the `start_c_of_end_wd` as the end char for current row.

      _end_char_and_filled_cols(
        text,
        buffer_line,
        current_line,
        start_c_of_end_wd - 1,
        end_width,
      )
    } else {
      // Part-1.2, cut this word and force rendering it and ignores line-break
      // behavior.
      debug_assert!(start_c_of_end_wd <= start_char);
      // Save the position (`end_width_char`) where we cut the words into
      // pieces.
      *last_word_is_too_long = Some((
        *end_wd_idx,
        start_c_of_end_wd,
        end_c_of_end_wd,
        end_width_char,
      ));

      // NOTE: Here we fallback to the same behavior of
      // `wrap_nolinebreak_line_process`.
      //
      // If `the width of end_width_char > end_width`, the `end_width_char`
      // itself is the end char.
      _end_char_and_filled_cols(
        text,
        buffer_line,
        current_line,
        end_width_char,
        end_width,
      )
    }
  } else {
    debug_assert_eq!(end_width_char + 1, end_c_of_end_wd);
    // The current word is not long, it can be put in current row.
    let c_next = std::cmp::min(end_c_of_end_wd, buffer_line.len_chars());
    (c_next, 0_usize)
  }
}

#[allow(unused_assignments)]
/// Returns `rows`, `start_fills`, `end_fills`, `last_row` (in `rows`).
fn wrap_linebreak_line_process(
  text: &Text,
  start_column: usize,
  current_line: usize,
  current_row: u16,
  window_height: u16,
  window_width: u16,
) -> (LiteMap<u16, RowViewport>, usize, usize, u16) {
  let buffer_line = text.rope().line(current_line);

  let mut rows: LiteMap<u16, RowViewport> =
    LiteMap::with_capacity(std::cmp::min(
      buffer_line.len_chars() / (window_width as usize),
      window_height as usize,
    ));

  let mut current_row = current_row;
  let mut start_fills: usize = 0;
  let mut end_fills: usize = 0;

  if buffer_line.len_chars() == 0 {
    rows.insert(current_row, RowViewport::new(0..0));
    start_fills = 0;
    end_fills = 0;
  } else {
    // Here clone the line with the max chars that can hold by current window/viewport,
    // i.e. the `height * width` cells count as the max chars in the line. This helps avoid
    // performance issue when iterating on super long lines.

    // Clone this line from `cloned_start_char`, thus we can limit the cloned text within the
    // window's size (i.e. height * width).
    let cloned_start_char = text
      .char_before(current_line, start_column)
      .unwrap_or(0_usize);
    let cloned_line = text
      .clone_line(
        current_line,
        cloned_start_char,
        (window_height as usize + 1) * (window_width as usize + 1) * 2 + 1,
      )
      .unwrap();

    trace!(
      "cloned_line({}):{:?}, cloned_start_char:{}, start_column:{}",
      cloned_line.len(),
      cloned_line.as_str(),
      cloned_start_char,
      start_column
    );

    // Maps word index => its start char index and end char index
    //
    // NOTE: The char index of a word is the char index in current line. The
    // end char index is also the start char index of next word.
    let words_boundary_char =
      WordSegmenter::new_auto(WordBreakInvariantOptions::default())
        .segment_str(&cloned_line)
        .tuple_windows()
        .map(|(i, j)|
          // Words
          &cloned_line[i..j])
        .enumerate()
        .scan(cloned_start_char, |state, (i, wd)| {
          let sc = *state;
          *state += wd.chars().count();
          let ec = *state;
          Some((i, (sc, ec)))
        })
        .collect::<FoldMap<usize, (usize, usize)>>();

    // Maps every char index => its belonged word index.
    let mut words_char_to_index: FoldMap<usize, usize> =
      FoldMap::with_capacity(cloned_line.len());
    for (wd_index, wd_bound) in words_boundary_char.iter() {
      for c in wd_bound.0..wd_bound.1 {
        words_char_to_index.insert(c, *wd_index);
      }
    }

    // trace!("words:{:?}", words);
    // trace!("words_end_char:{:?}", words_end_char);
    // trace!("words_boundary_char:{:?}", words_boundary_char);
    // trace!("words_char_to_index:{:?}", words_char_to_index);

    if let Some(mut start_char) = text.char_after(current_line, start_column) {
      start_fills = {
        let width_before = text.width_before(current_line, start_char);
        width_before.saturating_sub(start_column)
      };

      let mut end_width = start_column + window_width as usize;

      // Saved last word info, if it is too long to put in an entire row of
      // window.
      //
      // This tuple is:
      // 1. Word index.
      // 2. Start char of the word.
      // 3. End char of the word.
      // 4. Continued start char index of the word (which should be continued to rendering on
      //    current row).
      let mut last_word_is_too_long: Option<(usize, usize, usize, usize)> =
        None;

      debug_assert!(current_row < window_height);
      while current_row < window_height {
        let (end_char, end_f) = match text.char_at(current_line, end_width) {
          Some(end_width_char) => {
            match last_word_is_too_long {
              Some((
                last_wd_idx,
                start_c_of_last_wd,
                end_c_of_last_wd,
                _continued_c_of_last_wd,
              )) => {
                // Part-2
                // This is the following logic of part-1.2, you should see it
                // before this.
                //
                // If the word is too long to put in an entire row, and we cut
                // it into pieces. In this part, we need to continue rendering
                // the rest part of the word on current row.
                //
                // Here we also have two sub-cases:
                // 1. If the rest part of the word is still too long to put in
                //    current row.
                // 2. If the rest part of the word is not long and can be put
                //    in current row.

                if end_c_of_last_wd > end_width_char {
                  // Part-2.1, the rest part of the word is still too long.

                  // Record the position (c) where we cut the words into pieces.
                  last_word_is_too_long = Some((
                    last_wd_idx,
                    start_c_of_last_wd,
                    end_c_of_last_wd,
                    end_width_char,
                  ));

                  // If the char `c` width is greater than `end_width`, the `c` itself is
                  // the end char.
                  _end_char_and_filled_cols(
                    text,
                    &buffer_line,
                    current_line,
                    end_width_char,
                    end_width,
                  )
                } else {
                  // Part-2.2, the rest part of the word is not long.
                  // Thus we can go back to *normal* behavior, i.e. part-1.

                  _part1(
                    &words_boundary_char,
                    &words_char_to_index,
                    text,
                    &buffer_line,
                    current_line,
                    end_width_char,
                    end_width,
                    start_char,
                    &mut last_word_is_too_long,
                  )
                }
              }
              None => {
                // Part-1
                _part1(
                  &words_boundary_char,
                  &words_char_to_index,
                  text,
                  &buffer_line,
                  current_line,
                  end_width_char,
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
            (buffer_line.len_chars(), 0_usize)
          }
        };
        end_fills = end_f;

        rows.insert(current_row, RowViewport::new(start_char..end_char));

        // Goes out of line.
        debug_assert!(buffer_line.len_chars() > 0);
        if end_char
          > text
            .last_char_idx_on_line_exclude_eol(current_line)
            .unwrap_or(0_usize)
        {
          break;
        }

        // Prepare next row.
        current_row += 1;
        start_char = end_char;
        end_width =
          text.width_before(current_line, end_char) + window_width as usize;
      }
    } else {
      start_fills = 0;
      end_fills = 0;
    }
  }

  (rows, start_fills, end_fills, current_row)
}

/// Implements [`sync`] with option `wrap=true` and `line-break=true`.
fn wrap_linebreak_sync(
  text: &Text,
  size: &U16Size,
  start_line: usize,
  start_column: usize,
) -> (ViewportLineRange, LiteMap<usize, LineViewport>) {
  let height = size.height();
  let width = size.width();
  let buffer_len_lines = text.rope().len_lines();

  let mut line_viewports: LiteMap<usize, LineViewport> =
    LiteMap::with_capacity(height as usize);

  // The first `current_row` in the window maps to the `start_line` in the buffer.
  let mut current_row = 0_u16;
  let mut current_line = start_line;

  if current_line < buffer_len_lines {
    // If `current_row` goes out of window, `current_line` goes out of buffer.
    while current_row < height && current_line < buffer_len_lines {
      let (rows, start_fills, end_fills, last_row) =
        wrap_linebreak_line_process(
          text,
          start_column,
          current_line,
          current_row,
          height,
          width,
        );
      current_row = last_row;

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
  use super::*;

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

  pub fn cursor_width_to_left_no_eol(
    text: &Text,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> usize {
    let mut target_cursor_width =
      text.width_before(target_cursor_line, target_cursor_char);

    // For eol, subtract these eol width, i.e. treat them as 0-width.
    let target_is_eol = text.is_eol(target_cursor_line, target_cursor_char);
    if target_is_eol {
      target_cursor_width =
        match text.last_char_idx_on_line_exclude_eol(target_cursor_line) {
          Some(last_visible_char) => {
            text.width_before(target_cursor_line, last_visible_char)
          }
          None => target_cursor_width.saturating_sub(1),
        };
    }

    target_cursor_width
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
    text: &Text,
    _window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    if cfg!(debug_assertions) {
      match text.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }
    }

    let target_cursor_width = detail::cursor_width_to_left_no_eol(
      text,
      target_cursor_line,
      target_cursor_char,
    );

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
    text: &Text,
    window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    let width = window_actual_size.width();
    let viewport_end_column = target_viewport_start_column + width as usize;

    if cfg!(debug_assertions) {
      let target_viewport_start_char = match text
        .char_after(target_cursor_line, target_viewport_start_column)
      {
        Some(c) => {
          format!("{}({:?})", c, text.rope().line(target_cursor_line).char(c))
        }
        None => "None".to_string(),
      };
      let viewport_end_char =
        match text.char_at(target_cursor_line, viewport_end_column) {
          Some(c) => {
            format!("{}({:?})", c, text.rope().line(target_cursor_line).char(c))
          }
          None => "None".to_string(),
        };
      trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{},viewport_end_column:{},viewport_end_char:{}",
        target_cursor_line,
        target_cursor_char,
        text
          .rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        target_viewport_start_column,
        target_viewport_start_char,
        viewport_end_column,
        viewport_end_char,
      );
    }

    let target_is_eol = text.is_eol(target_cursor_line, target_cursor_char);
    let target_cursor_width = text
      .width_until(target_cursor_line, target_cursor_char)
      + if target_is_eol { 1 } else { 0 }; // For eol, add extra 1 column.
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
    text: &Text,
    window_actual_size: &U16Size,
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
            text,
            window_actual_size,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_left_side = to_left(
        text,
        window_actual_size,
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
            text,
            window_actual_size,
            start_column,
            target_cursor_line,
            target_cursor_char,
          )
          .is_none()
        );
      }
    } else {
      let start_column_on_right_side = to_right(
        text,
        window_actual_size,
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
    /* text */ &Text,
    /* window_actual_size */ &U16Size,
    /* start_line */ usize,
    /* start_column */ usize,
  ) -> (
    /* line range */ ViewportLineRange,
    /* lines_viewport */ LiteMap<usize, LineViewport>,
  );

  // Type alias for `xxx_line_process` functions.
  pub type LineProcessFn = fn(
    /* text */ &Text,
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

  pub type SearchFn =
    fn(
      /* sync_fn */ SyncFn,
      /* line_process_fn */ LineProcessFn,
      /* viewport */ &Viewport,
      /* cursor_viewport */ &CursorViewport,
      /* opts */ &WindowOptions,
      /* text */ &Text,
      /* size */ &U16Size,
      /* new_start_line */ usize,
      /* mut new_start_column */ usize,
      /* target_cursor_line */ usize,
      /* target_cursor_char */ usize,
    ) -> (/* start_line */ usize, /* start_column */ usize);

  pub fn maximized_viewport_height(height: u16) -> u16 {
    height.saturating_add(3)
  }

  fn find_start_char(
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
    target_cursor_line: usize,
    target_cursor_char: usize,
    mut start_column: usize,
  ) -> usize {
    let bufline = text.rope().line(target_cursor_line);
    let bufline_len_char = bufline.len_chars();
    let bufline_chars_width =
      text.width_until(target_cursor_line, bufline_len_char);

    while start_column < bufline_chars_width {
      let (rows, _start_fills, _end_fills, _) = proc_fn(
        text,
        start_column,
        target_cursor_line,
        0_u16,
        window_actual_size.height(),
        window_actual_size.width(),
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
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> usize {
    let target_is_eol = text.is_eol(target_cursor_line, target_cursor_char);
    let target_cursor_width = text
      .width_until(target_cursor_line, target_cursor_char)
      + if target_is_eol { 1 } else { 0 }; // For eol, add extra 1 column.

    let approximate_start_column = target_cursor_width.saturating_sub(
      (window_actual_size.height() as usize)
        * (window_actual_size.width() as usize),
    );

    find_start_char(
      proc_fn,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
      approximate_start_column,
    )
  }

  // For case-1
  fn to_left_1(
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    let mut start_column = target_viewport_start_column;

    if cfg!(debug_assertions) {
      match text.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
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

    debug_assert!(text.rope().get_line(target_cursor_line).is_some());
    let last_char = text
      .last_char_idx_on_line_include_eol(target_cursor_line) // Also consider eol.
      .unwrap_or(0_usize);

    let (
      preview_target_rows,
      _preview_target_start_fills,
      _preview_target_end_fills,
      _,
    ) = proc_fn(
      text,
      start_column,
      target_cursor_line,
      0_u16,
      maximized_viewport_height(window_actual_size.height()),
      window_actual_size.width(),
    );

    let extra_space_left = match preview_target_rows.last() {
      Some((_last_row_idx, last_row_viewport)) => {
        last_row_viewport.end_char_idx() > last_char
      }
      None => true,
    };

    // If there is extra space left in viewport, i.e. viewport is not fully used, we need to do a
    // reverse search to try to locate the better `start_column`.
    if extra_space_left {
      let start_column_include_last_visible_char = reverse_search_start_column(
        proc_fn,
        text,
        window_actual_size,
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
    let target_cursor_width = detail::cursor_width_to_left_no_eol(
      text,
      target_cursor_line,
      target_cursor_char,
    );

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
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    let height = window_actual_size.height();
    let width = window_actual_size.width();

    let (
      preview_target_rows,
      _preview_target_start_fills,
      _preview_target_end_fills,
      _,
    ) = proc_fn(
      text,
      target_viewport_start_column,
      target_cursor_line,
      0_u16,
      height,
      width,
    );

    debug_assert!(preview_target_rows.last().is_some());
    let (_last_row_idx, last_row_viewport) =
      preview_target_rows.last().unwrap();

    let on_right_side = last_row_viewport.end_char_idx()
      > last_row_viewport.start_char_idx()
      && target_cursor_char >= last_row_viewport.end_char_idx();

    if on_right_side {
      let start_column = reverse_search_start_column(
        proc_fn,
        text,
        window_actual_size,
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
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
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
            text,
            window_actual_size,
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
        text,
        window_actual_size,
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
            text,
            window_actual_size,
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
        text,
        window_actual_size,
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
    _proc_fn: LineProcessFn,
    text: &Text,
    _window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    if cfg!(debug_assertions) {
      match text.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }

      let target_cursor_width =
        text.width_before(target_cursor_line, target_cursor_char);
      debug_assert_eq!(target_viewport_start_column, 0_usize);
      let on_left_side = target_cursor_width < target_viewport_start_column;
      debug_assert!(!on_left_side);
    }

    None
  }

  fn to_right_2_1(
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    debug_assert_eq!(target_viewport_start_column, 0_usize);
    let height = window_actual_size.height();
    let width = window_actual_size.width();

    let (
      preview_target_rows,
      _preview_target_start_fills,
      _preview_target_end_fills,
      _,
    ) = proc_fn(
      text,
      target_viewport_start_column,
      target_cursor_line,
      0_u16,
      height,
      width,
    );

    debug_assert!(preview_target_rows.last().is_some());
    let (_last_row_idx, last_row_viewport) =
      preview_target_rows.last().unwrap();

    let on_right_side = last_row_viewport.end_char_idx()
      > last_row_viewport.start_char_idx()
      && target_cursor_char >= last_row_viewport.end_char_idx();

    if on_right_side {
      // The `on_right_side=true` happens only when `target_cursor_char` is the eol, and the
      // `target_cursor_char` is out of viewport.
      debug_assert!(text.is_eol(target_cursor_line, target_cursor_char));
      let start_column = reverse_search_start_column(
        proc_fn,
        text,
        window_actual_size,
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
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
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
            text,
            window_actual_size,
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
        text,
        window_actual_size,
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
            text,
            window_actual_size,
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
        text,
        window_actual_size,
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
    _proc_fn: LineProcessFn,
    text: &Text,
    _window_actual_size: &U16Size,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<usize> {
    if cfg!(debug_assertions) {
      match text.char_at(target_cursor_line, target_viewport_start_column) {
        Some(target_viewport_start_char) => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:{}({:?})",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
          target_viewport_start_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_viewport_start_char)
            .unwrap_or('?')
        ),
        None => trace!(
          "target_cursor_line:{},target_cursor_char:{}({:?}),viewport_start_column:{},viewport_start_char:None",
          target_cursor_line,
          target_cursor_char,
          text
            .rope()
            .line(target_cursor_line)
            .get_char(target_cursor_char)
            .unwrap_or('?'),
          target_viewport_start_column,
        ),
      }

      let target_cursor_width =
        text.width_before(target_cursor_line, target_cursor_char);
      debug_assert_eq!(target_viewport_start_column, 0_usize);
      let on_left_side = target_cursor_width < target_viewport_start_column;
      debug_assert!(!on_left_side);
    }

    None
  }

  fn to_right_2_2(
    proc_fn: LineProcessFn,
    lines_viewport: &LiteMap<usize, LineViewport>,
    text: &Text,
    window_actual_size: &U16Size,
    target_viewport_start_line: usize,
    target_viewport_start_column: usize,
    target_cursor_line: usize,
    target_cursor_char: usize,
  ) -> Option<(usize, usize)> {
    debug_assert_eq!(target_viewport_start_column, 0_usize);

    let height = window_actual_size.height();
    let width = window_actual_size.width();

    debug_assert!(lines_viewport.contains_key(&target_cursor_line));
    let current_target_rows =
      lines_viewport.get(&target_cursor_line).unwrap().rows();
    debug_assert!(current_target_rows.last().is_some());
    let (current_last_row_idx, current_last_row_viewport) =
      current_target_rows.last().unwrap();

    let (
      preview_target_rows,
      _preview_target_start_fills,
      _preview_target_end_fills,
      _,
    ) = proc_fn(
      text,
      target_viewport_start_column,
      target_cursor_line,
      0_u16,
      height,
      width,
    );

    let fully_show = preview_target_rows.len() == current_target_rows.len();
    let is_eol = text.is_eol(target_cursor_line, target_cursor_char);
    let is_last_row = *current_last_row_idx == height.saturating_sub(1);
    let out_of_view = current_last_row_viewport.end_char_idx()
      > current_last_row_viewport.start_char_idx()
      && target_cursor_char >= current_last_row_viewport.end_char_idx();
    let on_right_side = fully_show && is_eol && is_last_row && out_of_view;

    if on_right_side {
      // The `target_cursor_line` must not to be the 1st line in the viewport (because in
      // case-2.1, the viewport contains multiple lines and the eol of target cursor line is out of
      // viewport, it has to be at the bottom-right corner).
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
    proc_fn: LineProcessFn,
    lines_viewport: &LiteMap<usize, LineViewport>,
    text: &Text,
    window_actual_size: &U16Size,
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
            text,
            window_actual_size,
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
        text,
        window_actual_size,
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
            text,
            window_actual_size,
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
        text,
        window_actual_size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );

      if let Some((start_line_right, start_column_right)) =
        start_line_column_on_right_side
      {
        return (start_line_right, start_column_right);
      }
    }

    (start_line, start_column)
  }

  pub fn reverse_search_start_line(
    proc_fn: LineProcessFn,
    text: &Text,
    window_actual_size: &U16Size,
    target_cursor_line: usize,
  ) -> usize {
    let height = window_actual_size.height();
    let width = window_actual_size.width();

    let mut n = 0_usize;
    let mut current_line = target_cursor_line as isize;

    while (n < height as usize) && (current_line >= 0) {
      let (rows, _start_fills, _end_fills, _) =
        proc_fn(text, 0_usize, current_line as usize, 0_u16, height, width);
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
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must move downward.
  debug_assert!(target_cursor_line >= viewport.start_line_idx());

  let buffer_len_lines = text.rope().len_lines();
  let target_cursor_line =
    std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    text
      .last_char_idx_on_line_include_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_downward_nowrap(
      viewport,
      text,
      size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_downward_wrap(
      wrap_nolinebreak_sync,
      wrap_nolinebreak_line_process,
      viewport,
      text,
      size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_downward_wrap(
      wrap_linebreak_sync,
      wrap_linebreak_line_process,
      viewport,
      text,
      size,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_downward_nowrap(
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_size.height();
  let width = window_actual_size.width();

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
      let (rows, _start_fills, _end_fills, _last_row) = nowrap_line_process(
        text,
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
    text,
    window_actual_size,
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
//      case when considering the eol:
//        a) The last visible char of target cursor line is at the bottom-right corner of the
//           viewport, and thus the eol is actually out of viewport.
//        b) Otherwise the eol of target cursor line is not out of viewport.
//    2.2 The viewport not only contains the target cursor line, i.e. it contains at least 2
//      lines. And we have a very specific edge case for eol:
//        a) The target cursor line is the last line in viewport, and its last visible char is at
//           the bottom-right corner, and thus the eol is out of viewport.
//        b) Otherwise the eol of target cursor line is not out of viewport.
fn search_anchor_downward_wrap(
  sync_fn: wrap_detail::SyncFn,
  proc_fn: wrap_detail::LineProcessFn,
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_size.height();
  let width = window_actual_size.width();

  let (
    preview_target_rows,
    _preview_target_start_fills,
    _preview_target_end_fills,
    _,
  ) = proc_fn(
    text,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line =
    preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line =
    preview_target_rows.len() == height as usize;

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
      text,
      window_actual_size,
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
      text,
      window_actual_size,
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
      text,
      window_actual_size,
      target_cursor_line,
    );
    let start_line = std::cmp::max(start_line, viewport_start_line);
    let start_column = 0_usize;
    let (_new_line_range, new_lines_viewport) =
      sync_fn(text, window_actual_size, start_line, start_column);
    wrap_detail::adjust_wrap_2_2(
      detail::AdjustOptions::all(),
      proc_fn,
      &new_lines_viewport,
      text,
      window_actual_size,
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
  opts: &WindowOptions,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must move upward.
  debug_assert!(target_cursor_line < viewport.end_line_idx());

  let buffer_len_lines = text.rope().len_lines();
  let target_cursor_line =
    std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    text
      .last_char_idx_on_line_include_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_upward_nowrap(
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_upward_wrap(
      wrap_nolinebreak_sync,
      wrap_nolinebreak_line_process,
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_upward_wrap(
      wrap_linebreak_sync,
      wrap_linebreak_line_process,
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_upward_nowrap(
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
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
    text,
    window_actual_size,
    target_cursor_line,
    target_cursor_char,
    start_line,
    viewport.start_column_idx(),
  )
}

fn search_anchor_upward_wrap(
  sync_fn: wrap_detail::SyncFn,
  proc_fn: wrap_detail::LineProcessFn,
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_size.height();
  let width = window_actual_size.width();

  let (
    preview_target_rows,
    _preview_target_start_fills,
    _preview_target_end_fills,
    _,
  ) = proc_fn(
    text,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line =
    preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line =
    preview_target_rows.len() == height as usize;

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
      text,
      window_actual_size,
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
      text,
      window_actual_size,
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
      sync_fn(text, window_actual_size, start_line, start_column);
    wrap_detail::adjust_wrap_2_2(
      detail::AdjustOptions::all(),
      proc_fn,
      &new_lines_viewport,
      text,
      window_actual_size,
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
  opts: &WindowOptions,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must stay in viewport.
  debug_assert!(
    target_cursor_line >= viewport.start_line_idx()
      && target_cursor_line < viewport.end_line_idx()
  );

  let buffer_len_lines = text.rope().len_lines();
  let target_cursor_line =
    std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    text
      .last_char_idx_on_line_include_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_leftward_nowrap(
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_leftward_wrap(
      wrap_nolinebreak_line_process,
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_leftward_wrap(
      wrap_linebreak_line_process,
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_leftward_nowrap(
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // adjust horizontally
  let start_line = viewport.start_line_idx();
  let start_column = viewport.start_column_idx();

  nowrap_detail::adjust_nowrap(
    detail::AdjustOptions::no_rightward(),
    text,
    window_actual_size,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

fn search_anchor_leftward_wrap(
  proc_fn: wrap_detail::LineProcessFn,
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_size.height();
  let width = window_actual_size.width();

  let (
    preview_target_rows,
    _preview_target_start_fills,
    _preview_target_end_fills,
    _,
  ) = proc_fn(
    text,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line =
    preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line =
    preview_target_rows.len() == height as usize;

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
      text,
      window_actual_size,
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
      text,
      window_actual_size,
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
      text,
      window_actual_size,
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
  opts: &WindowOptions,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // The cursor must stay in viewport.
  debug_assert!(
    target_cursor_line >= viewport.start_line_idx()
      && target_cursor_line < viewport.end_line_idx()
  );

  let buffer_len_lines = text.rope().len_lines();
  let target_cursor_line =
    std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    text
      .last_char_idx_on_line_include_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  match (opts.wrap(), opts.line_break()) {
    (false, _) => search_anchor_rightward_nowrap(
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, false) => search_anchor_rightward_wrap(
      wrap_nolinebreak_line_process,
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
    (true, true) => search_anchor_rightward_wrap(
      wrap_linebreak_line_process,
      viewport,
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
    ),
  }
}

fn search_anchor_rightward_nowrap(
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // adjust horizontally
  let start_line = viewport.start_line_idx();
  let start_column = viewport.start_column_idx();

  nowrap_detail::adjust_nowrap(
    detail::AdjustOptions::no_leftward(),
    text,
    window_actual_size,
    target_cursor_line,
    target_cursor_char,
    start_line,
    start_column,
  )
}

fn search_anchor_rightward_wrap(
  proc_fn: wrap_detail::LineProcessFn,
  viewport: &Viewport,
  text: &Text,
  window_actual_size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let height = window_actual_size.height();
  let width = window_actual_size.width();

  let (
    preview_target_rows,
    _preview_target_start_fills,
    _preview_target_end_fills,
    _,
  ) = proc_fn(
    text,
    0,
    target_cursor_line,
    0_u16,
    wrap_detail::maximized_viewport_height(height),
    width,
  );
  let cannot_fully_contains_target_cursor_line =
    preview_target_rows.len() > height as usize;
  let only_contains_target_cursor_line =
    preview_target_rows.len() == height as usize;

  if cannot_fully_contains_target_cursor_line {
    // Case-1
    // For `start_line`, force it to be `target_cursor_line`, because viewport
    // can only contain this line (and still cannot put all of it inside).
    // For `start_column`, still use old `viewport_start_column` and wait to be adjusted.
    let start_line = target_cursor_line;
    let start_column = viewport_start_column;
    wrap_detail::adjust_wrap_1(
      detail::AdjustOptions::no_leftward(),
      proc_fn,
      text,
      window_actual_size,
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
      text,
      window_actual_size,
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
      text,
      window_actual_size,
      target_cursor_line,
      target_cursor_char,
      start_line,
      start_column,
    )
  }
}

// Search a new viewport anchor (`start_line`, `start_column`).
//
// The new viewport anchor can help cursor moves and even scrolling the buffer
// if cursor reaches the border of the window viewport.
//
// Returns `(start_line, start_column)` for the new viewport.
pub fn search(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let buffer_len_lines = text.rope().len_lines();
  let target_cursor_line =
    std::cmp::min(target_cursor_line, buffer_len_lines.saturating_sub(1));
  let target_cursor_char = std::cmp::min(
    target_cursor_char,
    text
      .last_char_idx_on_line_include_eol(target_cursor_line)
      .unwrap_or(0_usize),
  );

  let (sync_fn, line_process_fn, search_left_fn, search_right_fn): (
    wrap_detail::SyncFn,
    wrap_detail::LineProcessFn,
    wrap_detail::SearchFn,
    wrap_detail::SearchFn,
  ) = match (opts.wrap(), opts.line_break()) {
    (false, _) => (
      nowrap_sync,
      nowrap_line_process,
      nowrap_search_left,
      nowrap_search_right,
    ),
    (true, false) => (
      wrap_nolinebreak_sync,
      wrap_nolinebreak_line_process,
      wrap_search_left,
      wrap_search_right,
    ),
    (true, true) => (
      wrap_linebreak_sync,
      wrap_linebreak_line_process,
      wrap_search_left,
      wrap_search_right,
    ),
  };
  if target_cursor_line < cursor_viewport.line_idx() {
    // Cursor moves upward
    search_up(
      sync_fn,
      line_process_fn,
      search_left_fn,
      search_right_fn,
      viewport,
      cursor_viewport,
      opts,
      text,
      size,
      target_cursor_line,
      target_cursor_char,
    )
  } else {
    // Cursor moves downward, or just moves to left/right side. But in this
    // algorithm, we have to moves to downward (even just for 0-lines) before
    // moving to left/right side.
    search_down(
      sync_fn,
      line_process_fn,
      search_left_fn,
      search_right_fn,
      viewport,
      cursor_viewport,
      opts,
      text,
      size,
      target_cursor_line,
      target_cursor_char,
    )
  }
}

// Detect if we can keep current `viewport.start_line_idx()` unchanged, this
// will reduce the viewport scrolls as small as we can, avoid too big jumps for
// user eyes.
//
// It returns 3 booleans:
// 1. Whether target cursor line is already in current viewport.
// 2. Whether target cursor line is already in current viewport, and it is at
//    the bottom/last line.
// 3. Whether target cursor line is already in current viewport, and it is been
//    fully shown in current viewport. Because we support the bottom line
//    partial rendering, when option `wrap = true`, if target cursor happens to
//    be the bottom line but only part of it is shown, it could lead to some
//    issues. For target cursor, we always try to show all of the line, except
//    the target cursor line is just too long to put in the entire window.
fn _if_can_keep_current_viewport_start_line(
  line_process_fn: wrap_detail::LineProcessFn,
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  _target_cursor_char: usize,
) -> (bool, bool, bool) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let window_height = size.height();
  let window_width = size.width();
  let buffer_len_lines = text.rope().len_lines();

  let (end_line, current_cursor_line_rows) = {
    let mut current_row: u16 = 0;
    let mut current_line: isize = viewport_start_line as isize;
    let mut current_cursor_line_rows: Option<usize> = None;

    // Start with `viewport_start_line`, iterate lines from top to bottom in the
    // viewport.
    while (current_row < window_height)
      && (current_line < buffer_len_lines as isize)
    {
      let (rows, _start_fills, _end_fills, _last_row) = line_process_fn(
        text,
        viewport_start_column,
        current_line as usize,
        current_row,
        window_height,
        window_width,
      );
      if current_line == cursor_viewport.line_idx() as isize {
        current_cursor_line_rows = Some(rows.len());
      }
      current_row += rows.len() as u16;
      current_line += 1;
    }
    (current_line, current_cursor_line_rows)
  };

  // Target cursor line is already in current viewport, i.e. we don't have to
  // change `viewport_start_line` for a new viewport.
  let target_cursor_is_in_current_viewport = (viewport_start_line
    <= target_cursor_line)
    && (end_line > target_cursor_line as isize);

  // Target cursor line is at the bottom line in current viewport.
  let target_cursor_is_in_bottom_line = if target_cursor_is_in_current_viewport
  {
    end_line == (target_cursor_line + 1) as isize
  } else {
    false
  };

  // Target cursor line is fully shown in current viewport, since our viewing
  // algorithm support partial rendering for the bottom line.
  let target_cursor_is_fully_shown_in_current_viewport =
    match current_cursor_line_rows {
      Some(current_cursor_line_rows) => {
        match viewport.lines.get(&cursor_viewport.line_idx()) {
          Some(line_viewport) => {
            line_viewport.rows.len() == current_cursor_line_rows
          }
          None => false,
        }
      }
      None => false,
    };

  (
    target_cursor_is_in_current_viewport,
    target_cursor_is_in_bottom_line,
    target_cursor_is_fully_shown_in_current_viewport,
  )
}

fn search_down(
  sync_fn: wrap_detail::SyncFn,
  line_process_fn: wrap_detail::LineProcessFn,
  search_left_fn: wrap_detail::SearchFn,
  search_right_fn: wrap_detail::SearchFn,
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let window_height = size.height();
  let window_width = size.width();
  let buffer_len_lines = text.rope().len_lines();

  // Step-1: Try to keep current `viewport_start_line` unchanged, this will
  // keep the viewport scrolls as small as we can, and thus avoid too big jumps
  // for users' eye.
  let (
    target_cursor_is_in_current_viewport,
    target_cursor_is_in_bottom_line,
    target_cursor_is_fully_shown_in_current_viewport,
  ) = _if_can_keep_current_viewport_start_line(
    line_process_fn,
    viewport,
    cursor_viewport,
    text,
    size,
    target_cursor_line,
    target_cursor_char,
  );

  let current_cursor_column =
    text.width_before(cursor_viewport.line_idx(), cursor_viewport.char_idx());
  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  // Whether `target_cursor_line` is inside step-1 iteration result.
  if target_cursor_is_in_current_viewport
    && !(target_cursor_is_in_bottom_line
      && !target_cursor_is_fully_shown_in_current_viewport)
  {
    // Yes it contains, this means we don't have to scroll the window viewport,
    // we can still use the `viewport_start_line` as the first line for the new
    // viewport.

    // Cursor moves to left side.
    if target_cursor_column < current_cursor_column {
      search_left_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        viewport_start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // Cursor moves to right side (even just for 0-chars).
      search_right_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        viewport_start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  } else {
    // Otherwise `target_cursor_line` is outside of step-1 iteration result. We
    // have to do an extra reverse-iteration to find out the suitable first
    // line for the new viewport.

    debug_assert!(target_cursor_line as isize >= end_line);

    let start_line = {
      // This time, we iterate in reverse order.
      let mut current_row: usize = 0;
      let mut current_line: isize = target_cursor_line as isize;

      while (current_row < window_height as usize) && (current_line >= 0) {
        let (rows, _start_fills, _end_fills, _last_row) = line_process_fn(
          text,
          viewport_start_column,
          current_line as usize,
          0,
          window_height,
          window_width,
        );
        current_row += rows.len();
        current_line -= 1;
      }
      (current_line + 1) as usize
    };

    if target_cursor_column < current_cursor_column {
      // To left side
      search_left_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // To right side
      search_right_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  }
}

fn search_up(
  sync_fn: wrap_detail::SyncFn,
  line_process_fn: wrap_detail::LineProcessFn,
  search_left_fn: wrap_detail::SearchFn,
  search_right_fn: wrap_detail::SearchFn,
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let viewport_start_line = viewport.start_line_idx();
  let viewport_start_column = viewport.start_column_idx();
  let window_height = size.height();
  let window_width = size.width();
  let buffer_len_lines = text.rope().len_lines();

  // Step-1: Try to keep current `viewport_start_line` unchanged, this will
  // keep the viewport scrolls as small as we can, and thus avoid too big jumps
  // for users' eye.
  let (
    target_cursor_is_in_current_viewport,
    target_cursor_is_in_bottom_line,
    target_cursor_is_fully_shown_in_current_viewport,
  ) = _if_can_keep_current_viewport_start_line(
    line_process_fn,
    viewport,
    cursor_viewport,
    text,
    size,
    target_cursor_line,
    target_cursor_char,
  );

  let current_cursor_column =
    text.width_before(cursor_viewport.line_idx(), cursor_viewport.char_idx());
  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  // Whether `target_cursor_line` is inside step-1 iteration result.
  if target_cursor_is_in_current_viewport
    && !(target_cursor_is_in_bottom_line
      && !target_cursor_is_fully_shown_in_current_viewport)
  {
    // Yes it contains, this means we don't have to scroll the window viewport,
    // we can still use the `viewport_start_line` as the first line for the new
    // viewport.

    // Cursor moves to left side.
    if target_cursor_column < current_cursor_column {
      search_left_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        viewport_start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // Cursor moves to right side (even just for 0-chars).
      search_right_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        viewport_start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  } else {
    // Otherwise `target_cursor_line` is outside of step-1 iteration result. We
    // have to do an extra reverse-iteration to find out the suitable first
    // line for the new viewport.

    debug_assert!(target_cursor_line as isize >= end_line);

    let start_line = {
      // This time, we iterate in reverse order.
      let mut current_row: usize = 0;
      let mut current_line: isize = target_cursor_line as isize;

      while (current_row < window_height as usize) && (current_line >= 0) {
        let (rows, _start_fills, _end_fills, _last_row) = line_process_fn(
          text,
          viewport_start_column,
          current_line as usize,
          0,
          window_height,
          window_width,
        );
        current_row += rows.len();
        current_line -= 1;
      }
      (current_line + 1) as usize
    };

    if target_cursor_column < current_cursor_column {
      // To left side
      search_left_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // To right side
      search_right_fn(
        sync_fn,
        line_process_fn,
        viewport,
        cursor_viewport,
        opts,
        text,
        size,
        start_line,
        viewport_start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  }
}

fn nowrap_search_left(
  _sync_fn: wrap_detail::SyncFn,
  _line_process_fn: wrap_detail::LineProcessFn,
  _viewport: &Viewport,
  _cursor_viewport: &CursorViewport,
  _opts: &WindowOptions,
  text: &Text,
  _size: &U16Size,
  new_start_line: usize,
  new_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  if cfg!(debug_assertions) {
    match text.char_at(target_cursor_line, new_start_column) {
      Some(new_start_char) => trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),new_start_column:{},new_start_char:{}({:?})",
        target_cursor_line,
        target_cursor_char,
        text
          .rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        new_start_column,
        new_start_char,
        text
          .rope()
          .line(target_cursor_line)
          .get_char(new_start_char)
          .unwrap_or('?')
      ),
      None => trace!(
        "target_cursor_line:{},target_cursor_char:{}({:?}),new_start_column:{},new_start_char:None",
        target_cursor_line,
        target_cursor_char,
        text
          .rope()
          .line(target_cursor_line)
          .get_char(target_cursor_char)
          .unwrap_or('?'),
        new_start_column,
      ),
    }
  }

  let mut new_start_column = new_start_column;
  let mut target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  // If `target_cursor_char` is eol or line end, and now we are moving to left
  // side. It could be something like:
  //
  // ```
  //            +----------+
  //  AAAAAAAAAA|BBBBBBBBBB|\n   <- line-0
  //  CCCCCCCCCC|_\n       |     <- line-1
  //  3rd.\n    |          |     <- line-2
  //            +----------+
  // ```
  //
  // Note: The cursor is at line-1, the first column.
  // In such case, it will be better to move `start_column` to left for 1 more
  // column, and showing the first visible char (i.e. neither eol nor line
  // end). Then the new viewport will become something like:
  //
  // ```
  //           +----------+
  //  AAAAAAAAA|ABBBBBBBBB|B\n  <- line-0
  //  CCCCCCCCC|C_\n      |     <- line-1
  //  3rd.\n   |          |     <- line-2
  //           +----------+
  // ```
  //
  // Now it looks much better.

  if text.is_eol_or_line_end(target_cursor_line, target_cursor_char) {
    target_cursor_column = target_cursor_column.saturating_sub(1);
  }

  if target_cursor_column < new_start_column {
    new_start_column = target_cursor_column;
  }

  (new_start_line, new_start_column)
}

fn wrap_search_left(
  sync_fn: wrap_detail::SyncFn,
  line_process_fn: wrap_detail::LineProcessFn,
  _viewport: &Viewport,
  _cursor_viewport: &CursorViewport,
  _opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  new_start_line: usize,
  new_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_height = size.height();
  let window_width = size.width();

  // Try preview put the target cursor line with `start_column = 0`, start from
  // `current_row = 0`.
  let (
    preview_target_rows,
    _preview_target_start_fills,
    _preview_target_end_fills,
    _,
  ) = line_process_fn(
    text,
    0,
    target_cursor_line,
    0_u16,
    window_height,
    window_width,
  );

  // Current window cannot contain the target cursor line, i.e. target cursor
  // line is just too long to be put in current window.
  let cannot_completely_contain_target_cursor_line =
    preview_target_rows.len() > window_height as usize;

  // Current window can exactly contain the target cursor line, i.e. target
  // cursor line just happens to use all the rows in current window.
  let exactly_contains_target_cursor_line =
    preview_target_rows.len() == window_height as usize;

  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  if cannot_completely_contain_target_cursor_line
    || exactly_contains_target_cursor_line
  {
    // Case-1 and Case-2

    // For `start_line`, force it to be `target_cursor_line`, because viewport
    // can only contain this line (and still cannot put all of it inside).
    let start_line = target_cursor_line;

    // For `start_column`, simply pick the smaller one between
    // `target_cursor_column` and `new_start_column` as the new viewport
    // `start_column`.
    let start_column = std::cmp::min(new_start_column, target_cursor_column);

    (start_line, start_column)
  } else {
    // Case-3

    // For `start_column`, force it to be 0.
    let start_column = 0;

    // For `start_line`, we have two cases:
    // 1. If `target_cursor_char` is at right-bottom corner of the window, and
    //    it happens to be either end-of-line or line end (i.e. out of last
    //    visible char in cursor line). Mean while, our viewing algorithm
    //    (`sync`) will not render eol (`\n`) in viewport. For example:
    //
    //    ```
    //    +----------+
    //    |AAAAAAAAAA|   <- line-0
    //    |BBBBBBBBBB|\n
    //    |CCCCCCCCCC|\n <- line-1
    //    |3rd.\n    |   <- line-2
    //    +----------+
    //    ```
    //
    //    The window width is 10, and the width of line-0 and line-1 happen to
    //    be 20 and 10, and their eol (`\n`) don't render in the viewport.
    //    In such case, if cursor is in **insert** mode in line-0 and char-20
    //    (this position allow user inserts at the end of line-0), then we need
    //    to put the cursor position at row-2 and column-0, in this example, it
    //    is the `_` in the beginning of line-1:
    //
    //    ```
    //    +----------+
    //    |AAAAAAAAAA|   <- line-0
    //    |BBBBBBBBBB|\n
    //    |_CCCCCCCCC|\n <- line-1
    //    |3rd.\n    |   <- line-2
    //    +----------+
    //    ```
    //
    //    In such case, if `target_cursor_line` and `target_cursor_char` is at
    //    right-bottom corner of the window, in this example, it is the `_` in
    //    line-2:
    //
    //    ```
    //    +----------+
    //    |AAAAAAAAAA|   <- line-0
    //    |BBBBBBBBBB|\n
    //    |CCCCCCCCCC|\n <- line-1
    //    |DDDDDDDDDD|_\n <- line-2
    //    +----------+
    //    ```
    //
    //    In this example, we cannot put the target cursor at next row and
    //    column-0. So we need to change the `start_line` to `start_line + 1`,
    //    this would give the bottom line 1 more row to put the target cursor.
    //
    // 2. Otherwise we just use `new_start_line`.

    let (_preview_viewport_range, preview_viewport) =
      sync_fn(text, size, new_start_line, start_column);

    let (cursor_is_in_bottom_line, cursor_is_at_right_bottom) =
      match preview_viewport.last() {
        Some((last_preview_line, last_preview_line_viewport)) => {
          // Target cursor line is at the bottom line in preview viewport.
          let is_bottom_line = *last_preview_line == target_cursor_line;

          // Target cursor is at the right-bottom corner in current window or
          // preview viewport.
          let at_right_bottom = if is_bottom_line {
            if let Some((_last_preview_row, last_preview_row_viewport)) =
              last_preview_line_viewport.rows().last()
            {
              // How do we detect whether target cursor is at right-bottom
              // corner?
              //
              // 1. The last row of the preview viewport is not empty
              let last_row_not_empty = last_preview_row_viewport.end_char_idx()
                > last_preview_row_viewport.start_char_idx();
              // 2. The end char of last row == `target_cursor_char`
              let at_last_row =
                last_preview_row_viewport.end_char_idx() == target_cursor_char;
              // 3. The width of last row >= `window_width`
              let last_row_full_width = text
                .width_before(
                  target_cursor_line,
                  last_preview_row_viewport.end_char_idx(),
                )
                .saturating_sub(text.width_before(
                  target_cursor_line,
                  last_preview_row_viewport.start_char_idx(),
                ))
                >= window_width as usize;

              last_row_not_empty && at_last_row && last_row_full_width
            } else {
              false
            }
          } else {
            false
          };
          (is_bottom_line, at_right_bottom)
        }
        None => (false, false),
      };

    let start_line = if cursor_is_in_bottom_line && cursor_is_at_right_bottom {
      new_start_line + 1
    } else {
      new_start_line
    };

    (start_line, start_column)
  }
}

fn nowrap_search_right(
  _sync_fn: wrap_detail::SyncFn,
  _line_process_fn: wrap_detail::LineProcessFn,
  _viewport: &Viewport,
  _cursor_viewport: &CursorViewport,
  _opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  new_start_line: usize,
  new_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_width = size.width();
  let new_end_column = new_start_column + window_width as usize;

  if cfg!(debug_assertions) {
    let new_start_char =
      match text.char_at(target_cursor_line, new_start_column) {
        Some(c) => {
          format!("{}({:?})", c, text.rope().line(target_cursor_line).char(c))
        }
        None => "None".to_string(),
      };
    let new_end_char = match text.char_at(target_cursor_line, new_end_column) {
      Some(c) => {
        format!("{}({:?})", c, text.rope().line(target_cursor_line).char(c))
      }
      None => "None".to_string(),
    };
    trace!(
      "target_cursor_line:{},target_cursor_char:{}({:?}),new_start_column:{}({:?}),new_end_column:{}({:?})",
      target_cursor_line,
      target_cursor_char,
      text
        .rope()
        .line(target_cursor_line)
        .get_char(target_cursor_char)
        .unwrap_or('?'),
      new_start_column,
      new_start_char,
      new_end_column,
      new_end_char,
    );
  }

  let mut new_start_column = new_start_column;
  let out_of_line =
    text.is_eol_or_line_end(target_cursor_line, target_cursor_char);
  // For eol or line-end, add 1 more column
  let target_cursor_width = text
    .width_until(target_cursor_line, target_cursor_char)
    + if out_of_line { 1 } else { 0 };

  if target_cursor_width > new_end_column {
    new_start_column =
      target_cursor_width.saturating_sub(window_width as usize);
  }

  (new_start_line, new_start_column)
}

fn wrap_search_right(
  sync_fn: wrap_detail::SyncFn,
  line_process_fn: wrap_detail::LineProcessFn,
  _viewport: &Viewport,
  _cursor_viewport: &CursorViewport,
  _opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  new_start_line: usize,
  new_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_height = size.height();
  let window_width = size.width();

  // Try preview put the target cursor line with `start_column = 0`, start from
  // `current_row = 0`.
  let (
    preview_target_rows,
    _preview_target_start_fills,
    _preview_target_end_fills,
    _,
  ) = line_process_fn(
    text,
    0,
    target_cursor_line,
    0_u16,
    window_height,
    window_width,
  );

  // Current window cannot contain the target cursor line, i.e. target cursor
  // line is just too long to be put in current window.
  let cannot_completely_contain_target_cursor_line =
    preview_target_rows.len() > window_height as usize;

  // Current window can exactly contain the target cursor line, i.e. target
  // cursor line just happens to use all the rows in current window.
  let exactly_contains_target_cursor_line =
    preview_target_rows.len() == window_height as usize;

  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  if cannot_completely_contain_target_cursor_line
    || exactly_contains_target_cursor_line
  {
    // Case-1 and Case-2

    // For `start_line`, force it to be `target_cursor_line`, because viewport
    // can only contain this line (and still cannot put all of it inside).
    let start_line = target_cursor_line;

    // For `start_column`, first calculate the `target_cursor_start_column`
    // based on the `target_cursor_column` as the end column in the window.
    // Then simply pick the smaller one between `target_cursor_start_column`
    // and `new_start_column` as the new viewport `start_column`.
    let target_cursor_end_column =
      if text.is_eol_or_line_end(target_cursor_line, target_cursor_char) {
        target_cursor_column + 1
      } else {
        target_cursor_column
      };
    let target_cursor_start_column =
      target_cursor_end_column.saturating_sub(window_width as usize);
    let start_column =
      std::cmp::max(new_start_column, target_cursor_start_column);

    (start_line, start_column)
  } else {
    // Case-3

    // For `start_column`, force it to be 0.
    let start_column = 0;

    // For `start_line`, we have two cases:
    // 1. If `target_cursor_char` is at right-bottom corner of the window, and
    //    it happens to be either end-of-line or line end (i.e. out of last
    //    visible char in cursor line). Mean while, our viewing algorithm
    //    (`sync`) will not render eol (`\n`) in viewport. For example:
    //
    //    ```
    //    +----------+
    //    |AAAAAAAAAA|   <- line-0
    //    |BBBBBBBBBB|\n
    //    |CCCCCCCCCC|\n <- line-1
    //    |3rd.\n    |   <- line-2
    //    +----------+
    //    ```
    //
    //    The window width is 10, and the width of line-0 and line-1 happen to
    //    be 20 and 10, and their eol (`\n`) don't render in the viewport.
    //    In such case, if cursor is in **insert** mode in line-0 and char-20
    //    (this position allow user inserts at the end of line-0), then we need
    //    to put the cursor position at row-2 and column-0, in this example, it
    //    is the `_` in the beginning of line-1:
    //
    //    ```
    //    +----------+
    //    |AAAAAAAAAA|   <- line-0
    //    |BBBBBBBBBB|\n
    //    |_CCCCCCCCC|\n <- line-1
    //    |3rd.\n    |   <- line-2
    //    +----------+
    //    ```
    //
    //    In such case, if `target_cursor_line` and `target_cursor_char` is at
    //    right-bottom corner of the window, in this example, it is the `_` in
    //    line-2:
    //
    //    ```
    //    +----------+
    //    |AAAAAAAAAA|   <- line-0
    //    |BBBBBBBBBB|\n
    //    |CCCCCCCCCC|\n <- line-1
    //    |DDDDDDDDDD|_\n <- line-2
    //    +----------+
    //    ```
    //
    //    In this example, we cannot put the target cursor at next row and
    //    column-0. So we need to change the `start_line` to `start_line + 1`,
    //    this would give the bottom line 1 more row to put the target cursor.
    //
    // 2. Otherwise we just use `new_start_line`.

    let (_preview_viewport_range, preview_viewport) =
      sync_fn(text, size, new_start_line, start_column);

    let (cursor_is_in_bottom_line, cursor_is_at_right_bottom) =
      match preview_viewport.last() {
        Some((last_preview_line, last_preview_line_viewport)) => {
          // Target cursor line is at the bottom line in preview viewport.
          let is_bottom_line = *last_preview_line == target_cursor_line;

          // Target cursor is at the right-bottom corner in current window or
          // preview viewport.
          let at_right_bottom = if is_bottom_line {
            if let Some((_last_preview_row, last_preview_row_viewport)) =
              last_preview_line_viewport.rows().last()
            {
              // How do we detect whether target cursor is at right-bottom
              // corner?
              //
              // 1. The last row of the preview viewport is not empty
              let last_row_not_empty = last_preview_row_viewport.end_char_idx()
                > last_preview_row_viewport.start_char_idx();
              // 2. The end char of last row == `target_cursor_char`
              let at_last_row =
                last_preview_row_viewport.end_char_idx() == target_cursor_char;
              // 3. The width of last row >= `window_width`
              let last_row_full_width = text
                .width_before(
                  target_cursor_line,
                  last_preview_row_viewport.end_char_idx(),
                )
                .saturating_sub(text.width_before(
                  target_cursor_line,
                  last_preview_row_viewport.start_char_idx(),
                ))
                >= window_width as usize;

              last_row_not_empty && at_last_row && last_row_full_width
            } else {
              false
            }
          } else {
            false
          };
          (is_bottom_line, at_right_bottom)
        }
        None => (false, false),
      };

    let start_line = if cursor_is_in_bottom_line && cursor_is_at_right_bottom {
      new_start_line + 1
    } else {
      new_start_line
    };

    (start_line, start_column)
  }
}
