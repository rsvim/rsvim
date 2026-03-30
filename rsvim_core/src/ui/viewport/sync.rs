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

/// Find word index by char index.
///
/// Returns:
/// 1. The word index, which contains this `char_idx`.
/// 2. The first char index of this word.
/// 3. The end char index of this word.
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

// Part-1 of the processing algorithm in [`wrap_linebreak_line_process`].
// Returns `(end_char, end_filled_cols)`.
fn _part1(
  words: &[&str],
  words_end_char_idx: &LiteMap<usize, usize>,
  text: &Text,
  buffer_line: &RopeSlice,
  current_line: usize,
  end_width_char: usize,
  end_width: usize,
  start_char: usize,
  last_word_is_too_long: &mut Option<(usize, usize, usize, usize)>,
) -> (usize, usize) {
  let (end_wd_idx, start_c_of_end_wd, end_c_of_end_wd) =
    _find_word_by_char(words, words_end_char_idx, end_width_char);

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

    let segmenter =
      WordSegmenter::new_auto(WordBreakInvariantOptions::default());
    // Words.
    let words: Vec<&str> = segmenter
      .segment_str(&cloned_line)
      .tuple_windows()
      .map(|(i, j)| &cloned_line[i..j])
      .collect();
    // Word index => its end char index
    let words_end_char_idx = words
      .iter()
      .enumerate()
      .scan(cloned_start_char, |state, (i, wd)| {
        *state += wd.chars().count();
        Some((i, *state))
      })
      .collect::<LiteMap<usize, usize>>();

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
                    &words,
                    &words_end_char_idx,
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
                  &words,
                  &words_end_char_idx,
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

type WrapSyncFn = fn(
  /* text */ &Text,
  /* window_actual_size */ &U16Size,
  /* start_line */ usize,
  /* start_column */ usize,
) -> (
  /* line range */ ViewportLineRange,
  /* lines_viewport */ LiteMap<usize, LineViewport>,
);

// Type alias for `wrap_xxx_line_process` functions.
type WrapLineProcessFn = fn(
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

type WrapHorizontalSearchFn =
  fn(
    /* sync_fn */ WrapSyncFn,
    /* line_process_fn */ WrapLineProcessFn,
    /* viewport */ &Viewport,
    /* cursor_viewport */ &CursorViewport,
    /* opts */ &WindowOptions,
    /* text */ &Text,
    /* size */ &U16Size,
    /* suggest_start_line */ usize,
    /* suggest_start_column */ usize,
    /* target_cursor_line */ usize,
    /* target_cursor_char */ usize,
  ) -> (/* start_line */ usize, /* start_column */ usize);

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
  let target_cursor_line_end_char =
    match text.last_char_idx_on_line_include_eol(target_cursor_line) {
      Some(last_char) => {
        if !text.is_eol(target_cursor_line, last_char)
          && target_cursor_char > last_char
        {
          // If the `last_char` is not a eol, and `target_cursor_char` is still
          // greater than it. It means `target_cursor_char` wants the line end,
          // which is actually out of the line.
          // This only happens when a line doesn't end with eol (i.e. `\n` or
          // `\r\n`). If a line ends with eol (`\n` or `\r\n`), the
          // `target_cursor_char` must be less than or equal to the eol.
          last_char + 1
        } else {
          last_char
        }
      }
      None => {
        // The line is empty
        0
      }
    };
  let target_cursor_char =
    std::cmp::min(target_cursor_char, target_cursor_line_end_char);

  let (sync_fn, line_process_fn, search_left_fn, search_right_fn): (
    WrapSyncFn,
    WrapLineProcessFn,
    WrapHorizontalSearchFn,
    WrapHorizontalSearchFn,
  ) = if opts.line_break() {
    (
      wrap_linebreak_sync,
      wrap_linebreak_line_process,
      wrap_search_left,
      wrap_search_right,
    )
  } else {
    (
      wrap_nolinebreak_sync,
      wrap_nolinebreak_line_process,
      wrap_search_left,
      wrap_search_right,
    )
  };

  if target_cursor_line < cursor_viewport.line_idx() {
    // Cursor moves upward
    if opts.wrap() {
      wrap_search_up(
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
      nowrap_search_up(
        viewport,
        cursor_viewport,
        text,
        size,
        target_cursor_line,
        target_cursor_char,
      )
    }
  } else {
    // Cursor moves downward, or just moves to left/right side. But in this
    // algorithm, we have to moves to downward (even just for 0-lines) before
    // moving to left/right side.
    if opts.wrap() {
      wrap_search_down(
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
      nowrap_search_down(
        viewport,
        cursor_viewport,
        text,
        size,
        target_cursor_line,
        target_cursor_char,
      )
    }
  }
}

// Returns if current viewport contains the `target_cursor_line`.
fn _if_contains_target_cursor_line(
  viewport: &Viewport,
  target_cursor_line: usize,
) -> bool {
  target_cursor_line >= viewport.start_line_idx()
    && target_cursor_line < viewport.end_line_idx()
}

// Returns whether the `target_cursor_line` is:
// 1. If the window cannot even contain it, because it is just too long.
// 2. If the window can exactly contain it, i.e. it will use the same rows that
//    equals to the window height.
fn _can_fully_contain_target_cursor_line(
  line_process_fn: WrapLineProcessFn,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
) -> (bool, bool) {
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
    // Note: here use window_height + 3 to give a larger window, to test if the
    // line can use more rows.
    window_height.saturating_add(3),
    window_width,
  );

  // Current window cannot contain the target cursor line, i.e. target cursor
  // line is just too long to be put in current window.
  let cannot_fully_contain_target_cursor_line =
    preview_target_rows.len() > window_height as usize;

  // Current window can exactly contain the target cursor line, i.e. target
  // cursor line just happens to use all the rows in current window.
  let can_exactly_contain_target_cursor_line =
    preview_target_rows.len() == window_height as usize;

  (
    cannot_fully_contain_target_cursor_line,
    can_exactly_contain_target_cursor_line,
  )
}

fn nowrap_search_down(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_height = size.height();

  // Try to keep current `viewport.start_line_idx` unchanged, this will keep
  // the viewport scrolls as small as we can, and thus avoid too big jumps for
  // users' eye.
  let already_contains_target_cursor_line =
    _if_contains_target_cursor_line(viewport, target_cursor_line);

  let current_cursor_column =
    text.width_before(cursor_viewport.line_idx(), cursor_viewport.char_idx());
  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  if already_contains_target_cursor_line {
    // Yes it contains, this means we don't have to scroll the window viewport,
    // we can still use the `viewport.start_line_idx` as the first line for
    // the new viewport.

    let start_line = viewport.start_line_idx();
    let start_column = viewport.start_column_idx();

    if target_cursor_column < current_cursor_column {
      trace!(
        "viewport:{}/{},cursor_viewport:{:?},start_line/column:{}/{},target_cursor:{}/{}",
        viewport.start_line_idx(),
        viewport.start_column_idx(),
        cursor_viewport,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      );
      // Cursor moves to left side.
      nowrap_search_left(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // Cursor moves to right side (even just for 0-chars).
      nowrap_search_right(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  } else {
    // Otherwise `target_cursor_line` is out of current viewport. We have to
    // find out the correct first line for the new viewport.

    let start_line = std::cmp::max(
      0,
      (target_cursor_line as isize) - (window_height as isize) + 1,
    ) as usize;
    let start_column = viewport.start_column_idx();

    if target_cursor_column < current_cursor_column {
      // To left side
      nowrap_search_left(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // To right side
      nowrap_search_right(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  }
}

fn nowrap_search_up(
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  // Try to keep current `viewport.start_line_idx` unchanged, this will keep
  // the viewport scrolls as small as we can, and thus avoid too big jumps for
  // users' eye.
  let already_contains_target_cursor_line =
    _if_contains_target_cursor_line(viewport, target_cursor_line);

  let current_cursor_column =
    text.width_before(cursor_viewport.line_idx(), cursor_viewport.char_idx());
  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  if already_contains_target_cursor_line {
    // Yes it contains, this means we don't have to scroll the window viewport,
    // we can still use the `viewport.start_line_idx` as the first line for the
    // new viewport.

    let start_line = viewport.start_line_idx();
    let start_column = viewport.start_column_idx();

    if target_cursor_column < current_cursor_column {
      // Cursor moves to left side.
      nowrap_search_left(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // Cursor moves to right side (even just for 0-chars).
      nowrap_search_right(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  } else {
    // Otherwise `target_cursor_line` is out of current viewport. We have to
    // find out the correct first line for the new viewport.

    let start_line = target_cursor_line;
    let start_column = viewport.start_column_idx();

    if target_cursor_column < current_cursor_column {
      // To left side
      nowrap_search_left(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    } else {
      // To right side
      nowrap_search_right(
        text,
        size,
        start_line,
        start_column,
        target_cursor_line,
        target_cursor_char,
      )
    }
  }
}

// When cursor moves to downward and scrolls the window, we need to set a
// larger `start_line` for the new viewport to "contain" the target cursor line
// inside the window.
//
// In such case, how could we know what `start_line` we should use for the new
// viewport?
//
// We still iterate the lines (in the buffer) one by one, but from the
// `target_cursor_line` reversely, from bottom to top, until we find the first
// line which cannot "contain" the `target_cursor_line` any more. Then the
// `first_line + 1` is our `start_line`.
//
// Returns `start_line` for the new viewport.
fn _reverse_search_target_cursor_line(
  sync_fn: WrapSyncFn,
  line_process_fn: WrapLineProcessFn,
  viewport: &Viewport,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> usize {
  let window_height = size.height();
  let window_width = size.width();

  let mut current_row: usize = 0;
  let mut current_line: isize = target_cursor_line as isize;

  // Here we set the `current_line` iterates start from `target_cursor_line`,
  // until it cannot fully renders the `target_cursor_line`.
  //
  // In most happy case, the `current_line + 1` will be the `start_line` for
  // new viewport, the `target_cursor_line` will just be the last line in the
  // new viewport, which looks good for users.
  // For example:
  //
  // ```
  //  AAAAAAAAAA    <- current_line
  // +----------+
  // |BBBBBBBBBB|   <- current_line + 1
  // |BBBBBB.\n |
  // |CCCCCCCCCC|   <- target_cursor_line
  // |CCC.\n    |
  // +----------+
  // ```

  while (current_row < window_height as usize) && (current_line >= 0) {
    let (rows, _start_fills, _end_fills, _last_row) = line_process_fn(
      text,
      0,
      current_line as usize,
      0,
      window_height,
      window_width,
    );
    current_row += rows.len();
    current_line -= 1;
  }

  // Here we have an edge case: the `target_cursor_line` is partially rendered,
  // i.e. the `target_cursor_line` will not be fully shown in the new viewport.
  // For example:
  //
  // ```
  //  AAAAAAAAAA    <- current_line
  // +----------+
  // |BBBBBBBBBB|   <- current_line + 1
  // |BBBBBBBBBB|
  // |BB.\n     |
  // |CCCCCCCCCC|   <- target_cursor_line
  // +----------+
  //  CCC.\n
  // ```
  //
  // When `wrap = true` we always try to put the entire `target_cursor_line`
  // inside the window/viewport. So for this case, we use `current_line + 2`
  // as `start_line` for the new viewport.

  let start_line = if current_row > window_height as usize {
    debug_assert!(current_line + 2 <= target_cursor_line as isize);
    current_line + 2
  } else {
    debug_assert!(current_line < target_cursor_line as isize);
    current_line + 1
  };
  debug_assert!(start_line >= 0);
  debug_assert!(start_line <= target_cursor_line as isize);
  let start_line =
    std::cmp::max(viewport.start_line_idx(), start_line as usize);

  // Here we have another edge case: the `target_cursor_line` is fully rendered,
  // but `target_cursor_char` is eol or line end. Since our rendering algorithm
  // will not render eol (`\n` or `\r\n`), while in **insert** mode, cursor
  // will want the line end position (for appending characters at the end of
  // line). And it also happens that `target_cursor_line` is the last line, so
  // the cursor actually wants a position that is next to the right-bottom
  // corner of the window. For example:
  //
  // ```
  //  AAAAAAAAAA      <- current_line
  // +----------+
  // |BBBBBBBBBB|     <- current_line + 1
  // |BBBBBBBBBB|
  // |BB.\n     |
  // |CCCCCCCCCC|_\n  <- target_cursor_line, `_` is target_cursor_char
  // +----------+
  //  DDDDDD.\n
  // ```
  //
  // And there are two sub-cases:
  // 1. If the `target_cursor_line` is just too long to be put in current
  //    window. Then the current window will only have 1 line, i.e. the
  //    `target_cursor_line`. And we don't have to do anything, leave the
  //    left/right movements to other methods.
  // 2. If the `target_cursor_line` is not too long, and current window can
  //    contain more than 1 lines, include the `target_cursor_line`, just like
  //    the above example. Then we move 1 more line down to ensure the
  //    `target_cursor_char` can be safely put to the next row.
  //
  //     ```
  //      AAAAAAAAAA      <- current_line
  //      BBBBBBBBBB      <- current_line + 1
  //      BBBBBBBBBB
  //      BB.\n
  //     +----------+
  //     |CCCCCCCCCC|_\n  <- target_cursor_line, `_` is target_cursor_char
  //     |*DDDDD.\n |     <- `*` is the cursor rendered in terminal, it is put
  //     |          |        to next row.
  //     |          |
  //     +----------+
  //     ```

  let (
    cannot_fully_contain_target_cursor_line,
    can_exactly_contain_target_cursor_line,
  ) = _can_fully_contain_target_cursor_line(
    line_process_fn,
    text,
    size,
    target_cursor_line,
  );

  if cannot_fully_contain_target_cursor_line
    || can_exactly_contain_target_cursor_line
  {
    return start_line;
  }

  let (_preview_line_range, preview_viewport) =
    sync_fn(text, size, start_line, 0);
  debug_assert!(preview_viewport.last().is_some());
  let (last_preview_line, last_preview_line_viewport) =
    preview_viewport.last().unwrap();

  let target_cursor_char_is_at_right_bottom_corner = {
    // Target cursor line is the last line in preview viewport.
    let is_last_line = *last_preview_line == target_cursor_line;

    // Target cursor char is at right-bottom corner in the preview viewport.
    if is_last_line {
      if let Some((_last_preview_row, last_preview_row_viewport)) =
        last_preview_line_viewport.rows().last()
      {
        // 1. The last row of the preview viewport is not empty
        let last_row_not_empty = last_preview_row_viewport.end_char_idx()
          > last_preview_row_viewport.start_char_idx();

        // 2. The end char of last row <= `target_cursor_char`
        let at_last_row =
          last_preview_row_viewport.end_char_idx() <= target_cursor_char;

        // 3. The width of last row >= `window_width`
        let last_row_end_column = text.width_before(
          target_cursor_line,
          last_preview_row_viewport.end_char_idx(),
        );
        let last_row_start_column = text.width_before(
          target_cursor_line,
          last_preview_row_viewport.start_char_idx(),
        );
        let last_row_width =
          last_row_end_column.saturating_sub(last_row_start_column);
        let last_row_use_full_width = last_row_width >= window_width as usize;

        last_row_not_empty && at_last_row && last_row_use_full_width
      } else {
        false
      }
    } else {
      false
    }
  };

  let start_line = if target_cursor_char_is_at_right_bottom_corner {
    // Add 1 more line, but don't be greater than `target_cursor_line` itself.
    start_line + 1
  } else {
    start_line
  };
  std::cmp::min(start_line, target_cursor_line)
}

fn wrap_search_down(
  sync_fn: WrapSyncFn,
  line_process_fn: WrapLineProcessFn,
  search_left_fn: WrapHorizontalSearchFn,
  search_right_fn: WrapHorizontalSearchFn,
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let current_cursor_column =
    text.width_before(cursor_viewport.line_idx(), cursor_viewport.char_idx());
  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  let start_line = _reverse_search_target_cursor_line(
    sync_fn,
    line_process_fn,
    viewport,
    text,
    size,
    target_cursor_line,
    target_cursor_char,
  );
  let start_column = viewport.start_column_idx();

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
      start_line,
      start_column,
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
      start_line,
      start_column,
      target_cursor_line,
      target_cursor_char,
    )
  }
}

fn wrap_search_up(
  sync_fn: WrapSyncFn,
  line_process_fn: WrapLineProcessFn,
  search_left_fn: WrapHorizontalSearchFn,
  search_right_fn: WrapHorizontalSearchFn,
  viewport: &Viewport,
  cursor_viewport: &CursorViewport,
  opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let current_cursor_column =
    text.width_before(cursor_viewport.line_idx(), cursor_viewport.char_idx());
  let target_cursor_column =
    text.width_before(target_cursor_line, target_cursor_char);

  let start_line = std::cmp::min(target_cursor_line, viewport.start_line_idx());
  let start_column = viewport.start_column_idx();

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
      start_line,
      start_column,
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
      start_line,
      start_column,
      target_cursor_line,
      target_cursor_char,
    )
  }
}

// For/to leftward
fn _find_target_cursor_column_exclude_eol(
  text: &Text,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> usize {
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
    if cfg!(debug_assertions) {
      if let Some(last_char_exclude_eol) =
        text.last_char_idx_on_line_exclude_eol(target_cursor_line)
      {
        debug_assert!(
          target_cursor_char > last_char_exclude_eol
            && target_cursor_char <= last_char_exclude_eol + 2
        );
      }
    }
    let last_char_exclude_eol = text
      .last_char_idx_on_line_exclude_eol(target_cursor_line)
      .unwrap_or(0);
    target_cursor_column =
      text.width_before(target_cursor_line, last_char_exclude_eol);
  }

  target_cursor_column
}

// For/to rightward
fn _find_target_cursor_column_include_eol(
  text: &Text,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> usize {
  let out_of_line =
    text.is_eol_or_line_end(target_cursor_line, target_cursor_char);

  // For eol or line-end, add 1 more column
  text.width_until(target_cursor_line, target_cursor_char)
    + if out_of_line { 1 } else { 0 }
}

fn nowrap_search_left(
  text: &Text,
  _size: &U16Size,
  suggest_start_line: usize,
  suggest_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let mut suggest_start_column = suggest_start_column;
  let target_cursor_column = _find_target_cursor_column_exclude_eol(
    text,
    target_cursor_line,
    target_cursor_char,
  );

  if target_cursor_column < suggest_start_column {
    suggest_start_column = target_cursor_column;
  }

  trace!(
    "suggest_line/column:{}/{},target_line/char/column:{}/{}/{}",
    suggest_start_line,
    suggest_start_column,
    target_cursor_line,
    target_cursor_char,
    target_cursor_column,
  );
  (suggest_start_line, suggest_start_column)
}

fn nowrap_search_right(
  text: &Text,
  size: &U16Size,
  suggest_start_line: usize,
  suggest_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_width = size.width();
  let suggest_end_column = suggest_start_column + window_width as usize;
  let mut suggest_start_column = suggest_start_column;

  let target_cursor_column = _find_target_cursor_column_include_eol(
    text,
    target_cursor_line,
    target_cursor_char,
  );
  if target_cursor_column > suggest_end_column {
    suggest_start_column =
      target_cursor_column.saturating_sub(window_width as usize);
  }

  // This "search_right" method can be called if the
  // `target_cursor_column == current_cursor_column`, which means this method
  // also need to consider non-right case, or even search to leftward case.
  //
  // If in `target_cursor_line`, the `target_cursor_char` is already the last
  // char, and it is eol or line end, and there is no other visible char in
  // `target_cursor_line`, we should actually move the `suggest_start_column`
  // to leftward for 1 visible char, to ensure the `target_cursor_line`
  // contains at least 1 visible char.
  let last_char = text
    .last_char_idx_on_line_exclude_eol(target_cursor_line)
    .unwrap_or(0);
  let last_char_column = text.width_before(target_cursor_line, last_char);
  let suggest_start_column =
    std::cmp::min(suggest_start_column, last_char_column);

  (suggest_start_line, suggest_start_column)
}

fn wrap_search_left(
  _sync_fn: WrapSyncFn,
  line_process_fn: WrapLineProcessFn,
  _viewport: &Viewport,
  _cursor_viewport: &CursorViewport,
  _opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  suggest_start_line: usize,
  suggest_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_height = size.height();
  let window_width = size.width();

  let (
    cannot_fully_contain_target_cursor_line,
    can_exactly_contain_target_cursor_line,
  ) = _can_fully_contain_target_cursor_line(
    line_process_fn,
    text,
    size,
    target_cursor_line,
  );

  let target_cursor_column = _find_target_cursor_column_exclude_eol(
    text,
    target_cursor_line,
    target_cursor_char,
  );

  if cannot_fully_contain_target_cursor_line
    || can_exactly_contain_target_cursor_line
  {
    // For `start_line`, force it to be `target_cursor_line`, because viewport
    // can only contain this line (and still cannot put all of it inside).
    let start_line = target_cursor_line;

    // For `start_column`, it seems that we only need to pick the smaller one
    // between `target_cursor_column` and `suggest_start_column`. But there is an
    // edge case we need to consider, for example:
    //
    // ```
    //           +----------+
    // AAAAAAAAAA|AAAAAAAAAA|     <- line-0
    //           |AAAAAAAAAA|
    //           |AAAAAAAAA_|\n   <- Now cursor is at line-0, char-39. `start_column` is 10.
    //           +----------+
    //            BBBBBBBBBB      <- line-1
    //            BBBBBBBBBB
    //            BBBBBBBBBB
    //            BB_\n           <- Cursor wants line-1, char-32.
    // ```
    //
    // Cursor wants to move down to line-1, move left to char-32.
    // In such case, the `suggest_start_column` is 10 (it is the current viewport
    // `start_column`), the `target_cursor_column` is 32. If we simply use the
    // smaller one between `suggest_start_column` and `target_cursor_column`, then
    // the `start_column` is 10. And the viewport becomes:
    //
    // ```
    //            AAAAAAAAAA     <- line-0
    //            AAAAAAAAAA
    //            AAAAAAAAAA
    //            AAAAAAAAA\n    <- Previous cursor is at line-0, char-39.
    //           +----------+
    // BBBBBBBBBB|BBBBBBBBBB|    <- line-1
    //           |BBBBBBBBBB|
    //           |BB_\n     |    <- Cursor is at line-1, char-32, `start_column` is 10.
    //           +----------+
    // ```
    //
    // The looking is weird because there still are some empty columns left at
    // the end of the last row, while at the beginning of the line, 10 `B`
    // characters are not rendered in the viewport. The window spaces are
    // wasted.

    let start_column =
      std::cmp::min(suggest_start_column, target_cursor_column);

    // So we try to do some more additional leftward movement on
    // the `target_cursor_column`, to make give the new viewport can
    // contain the `target_cursor_char`.
    let target_cursor_line_end_char = text
      .last_char_idx_on_line_include_eol(target_cursor_line)
      .unwrap_or(0);
    let target_cursor_line_end_column = text
      .width_until(target_cursor_line, target_cursor_line_end_char)
      + if text
        .is_eol_or_line_end(target_cursor_line, target_cursor_line_end_char)
      {
        1
      } else {
        0
      };
    let target_cursor_line_start_column = target_cursor_line_end_column
      .saturating_sub((window_width as usize) * (window_height as usize));

    // NOTE: only contain 1 line
    debug_assert!(
      cannot_fully_contain_target_cursor_line
        || can_exactly_contain_target_cursor_line
    );
    let target_cursor_line_start_column = _reverse_search_start_column(
      line_process_fn,
      text,
      size,
      start_line,
      target_cursor_line_start_column,
      target_cursor_line,
      target_cursor_line_end_char,
    );

    let start_column =
      std::cmp::min(start_column, target_cursor_line_start_column);

    (start_line, start_column)
  } else {
    // For `start_column`, force it to be 0.
    let start_column = 0;
    let start_line = suggest_start_line;

    (start_line, start_column)
  }
}

// By the formula:
//
// ```
// target_cursor_start_column = target_cursor_end_column - (window_height * window_width)
// ```
//
// It cannot handle the potential empty columns at the end of rows, "potential"
// means:
//
// 1. Bad case: There are some empty columns. In this case, the final rendered
//    columns of `[target_cursor_start_column,target_cursor_end_column)` (or
//    `[target_cursor_start_column,target_cursor_char)`) will be longer than
//    `(window_height * window_width)`, because these extra empty columns are
//    wasted.
// 2. Good case: There are no empty columns. In this case, the final rendered
//    columns is exactly `(window_height * window_width)`.
//
// We can see the `target_cursor_start_column` can be still or larger, but
// can never be smaller.
//
// This method try to use a larger `start_column`, based on the
// `target_cursor_start_column` we calculated with above formula. It repeatedly
// searches to rightward by `target_cursor_start_column += 1`, and check if the
// result are better.
fn _reverse_search_start_column(
  line_process_fn: WrapLineProcessFn,
  text: &Text,
  size: &U16Size,
  _suggest_start_line: usize,
  suggest_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> usize {
  let window_height = size.height();
  let window_width = size.width();
  let bufline = text.rope().line(target_cursor_line);
  let bufline_len_char = bufline.len_chars();
  let bufline_chars_width =
    text.width_until(target_cursor_line, bufline_len_char);

  let eol_or_line_end =
    text.is_eol_or_line_end(target_cursor_line, target_cursor_char);
  let mut suggest_start_column = suggest_start_column;

  while suggest_start_column < bufline_chars_width {
    let (preview_target_rows, _preview_start_fills, _preview_end_fills, _) =
      line_process_fn(
        text,
        suggest_start_column,
        target_cursor_line,
        0_u16,
        window_height,
        window_width,
      );

    // This method is only used when `wrap = true`, and the line is long enough
    // that 1 single line uses the entier window/viewport.
    debug_assert!(preview_target_rows.len() <= window_height as usize);

    // If this preview viewport can contain `target_cursor_char`.
    let contains_target_cursor_char =
      preview_target_rows.iter().any(|(_row_idx, row_viewport)| {
        target_cursor_char >= row_viewport.start_char_idx()
          && target_cursor_char < row_viewport.end_char_idx()
      });
    if contains_target_cursor_char {
      return suggest_start_column;
    }

    // 1. If `target_cursor_char` is eol or line end
    // 2. If this preview viewport last row has the eol or line end
    // 3. The last row doesn't use all the columns in the row, i.e. it has at
    //    least 1 empty column to put the `target_cursor_char` at line end.
    if eol_or_line_end {
      let contains_target_cursor_char_as_eol =
        preview_target_rows.iter().any(|(_row_idx, row_viewport)| {
          target_cursor_char == row_viewport.end_char_idx()
        });

      if contains_target_cursor_char_as_eol {
        // The width of last row == `window_width`, i.e. the last row already
        // uses all columns (full width).
        // In such case, if the `target_cursor_char` is eol, we will need to
        // give it 1 more column for it.
        let last_row_use_full_width = {
          debug_assert!(preview_target_rows.last().is_some());
          let (_last_preview_row_idx, last_preview_row_viewport) =
            preview_target_rows.last().unwrap();
          let last_row_end_column = text.width_before(
            target_cursor_line,
            last_preview_row_viewport.end_char_idx(),
          );
          let last_row_start_column = text.width_before(
            target_cursor_line,
            last_preview_row_viewport.start_char_idx(),
          );
          let last_row_width =
            last_row_end_column.saturating_sub(last_row_start_column);
          last_row_width >= window_width as usize
        };

        if !last_row_use_full_width {
          return suggest_start_column;
        }
      }
    }

    suggest_start_column += 1;
  }

  unreachable!()
}

fn wrap_search_right(
  _sync_fn: WrapSyncFn,
  line_process_fn: WrapLineProcessFn,
  _viewport: &Viewport,
  _cursor_viewport: &CursorViewport,
  _opts: &WindowOptions,
  text: &Text,
  size: &U16Size,
  suggest_start_line: usize,
  suggest_start_column: usize,
  target_cursor_line: usize,
  target_cursor_char: usize,
) -> (usize, usize) {
  let window_height = size.height();
  let window_width = size.width();

  let (
    cannot_fully_contain_target_cursor_line,
    can_exactly_contain_target_cursor_line,
  ) = _can_fully_contain_target_cursor_line(
    line_process_fn,
    text,
    size,
    target_cursor_line,
  );

  let target_cursor_column =
    text.width_until(target_cursor_line, target_cursor_char);

  if cannot_fully_contain_target_cursor_line
    || can_exactly_contain_target_cursor_line
  {
    // For `start_line`, force it to be `target_cursor_line`, because viewport
    // can only contain this line (and still cannot put all of it inside).
    let start_line = target_cursor_line;

    // For `start_column`, calculate the `target_cursor_start_column` based on
    // the `target_cursor_column` as the end column in the window.
    //
    // But when `wrap = true` (no matter `linebreak` is `true` or `false`),
    // there is an edge case, that some character or word can leaves columns
    // not fully filled at the end of a window row.
    //
    // When `wrap = true, linebreak = false`:
    //
    // Example-1.1
    //
    // ```
    // +----------+
    // |AAAAAAAA<<|   <- row-0
    // |\tBBB.\n  |   <- row-1
    // +----------+
    // ```
    //
    // Example-1.2
    //
    // ```
    // +----------+
    // |AAAAAAAAA<|   <- row-0
    // |你好.\n   |   <- row-1
    // +----------+
    // ```
    //
    // For example-1.1, at the end of row-0, there are 2 empty columns because
    // the following character `\t` are rendered as 8 columns, but the 2
    // columns at the end of row-0 is not enough for `\t`. The rendering
    // algorithm force to put the `\t` to row-1, and leaves 2 empty columns at
    // the end of row-0.
    //
    // For example-1.2, there are 1 empty column at the end of row-0, because
    // the following character `你` are rendered as 2 columns, but the 1 column
    // at the end of row-0 is not enough.
    //
    // When `wrap = true, linebreak = true`:
    //
    // Example-2.1
    //
    // ```
    // +----------+
    // |This is <<|   <- row-0
    // |ours.\n   |   <- row-1
    // +----------+
    // ```
    //
    // There are 2 empty columns at the end of row-0, because the following
    // word `ours` are rendered as 4 columns, and the rendering algorithm
    // force to put a word in 1 row when `linebreak=true`, thus it leaves 2
    // empty columns at the end of row-0.

    // If `target_cursor_char` is a eol or line end, we move to right for 1
    // more column to allow the invisible eol or line end.
    let target_cursor_end_column =
      if text.is_eol_or_line_end(target_cursor_line, target_cursor_char) {
        target_cursor_column + 1
      } else {
        target_cursor_column
      };

    // In such case, we cannot simply use `target_cursor_end_column -
    // (window_height * window_width)` to calculate the
    // `target_cursor_start_column`.
    let target_cursor_start_column = target_cursor_end_column
      .saturating_sub((window_width as usize) * (window_height as usize));

    // NOTE: only contain 1 line
    debug_assert!(
      cannot_fully_contain_target_cursor_line
        || can_exactly_contain_target_cursor_line
    );
    // We try to do some more additional rightward movement on
    // `target_cursor_start_column`, to make sure the new viewport can
    // contain the `target_cursor_char`.
    let target_cursor_start_column = _reverse_search_start_column(
      line_process_fn,
      text,
      size,
      start_line,
      target_cursor_start_column,
      target_cursor_line,
      target_cursor_char,
    );

    let start_column =
      std::cmp::max(suggest_start_column, target_cursor_start_column);

    (start_line, start_column)
  } else {
    // For `start_column`, force it to be 0.
    let start_column = 0;
    let start_line = suggest_start_line;

    (start_line, start_column)
  }
}
