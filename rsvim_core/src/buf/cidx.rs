//! Display width index (char-wise) for each unicode char in vim buffer.

use crate::buf::opt::BufferLocalOptions;
use crate::buf::unicode;
use ropey::RopeSlice;

use std::collections::BTreeMap;
// use tracing::trace;

#[derive(Debug, Default, Clone)]
/// Display width index (char-wise) for each unicode char in vim buffer. For each line, the
/// char/column index starts from 0.
///
/// This structure is actually a prefix-sum tree structure. For example now we have a line:
///
/// ```text
///                           25
/// 0      7       15       25|
/// |      |       |         ||
/// This is<--HT-->an example.\n
/// |      |                 ||
/// 0      7                18|
///                           19
/// ```
///
/// Here we have some facts:
/// 1. The first char (`T`) index is 0, the display width of char range `[0,0]` is 1.
/// 2. The char (`.`) before the last char index is 18, the display width of char range `[0,18]` is
///    25, there's a tab char (`\t`) which display width is 8 cells.
/// 3. The last char (`\n`) index is 19, the display width of char range `[0,19]` is also 25,
///    because the last char display width is 0 cells.
///
/// Here we have below terms:
/// - **Prefix (Display) Width**: the display width from the first char to current char, inclusive
///   on both side.
pub struct ColumnIndex {
  // Char index maps to its prefix display width.
  char2width: Vec<usize>,

  // Prefix display width maps to the right-most char index, i.e. the reversed mapping of
  // `char2width`.
  //
  // NOTE:
  // 1. Some unicodes use more than 1 cells, thus the keys/widths could be non-continuous.
  // 2. Some unicodes use 0 cells (such as LF, CR), thus multiple char index could have same width.
  //    In such case, the width maps to the right-most char index, i.e. try to cover wider char
  //    index range.
  width2char: BTreeMap<usize, usize>,
}

impl ColumnIndex {
  /// Create new index.
  pub fn new() -> Self {
    Self {
      char2width: Vec::new(),
      width2char: BTreeMap::new(),
    }
  }

  #[cfg(not(debug_assertions))]
  pub fn _internal_check(&self) {}

  #[cfg(debug_assertions)]
  pub fn _internal_check(&self) {
    // Check length.
    assert!(self.char2width.len() >= self.width2char.len());

    // Check indexing.
    let mut last_width: Option<usize> = None;
    for (i, w) in self.char2width.iter().enumerate() {
      match last_width {
        Some(last_width1) => {
          assert!(*w >= last_width1);
        }
        None => { /* Skip */ }
      }
      last_width = Some(*w);
      assert!(self.width2char.contains_key(w));
      let c = self.width2char[w];
      // trace!("char2width[{i}]:{w:?}, width2char[{w}]:{c:?}");
      assert!(i >= c);
    }
  }

  // Build cache beyond the bound by `char_idx` or `width`.
  fn _build_cache(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx_bound: Option<usize>,
    width_bound: Option<usize>,
  ) {
    let n = rope_line.len_chars();

    let start_idx = self.char2width.len();
    let mut prefix: usize = if start_idx == 0 {
      0_usize
    } else {
      self.char2width[start_idx - 1]
    };

    let mut rope_chars = rope_line.chars_at(start_idx);
    for i in start_idx..n {
      let c = rope_chars.next().unwrap();
      prefix += unicode::char_width(options, c);

      // Update `char2width`
      self.char2width.push(prefix);

      // Update `width2char`
      let c = self.char2width.len() - 1;
      debug_assert_eq!(i, c);
      match self.width2char.get(&prefix) {
        Some(c1) => {
          if *c1 > c {
            self.width2char.insert(prefix, c);
          }
        }
        None => {
          self.width2char.insert(prefix, c);
        }
      }

      if let Some(char_idx) = char_idx_bound {
        if i > char_idx {
          return;
        }
      }
      if let Some(width) = width_bound {
        if prefix > width {
          return;
        }
      }
    }
  }

  // Build cache until `char_idx`.
  fn _build_cache_until_char(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx: usize,
  ) {
    self._build_cache(options, rope_line, Some(char_idx), None);
  }

