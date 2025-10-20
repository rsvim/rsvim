//! Ex command attributes.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_impl;
use crate::js::converter::*;
use crate::to_v8_impl;
use compact_str::CompactString;
use std::str::FromStr;

flags_impl!(Flags, u8, BANG);

/// Command attribute name.
pub const BANG: &str = "bang";
pub const NARGS: &str = "nargs";

/// Default command attributes.
pub const NARGS_DEFAULT: Nargs = Nargs::Zero;
pub const BANG_DEFAULT: bool = false;

// bang=false
const FLAGS: Flags = Flags::empty();

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum Nargs {
  #[strum(serialize = "0")]
  /// No arguments
  Zero,

  #[strum(serialize = "1")]
  /// 1 argument
  One,

  #[strum(serialize = "?")]
  /// 0 or 1 argument
  Optional,

  #[strum(serialize = "+")]
  /// 1 or more arguments
  More,

  #[strum(serialize = "*")]
  /// Any arguments
  Any,
}

impl StringFromV8 for Nargs {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::String>,
  ) -> Self {
    let nargs = CompactString::from_v8(scope, value);
    Nargs::from_str(&nargs).unwrap()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandAttributes {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // bang
  flags: Flags,

  #[builder(default = NARGS_DEFAULT)]
  nargs: Nargs,
}

flags_builder_impl!(CommandAttributesBuilder, flags, Flags, bang);

impl CommandAttributes {
  pub fn bang(&self) -> bool {
    self.flags.contains(Flags::BANG)
  }

  pub fn nargs(&self) -> Nargs {
    self.nargs
  }
}

from_v8_impl!(CommandAttributes, [(bool, bang), (Nargs, nargs)], []);
to_v8_impl!(CommandAttributes, [bang, nargs], [], [], []);
