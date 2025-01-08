//! Display width index (line-wise) for each unicode char in vim buffer.

use crate::buf::opt::BufferLocalOptions;
use crate::buf::unicode;
use ropey::RopeSlice;

use std::collections::BTreeMap;
// use tracing::trace;

#[derive(Debug, Default, Clone)]
/// Display width index (line-wise) for each unicode char in vim buffer. For each line, the
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
pub struct BufWindex {
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

impl BufWindex {
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
      assert!(i <= c);
    }
  }

  // Build cache from char index 0 to `char_idx`.
  fn _build_cache(&mut self, options: &BufferLocalOptions, rope_line: &RopeSlice, char_idx: usize) {
    // Line is empty, no need to build.
    if rope_line.len_chars() == 0 {
      return;
    }

    // If not cached.
    if char_idx >= self.char2width.len() {
      // Build the cache until either `char_idx` or the end of the line.
      let build_idx = std::cmp::min(char_idx, rope_line.len_chars() - 1);

      let start_idx = self.char2width.len();
      let mut prefix_width: usize = if start_idx == 0 {
        0_usize
      } else {
        self.char2width[start_idx - 1]
      };

      let mut rope_chars = rope_line.chars().skip(start_idx);
      for _i in start_idx..=build_idx {
        let c = rope_chars.next().unwrap();
        prefix_width += unicode::char_width(options, c);

        // Update `char2width`
        self.char2width.push(prefix_width);

        // Update `width2char`
        let w = prefix_width;
        let c = self.char2width.len() - 1;
        match self.width2char.get(&w) {
          Some(c1) => {
            if *c1 < c {
              self.width2char.insert(w, c);
            }
          }
          None => {
            self.width2char.insert(w, c);
          }
        }
      }
    }
  }

  /// Get the prefix display width in char index range `[0,char_idx)`, left-inclusive and
  /// right-exclusive.
  ///
  /// NOTE: This is equivalent to `width_inclusive(char_idx-1)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if `char_idx <= 0`.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than the line
  ///    length.
  pub fn width(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx: usize,
  ) -> usize {
    self._build_cache(options, rope_line, char_idx);
    self._internal_check();

    if char_idx == 0 || self.char2width.is_empty() {
      assert_eq!((rope_line.len_chars() == 0), self.char2width.is_empty());
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

  /// Get the prefix display width in char index range `[0,char_idx]`, both sides are inclusive.
  ///
  /// NOTE: This is equivalent to `width(char_idx+1)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if the line length is 0, i.e. the line itself is empty.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than or equal to
  ///    the line length.
  pub fn width_inclusive(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx: usize,
  ) -> usize {
    self._build_cache(options, rope_line, char_idx);
    self._internal_check();

    if self.char2width.is_empty() {
      assert!(rope_line.len_chars() == 0);
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

  /// Get the display width between char index range `[a, b)`, left-inclusive and right-exclusive.
  ///
  /// This is equivalent to `width(b)-width(a)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if the line length is 0, i.e. the line itself is empty.
  /// 2. It returns the prefix display width if the range is inside the line.
  /// 3. It returns the truncated actual display width of the range is outside of the line.
  pub fn width_between(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx_range: std::ops::Range<usize>,
  ) -> usize {
    assert!(char_idx_range.end >= char_idx_range.start);
    self._build_cache(options, rope_line, char_idx_range.end);
    self._internal_check();
    let start_idx = char_idx_range.start;
    let last_idx = char_idx_range.end;
    let start_width = self.width(options, rope_line, start_idx);
    let last_width = self.width(options, rope_line, last_idx);
    assert!(last_width >= start_width);
    last_width - start_width
  }

  /// Get the display width between char index range `[a, b]`, both sides are inclusive.
  ///
  /// This is equivalent to `width_inclusive(b)-width(a)`.
  ///
  /// # Return
  ///
  /// 1. It returns 0 if the line length is 0, i.e. the line itself is empty.
  /// 2. It returns the prefix display width if the range is inside the line.
  /// 3. It returns the truncated actual display width of the range is outside of the line.
  pub fn width_between_inclusive(
    &mut self,
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    char_idx_range: std::ops::RangeInclusive<usize>,
  ) -> usize {
    assert!(char_idx_range.end() >= char_idx_range.start());
    self._build_cache(options, rope_line, *char_idx_range.end());
    self._internal_check();
    let start_idx = *char_idx_range.start();
    let last_idx = *char_idx_range.end();
    let start_width = self.width(options, rope_line, start_idx);
    let last_width = self.width_inclusive(options, rope_line, last_idx);
    assert!(last_width >= start_width);
    last_width - start_width
  }

  /// Get the first char index which width is greater or equal than the specified width.
  ///
  /// Here the *greater or equal than* indicates that:
  /// 1. If the width is exactly the width on a char index, it returns the char index.
  /// 2. Otherwise, it returns the first char which width is greater than it.
  ///
  /// # Return
  ///
  /// It returns the first char index if the `width` is inside the index.
  /// It returns `None` if the `width` is out of the index range.
  pub fn char_at(&self, _width: usize) -> Option<usize> {
    unimplemented!();
  }

  /// Truncate the width since specified char index.
  pub fn truncate(&mut self, _char_idx: usize) {
    unimplemented!();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::test::buf::make_rope_from_lines;
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  use tracing::info;

  fn assert_width_inclusive(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut BufWindex,
    expect: &[usize],
  ) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width_inclusive(options, rope_line, i);
      info!("width_inclusive:{i} actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
      let a = actual.width_between_inclusive(options, rope_line, 0..=i);
      info!("width_between_inclusive:[0,{i}] actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_inclusive_rev(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut BufWindex,
    expect: &[(usize, usize)],
  ) {
    for (e, i) in expect.iter() {
      let a = actual.width_inclusive(options, rope_line, *i);
      info!("width_inclusive:{i}, actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
      let a = actual.width_between_inclusive(options, rope_line, 0..=*i);
      info!("width_between_inclusive:[0,{i}] actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut BufWindex,
    expect: &[usize],
  ) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width(options, rope_line, i);
      info!("width:{i} actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
      let a = actual.width_between(options, rope_line, 0..i);
      info!("width_between:[0,{i}] actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_rev(
    options: &BufferLocalOptions,
    rope_line: &RopeSlice,
    actual: &mut BufWindex,
    expect: &[(usize, usize)],
  ) {
    for (e, i) in expect.iter() {
      let a = actual.width(options, rope_line, *i);
      info!("width:{i}, actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
      let a = actual.width_between(options, rope_line, 0..*i);
      info!("width_between:[0,{i}] actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  #[test]
  fn width1() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let mut actual = BufWindex::new();

    let expect: Vec<usize> =
      [(1..=6).collect(), (14..=20).collect(), vec![20, 20, 20, 20]].concat();
    assert_width_inclusive(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_inclusive_rev(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [(0..=6).collect(), (14..=20).collect(), vec![20, 20, 20]].concat();
    assert_width(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width2() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["This is a quite simple and small test lines.\n"]);
    let mut actual = BufWindex::new();

    assert_eq!(actual.width(&options, &rope.line(0), 5), 5);
    assert_eq!(actual.width_inclusive(&options, &rope.line(0), 43), 44);

    let expect: Vec<usize> = [(1..=44).collect(), vec![44, 44, 44, 44]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_inclusive_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width_inclusive(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [(0..=44).collect(), vec![44, 44, 44]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width3() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["But still\tit\\包含了好几种东西we want to test:\n"]);
    let mut actual = BufWindex::new();

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
    assert_width_inclusive(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_inclusive_rev(&options, &rope.line(0), &mut actual, &expect);

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
    assert_width(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width4() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["  1. When the\r"]);
    let mut actual = BufWindex::new();

    assert_eq!(actual.width(&options, &rope.line(0), 11), 11);
    assert_eq!(actual.width_inclusive(&options, &rope.line(0), 10), 11);

    let expect: Vec<usize> = [(1..=13).collect(), vec![13, 13, 13, 13]].concat();
    assert_width_inclusive(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_inclusive_rev(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [(0..=13).collect(), vec![13, 13, 13]].concat();
    assert_width(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width5() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "一行文本小到可以放入一个窗口中，那么line-wrap和word-wrap选项就不会影响排版。\n",
    ]);
    let mut actual = BufWindex::new();

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
    assert_width_inclusive(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_inclusive_rev(&options, &rope.line(0), &mut actual, &expect);

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
    assert_width(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_rev(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width6() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "\t\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
    ]);
    let mut actual = BufWindex::new();

    assert_eq!(actual.width(&options, &rope.line(0), 1), 8);
    assert_eq!(actual.width(&options, &rope.line(0), 2), 16);
    assert_eq!(actual.width_inclusive(&options, &rope.line(0), 2), 17);

    let expect: Vec<usize> = [vec![8, 16], (17..=129).collect(), vec![129, 129, 129, 129]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_inclusive_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width_inclusive(&options, &rope.line(0), &mut actual, &expect);

    let expect: Vec<usize> = [vec![0, 8, 16], (17..=129).collect(), vec![129, 129, 129]].concat();

    let expect1: Vec<(usize, usize)> = expect
      .iter()
      .enumerate()
      .map(|(i, e)| (*e, i))
      .rev()
      .collect();
    assert_width_rev(&options, &rope.line(0), &mut actual, &expect1);

    assert_width(&options, &rope.line(0), &mut actual, &expect);
  }

  #[test]
  fn width_between1() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let mut widx = BufWindex::new();

    // inclusive {
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=6),
      widx.width_inclusive(&options, &rope.line(0), 6)
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=6),
      6 + 8
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 4..=8),
      4 + 8
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 13..=13),
      0
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 12..=13),
      1
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=13),
      widx.width_inclusive(&options, &rope.line(0), 13)
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=13),
      12 + 8
    );
    // inclusive }

    // non-inclusive {
    assert_eq!(
      widx.width_between(&options, &rope.line(0), 0..6),
      widx.width(&options, &rope.line(0), 6)
    );
    assert_eq!(widx.width_between(&options, &rope.line(0), 0..6), 6);
    assert_eq!(widx.width_between(&options, &rope.line(0), 4..8), 3 + 8);
    assert_eq!(widx.width_between(&options, &rope.line(0), 13..13), 0);
    assert_eq!(widx.width_between(&options, &rope.line(0), 12..13), 1);
    assert_eq!(
      widx.width_between(&options, &rope.line(0), 0..13),
      widx.width(&options, &rope.line(0), 13)
    );
    assert_eq!(widx.width_between(&options, &rope.line(0), 0..13), 12 + 8);
    // non-inclusive }
  }

  #[test]
  fn width_between2() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["This is a quite simple and small test lines.\n"]);
    let mut widx = BufWindex::new();

    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=43),
      44
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=43),
      widx.width_inclusive(&options, &rope.line(0), 43),
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=44),
      44
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=44),
      widx.width_inclusive(&options, &rope.line(0), 44),
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 7..=15),
      9
    );
  }

  #[test]
  fn width_between3() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["But still\tit\\包含了好几种东西we want to test:\n"]);
    let mut widx = BufWindex::new();

    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=15),
      12 + 8 + 6
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=15),
      widx.width_inclusive(&options, &rope.line(0), 15)
    );
  }

  #[test]
  fn width_between4() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "一行文本小到可以放入一个窗口中，那么line-wrap和word-wrap选项就不会影响排版。\n",
    ]);
    let mut widx = BufWindex::new();

    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 15..=27),
      8 + 9
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=27),
      19 * 2 + 9
    );
    assert_eq!(
      widx.width_between_inclusive(&options, &rope.line(0), 0..=27),
      widx.width_inclusive(&options, &rope.line(0), 27)
    );
  }
}
