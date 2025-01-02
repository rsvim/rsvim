//! Display width index (line-wise) for each unicode char in vim buffer.

use crate::buf::opt::BufferLocalOptions;
use crate::buf::unicode;
use ropey::Rope;

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
/// Display width index (line-wise) for each unicode char in vim buffer. For each line, the
/// char/column index starts from 0.
///
/// This structure is mostly like a prefix-sum tree structure.
pub struct BufWindex {
  // Char index maps to its prefix display width, i.e. from the first char/column (0) to current
  // char/column, not just the current char's display width.
  char2width: Vec<usize>,

  // Prefix display width maps to its char index. This is the reversed mapping of `char2width`.
  // NOTE: The keys, i.e. the widths could be non-continuous since one unicode char could use
  // more than 1 cells.
  width2char: BTreeMap<usize, usize>,
}

impl BufWindex {
  /// Create and initialize index for the line.
  ///
  /// # Panics
  ///
  /// It panics if the line doesn't exist in the rope.
  pub fn new(options: &BufferLocalOptions, rope: &Rope, line_idx: usize) -> Self {
    let char2width: Vec<usize> = rope
      .get_line(line_idx)
      .unwrap()
      .chars()
      .scan(0_usize, |acc, c| {
        let width = *acc;
        *acc += unicode::char_width(options, c);
        Some(width)
      })
      .collect();
    let width2char: BTreeMap<usize, usize> = char2width
      .iter()
      .enumerate()
      .map(|(i, w)| (*w, i))
      .collect();
    Self {
      char2width,
      width2char,
    }
  }

  #[cfg(not(debug_assertions))]
  pub fn _internal_check(&self) {}

