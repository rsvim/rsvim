//! Indexes mappings between character and its display width.

use crate::buf::text::opt::TextOptions;
use crate::buf::unicode;
use ropey::RopeSlice;

use smallvec::SmallVec;
use std::collections::BTreeMap;
// use tracing::trace;

#[derive(Debug, Default, Clone)]
/// Display width index (char-wise) for each unicode char in vim buffer. For each line, the
/// char/column index starts from 0.
///
/// A unicode char's width can also be 0 (line-break), 2 (Chinese/Japanese/Korean char) and
/// 8 (tab). Thus we need to maintain the mappings between the char and its display width/column.
///
/// This structure is actually a prefix-sum tree structure. For example now we have a line:
///
/// ```text
/// column index:
///                           25
/// 0      7       15       25|
/// |      |       |         ||
/// This is<--HT-->an example.\n
/// |      |                 ||
/// 0      7                18|
///                           19
/// char index:
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
/// - **Column**: the column index is the its display width - 1.
/// - **Current** char: the char index that its covers the display width.
/// - **Previous** char: the char index before the **current** char.
/// - **Next** char: the char index after the **current** char.
///
/// For example:
/// - The **current** char on width 8 is `<--HT-->` (column:7, char:7), the **current** char on
///   width 10 is still `<--HT-->` (column:9, char:7). The width on **current** char 7 is 16.
/// - The **current** char on width 0 doesn't exist (because the width 0 actually don't have a char
///   on it), the **current** char on width 1 is `T` (column:0, char:0). The width on **current**
///   char 0 is 1.
/// - The **current** char on width 26 is `.` (column:25, char:18). NOTE: The width 26 has 2 chars
///   on it: `.` and `\n`, but here we always locate to the 1st char from the beginning. The width
///   on **current** char 18 is 26.
pub struct ColumnIndex {
  // Char index maps to its prefix display width.
  char2width: SmallVec<[usize; 80]>,

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
  pub fn with_capacity(size: usize) -> Self {
    Self {
      char2width: SmallVec::with_capacity(size),
      width2char: BTreeMap::new(),
    }
  }

  /// Create new index.
  pub fn new() -> Self {
    Self {
      char2width: SmallVec::new(),
      width2char: BTreeMap::new(),
    }
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    // Check length.
    debug_assert!(self.char2width.len() >= self.width2char.len());

    // Check indexing.
    let mut last_width: Option<usize> = None;
    for (i, w) in self.char2width.iter().enumerate() {
      match last_width {
        Some(last_width1) => {
          debug_assert!(*w >= last_width1);
        }
        None => { /* Skip */ }
      }
      last_width = Some(*w);
      debug_assert!(self.width2char.contains_key(w));
      let c = self.width2char[w];
      // trace!("char2width[{i}]:{w:?}, width2char[{w}]:{c:?}");
      debug_assert!(i >= c);
    }
  }

  // Build cache beyond the bound by `char_idx` or `width`.
  fn _build_cache(
    &mut self,
    options: &TextOptions,
    buf_line: &RopeSlice,
    char_idx_bound: Option<usize>,
    width_bound: Option<usize>,
  ) {
    let n = buf_line.len_chars();

    let start_idx = self.char2width.len();
    let mut prefix: usize = if start_idx == 0 {
      0_usize
    } else {
      self.char2width[start_idx - 1]
    };

    let mut rope_chars = buf_line.chars_at(start_idx);
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
    options: &TextOptions,
    buf_line: &RopeSlice,
    char_idx: usize,
  ) {
    self._build_cache(options, buf_line, Some(char_idx), None);
  }