  /// Get the prefix display width in char index range `[0,char_idx)`, left-inclusive and
  /// right-exclusive.
  ///
  /// NOTE: This is equivalent to `width_until(char_idx-1)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if the `char_idx` is out of the line, there're below cases:
  ///    - The `char_idx` is 0.
  ///    - The line is empty.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than the line
  ///    length.
  pub fn width_before(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx: usize,
  ) -> usize {
    self._build_cache_until_char(options, rope_line, char_idx);
    self._internal_check();

    if char_idx == 0 {
      0
    } else if self.char2width.is_empty() {
      assert_eq!(rope_line.len_chars(), 0);
      0
    } else {
      assert!(!self.char2width.is_empty());
      assert!(rope_line.len_chars() > 0);
      if char_idx - 1 < self.char2width.len() {
        // Find width from the cache.
        self.char2width[char_idx - 1]
      } else {
        // If outside of the cache, returns the whole width.
        self.char2width[self.char2width.len() - 1]
      }
    }
  }

  /// Get the display width in char index range `[0,char_idx]`, both sides are inclusive.
  ///
  /// NOTE: This is equivalent to `width_before(char_idx+1)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if the `char_idx` is out of the line, there're below cases:
  ///    - The line is empty.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than or equal to
  ///    the line length.
  pub fn width_at(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx: usize,
  ) -> usize {
    self._build_cache_until_char(options, rope_line, char_idx);
    self._internal_check();

    if self.char2width.is_empty() {
      assert_eq!(rope_line.len_chars(), 0);
      0
    } else {
      assert!(!self.char2width.is_empty());
      assert!(rope_line.len_chars() > 0);
      if char_idx < self.char2width.len() {
        // Find width from the cache.
        self.char2width[char_idx]
      } else {
        // If outside of the cache, returns the whole width.
        self.char2width[self.char2width.len() - 1]
      }
    }
  }

  // Build cache until specified `width`.
  fn _build_cache_until_width(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    width: usize,
  ) {
    self._build_cache(options, rope_line, None, Some(width));
  }

  /// Get the **previous** char index which the width is less than the specified width.
  ///
  /// NOTE: A unicode char's width can also be 0 (line-break), 2 (Chinese/Japanese/Korean char) and
  /// 8 (default tab). The **current** char index is the one that its width range covers the
  /// specified `width`. The **previous** char is the one before the **current**, the **next** char
  /// is the one after the **current**.
  ///
  /// # Return
  ///
  /// 1. It returns None if the `width` is out of the line, there're below cases:
  ///    - The line is empty.
  ///    - The `width` is too small thus there's no such char exists. For example:
  ///      - The `width` is 0, and there's no 0-width chars (e.g. line-break) before it.
  ///      - The `width` is 1 (positive integer), but the 1st char is CJK unicode, its display
  ///        width is 2 thus there's still no such char exists.
  ///    - The `width` is greater than the whole line's display width, thus there's no such char
  ///      exists.
  /// 2. It returns the previous char index otherwise.
  pub fn char_before(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, rope_line, width);
    self._internal_check();

