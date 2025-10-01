//! Ex command attributes.

use compact_str::CompactString;

use crate::js::converter::*;
use crate::prelude::*;
use std::str::FromStr;

/// Command attribute name.
pub const BANG_NAME: &str = "bang";
pub const NARGS_NAME: &str = "nargs";

/// Default command attributes.
pub const NARGS_VALUE: Nargs = Nargs::Zero;
pub const BANG_VALUE: bool = false;

#[derive(
  Debug,
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

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct CommandAttributes {
  #[builder(default = BANG_VALUE)]
  pub bang: bool,

  #[builder(default = NARGS_VALUE)]
  pub nargs: Nargs,
}

impl FromV8 for CommandAttributes {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    let mut builder = CommandAttributesBuilder::default();

    // bang
    let bang_name = to_v8(scope, BANG_NAME).unwrap();
    if let Some(bang_value) = value.get(scope, bang_name.into()) {
      builder.bang(from_v8::<bool>(scope, bang_value).unwrap());
    }

    // nargs
    let nargs_name = to_v8(scope, NARGS_NAME).unwrap();
    if let Some(nargs_value) = value.get(scope, nargs_name.into()) {
      let nargs = from_v8::<CompactString>(scope, nargs_value).unwrap();
      if let Ok(nargs) = Nargs::from_str(&nargs) {
        builder.nargs(nargs);
      }
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
    let bang_field = v8::String::new(scope, "bang").unwrap();
    let bang_value = v8::Boolean::new(scope, self.bang);
    obj.set(scope, bang_field.into(), bang_value.into());

    // nargs
    let nargs_field = v8::String::new(scope, "nargs").unwrap();
    let nargs_value = v8::String::new(scope, &self.nargs.to_string()).unwrap();
    obj.set(scope, nargs_field.into(), nargs_value.into());

    obj.into()
  }
}