  /// Get the prefix display width in of **previous** char by `char_idx`, i.e. width range is
  /// `[0,char_idx)`.
  ///
  /// NOTE: This is equivalent to `width_at(char_idx-1)`.
  ///
  /// # Returns
  ///
  /// 1. It returns 0 if the `char_idx` is out of the line, there're below cases:
  ///    - The `char_idx` is 0.
  ///    - The line is empty.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than the line
  ///    length.
  pub fn width_before(
    &mut self,
    options: &TextOptions,
    buf_line: &RopeSlice,
    char_idx: usize,
  ) -> usize {
    self._build_cache_until_char(options, buf_line, char_idx);
    self._internal_check();

    if char_idx == 0 {
      0
    } else if self.char2width.is_empty() {
      debug_assert_eq!(buf_line.len_chars(), 0);
      0
    } else {
      debug_assert!(!self.char2width.is_empty());
      debug_assert!(buf_line.len_chars() > 0);
      if char_idx - 1 < self.char2width.len() {
        // Find width from the cache.
        self.char2width[char_idx - 1]
      } else {
        // If outside of the cache, returns the whole width.
        self.char2width[self.char2width.len() - 1]
      }
    }
  }

  /// Get the display width until **current** char by `char_idx`, i.e. width range is
  /// `[0,char_idx]`.
  ///
  /// NOTE: This is equivalent to `width_before(char_idx+1)`.
  ///
  /// # Returns
  ///
  /// 1. It returns 0 if the `char_idx` is out of the line, there're below cases:
  ///    - The line is empty.
  /// 2. It returns the prefix display width if `char_idx` is inside the line.
  /// 3. It returns the whole display width of the line if `char_idx` is greater than or equal to
  ///    the line length.
  pub fn width_until(
    &mut self,
    options: &TextOptions,
    buf_line: &RopeSlice,
    char_idx: usize,
  ) -> usize {
    self._build_cache_until_char(options, buf_line, char_idx);
    self._internal_check();

    if self.char2width.is_empty() {
      debug_assert_eq!(buf_line.len_chars(), 0);
      0
    } else {
      debug_assert!(!self.char2width.is_empty());
      debug_assert!(buf_line.len_chars() > 0);
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
    options: &TextOptions,
    buf_line: &RopeSlice,
    width: usize,
  ) {
    self._build_cache(options, buf_line, None, Some(width));
  }

  /// Get the **previous** char index before the char at `width`.
  ///
  /// # Returns
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
    options: &TextOptions,
    buf_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, buf_line, width);
    self._internal_check();

    if width == 0 {
      None
    } else if self.width2char.is_empty() {
      debug_assert_eq!(buf_line.len_chars(), 0);
      None
    } else {
      debug_assert!(!self.width2char.is_empty());
      debug_assert!(buf_line.len_chars() > 0);

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

  /// Get the **current** char index at `width`.
  ///
  /// # Return
  ///
  /// 1. It returns None if the `width` is out of the line, there're below cases:
  ///    - The line is empty.
  ///    - The `width` is greater than the whole line's display width, thus there's no such char
  ///      exists.
  ///    - The `width` is 0, and the 1st (only) char is 0-width char (e.g. line-break).
  /// 2. It returns the **current** char index otherwise.
  pub fn char_at(
    &mut self,
    options: &TextOptions,
    buf_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, buf_line, width);
    self._internal_check();

    if self.width2char.is_empty() {
      debug_assert_eq!(buf_line.len_chars(), 0);
      None
    } else {
      debug_assert!(!self.width2char.is_empty());
      debug_assert!(buf_line.len_chars() > 0);

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

  /// Get the **next** char index after the char at `width`.
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
    options: &TextOptions,
    buf_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, buf_line, width + 1);
    self._internal_check();

    let n = buf_line.len_chars();
    if self.char2width.is_empty() {
      debug_assert_eq!(n, 0);
      return None;
    }

    if width == 0 {
      return Some(0);
    }

    if let Some(char_idx) = self.char_at(options, buf_line, width) {
      if char_idx + 1 < n {
        return Some(char_idx + 1);
      }
    }

    None
  }

