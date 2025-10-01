//! Ex command attributes.

use crate::js::converter::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::str::FromStr;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const NARGS: &str = "nargs";

/// Default command attributes.
pub const NARGS_DEFAULT: Nargs = Nargs::Zero;
pub const BANG_DEFAULT: bool = false;

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

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandAttributes {
  #[builder(default = BANG_DEFAULT)]
  pub bang: bool,

  #[builder(default = NARGS_DEFAULT)]
  pub nargs: Nargs,
}

impl FromV8 for CommandAttributes {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    let mut builder = CommandAttributesBuilder::default();
    if value.is_object() {
      let obj = value.to_object(scope).unwrap();

      // bang
      let bang_name = to_v8(scope, BANG.to_compact_string()).unwrap();
      if let Some(bang_value) = obj.get(scope, bang_name) {
        builder.bang(from_v8::<bool>(scope, bang_value).unwrap());
      }

      // nargs
      let nargs_name = to_v8(scope, NARGS.to_compact_string()).unwrap();
      if let Some(nargs_value) = obj.get(scope, nargs_name) {
        let nargs = from_v8::<CompactString>(scope, nargs_value).unwrap();
        if let Ok(nargs) = Nargs::from_str(&nargs) {
          builder.nargs(nargs);
        }
      }

      Some(builder.build().unwrap())
    } else {
      None
    }
  }
}

impl ToV8 for CommandAttributes {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let obj = v8::Object::new(scope);

    // bang
    let bang_field = to_v8(scope, BANG.to_compact_string()).unwrap();
    let bang_value = to_v8(scope, self.bang).unwrap();
    obj.set(scope, bang_field, bang_value);

    // nargs
    let nargs_field = to_v8(scope, NARGS.to_compact_string()).unwrap();
    let nargs_value = to_v8(scope, self.nargs.to_compact_string()).unwrap();
    obj.set(scope, nargs_field, nargs_value);

    Some(obj.into())
  }
}
