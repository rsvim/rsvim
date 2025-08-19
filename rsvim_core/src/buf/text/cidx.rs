//! Indexes mappings between character and its display width.

use crate::buf::opt::BufferOptions;
use crate::buf::unicode;
use ropey::RopeSlice;

use smallvec::SmallVec;
use std::collections::BTreeMap;

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
    options: &BufferOptions,
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
    options: &BufferOptions,
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
    options: &BufferOptions,
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
    options: &BufferOptions,
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
    options: &BufferOptions,
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
    options: &BufferOptions,
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

      let (last_width, _last_char_idx) =
        self.width2char.last_key_value().unwrap();
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
    options: &BufferOptions,
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

      let (last_width, _last_char_idx) =
        self.width2char.last_key_value().unwrap();
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
    options: &BufferOptions,
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
    options: &BufferOptions,
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
      let (last_width, _last_char_idx) =
        self.width2char.last_key_value().unwrap();
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