  /// Get the **last** char index which has the biggest width, while still less than or equal to
  /// the specified `width`.
  ///
  /// # Return
  ///
  /// 1. It returns None if the `width` is out of the line, there're below cases:
  ///    - The line is empty.
  ///    - The `width` is 0, and the 1st char is not 0-width char (e.g. line-break).
  /// 2. It returns the last char index otherwise. If the `width` is longer than the whole line, it
  ///    returns the last char index.
  pub fn last_char_until(
    &mut self,
    options: &TextOptions,
    buf_line: &RopeSlice,
    width: usize,
  ) -> Option<usize> {
    self._build_cache_until_width(options, buf_line, width);
    self._internal_check();

    if width == 0 {
      if *self.char2width.first().unwrap() == 0 {
        return Some(0);
      } else {
        return None;
      }
    }

    let (last_width, last_char_idx) = self.width2char.last_key_value().unwrap();
    if width > *last_width {
      return Some(*last_char_idx);
    } else if let Some(char_idx) = self.char_at(options, buf_line, width) {
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
      self.char2width.truncate(char_idx.saturating_sub(1));
      let end_char = self.char2width.len();
      self.width2char.retain(|&_w, &mut c| c < end_char);
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

// spellchecker:off
#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::{BufferLocalOptionsBuilder, Text};
  use crate::test::log::init as test_log_init;

  use ropey::{Rope, RopeBuilder};
  use tracing::info;

  fn make_default_text_opts() -> TextOptions {
    TextOptions::from(&BufferLocalOptionsBuilder::default().build().unwrap())
  }

  fn make_rope_from_lines(lines: Vec<&str>) -> Rope {
    let mut rb: RopeBuilder = RopeBuilder::new();
    for line in lines.iter() {
      rb.append(line);
    }
    rb.finish()
  }

  fn make_text_from_rope(terminal_height: u16, opts: TextOptions, rp: Rope) -> Text {
    Text::new(terminal_height, rp, opts)
  }

  #[allow(clippy::unused_enumerate_index)]
  fn print_text_line_details(text: Text, line_idx: usize, msg: &str) {
    let line = text.rope().get_line(line_idx).unwrap();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
      .with_line_number(false)
      .with_target(false)
      .with_level(true)
      .with_ansi(true)
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .with_writer(std::io::stdout)
      .without_time()
      .finish();

    tracing::subscriber::with_default(subscriber, || {
      if !msg.is_empty() {
        info!("line: {}", msg);
      } else {
        info!("line");
      }

      let mut payload = String::new();
      for c in line.chars() {
        let (cs, _cw) = text.char_symbol(c);
        payload.push_str(cs.as_ref());
      }
      info!("-{}-", payload);

      {
        let mut builder = String::new();
        let mut n = 0_usize;
        let mut w = 0_usize;
        let mut zero_width_chars: Vec<String> = vec![];
        let mut big_width_chars: Vec<String> = vec![];
        for (i, c) in line.chars().enumerate() {
          let (_cs, cw) = text.char_symbol(c);
          w += cw;
          n += 1;
          if cw == 0 {
            zero_width_chars.push(format!("{}", i));
          }
          if cw > 1 {
            big_width_chars.push(format!("{}", i));
          }
          if i % 5 == 0 {
            builder.push_str(&format!("{}", i));
          }
          if builder.len() < w {
            let diff = w - builder.len();
            builder.push_str(&" ".repeat(diff));
          }
        }
        info!(
          "-{}- Char Index, total:{} (width = 0 chars: count:{} indexes:{}, width > 1 chars: count:{} indexes:{})",
          builder,
          n,
          zero_width_chars.len(),
          zero_width_chars.join(","),
          big_width_chars.len(),
          big_width_chars.join(",")
        );
      }

      {
        let mut builder1 = String::new();
        let mut builder2 = String::new();
        let mut builder3 = String::new();
        let mut show2 = false;
        let mut show3 = false;
        let mut w = 0_usize;
        for (_i, c) in line.chars().enumerate() {
          let (_cs, cw) = text.char_symbol(c);
          w += cw;
          if w == 1 || w % 5 == 0 {
            if builder1.is_empty() || builder1.ends_with(" ") {
              builder1.push_str(&format!("{}", w));
            } else if cw > 0 {
              builder2.push_str(&format!("{}", w));
              show2 = true;
            } else {
              builder3.push_str(&format!("{}", w));
              show3 = true;
            }
          }

          if builder1.len() < w {
            let diff = w - builder1.len();
            builder1.push_str(&" ".repeat(diff));
          }
          if builder2.len() < w {
            let diff = w - builder2.len();
            builder2.push_str(&" ".repeat(diff));
          }
          if builder3.len() < w {
            let diff = w - builder3.len();
            builder3.push_str(&" ".repeat(diff));
          }
        }
        info!("-{}- Display Width, total width:{}", builder1, w);
        if show2 {
          info!(
            "-{}- Display Width (extra, conflicted with the above one)",
            builder2
          );
        }
        if show3 {
          info!("-{}- Display Width for width = 0 chars", builder3);
        }
      }

      {
        let mut builder = String::new();
        let mut w = 0_usize;
        let mut show = false;
        for (_i, c) in line.chars().enumerate() {
          let (_cs, cw) = text.char_symbol(c);
          w += cw;
          if cw > 1 && (builder.is_empty() || builder.ends_with(" ")) {
            builder.push_str(&" ".repeat(cw - 1));
            builder.push_str(&format!("{}", w));
            show = true;
          }

          if builder.len() < w {
            let diff = w - builder.len();
            builder.push_str(&" ".repeat(diff));
          }
        }
        if show {
          info!("-{}- Display Width for width > 1 chars", builder);
        }
      }
    });
  }

  fn assert_width_at(
    options: &TextOptions,
    buf_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[usize],
  ) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width_until(options, buf_line, i);
      info!("width_at:{i} actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_at_rev(
    options: &TextOptions,
    buf_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[(usize, usize)],
  ) {
    for (e, i) in expect.iter() {
      let a = actual.width_until(options, buf_line, *i);
      info!("width_at_rev:{i}, actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_before(
    options: &TextOptions,
    buf_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[usize],
  ) {
    for (i, e) in expect.iter().enumerate() {
      let a = actual.width_before(options, buf_line, i);
      info!("width_before:{i} actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  fn assert_width_before_rev(
    options: &TextOptions,
    buf_line: &RopeSlice,
    actual: &mut ColumnIndex,
    expect: &[(usize, usize)],
  ) {
    for (e, i) in expect.iter() {
      let a = actual.width_before(options, buf_line, *i);
      info!("width_before:{i}, actual:{a:?}, expect:{e:?}");
      assert_eq!(a, *e);
    }
  }

  #[test]
  fn width1() {
    test_log_init();

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "width1");

    let mut actual = ColumnIndex::with_capacity(10);

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

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["This is a quite simple and small test lines.\n"]);

    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "width2");

    let mut actual = ColumnIndex::with_capacity(10);

    assert_eq!(actual.width_before(&options, &rope.line(0), 5), 5);
    assert_eq!(actual.width_until(&options, &rope.line(0), 43), 44);

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

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["But still\tit\\包含了好几种东西we want to test:\n"]);

    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "width3");

    let mut actual = ColumnIndex::with_capacity(10);

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

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["  1. When the\r"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "width4");

    let mut actual = ColumnIndex::with_capacity(10);

    assert_eq!(actual.width_before(&options, &rope.line(0), 11), 11);
    assert_eq!(actual.width_until(&options, &rope.line(0), 10), 11);

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

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec![
      "一行文本小到可以放入一个窗口中，那么line-wrap和word-wrap选项就不会影响排版。\n",
    ]);

    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "width5");

