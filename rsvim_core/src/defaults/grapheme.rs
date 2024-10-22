//! Grapheme cluster and unicode.

use crate::error::AnyErr;

#[derive(Debug, Copy, Clone)]
/// ASCII control code.
/// See: <https://en.wikipedia.org/wiki/ASCII>.
/// See: <https://en.wikipedia.org/wiki/C0_and_C1_control_codes>.
pub enum AsciiControlCode {
  Nul = 0, // \0
  Soh = 1,
  Stx = 2,
  Etx = 3,
  Eot = 4,
  Enq = 5,
  Ack = 6,
  Bel = 7,
  Bs = 8,
  Ht = 9,  // \t
  Lf = 10, // \n
  Vt = 11,
  Ff = 12,
  Cr = 13, // \r
  So = 14,
  Si = 15,
  Dle = 16,
  Dc1 = 17,
  Dc2 = 18,
  Dc3 = 19,
  Dc4 = 20,
  Nak = 21,
  Syn = 22,
  Etb = 23,
  Can = 24,
  Em = 25,
  Sub = 26,
  Esc = 27,
  Fs = 28,
  Gs = 29,
  Rs = 30,
  Us = 31,
}

impl std::fmt::Display for AsciiControlCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AsciiControlCode::Nul => write!(f, "^@"),
      AsciiControlCode::Soh => write!(f, "^A"),
      AsciiControlCode::Stx => write!(f, "^B"),
      AsciiControlCode::Etx => write!(f, "^C"),
      AsciiControlCode::Eot => write!(f, "^D"),
      AsciiControlCode::Enq => write!(f, "^E"),
      AsciiControlCode::Ack => write!(f, "^F"),
      AsciiControlCode::Bel => write!(f, "^G"),
      AsciiControlCode::Bs => write!(f, "^H"),
      AsciiControlCode::Ht => write!(f, "\t"), // \t
      AsciiControlCode::Lf => write!(f, "\n"), // \n
      AsciiControlCode::Vt => write!(f, "^K"),
      AsciiControlCode::Ff => write!(f, "^L"),
      AsciiControlCode::Cr => write!(f, "\r"), // \r
      AsciiControlCode::So => write!(f, "^N"),
      AsciiControlCode::Si => write!(f, "^0"),
      AsciiControlCode::Dle => write!(f, "^P"),
      AsciiControlCode::Dc1 => write!(f, "^Q"),
      AsciiControlCode::Dc2 => write!(f, "^R"),
      AsciiControlCode::Dc3 => write!(f, "^S"),
      AsciiControlCode::Dc4 => write!(f, "^T"),
      AsciiControlCode::Nak => write!(f, "^U"),
      AsciiControlCode::Syn => write!(f, "^V"),
      AsciiControlCode::Etb => write!(f, "^W"),
      AsciiControlCode::Can => write!(f, "^X"),
      AsciiControlCode::Em => write!(f, "^Y"),
      AsciiControlCode::Sub => write!(f, "^Z"),
      AsciiControlCode::Esc => write!(f, "^["),
      AsciiControlCode::Fs => write!(f, "^\\"),
      AsciiControlCode::Gs => write!(f, "^]"),
      AsciiControlCode::Rs => write!(f, "^^"),
      AsciiControlCode::Us => write!(f, "^_"),
    }
  }
}

impl std::convert::TryFrom<u8> for AsciiControlCode {
  type Error = AnyErr;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    Ok(AsciiControlCode::Bs)
  }
}

impl std::convert::TryInto<u8> for AsciiControlCode {
  type Error = AnyErr;

  fn try_into(self) -> Result<u8, Self::Error> {
    Ok(0)
  }
}