  #[cfg(debug_assertions)]
  pub fn _internal_check(&self) {
    // Check length.
    assert!(self.char2width.len() >= self.width2char.len());

    // Check char index continuous.
    let mut last_width: Option<usize> = None;
    for (i, w) in self.char2width.iter().enumerate() {
      if i == 0 {
        assert!(self.char2width[0] == 0);
      }
      match last_width {
        Some(last_width1) => {
          assert!(*w >= last_width1);
        }
        None => { /* Skip */ }
      }
      last_width = Some(*w);
    }

    // Check char index always exists in width index.
    for (i, w) in self.char2width.iter().enumerate() {
      assert!(self.width2char.contains_key(w));
      assert_eq!(i, self.width2char[w]);
    }
  }

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.char2width.is_empty()
  }

  pub fn len(&self) -> usize {
    self._internal_check();
    self.char2width.len()
  }

  /// Get the prefix display width starts from the first char 0 until the specified char.
  ///
  /// NOTE: This is equivalent to `width_between(0..=char_idx)`.
  ///
  /// # Return
  ///
  /// It returns the prefix display width if `char_idx` is inside the index.
  /// It returns `None` if the `char_idx` is out of index range.
  pub fn width_at(&self, char_idx: usize) -> Option<usize> {
    self._internal_check();
    if char_idx < self.char2width.len() {
      Some(self.char2width[char_idx])
    } else {
      None
    }
  }

  /// Get the display width in the inclusive range, i.e. [a, b].
  ///
  /// # Return
  ///
  /// It returns the display width of the `char_idx_range` if the range is inside the index.
  /// It returns `None` if the `char_idx_range` is out of index range.
  pub fn width_between(&self, char_idx_range: std::ops::RangeInclusive<usize>) -> Option<usize> {
    self._internal_check();
    let c_start = *char_idx_range.start();
    let c_end = *char_idx_range.end();
    if c_start < self.char2width.len() && c_end < self.char2width.len() {
      assert!(self.char2width[c_start] <= self.char2width[c_end]);
      Some(self.char2width[c_start] - self.char2width[c_end])
    } else {
      None
    }
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
  pub fn char_at(&self, width: usize) -> Option<usize> {
    self._internal_check();
    if !self.is_empty() && width <= *self.width2char.last_key_value().unwrap().1 {
      for w in width.. {
        match self.width2char.get(&w) {
          Some(c) => {
            // Early returns.
            return Some(*c);
          }
          None => { /* Skip */ }
        }
      }
      unreachable!();
    } else {
      None
    }
  }

  /// Set/update a specified char's width, and re-calculate all display width since this char.
  ///
  /// NOTE: This operation is `O(N)`, where `N` is the chars count of current line.
  pub fn set_width_at(&mut self, char_idx: usize, width: usize) {
    self._internal_check();
    assert!(char_idx < self.char2width.len());
    match width.cmp(&self.char2width[char_idx]) {
      std::cmp::Ordering::Less => {
        let diff = self.char2width[char_idx] - width;
        for w in self.char2width.iter_mut().skip(char_idx) {
          *w -= diff;
        }
      }
      std::cmp::Ordering::Greater => {
        let diff = width - self.char2width[char_idx];
        for w in self.char2width.iter_mut().skip(char_idx) {
          *w += diff;
        }
      }
      _ => { /* Do nothing */ }
    }
  }

  /// Set/update a range of chars and their width, and re-calculate all display width since the first
  /// char in the range.
  ///
  /// NOTE: This operation is `O(N)`, where `N` is the chars count of current line.
  ///
  /// # Panics
  ///
  /// It panics if the provided parameter `char2width` keys are not continuous, i.e. the chars
  /// index must be continuous.
  pub fn set_width_between(&mut self, widths: &BTreeMap<usize, usize>) {
    if widths.is_empty() {
      return;
    }

    self._internal_check();

    let (start_c, _start_w) = widths.first_key_value().unwrap();
    let (last_c, _last_w) = widths.last_key_value().unwrap();
    assert!(*start_c < self.char2width.len());
    assert!(*last_c < self.char2width.len());
    let mut last_key: Option<usize> = None;
    for (k, _v) in widths.iter() {
      match last_key {
        Some(last_key1) => assert_eq!(last_key1 + 1, *k),
        None => { /* Skip */ }
      }
      last_key = Some(*k);
    }

    let mut result: Vec<usize> = self.char2width.iter().take(*start_c).cloned().collect();
    let init_width = if *start_c > 0 {
      self.char2width[*start_c - 1]
    } else {
      0_usize
    };
    let result2: Vec<usize> = self
      .char2width
      .iter()
      .enumerate()
      .skip(*start_c)
      .scan(init_width, |acc, (i, _w)| {
        let width = *acc + widths.get(&i).unwrap();
        *acc = width;
        Some(width)
      })
      .collect();
    result.extend(result2);
    self.char2width = result;
  }

  /// Push/append a specified char's width.
  ///
  /// NOTE: This operation is `O(1)`.
  pub fn push(&mut self, _width: usize) {
    unimplemented!();
  }

  /// Extend/append multiple chars and their display width, and re-calculate all display width
  /// for the extended chars.
  ///
  /// NOTE: This operation is `O(M)`, where `M` is the chars count of the extended chars.
  pub fn extend(&mut self, _widths: &[usize]) {
    unimplemented!();
  }

  /// Replace a range of chars and their display width, with a new range, and re-calculate all
  /// display width since the first char in the newly added range of chars.
  ///
  /// NOTE: This operation is `O(N+M)`, where `N` is the chars count of current line, `M` is the
  /// chars count of the new range.
  pub fn splice(&mut self) {
    unimplemented!();
  }

  /// Shorten (remove/truncate) the chars since a specified char index. This operation doesn't need
  /// to trigger re-calculation.
  ///
  /// NOTE: This operation is `O(1)`.
  pub fn truncate(&mut self) {
    unimplemented!();
  }

  /// Remove a specified range of chars, and re-calculate all display width since the start index
  /// in the removed range.
  ///
  /// NOTE: This operation is `O(N)`, where `N` is the chars count of current line.
  pub fn drain(&mut self) {
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

  fn ensure_width_at(actual: &BufWindex, expect: &Vec<Option<usize>>) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width_at(i);
      info!("actual[{i}]:{a:?}, expect[{i}]:{e:?}");
      assert_eq!(a, e.clone());
    }
  }

  #[test]
  fn width_at1() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let actual = BufWindex::new(&options, &rope, 0);
    // 0-6, 14-20
    let expect: Vec<Option<usize>> = [
      (0..=6).map(|i| Some(i)).collect(),
      (14..=20).map(|i| Some(i)).collect(),
      vec![None, None, None],
    ]
    .concat();
    ensure_width_at(&actual, &expect);
  }

  #[test]
  fn width_at2() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["This is a quite simple and small test lines.\n"]);
    let actual = BufWindex::new(&options, &rope, 0);
    // 0-43, 43
    let expect: Vec<Option<usize>> = [
      (0..=43).map(|i| Some(i)).collect(),
      vec![Some(43), None, None, None],
    ]
    .concat();
    ensure_width_at(&actual, &expect);
  }

  #[test]
  fn width_at3() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["But still\tit\\包含了好几种东西we want to test:\n"]);
    let actual = BufWindex::new(&options, &rope, 0);
    // 0-8, 16-18, 19-33, 35-51, 51
    let expect: Vec<Option<usize>> = [
      (0..=8).map(|i| Some(i)).collect(),
      (16..=18).map(|i| Some(i)).collect(),
      (19..=26)
        .scan(19, |state, i| {
          let diff: usize = i - *state;
          Some(Some(*state + 2 * diff))
        })
        .collect(),
      (35..=51).map(|i| Some(i)).collect(),
      vec![Some(51), None, None, None],
    ]
    .concat();
    ensure_width_at(&actual, &expect);
  }

  #[test]
  fn width_at4() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec!["  1. When the\r"]);
    let actual = BufWindex::new(&options, &rope, 0);
    // 0-12, 12
    let expect: Vec<Option<usize>> = [
      (0..=12).map(|i| Some(i)).collect(),
      vec![Some(12), None, None, None],
    ]
    .concat();
    ensure_width_at(&actual, &expect);
  }

  #[test]
  fn width_at5() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "一行文本小到可以放入一个窗口中，那么line-wrap和word-wrap选项就不会影响排版。\n",
    ]);
    let actual = BufWindex::new(&options, &rope, 0);
    // 0-35, 36-45, 46, 48-57, 58-77, 77
    let expect: Vec<Option<usize>> = [
      (0..=17).map(|i| Some(i * 2)).collect(),
      (36..=45).map(|i| Some(i)).collect(),
      vec![Some(46)],
      (48..=57).map(|i| Some(i)).collect(),
      (58..=67)
        .scan(58, |state, i| {
          let diff: usize = i - *state;
          Some(Some(*state + 2 * diff))
        })
        .collect(),
      vec![Some(77), None, None, None],
    ]
    .concat();
    ensure_width_at(&actual, &expect);
  }

  #[test]
  fn width_at6() {
    test_log_init();

    let options = BufferLocalOptions::default();
    let rope = make_rope_from_lines(vec![
      "\t\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
    ]);
    let actual = BufWindex::new(&options, &rope, 0);
    // 0, 8, 16-129, 129
    let expect: Vec<Option<usize>> = [
      vec![Some(0), Some(8)],
      (16..=129).map(|i| Some(i)).collect(),
      vec![Some(129), None, None, None],
    ]
    .concat();
    ensure_width_at(&actual, &expect);
  }
}
