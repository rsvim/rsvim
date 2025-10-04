//! Ex command attributes.

use crate::js::converter::*;
use bitflags::Flag;
use bitflags::bitflags;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::fmt::Debug;
use std::str::FromStr;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const NARGS: &str = "nargs";

/// Default command attributes.
pub const NARGS_DEFAULT: Nargs = Nargs::Zero;
pub const BANG_DEFAULT: bool = false;

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

bitflags! {
  #[derive(Copy, Clone, PartialEq, Eq)]
  struct Flags: u8 {
    const BANG = 1;
  }
}

impl Debug for Flags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Flags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

// bang=false
const FLAGS: Flags = Flags::empty();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandAttributes {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // bang
  flags: Flags,

  #[builder(default = NARGS_DEFAULT)]
  nargs: Nargs,
}

impl CommandAttributesBuilder {
  pub fn bang(&mut self, value: bool) -> &mut Self {
    let mut flags = match self.flags {
      Some(flags) => flags,
      None => FLAGS,
    };
    if value {
      flags.insert(Flags::BANG);
    } else {
      flags.remove(Flags::BANG);
    }
    self.flags = Some(flags);
    self
  }
}

impl CommandAttributes {
  pub fn bang(&self) -> bool {
    self.flags.contains(Flags::BANG)
  }

  pub fn set_bang(&mut self, value: bool) {
    if value {
      self.flags.insert(Flags::BANG);
    } else {
      self.flags.remove(Flags::BANG);
    }
  }

  pub fn nargs(&self) -> Nargs {
    self.nargs
  }

  pub fn set_nargs(&mut self, value: Nargs) {
    self.nargs = value;
  }
}

impl FromV8 for CommandAttributes {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    let mut builder = CommandAttributesBuilder::default();
    let obj = value.to_object(scope).unwrap();

    // bang
    let bang_name = to_v8(scope, BANG);
    if let Some(bang_value) = obj.get(scope, bang_name) {
      builder.bang(from_v8::<bool>(scope, bang_value));
    }

    // nargs
    let nargs_name = to_v8(scope, NARGS);
    if let Some(nargs_value) = obj.get(scope, nargs_name) {
      let nargs = from_v8::<CompactString>(scope, nargs_value);
      builder.nargs(Nargs::from_str(&nargs).unwrap());
    }

    builder.build().unwrap()
  }
}

impl ToV8 for CommandAttributes {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // bang
    let bang_field = to_v8(scope, BANG);
    let bang_value = to_v8(scope, self.bang);
    obj.set(scope, bang_field, bang_value);

    // nargs
    let nargs_field = to_v8(scope, NARGS);
    let nargs_value = to_v8(scope, self.nargs.to_compact_string());
    obj.set(scope, nargs_field, nargs_value);

    obj.into()
  }
}
