//! Unicode utils.

use crate::buf::opt::BufferOptions;
use crate::buf::opt::FileFormatOption;
use ascii::AsciiChar;
use compact_str::CompactString;
use icu::properties::CodePointMapData;
use icu::properties::props::EastAsianWidth;
use std::fmt;

/// The formatter for ASCII control code in [`AsciiChar`], helps implement the
/// `Debug`/`Display` trait.
pub struct AsciiControlCodeFormatter {
  value: AsciiChar,
}

/// Build the ASCII char formatter from it.
///
/// # Panics
///
/// If the value is not a valid ASCII control code.
impl From<AsciiChar> for AsciiControlCodeFormatter {
  fn from(value: AsciiChar) -> Self {
    debug_assert!(value.is_ascii_control());
    AsciiControlCodeFormatter { value }
  }
}

impl fmt::Display for AsciiControlCodeFormatter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match self.value {
      AsciiChar::Null => write!(f, "^@"),
      AsciiChar::SOH => write!(f, "^A"),
      AsciiChar::SOX => write!(f, "^B"),
      AsciiChar::ETX => write!(f, "^C"),
      AsciiChar::EOT => write!(f, "^D"),
      AsciiChar::ENQ => write!(f, "^E"),
      AsciiChar::ACK => write!(f, "^F"),
      AsciiChar::Bell => write!(f, "^G"),
      AsciiChar::BackSpace => write!(f, "^H"),
      AsciiChar::Tab => write!(f, "\t"), // \t
      AsciiChar::LineFeed => writeln!(f, "^J"), // \n
      AsciiChar::VT => write!(f, "^K"),
      AsciiChar::FF => write!(f, "^L"),
      AsciiChar::CarriageReturn => write!(f, "^M"), // \r
      AsciiChar::SI => write!(f, "^N"),
      AsciiChar::SO => write!(f, "^0"),
      AsciiChar::DLE => write!(f, "^P"),
      AsciiChar::DC1 => write!(f, "^Q"),
      AsciiChar::DC2 => write!(f, "^R"),
      AsciiChar::DC3 => write!(f, "^S"),
      AsciiChar::DC4 => write!(f, "^T"),
      AsciiChar::NAK => write!(f, "^U"),
      AsciiChar::SYN => write!(f, "^V"),
      AsciiChar::ETB => write!(f, "^W"),
      AsciiChar::CAN => write!(f, "^X"),
      AsciiChar::EM => write!(f, "^Y"),
      AsciiChar::SUB => write!(f, "^Z"),
      AsciiChar::ESC => write!(f, "^["),
      AsciiChar::FS => write!(f, "^\\"),
      AsciiChar::GS => write!(f, "^]"),
      AsciiChar::RS => write!(f, "^^"),
      AsciiChar::US => write!(f, "^_"),
      _ => unreachable!(),
    }
  }
}

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