    if width == 0 {
      None
    } else if self.width2char.is_empty() {
      assert_eq!(rope_line.len_chars(), 0);
      None
    } else {
      assert!(!self.width2char.is_empty());
      assert!(rope_line.len_chars() > 0);

      let (last_width, _last_char_idx) = self.width2char.last_key_value().unwrap();
      if width > *last_width {
        return None;
      }

      for w in (1..width).rev() {
        if let Some(c) = self.width2char.get(&w) {
          return Some(*c);
        }
      }

      // Not exist.
      None
    }
  }

  /// Get the **current** char index which the width range covers the specified `width`.
  ///
  /// NOTE: For the term **current**, please refer to [`ColumnIndex::char_before`].
  ///
  /// # Return
  ///
  /// 1. It returns None if the `width` is out of the line, there're below cases:
  ///    - The line is empty.
  ///    - The `width` is greater than the whole line's display width, thus there's no such char
  ///      exists.
  ///    - The `width` is 0, and the 1st char is not 0-width char (e.g. line-break).
  /// 2. It returns the **current** char index otherwise.
  pub fn char_at(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, rope_line, width);
    self._internal_check();

    if self.width2char.is_empty() {
      assert_eq!(rope_line.len_chars(), 0);
      None
    } else {
      assert!(!self.width2char.is_empty());
      assert!(rope_line.len_chars() > 0);

      if width == 0 {
        if *self.char2width.first().unwrap() == 0 {
          return Some(0);
        } else {
          return None;
        }
      }

      let (last_width, _last_char_idx) = self.width2char.last_key_value().unwrap();
      if width > *last_width {
        return None;
      }

      for w in width..=*last_width {
        if let Some(c) = self.width2char.get(&w) {
          return Some(*c);
        }
      }

      // Not exist.
      None
    }
  }

  /// Get the **next** char index which is next to (after) the **current** char, which the width
  /// range covers the specified `width`.
  ///
  /// NOTE: For the term **next** and **current**, please refer to [`ColumnIndex::char_before`].
  ///
  /// # Return
  ///
  /// 1. It returns None if the `width` is out of the line, there're below cases:
  ///    - The line is empty.
  ///    - The `width` is greater than the whole line's display width, thus there's no such char
  ///      exists.
  /// 2. It returns the next char index otherwise.
  pub fn char_after(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, rope_line, width + 1);
    self._internal_check();

    let n = rope_line.len_chars();
    if self.char2width.is_empty() {
      assert_eq!(n, 0);
      return None;
    }

    if width == 0 {
      return Some(0);
    }

    if let Some(char_idx) = self.char_at(options, rope_line, width) {
      if char_idx < n {
        return Some(char_idx + 1);
      }
    }

    None
  }

  /// Get the last char index which the width is less than or equal to the specified `width`.
  ///
  /// # Return
  ///
  /// 1. It returns None if the `width` is out of the line, there're below cases:
  ///    - The line is empty.
  /// 2. It returns the last char index otherwise. If the `width` is longer than the whole line, it
  ///    returns the last char index.
  pub fn last_char_until(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, rope_line, width);
    self._internal_check();

    let (last_width, last_char_idx) = self.width2char.last_key_value().unwrap();
    if width > *last_width {
      return Some(*last_char_idx);
    } else if let Some(char_idx) = self.char_at(options, rope_line, width) {
      return Some(char_idx);
    }

    None
  }

  /// Truncate cache since char index.
  pub fn truncate_since_char(&mut self, char_idx: usize) {
    self._internal_check();

    if self.char2width.is_empty() || self.width2char.is_empty() {
      debug_assert_eq!(self.char2width.is_empty(), self.width2char.is_empty());
    } else if char_idx < self.char2width.len() {
      self.char2width.truncate(char_idx);
      self.width2char.retain(|&_w, &mut c| c < char_idx);
    }
  }

  /// Truncate cache since width.
  pub fn truncate_since_width(&mut self, width: usize) {
    self._internal_check();

    if self.char2width.is_empty() || self.width2char.is_empty() {
      debug_assert_eq!(self.char2width.is_empty(), self.width2char.is_empty());
    } else {
      let (last_width, _last_char_idx) = self.width2char.last_key_value().unwrap();
      if width <= *last_width {
        for w in (1..=width).rev() {
          match self.width2char.get(&w) {
            Some(c) => {
              self.truncate_since_char(*c);
              return;
            }
            None => { /* Skip */ }
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::test::buf::{make_buffer_from_rope, make_rope_from_lines, print_buffer_line_details};
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  use ropey::Rope;
  use tracing::info;

  fn assert_width_at(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[usize],
  ) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width_at(options, rope_line, i);
      info!("width_at:{i} actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_at_rev(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[(usize, usize)],
  ) {
    for (e, i) in expect.iter() {
      let a = actual.width_at(options, rope_line, *i);
      info!("width_at_rev:{i}, actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_before(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[usize],
  ) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width_before(options, rope_line, i);
      info!("width_before:{i} actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_before_rev(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[(usize, usize)],
  ) {
    for (e, i) in expect.iter() {
      let a = actual.width_before(options, rope_line, *i);
      info!("width_before:{i}, actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  #[test]
  fn width1() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "width1");

    let mut actual = ColumnIndex::new();

    let expect: Vec<usize> =
      [(1..=6).collect(), (14..=20).collect(), vec![20, 20, 20, 20]].concat();
    assert_width_at(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [(0..=6).collect(), (14..=20).collect(), vec![20, 20, 20]].concat();
    assert_width_before(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width2() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["This is a quite simple and small test lines.\n"]);

    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "width2");

    let mut actual = ColumnIndex::new();

    assert_eq!(actual.width_before(&options, &rope.line(0), 5), 5);
    assert_eq!(actual.width_at(&options, &rope.line(0), 43), 44);

    let expect: Vec<usize> = [(1..=44).collect(), vec![44, 44, 44, 44]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width_at(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [(0..=44).collect(), vec![44, 44, 44]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width_before(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width3() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["But still\tit\\包含了好几种东西we want to test:\n"]);

    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "width3");

    let mut actual = ColumnIndex::new();

    let expect: Vec<usize> = [
      (1..=9).collect(),
      (17..=20).collect(),
      (22..=29)
        .scan(22, |state, i| {
          let diff: usize = i - *state;
          Some(*state + 2 * diff)
        })
        .collect(),
      (37..=52).collect(),
      vec![52, 52, 52, 52],
    ]
    .concat();
    assert_width_at(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [
      (0..=9).collect(),
      (17..=20).collect(),
      (22..=29)
        .scan(22, |state, i| {
          let diff: usize = i - *state;
          Some(*state + 2 * diff)
        })
        .collect(),
      (37..=52).collect(),
      vec![52, 52, 52],
    ]
    .concat();
    assert_width_before(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width4() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["  1. When the\r"]);
    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "width4");

    let mut actual = ColumnIndex::new();

    assert_eq!(actual.width_before(&options, &rope.line(0), 11), 11);
    assert_eq!(actual.width_at(&options, &rope.line(0), 10), 11);

    let expect: Vec<usize> = [(1..=13).collect(), vec![13, 13, 13, 13]].concat();
    assert_width_at(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [(0..=13).collect(), vec![13, 13, 13]].concat();
    assert_width_before(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width5() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "一行文本小到可以放入一个窗口中，那么line-wrap和word-wrap选项就不会影响排版。\n",
    ]);

    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "width5");

    let mut actual = ColumnIndex::new();

    let expect: Vec<usize> = [
      (1..=18).map(|i| i * 2).collect(),
      (37..=45).collect(),
      vec![47],
      (48..=56).collect(),
      (58..=67)
        .scan(58, |state, i| {
          let diff: usize = i - *state;
          Some(*state + 2 * diff)
        })
        .collect(),
      vec![76, 76, 76, 76],
    ]
    .concat();
    assert_width_at(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [
      (0..=18).map(|i| i * 2).collect(),
      (37..=45).collect(),
      vec![47],
      (48..=56).collect(),
      (58..=67)
        .scan(58, |state, i| {
          let diff: usize = i - *state;
          Some(*state + 2 * diff)
        })
        .collect(),
      vec![76, 76, 76, 76],
    ]
    .concat();
    assert_width_before(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width6() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "\t\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
    ]);
    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "width6");

    let mut actual = ColumnIndex::new();

    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 8);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 16);
    assert_eq!(actual.width_at(&options, &rope.line(0), 2), 17);

    let expect: Vec<usize> = [vec![8, 16], (17..=129).collect(), vec![129, 129, 129, 129]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width_at(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [vec![0, 8, 16], (17..=129).collect(), vec![129, 129, 129]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width_before(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width7() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = Rope::new();

    let mut actual = ColumnIndex::new();

    assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 10), 0);

    let rope = make_rope_from_lines(vec![]);
    let mut actual = ColumnIndex::new();

    assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 10), 0);

    let rope = make_rope_from_lines(vec![""]);
    let mut actual = ColumnIndex::new();

    assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_at(&options, &rope.line(0), 10), 0);
  }

  fn assert_char_before(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_before: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_before.iter() {
      let actual = widx.char_before(options, rope_line, *w);
      info!("char_before expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_at(options, rope_line, c.unwrap());
        info!("width_until-1 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual < *w);
      }
    }
  }

  fn assert_char_at(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_until: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_until.iter() {
      let actual = widx.char_at(options, rope_line, *w);
      info!("char_at expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_at(options, rope_line, c.unwrap());
        info!("width_at-2 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual >= *w);
      } else {
        info!("width_at-2 char:{c:?} expect width:{w:?}");
      }
    }
  }

  fn assert_char_after(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_after: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_after.iter() {
      let actual = widx.char_after(options, rope_line, *w);
      info!("char_after expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_at(options, rope_line, c.unwrap());
        info!("width_until-3 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual >= *w);
      }
    }
  }

  fn assert_char(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_before: &[(usize, Option<usize>)],
    expect_until: &[(usize, Option<usize>)],
    expect_after: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_before.iter() {
      let actual = widx.char_before(options, rope_line, *w);
      info!("char_before expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_at(options, rope_line, c.unwrap());
        info!("width_until-1 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual < *w);
      }
    }
    for (w, c) in expect_until.iter() {
      let actual = widx.char_at(options, rope_line, *w);
      info!("char_until expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_at(options, rope_line, c.unwrap());
        info!("width_until-2 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual <= *w);
      }
    }
    for (w, c) in expect_after.iter() {
      let actual = widx.char_after(options, rope_line, *w);
      info!("char_after expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_at(options, rope_line, c.unwrap());
        info!("width_until-3 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual > *w);
      }
    }
  }

  #[test]
  fn char1() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["These are\t很简单的test\tlines.\n"]);
    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "char1");

    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, None),
      (5, Some(3)),
      (9, Some(7)),
      (10, Some(8)),
      (11, Some(8)),
      (16, Some(8)),
      (17, Some(8)),
      (18, Some(9)),
      (19, Some(9)),
      (20, Some(10)),
      (21, Some(10)),
      (22, Some(11)),
      (23, Some(11)),
      (24, Some(12)),
      (25, Some(12)),
      (26, Some(13)),
      (27, Some(14)),
      (28, Some(15)),
      (29, Some(16)),
      (30, Some(17)),
      (31, Some(17)),
      (32, Some(17)),
      (36, Some(17)),
      (37, Some(17)),
      (38, Some(18)),
      (39, Some(19)),
      (43, Some(23)),
      (44, None),
      (45, None),
    ];
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, Some(0)),
      (5, Some(4)),
      (8, Some(7)),
      (9, Some(8)),
      (10, Some(9)),
      (11, Some(9)),
      (12, Some(9)),
      (13, Some(9)),
      (14, Some(9)),
      (15, Some(9)),
      (16, Some(9)),
      (17, Some(9)),
      (18, Some(10)),
      (19, Some(10)),
      (20, Some(11)),
      (21, Some(11)),
      (22, Some(12)),
      (23, Some(12)),
      (24, Some(13)),
      (25, Some(13)),
      (26, Some(14)),
      (27, Some(15)),
      (28, Some(16)),
      (29, Some(17)),
      (30, Some(18)),
      (35, Some(18)),
      (36, Some(18)),
      (37, Some(18)),
      (38, Some(19)),
      (39, Some(20)),
      (40, Some(21)),
      (41, Some(22)),
      (42, Some(23)),
      (43, Some(24)),
      (44, None),
      (45, None),
    ];
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> = vec![
      (0, Some(0)),
      (1, Some(1)),
      (2, Some(2)),
      (5, Some(5)),
      (6, Some(6)),
      (7, Some(7)),
      (8, Some(8)),
      (9, Some(9)),
      (10, Some(10)),
      (11, Some(10)),
      (15, Some(10)),
      (16, Some(10)),
      (17, Some(10)),
      (18, Some(11)),
      (19, Some(11)),
      (20, Some(12)),
      (21, Some(12)),
      (22, Some(13)),
      (23, Some(13)),
      (24, Some(14)),
      (25, Some(14)),
      (26, Some(15)),
      (27, Some(16)),
      (28, Some(17)),
      (29, Some(18)),
      (30, Some(19)),
      (31, Some(19)),
      (32, Some(19)),
      (36, Some(19)),
      (37, Some(19)),
      (38, Some(20)),
      (39, Some(21)),
      (40, Some(22)),
      (41, Some(23)),
      (42, Some(24)),
      (43, Some(25)),
      (44, None),
      (45, None),
    ];
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
  }

  #[test]
  fn char2() {
    test_log_init();

    let options = BufferLocalOptions::default();

    let rope = make_rope_from_lines(vec!["\t"]);
    let buffer = make_buffer_from_rope(rope.clone());
    print_buffer_line_details(buffer.clone(), 0, "char2-1");
    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, None),
      (5, None),
      (7, None),
      (8, None),
      (9, Some(0)),
      (10, Some(0)),
    ];
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, None),
      (5, None),
      (7, None),
      (8, Some(0)),
      (9, Some(0)),
      (10, Some(0)),
    ];
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> = vec![
      (0, Some(0)),
      (1, Some(0)),
      (5, Some(0)),
      (7, Some(0)),
      (8, None),
      (9, None),
      (10, None),
    ];
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
  }

  #[test]
  fn char3() {
    test_log_init();

    let options = BufferLocalOptions::default();

    {
      let rope = Rope::new();

      let mut widx = ColumnIndex::new();

      let expect_before: Vec<(usize, Option<usize>)> =
        (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

      let expect_at: Vec<(usize, Option<usize>)> = (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

      let expect_after: Vec<(usize, Option<usize>)> =
        (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
    }

    {
      let rope = make_rope_from_lines(vec![]);
      let mut widx = ColumnIndex::new();

      let expect_before: Vec<(usize, Option<usize>)> =
        (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

      let expect_at: Vec<(usize, Option<usize>)> = (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

      let expect_after: Vec<(usize, Option<usize>)> =
        (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
    }

    {
      let rope = make_rope_from_lines(vec![""]);
      let buffer = make_buffer_from_rope(rope.clone());
      print_buffer_line_details(buffer.clone(), 0, "char3-1");

      let mut widx = ColumnIndex::new();

      let expect_before: Vec<(usize, Option<usize>)> =
        (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

      let expect_at: Vec<(usize, Option<usize>)> = (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

      let expect_after: Vec<(usize, Option<usize>)> =
        (0..50).into_iter().map(|i| (i, None)).collect();
      assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
    }
  }

  #[test]
  fn truncate1() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let mut widx = ColumnIndex::new();

    let expect: Vec<usize> =
      [(1..=6).collect(), (14..=20).collect(), vec![20, 20, 20, 20]].concat();
    for (i, e) in expect.iter().enumerate() {
      let actual = widx.width_at(&options, &rope.line(0), i);
      assert_eq!(actual, *e);
      widx.truncate_since_char(i);
    }

    let expect: Vec<usize> = [(0..=6).collect(), (14..=20).collect(), vec![20, 20, 20]].concat();
    for (i, e) in expect.iter().enumerate() {
      let a = widx.width_before(&options, &rope.line(0), i);
      assert_eq!(a, *e);
      widx.truncate_since_char(i);
    }

    let rope = make_rope_from_lines(vec!["This is a quite\t简单而且很小的test\tlines.\n"]);
    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, None),
      (5, Some(3)),
      (10, Some(8)),
      (15, Some(13)),
      (16, Some(14)),
      (17, Some(14)),
      (22, Some(14)),
      (23, Some(14)),
      (24, Some(15)),
      (25, Some(15)),
      (26, Some(16)),
      (27, Some(16)),
      (28, Some(17)),
    ];

    let expect_until: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, Some(0)),
      (5, Some(4)),
      (10, Some(9)),
      (15, Some(14)),
      (16, Some(14)),
      (17, Some(14)),
      (22, Some(14)),
      (23, Some(15)),
      (24, Some(15)),
      (25, Some(16)),
      (26, Some(16)),
      (27, Some(17)),
      (28, Some(17)),
      (29, Some(18)),
    ];
    for (w, i) in expect_before.iter() {
      assert_eq!(widx.char_before(&options, &rope.line(0), *w), *i);
      widx.truncate_since_width(*w);
    }
    for (w, i) in expect_until.iter() {
      assert_eq!(widx.char_at(&options, &rope.line(0), *w), *i);
      widx.truncate_since_width(*w);
    }
  }
}
