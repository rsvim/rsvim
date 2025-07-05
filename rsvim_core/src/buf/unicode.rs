//! Unicode utils.

use crate::buf::opt::{BufferLocalOptions, FileFormatOption};
use crate::defaults::ascii::AsciiControlCodeFormatter;

use ascii::AsciiChar;
use compact_str::CompactString;
//use tracing::trace;
use unicode_width::UnicodeWidthChar;

/// Get the display width for a `char`, supports both ASCI control codes and unicode.
///
/// The char display width follows the
/// [Unicode Standard Annex #11](https://www.unicode.org/reports/tr11/), implemented with
/// [UnicodeWidthChar], there's another equivalent crate
/// [icu::properties::EastAsianWidth](https://docs.rs/icu/latest/icu/properties/maps/fn.east_asian_width.html#).
pub fn char_width(opt: &BufferLocalOptions, c: char) -> usize {
  if c.is_ascii_control() {
    let ac = AsciiChar::from_ascii(c).unwrap();
    match ac {
      AsciiChar::Tab => opt.tab_stop() as usize,
      AsciiChar::LineFeed => 0,
      AsciiChar::CarriageReturn => {
        if opt.file_format() == FileFormatOption::Unix {
          let ascii_formatter = AsciiControlCodeFormatter::from(ac);
          format!("{ascii_formatter}").len()
        } else {
          0
        }
      }
      _ => {
        let ascii_formatter = AsciiControlCodeFormatter::from(ac);
        format!("{ascii_formatter}").len()
      }
    }
  } else {
    UnicodeWidthChar::width_cjk(c).unwrap()
  }
}

/// Get the printable cell symbol and its display width.
pub fn char_symbol(opt: &BufferLocalOptions, c: char) -> CompactString {
  if c.is_ascii_control() {
    let ac = AsciiChar::from_ascii(c).unwrap();
    match ac {
      AsciiChar::Tab => CompactString::from(" ".repeat(opt.tab_stop() as usize)),
      AsciiChar::LineFeed => CompactString::new(""),
      AsciiChar::CarriageReturn => {
        if opt.file_format() == FileFormatOption::Unix {
          let ascii_formatter = AsciiControlCodeFormatter::from(ac);
          CompactString::from(format!("{ascii_formatter}"))
        } else {
          CompactString::new("")
        }
      }
      _ => {
        let ascii_formatter = AsciiControlCodeFormatter::from(ac);
        CompactString::from(format!("{ascii_formatter}"))
      }
    }
  } else {
    CompactString::from(c.to_string())
  }
}

/// Get the display width for a unicode `str`.
pub fn str_width(opt: &BufferLocalOptions, s: &str) -> usize {
  s.chars().map(|c| char_width(opt, c)).sum()
}

/// Get the printable cell symbols and the display width for a unicode `str`.
pub fn str_symbols(opt: &BufferLocalOptions, s: &str) -> CompactString {
  s.chars().map(|c| char_symbol(opt, c)).fold(
    CompactString::with_capacity(s.len()),
    |mut init_symbol, mut symbol| {
      init_symbol.push_str(symbol.as_mut_str());
      init_symbol
    },
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::opt::BufferLocalOptionsBuilder;
  use crate::defaults::ascii::AsciiControlCodeFormatter;
  use crate::test::log::init as test_log_init;

  use tracing::info;

  #[test]
  fn char_width1() {
    test_log_init();

    for i in 0_u8..32_u8 {
      let c = i as char;
      let asciic = AsciiChar::from_ascii(c).unwrap();
      let opt = BufferLocalOptionsBuilder::default().build().unwrap();
      let asciifmt = AsciiControlCodeFormatter::from(asciic);
      let formatted = format!("{asciifmt}");
      let formatted_len = formatted.len();
      info!("i:{i},c:{c:?},ascii char:{asciic:?},ascii formatted:{formatted:?}({formatted_len})");
      assert_eq!(char_width(&opt, c), formatted_len);
    }
  }
}
