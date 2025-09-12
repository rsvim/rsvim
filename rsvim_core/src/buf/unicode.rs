//! Unicode utils.

use crate::buf::opt::BufferOptions;
use crate::buf::opt::FileFormatOption;
use crate::defaults::ascii::AsciiControlCodeFormatter;
use ascii::AsciiChar;
use compact_str::CompactString;
use icu::properties::CodePointMapData;
use icu::properties::props::EastAsianWidth;

/// Get the display width for a `char`, supports both ASCI control codes and
/// unicode.
///
/// The char display width follows the
/// [Unicode Standard Annex #11](https://www.unicode.org/reports/tr11/),
/// implemented with
/// [icu::properties::EastAsianWidth](https://docs.rs/icu/latest/icu/properties/maps/fn.east_asian_width.html#).
pub fn char_width(opt: &BufferOptions, c: char) -> usize {
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
    match CodePointMapData::<EastAsianWidth>::new().get(c) {
      EastAsianWidth::Wide => 2_usize,
      EastAsianWidth::Fullwidth => 2_usize,
      EastAsianWidth::Halfwidth => 1_usize,
      EastAsianWidth::Narrow => 1_usize,
      EastAsianWidth::Ambiguous => 1_usize,
      EastAsianWidth::Neutral => 1_usize,
      _ => 1_usize,
    }
  }
}

/// Get the printable cell symbol and its display width.
pub fn char_symbol(opt: &BufferOptions, c: char) -> CompactString {
  if c.is_ascii_control() {
    let ac = AsciiChar::from_ascii(c).unwrap();
    match ac {
      AsciiChar::Tab => {
        CompactString::from(" ".repeat(opt.tab_stop() as usize))
      }
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
pub fn str_width(opt: &BufferOptions, s: &str) -> usize {
  s.chars().map(|c| char_width(opt, c)).sum()
}

/// Get the printable cell symbols and the display width for a unicode `str`.
pub fn str_symbols(opt: &BufferOptions, s: &str) -> CompactString {
  s.chars().map(|c| char_symbol(opt, c)).fold(
    CompactString::with_capacity(s.len()),
    |mut init_symbol, mut symbol| {
      init_symbol.push_str(symbol.as_mut_str());
      init_symbol
    },
  )
}