    let mut actual = ColumnIndex::with_capacity(10);

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

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec![
      "\t\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
    ]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "width6");

    let mut actual = ColumnIndex::with_capacity(10);

    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 8);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 16);
    assert_eq!(actual.width_until(&options, &rope.line(0), 2), 17);

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

    let options = make_default_text_opts();
    let rope = Rope::new();

    let mut actual = ColumnIndex::with_capacity(10);

    assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 10), 0);

    let rope = make_rope_from_lines(vec![]);
    let mut actual = ColumnIndex::with_capacity(10);

    assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 10), 0);

    let rope = make_rope_from_lines(vec![""]);
    let mut actual = ColumnIndex::with_capacity(10);

    assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 0), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 1), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 2), 0);
    assert_eq!(actual.width_until(&options, &rope.line(0), 10), 0);
  }

  fn assert_char_before(
    options: &TextOptions,
    buf_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_before: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_before.iter() {
      let actual = widx.char_before(options, buf_line, *w);
      info!("char_before expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_until(options, buf_line, c.unwrap());
        info!("width_at-1 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual < *w);
      }
    }
  }

  fn assert_char_at(
    options: &TextOptions,
    buf_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_until: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_until.iter() {
      let actual = widx.char_at(options, buf_line, *w);
      info!("char_at expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_until(options, buf_line, c.unwrap());
        info!("width_at-2 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual >= *w);
      } else {
        info!("width_at-2 char:{c:?} expect width:{w:?}");
      }
    }
  }

  fn assert_char_after(
    options: &TextOptions,
    buf_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_after: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_after.iter() {
      let actual = widx.char_after(options, buf_line, *w);
      info!("char_after expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      if c.is_some() {
        let actual = widx.width_until(options, buf_line, c.unwrap());
        info!("width_at-3 char:{c:?} expect width:{w:?}, actual width:{actual:?}");
        assert!(actual >= *w);
      }
    }
  }

  fn assert_last_char_until(
    options: &TextOptions,
    buf_line: &RopeSlice,
    widx: &mut ColumnIndex,
    expect_until: &[(usize, Option<usize>)],
  ) {
    for (w, c) in expect_until.iter() {
      let actual = widx.last_char_until(options, buf_line, *w);
      info!("last_char_until expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
    }
  }

  #[test]
  fn char1() {
    test_log_init();

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["These are\t很简单的test\tlines.\n"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "char1");

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

    let expect_until: Vec<(usize, Option<usize>)> = vec![
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
      (44, Some(24)),
      (45, Some(24)),
      (46, Some(24)),
    ];
    assert_last_char_until(&options, &rope.line(0), &mut widx, &expect_until);
  }

  #[test]
  fn char2() {
    test_log_init();

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["\t"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "char2");
    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, None),
      (3, None),
      (6, None),
      (7, None),
      (8, None),
      (9, None),
      (10, None),
    ];
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, Some(0)),
      (2, Some(0)),
      (3, Some(0)),
      (4, Some(0)),
      (5, Some(0)),
      (6, Some(0)),
      (7, Some(0)),
      (8, Some(0)),
      (9, None),
      (10, None),
    ];
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> = vec![
      (0, Some(0)),
      (1, None),
      (2, None),
      (3, None),
      (5, None),
      (6, None),
      (7, None),
      (8, None),
      (9, None),
      (10, None),
    ];
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);

    let expect_until: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, Some(0)),
      (2, Some(0)),
      (3, Some(0)),
      (4, Some(0)),
      (5, Some(0)),
      (6, Some(0)),
      (7, Some(0)),
      (8, Some(0)),
      (9, Some(0)),
      (10, Some(0)),
      (11, Some(0)),
      (12, Some(0)),
    ];
    assert_last_char_until(&options, &rope.line(0), &mut widx, &expect_until);
  }

  #[test]
  fn char3() {
    test_log_init();

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["\n"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "char3");
    let mut widx = ColumnIndex::with_capacity(10);

    let expect_before: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, None),
      (3, None),
      (6, None),
      (7, None),
      (8, None),
      (9, None),
      (10, None),
    ];
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> = vec![
      (0, Some(0)),
      (1, None),
      (2, None),
      (3, None),
      (4, None),
      (5, None),
      (6, None),
      (7, None),
      (8, None),
      (9, None),
      (10, None),
    ];
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> = vec![
      (0, Some(0)),
      (1, None),
      (2, None),
      (3, None),
      (5, None),
      (6, None),
      (7, None),
      (8, None),
      (9, None),
      (10, None),
    ];
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);

    let expect_until: Vec<(usize, Option<usize>)> = vec![
      (0, Some(0)),
      (1, Some(0)),
      (2, Some(0)),
      (3, Some(0)),
      (4, Some(0)),
      (5, Some(0)),
      (6, Some(0)),
      (7, Some(0)),
      (8, Some(0)),
      (9, Some(0)),
      (10, Some(0)),
    ];
    assert_last_char_until(&options, &rope.line(0), &mut widx, &expect_until);
  }

  #[test]
  fn char4() {
    test_log_init();

    let options = make_default_text_opts();

    {
      let rope = Rope::new();

      let mut widx = ColumnIndex::new();

      let expect_before: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

      let expect_at: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

      let expect_after: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
    }

    {
      let rope = make_rope_from_lines(vec![]);
      let mut widx = ColumnIndex::new();

      let expect_before: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

      let expect_at: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

      let expect_after: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
    }

    {
      let rope = make_rope_from_lines(vec![""]);
      let buffer = make_text_from_rope(10, options, rope.clone());
      print_text_line_details(buffer, 0, "char3-3");

      let mut widx = ColumnIndex::new();

      let expect_before: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

      let expect_at: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

      let expect_after: Vec<(usize, Option<usize>)> = (0..50).map(|i| (i, None)).collect();
      assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
    }
  }

  #[test]
  fn truncate1() {
    test_log_init();

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "truncate1");

    let mut widx = ColumnIndex::new();

    let expect_at: Vec<usize> =
      [(1..=6).collect(), (14..=20).collect(), vec![20, 20, 20, 20]].concat();

    for (c, w) in expect_at.iter().enumerate() {
      let actual = widx.width_until(&options, &rope.line(0), c);
      info!("truncate1-width_at expect width:{w:?}, char:{c:}, actual width:{actual:?}");
      assert_eq!(actual, *w);
      widx.truncate_since_char(c);
    }

    let expect_before: Vec<usize> =
      [(0..=6).collect(), (14..=20).collect(), vec![20, 20, 20]].concat();

    for (c, w) in expect_before.iter().enumerate() {
      let actual = widx.width_before(&options, &rope.line(0), c);
      info!("truncate1-width_before expect width:{w:?}, char:{c:}, actual width:{actual:?}");
      assert_eq!(actual, *w);
      widx.truncate_since_char(c);
    }
  }

  #[test]
  fn truncate2() {
    test_log_init();

    let options = make_default_text_opts();
    let rope = make_rope_from_lines(vec!["This is a quite\t简单而且很小的test\tlines.\n"]);
    let buffer = make_text_from_rope(10, options, rope.clone());
    print_text_line_details(buffer, 0, "truncate2");
    let mut widx = ColumnIndex::with_capacity(10);

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
      (29, Some(17)),
      (30, Some(18)),
      (31, Some(18)),
      (32, Some(19)),
      (33, Some(19)),
      (34, Some(20)),
      (35, Some(20)),
      (36, Some(21)),
      (37, Some(21)),
      (38, Some(22)),
      (39, Some(23)),
    ];

    let expect_at: Vec<(usize, Option<usize>)> = vec![
      (0, None),
      (1, Some(0)),
      (5, Some(4)),
      (10, Some(9)),
      (15, Some(14)),
      (16, Some(15)),
      (17, Some(15)),
      (22, Some(15)),
      (23, Some(15)),
      (24, Some(16)),
      (25, Some(16)),
      (26, Some(17)),
      (27, Some(17)),
      (28, Some(18)),
      (29, Some(18)),
      (30, Some(19)),
      (31, Some(19)),
      (32, Some(20)),
      (33, Some(20)),
      (34, Some(21)),
      (35, Some(21)),
    ];

    for (w, c) in expect_before.iter() {
      let actual = widx.char_before(&options, &rope.line(0), *w);
      info!("truncate2-char_before expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      widx.truncate_since_width(*w);
    }
    for (w, c) in expect_at.iter() {
      let actual = widx.char_at(&options, &rope.line(0), *w);
      info!("truncate2-char_at expect char:{c:?} width:{w:?}, actual char:{actual:?}");
      assert_eq!(actual, *c);
      widx.truncate_since_width(*w);
    }
  }
}
// spellchecker:on
