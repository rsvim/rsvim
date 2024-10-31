//! Unicode utils for display.

use crate::buf::Buffer;
use crate::defaults::grapheme::AsciiControlCode;

use compact_str::CompactString;
use parking_lot::RwLockReadGuard;
use std::convert::From;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone)]
pub struct CharWidthOptions {
  pub tab_stop: u16,
}

impl From<&Buffer> for CharWidthOptions {
  fn from(value: &Buffer) -> Self {
    CharWidthOptions {
      tab_stop: value.tab_stop(),
    }
  }
}

impl From<&RwLockReadGuard<'_, Buffer>> for CharWidthOptions {
  fn from(value: &RwLockReadGuard<Buffer>) -> Self {
    CharWidthOptions {
      tab_stop: value.tab_stop(),
    }
  }
}

/// Display width for a `char`.
pub fn char_width(c: char, options: &CharWidthOptions) -> u16 {
  if c.is_ascii_control() {
    let cc = AsciiControlCode::try_from(c).unwrap();
    match cc {
      AsciiControlCode::Ht => options.tab_stop,
      AsciiControlCode::Lf => 0,
      _ => format!("{}", cc).len() as u16,
    }
  } else {
    UnicodeWidthChar::width_cjk(c).unwrap() as u16
  }
}

/// Returns printable cell symbol and display width.
pub fn char2cell(c: char, options: &CharWidthOptions) -> (CompactString, u16) {
  let width = char_width(c, options);
  if c.is_ascii_control() {
    let cc = AsciiControlCode::try_from(c).unwrap();
    match cc {
      AsciiControlCode::Ht => (
        CompactString::from(" ".repeat(options.tab_stop as usize)),
        width,
      ),
      AsciiControlCode::Lf => (CompactString::new(""), width),
      _ => (CompactString::from(format!("{}", cc)), width),
    }
  } else {
    (CompactString::from(c.to_string()), width)
  }
}
