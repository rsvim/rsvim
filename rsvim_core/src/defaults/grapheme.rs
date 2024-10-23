//! Grapheme cluster and unicode.

use crate::res::AnyErr;

#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
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
      AsciiControlCode::Lf => writeln!(f),     // \n
      AsciiControlCode::Vt => write!(f, "^K"),
      AsciiControlCode::Ff => write!(f, "^L"),
      AsciiControlCode::Cr => write!(f, "^M"), // \r
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

macro_rules! ascii_control_code_converter_impl {
  ($plain_type:ty, $method_name:tt) => {
    impl std::convert::TryFrom<$plain_type> for AsciiControlCode {
      type Error = AnyErr;

      fn try_from(value: $plain_type) -> Result<Self, Self::Error> {
        match num::FromPrimitive::$method_name(value) {
          Some(code) => Ok(code),
          None => anyhow::bail!(
            "Cannot convert {} to AsciiControlCode ({}-{})",
            value,
            Self::min() as $plain_type,
            Self::max() as $plain_type
          ),
        }
      }
    }

    #[allow(clippy::from_over_into)]
    impl std::convert::Into<$plain_type> for AsciiControlCode {
      fn into(self) -> $plain_type {
        self as $plain_type
      }
    }
  };
}

ascii_control_code_converter_impl!(i8, from_i8);
ascii_control_code_converter_impl!(u8, from_u8);
ascii_control_code_converter_impl!(i16, from_i16);
ascii_control_code_converter_impl!(u16, from_u16);
ascii_control_code_converter_impl!(i32, from_i32);
ascii_control_code_converter_impl!(u32, from_u32);
ascii_control_code_converter_impl!(i128, from_i128);
ascii_control_code_converter_impl!(u128, from_u128);
ascii_control_code_converter_impl!(isize, from_isize);
ascii_control_code_converter_impl!(usize, from_usize);

impl std::convert::TryFrom<char> for AsciiControlCode {
  type Error = AnyErr;

  fn try_from(value: char) -> Result<Self, Self::Error> {
    Self::try_from(value as u32)
  }
}

#[allow(clippy::from_over_into)]
impl std::convert::Into<char> for AsciiControlCode {
  fn into(self) -> char {
    self as u8 as char
  }
}

impl AsciiControlCode {
  /// Maximum code
  pub fn max() -> AsciiControlCode {
    AsciiControlCode::Us
  }

  /// Minimum code
  pub fn min() -> AsciiControlCode {
    AsciiControlCode::Nul
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn convert1() {
    for i in 0..32 {
      let code = AsciiControlCode::try_from(i as u8).unwrap();
      let code: u8 = code.into();
      assert!(code == i as u8);
      let code = AsciiControlCode::try_from(i as i8).unwrap();
      let code: i8 = code.into();
      assert!(code == i as i8);
      let code = AsciiControlCode::try_from(i as u16).unwrap();
      let code: u16 = code.into();
      assert!(code == i as u16);
      let code = AsciiControlCode::try_from(i as i16).unwrap();
      let code: i16 = code.into();
      assert!(code == i as i16);
      let code = AsciiControlCode::try_from(i as u32).unwrap();
      let code: u32 = code.into();
      assert!(code == i as u32);
      let code = AsciiControlCode::try_from(i).unwrap();
      let code: i32 = code.into();
      assert!(code == i);
    }
    for i in 32..128 {
      let code = AsciiControlCode::try_from(i as u8);
      assert!(code.is_err());
      let code = AsciiControlCode::try_from(i as i8);
      assert!(code.is_err());
      let code = AsciiControlCode::try_from(i as u16);
      assert!(code.is_err());
      let code = AsciiControlCode::try_from(i as i16);
      assert!(code.is_err());
      let code = AsciiControlCode::try_from(i as u32);
      assert!(code.is_err());
      let code = AsciiControlCode::try_from(i);
      assert!(code.is_err());
    }
  }

  #[test]
  fn display() {
    for i in (AsciiControlCode::min() as u32)..(AsciiControlCode::max() as u32 + 1) {
      println!("{}", AsciiControlCode::try_from(i).unwrap());
    }
  }
}
