//! Unicode utils for display.

use crate::buf::Buffer;
use crate::defaults::grapheme::AsciiControlCode;

use compact_str::CompactString;
use unicode_width::UnicodeWidthChar;

/// Display width for a `char`.
pub fn char_width(c: char, buf: &Buffer) -> u16 {
  if c.is_ascii_control() {
    let cc = AsciiControlCode::try_from(c).unwrap();
    match cc {
      AsciiControlCode::Ht => buf.tab_stop(),
      AsciiControlCode::Lf => 0,
      _ => format!("{}", cc).len() as u16,
    }
  } else {
    UnicodeWidthChar::width_cjk(c).unwrap() as u16
  }
}

/// Returns printable cell symbol and display width.
pub fn char2cell(c: char, buf: &Buffer) -> (CompactString, u16) {
  let width = char_width(c, buf);
  if c.is_ascii_control() {
    let cc = AsciiControlCode::try_from(c).unwrap();
    match cc {
      AsciiControlCode::Ht => (
        CompactString::from(" ".repeat(buf.tab_stop() as usize)),
        width,
      ),
      AsciiControlCode::Lf => (CompactString::new(""), width),
      _ => (CompactString::from(format!("{}", cc)), width),
    }
  } else {
    (CompactString::from(c.to_string()), width)
  }
}
