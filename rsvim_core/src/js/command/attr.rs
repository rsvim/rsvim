//! Ex command attributes.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_prop;
use crate::js::converter::*;
use crate::to_v8_prop;
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
    let nargs = value.to_rust_string_lossy(scope);
    Nargs::from_str(&nargs).unwrap()
  }
}

impl StringToV8 for Nargs {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::String> {
    self.to_string().to_v8(scope)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandAttributes {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // bang
  flags: Flags,

  #[builder(default = NARGS_DEFAULT)]
  pub nargs: Nargs,
}

flags_builder_impl!(CommandAttributes, flags, bang);

impl CommandAttributes {
  pub fn bang(&self) -> bool {
    self.flags.contains(Flags::BANG)
  }
}

#[allow(non_camel_case_types)]
type js_command_attr_Nargs = Nargs;

impl StructFromV8 for CommandAttributes {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = CommandAttributesBuilder::default();

    from_v8_prop!(builder, obj, scope, bool, bang);
    from_v8_prop!(builder, obj, scope, js_command_attr_Nargs, nargs);

    builder.build().unwrap()
  }
}

impl StructToV8 for CommandAttributes {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, bang, ());
    to_v8_prop!(self, obj, scope, nargs);

    obj
  }
}
