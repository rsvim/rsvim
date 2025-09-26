//! Ex command attributes.

use std::str::FromStr;

/// Command attribute name.
pub const BANG_NAME: &str = "bang";
pub const NARGS_NAME: &str = "nargs";

/// Default command attributes.
pub const NARGS_VALUE: Nargs = Nargs::Zero;
pub const BANG_VALUE: bool = false;

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

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
pub struct CommandAttributes {
  #[builder(default = BANG_VALUE)]
  pub bang: bool,

  #[builder(default = NARGS_VALUE)]
  pub nargs: Nargs,
}

impl CommandAttributes {
  pub fn from_v8_object<'a>(
    scope: &mut v8::HandleScope,
    value: v8::Local<'a, v8::Object>,
  ) -> Self {
    let mut builder = CommandAttributesBuilder::default();

    // bang
    let bang_name = v8::String::new(scope, BANG_NAME).unwrap();
    match value.get(scope, bang_name.into()) {
      Some(bang_value) => {
        let bang = bang_value.to_boolean(scope).boolean_value(scope);
        builder.bang(bang);
      }
      None => { /* do nothing */ }
    }

    // nargs
    let nargs_name = v8::String::new(scope, NARGS_NAME).unwrap();
    match value.get(scope, nargs_name.into()) {
      Some(nargs_value) => {
        let nargs = nargs_value.to_rust_string_lossy(scope);
        if let Ok(nargs) = Nargs::from_str(&nargs) {
          builder.nargs(nargs);
        }
      }
      None => { /* do nothing */ }
    }

    builder.build().unwrap()
  }

  pub fn into_v8_object<'a>(
    &self,
    scope: &mut v8::HandleScope<'a>,
  ) -> v8::Local<'a, v8::Object> {
    // obj
    let obj = v8::Object::new(scope);

    // internal fields
    {
      // bang
      let attr_bang_field = v8::String::new(scope, "bang").unwrap();
      let attr_bang_value = v8::Boolean::new(scope, self.bang);
      obj.set(scope, attr_bang_field.into(), attr_bang_value.into());

      // nargs
      let attr_nargs_field = v8::String::new(scope, "nargs").unwrap();
      let attr_nargs_value =
        v8::String::new(scope, &self.nargs.to_string()).unwrap();
      obj.set(scope, attr_nargs_field.into(), attr_nargs_value.into());
    }

    obj
  }
}
