//! Grapheme cluster and unicode.

use ascii::AsciiChar;
use std::fmt;

/// The formatter for ASCII control code in [`AsciiChar`], helps implement the `Debug`/`Display` trait.
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
    assert!(value.is_ascii_control());
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
      AsciiChar::Tab => write!(f, "\t"),  // \t
      AsciiChar::LineFeed => writeln!(f), // \n
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

#[cfg(test)]
mod tests {
  use crate::defaults::grapheme::AsciiControlCodeFormatter;
  use ascii::AsciiChar;

  #[test]
  fn display() {
    for i in 0_u32..32_u32 {
      let ac = AsciiChar::from_ascii(i).unwrap();
      let fmt = AsciiControlCodeFormatter::from(ac);
      println!("{}:{}", i, fmt);
    }
  }
}
